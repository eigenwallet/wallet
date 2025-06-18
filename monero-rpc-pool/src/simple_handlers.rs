use axum::{
    body::Body,
    extract::Path,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::Response,
};
use std::time::Instant;
use tracing::{debug, error, info_span, Instrument};

use crate::AppState;

// TODO: Only have a single handler for all requests (no simple_handler vs smart_pool)

#[derive(Debug)]
enum HandlerError {
    NoNodes,
    PoolError(String),
    RequestError(String),
    AllRequestsFailed,
}

async fn get_two_nodes(state: &AppState) -> Result<(String, String), HandlerError> {
    let node_pool_guard = state.node_pool.read().await;

    // Get first node
    let node1 = node_pool_guard
        .get_next_node()
        .await
        .map_err(|e| HandlerError::PoolError(e.to_string()))?
        .ok_or(HandlerError::NoNodes)?;

    // Get second node (different from first)
    let mut node2 = node_pool_guard
        .get_next_node()
        .await
        .map_err(|e| HandlerError::PoolError(e.to_string()))?
        .ok_or(HandlerError::NoNodes)?;

    // If we got the same node, try once more for diversity
    if node2 == node1 {
        if let Ok(Some(different_node)) = node_pool_guard.get_next_node().await {
            if different_node != node1 {
                node2 = different_node;
            }
        }
    }

    Ok((node1, node2))
}

async fn raw_http_request(
    node_url: &str,
    path: &str,
    method: &str,
    headers: &HeaderMap,
    body: Option<&[u8]>,
) -> Result<Response, HandlerError> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| HandlerError::RequestError(e.to_string()))?;

    let url = format!("{}{}", node_url, path);
    
    // Do not differentiate between POST and GET here, allow all methods
    let mut request_builder = match method {
        "POST" => client.post(&url),
        "GET" => client.get(&url),
        _ => return Err(HandlerError::RequestError("Unsupported method".to_string())),
    };

    // Forward body if present
    if let Some(body_bytes) = body {
        request_builder = request_builder.body(body_bytes.to_vec());
    }

    // Forward essential headers
    for (name, value) in headers.iter() {
        let header_name = name.as_str();
        // Forward important headers, skip hop-by-hop headers
        // TODO: What is this for?
        if !matches!(header_name.to_lowercase().as_str(), 
            "host" | "connection" | "transfer-encoding" | "upgrade" | 
            "proxy-authenticate" | "proxy-authorization" | "te" | "trailers") {
            if let Ok(header_value) = std::str::from_utf8(value.as_bytes()) {
                request_builder = request_builder.header(header_name, header_value);
            }
        }
    }

    let response = request_builder
        .send()
        .await
        .map_err(|e| HandlerError::RequestError(e.to_string()))?;

    // Convert to axum Response preserving everything
    let status = response.status();
    let response_headers = response.headers().clone();
    let body_bytes = response
        .bytes()
        .await
        .map_err(|e| HandlerError::RequestError(e.to_string()))?;

    let mut axum_response = Response::new(Body::from(body_bytes));
    *axum_response.status_mut() = 
        StatusCode::from_u16(status.as_u16()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

    // Copy response headers exactly
    for (name, value) in response_headers.iter() {
        if let (Ok(header_name), Ok(header_value)) = (
            axum::http::HeaderName::try_from(name.as_str()),
            axum::http::HeaderValue::try_from(value.as_bytes()),
        ) {
            axum_response.headers_mut().insert(header_name, header_value);
        }
    }

    Ok(axum_response)
}

async fn record_success(state: &AppState, node_url: &str, latency_ms: f64) {
    let node_pool_guard = state.node_pool.read().await;
    if let Err(e) = node_pool_guard.record_success(node_url, latency_ms).await {
        error!("Failed to record success for {}: {}", node_url, e);
    }
}

async fn record_failure(state: &AppState, node_url: &str) {
    let node_pool_guard = state.node_pool.read().await;
    if let Err(e) = node_pool_guard.record_failure(node_url).await {
        error!("Failed to record failure for {}: {}", node_url, e);
    }
}

async fn single_raw_request(
    state: &AppState,
    node_url: String,
    path: &str,
    method: &str,
    headers: &HeaderMap,
    body: Option<&[u8]>,
) -> Result<(Response, String, f64), HandlerError> {
    let start_time = Instant::now();

    match raw_http_request(&node_url, path, method, headers, body).await {
        Ok(response) => {
            let elapsed = start_time.elapsed();
            let latency_ms = elapsed.as_millis() as f64;
            Ok((response, node_url, latency_ms))
        }
        Err(e) => {
            record_failure(state, &node_url).await;
            Err(e)
        }
    }
}

async fn race_requests(
    state: &AppState,
    path: &str,
    method: &str,
    headers: &HeaderMap,
    body: Option<&[u8]>,
) -> Result<Response, HandlerError> {
    const MAX_RETRIES: usize = 20;
    let mut tried_nodes = std::collections::HashSet::new();
    let mut attempt = 0;

    while attempt < MAX_RETRIES {
        // Get two new nodes that we haven't tried yet
        let node_pool_guard = state.node_pool.read().await;
        
        let mut node1_option = None;
        let mut node2_option = None;
        
        // Try to get first node
        for _ in 0..10 { // Max 10 attempts to find an untried node
            if let Ok(Some(node)) = node_pool_guard.get_next_node().await {
                if !tried_nodes.contains(&node) {
                    node1_option = Some(node);
                    break;
                }
            }
        }
        
        // Try to get second node
        for _ in 0..10 { // Max 10 attempts to find an untried node
            if let Ok(Some(node)) = node_pool_guard.get_next_node().await {
                if !tried_nodes.contains(&node) && Some(&node) != node1_option.as_ref() {
                    node2_option = Some(node);
                    break;
                }
            }
        }
        
        drop(node_pool_guard); // Release the lock

        // If we can't get any new nodes, we've exhausted our options
        if node1_option.is_none() && node2_option.is_none() {
            break;
        }

        let mut requests = Vec::new();
        
        if let Some(node1) = node1_option {
            tried_nodes.insert(node1.clone());
            requests.push(single_raw_request(state, node1.clone(), path, method, headers, body));
        }
        
        if let Some(node2) = node2_option {
            tried_nodes.insert(node2.clone());
            requests.push(single_raw_request(state, node2.clone(), path, method, headers, body));
        }

        if requests.is_empty() {
            break;
        }

        debug!("Attempt {}: Racing {} requests to {}: {} nodes", 
               attempt + 1, method, path, requests.len());

        // Handle the requests based on how many we have
        let result = match requests.len() {
            1 => {
                // Only one request
                requests.into_iter().next().unwrap().await
            }
            2 => {
                // Two requests - race them
                let mut iter = requests.into_iter();
                let req1 = iter.next().unwrap();
                let req2 = iter.next().unwrap();
                
                tokio::select! {
                    result1 = req1 => result1,
                    result2 = req2 => result2,
                }
            }
            _ => unreachable!("We only add 1 or 2 requests"),
        };
        
        match result {
            Ok((response, winning_node, latency_ms)) => {
                debug!("{} response from {} ({}ms) - SUCCESS on attempt {}!", 
                       method, winning_node, latency_ms, attempt + 1);
                record_success(state, &winning_node, latency_ms).await;
                return Ok(response);
            }
            Err(_) => {
                debug!("Attempt {} failed, retrying with different nodes...", attempt + 1);
                attempt += 1;
                continue;
            }
        }
    }

    error!("All {} requests failed after trying {} nodes", method, tried_nodes.len());
    Err(HandlerError::AllRequestsFailed)
}

#[axum::debug_handler]
pub async fn simple_rpc_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> Response {
    let body_size = body.len();
    async move {
        debug!("Raw RPC request: {} bytes", body_size);

        // TODO: Some requests (e.g publish transactions) should be sent to multiple nodes (e.g at least 5 successful or 20 retries)
        match race_requests(&state, "/json_rpc", "POST", &headers, Some(&body)).await {
            Ok(response) => response,
            Err(_) => {
                let error_body = br#"{"jsonrpc":"2.0","error":{"code":-1,"message":"All nodes failed"},"id":null}"#;
                Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .header("content-type", "application/json")
                    .body(Body::from(&error_body[..]))
                    .unwrap_or_else(|_| Response::new(Body::empty()))
            }
        }
    }
    .instrument(info_span!("rpc_request", body_size = body_size))
    .await
}

#[axum::debug_handler]
pub async fn simple_http_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(endpoint): Path<String>,
) -> Response {
    let endpoint_clone = endpoint.clone();
    async move {
        debug!("Raw HTTP request: /{}", endpoint);

        match race_requests(&state, &format!("/{}", endpoint), "GET", &headers, None).await {
            Ok(response) => response,
            Err(_) => {
                Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(Body::from("All nodes failed"))
                    .unwrap_or_else(|_| Response::new(Body::empty()))
            }
        }
    }
    .instrument(info_span!("http_request", endpoint = %endpoint_clone))
    .await
}

#[axum::debug_handler]
pub async fn simple_stats_handler(State(state): State<AppState>) -> Response {
    async move {
        let node_pool_guard = state.node_pool.read().await;

        match node_pool_guard.get_current_status().await {
            Ok(status) => {
                let stats_json = serde_json::json!({
                    "status": "healthy",
                    "healthy_node_count": status.healthy_node_count,
                    "reliable_node_count": status.reliable_node_count,
                    "successful_health_checks": status.successful_health_checks,
                    "unsuccessful_health_checks": status.unsuccessful_health_checks,
                    "top_reliable_nodes": status.top_reliable_nodes
                });
                
                Response::builder()
                    .status(StatusCode::OK)
                    .header("content-type", "application/json")
                    .body(Body::from(stats_json.to_string()))
                    .unwrap_or_else(|_| Response::new(Body::empty()))
            }
            Err(e) => {
                error!("Failed to get pool status: {}", e);
                let error_json = r#"{"status":"error","message":"Failed to get pool status"}"#;
                Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .header("content-type", "application/json")
                    .body(Body::from(error_json))
                    .unwrap_or_else(|_| Response::new(Body::empty()))
            }
        }
    }
    .instrument(info_span!("stats_request"))
    .await
}
