use axum::{
    body::Body,
    extract::Path,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Json, Response},
};
use serde_json::Value;
use std::time::Instant;
use tokio::select;
use tracing::{debug, error};

use crate::AppState;

#[derive(Debug)]
enum HandlerError {
    NoNodes,
    PoolError(String),
    ClientError(String),
    RequestError(String),
    ParseError(String),
    AllRequestsFailed,
}

impl HandlerError {
    fn to_json_response(&self, id: Option<&Value>) -> Value {
        let (code, message) = match self {
            HandlerError::NoNodes => (-1, "No available nodes"),
            HandlerError::PoolError(_) => (-1, "Pool error"),
            HandlerError::ClientError(_) => (-1, "Client error"),
            HandlerError::RequestError(_) => (-1, "Request failed"),
            HandlerError::ParseError(_) => (-1, "Response parse error"),
            HandlerError::AllRequestsFailed => (-1, "All parallel requests failed"),
        };

        serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": {
                "code": code,
                "message": message
            }
        })
    }
}

async fn get_two_nodes(state: &AppState) -> Result<(String, String), HandlerError> {
    let smart_pool_guard = state.smart_pool.read().await;

    // Get first node
    let node1 = smart_pool_guard
        .get_next_node()
        .await
        .map_err(|e| HandlerError::PoolError(e.to_string()))?
        .ok_or(HandlerError::NoNodes)?;

    // Get second node (different from first)
    let mut node2 = smart_pool_guard
        .get_next_node()
        .await
        .map_err(|e| HandlerError::PoolError(e.to_string()))?
        .ok_or(HandlerError::NoNodes)?;

    // If we got the same node, try once more for diversity
    if node2 == node1 {
        if let Ok(Some(different_node)) = smart_pool_guard.get_next_node().await {
            if different_node != node1 {
                node2 = different_node;
            }
        }
    }

    Ok((node1, node2))
}

fn create_client() -> Result<reqwest::Client, HandlerError> {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| HandlerError::ClientError(e.to_string()))
}

async fn make_request(
    client: &reqwest::Client,
    node_url: &str,
    payload: &Value,
) -> Result<Response, HandlerError> {
    let response = client
        .post(&format!("{}/json_rpc", node_url))
        .json(payload)
        .send()
        .await
        .map_err(|e| HandlerError::RequestError(e.to_string()))?;

    // Convert reqwest::Response to axum::Response
    let status = response.status();
    let headers = response.headers().clone();
    let body = response
        .bytes()
        .await
        .map_err(|e| HandlerError::RequestError(e.to_string()))?;

    let mut axum_response = Response::new(Body::from(body));
    *axum_response.status_mut() =
        StatusCode::from_u16(status.as_u16()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

    // Copy all headers exactly as received
    for (name, value) in headers.iter() {
        if let (Ok(header_name), Ok(header_value)) = (
            axum::http::HeaderName::try_from(name.as_str()),
            axum::http::HeaderValue::try_from(value.as_bytes()),
        ) {
            axum_response
                .headers_mut()
                .insert(header_name, header_value);
        }
    }

    Ok(axum_response)
}

async fn make_http_request(
    client: &reqwest::Client,
    node_url: &str,
    endpoint: &str,
) -> Result<Response, HandlerError> {
    let response = client
        .get(&format!("{}{}", node_url, endpoint))
        .send()
        .await
        .map_err(|e| HandlerError::RequestError(e.to_string()))?;

    // Convert reqwest::Response to axum::Response
    let status = response.status();
    let headers = response.headers().clone();
    let body = response
        .bytes()
        .await
        .map_err(|e| HandlerError::RequestError(e.to_string()))?;

    let mut axum_response = Response::new(Body::from(body));
    *axum_response.status_mut() =
        StatusCode::from_u16(status.as_u16()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

    // Copy all headers exactly as received
    for (name, value) in headers.iter() {
        if let (Ok(header_name), Ok(header_value)) = (
            axum::http::HeaderName::try_from(name.as_str()),
            axum::http::HeaderValue::try_from(value.as_bytes()),
        ) {
            axum_response
                .headers_mut()
                .insert(header_name, header_value);
        }
    }

    Ok(axum_response)
}

async fn record_success(state: &AppState, node_url: &str, latency_ms: f64) {
    let smart_pool_guard = state.smart_pool.read().await;
    if let Err(e) = smart_pool_guard.record_success(node_url, latency_ms).await {
        error!("Failed to record success for {}: {}", node_url, e);
    }
}

async fn record_failure(state: &AppState, node_url: &str) {
    let smart_pool_guard = state.smart_pool.read().await;
    if let Err(e) = smart_pool_guard.record_failure(node_url).await {
        error!("Failed to record failure for {}: {}", node_url, e);
    }
}

async fn make_single_request(
    state: &AppState,
    client: &reqwest::Client,
    node_url: String,
    payload: &Value,
) -> Result<(Response, String, f64), HandlerError> {
    let start_time = Instant::now();

    match make_request(client, &node_url, payload).await {
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

async fn make_single_http_request(
    state: &AppState,
    client: &reqwest::Client,
    node_url: String,
    endpoint: &str,
) -> Result<(Response, String, f64), HandlerError> {
    let start_time = Instant::now();

    match make_http_request(client, &node_url, endpoint).await {
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

async fn handle_rpc_request(state: &AppState, payload: &Value) -> Result<Response, HandlerError> {
    let (node1, node2) = get_two_nodes(state).await?;
    let client = create_client()?;

    debug!("Racing JSON-RPC requests to nodes: {} and {}", node1, node2);

    // Race two parallel requests - clone the nodes so we don't move them
    let request1 = make_single_request(state, &client, node1.clone(), payload);
    let request2 = make_single_request(state, &client, node2.clone(), payload);

    select! {
        result1 = request1 => {
            match result1 {
                Ok((response, winning_node, latency_ms)) => {
                    debug!(
                        "JSON-RPC response from {} ({}ms) - WINNER! Racing against {}",
                        winning_node, latency_ms, node2
                    );
                    record_success(state, &winning_node, latency_ms).await;
                    Ok(response)
                }
                Err(_) => {
                    // First request failed, try the second one
                    debug!("First JSON-RPC request to {} failed, trying second request to {}", node1, node2);
                    let fallback_result = make_single_request(state, &client, node2.clone(), payload).await;
                    match fallback_result {
                        Ok((response, winning_node, latency_ms)) => {
                            debug!(
                                "JSON-RPC response from {} ({}ms) - fallback success after {} failed",
                                winning_node, latency_ms, node1
                            );
                            record_success(state, &winning_node, latency_ms).await;
                            Ok(response)
                        }
                        Err(_) => {
                            error!("Both parallel JSON-RPC requests failed: {} and {}", node1, node2);
                            Err(HandlerError::AllRequestsFailed)
                        }
                    }
                }
            }
        }
        result2 = request2 => {
            match result2 {
                Ok((response, winning_node, latency_ms)) => {
                    debug!(
                        "JSON-RPC response from {} ({}ms) - WINNER! Racing against {}",
                        winning_node, latency_ms, node1
                    );
                    record_success(state, &winning_node, latency_ms).await;
                    Ok(response)
                }
                Err(_) => {
                    // Second request failed, try the first one
                    debug!("Second JSON-RPC request to {} failed, trying first request to {}", node2, node1);
                    let fallback_result = make_single_request(state, &client, node1.clone(), payload).await;
                    match fallback_result {
                        Ok((response, winning_node, latency_ms)) => {
                            debug!(
                                "JSON-RPC response from {} ({}ms) - fallback success after {} failed",
                                winning_node, latency_ms, node2
                            );
                            record_success(state, &winning_node, latency_ms).await;
                            Ok(response)
                        }
                        Err(_) => {
                            error!("Both parallel JSON-RPC requests failed: {} and {}", node1, node2);
                            Err(HandlerError::AllRequestsFailed)
                        }
                    }
                }
            }
        }
    }
}

async fn handle_http_request(state: &AppState, endpoint: &str) -> Result<Response, HandlerError> {
    let (node1, node2) = get_two_nodes(state).await?;
    let client = create_client()?;

    debug!(
        "Racing HTTP requests to {}: {} and {}",
        endpoint, node1, node2
    );

    // Race two parallel requests - clone the nodes so we don't move them
    let request1 = make_single_http_request(state, &client, node1.clone(), endpoint);
    let request2 = make_single_http_request(state, &client, node2.clone(), endpoint);

    select! {
        result1 = request1 => {
            match result1 {
                Ok((response, winning_node, latency_ms)) => {
                    debug!(
                        "HTTP response from {} ({}ms) for {} - WINNER! Racing against {}",
                        winning_node, latency_ms, endpoint, node2
                    );
                    record_success(state, &winning_node, latency_ms).await;
                    Ok(response)
                }
                Err(_) => {
                    // First request failed, try the second one
                    debug!("First HTTP request to {} failed, trying second request to {}", node1, node2);
                    let fallback_result = make_single_http_request(state, &client, node2.clone(), endpoint).await;
                    match fallback_result {
                        Ok((response, winning_node, latency_ms)) => {
                            debug!(
                                "HTTP response from {} ({}ms) for {} - fallback success after {} failed",
                                winning_node, latency_ms, endpoint, node1
                            );
                            record_success(state, &winning_node, latency_ms).await;
                            Ok(response)
                        }
                        Err(_) => {
                            error!("Both parallel HTTP requests failed for {}: {} and {}", endpoint, node1, node2);
                            Err(HandlerError::AllRequestsFailed)
                        }
                    }
                }
            }
        }
        result2 = request2 => {
            match result2 {
                Ok((response, winning_node, latency_ms)) => {
                    debug!(
                        "HTTP response from {} ({}ms) for {} - WINNER! Racing against {}",
                        winning_node, latency_ms, endpoint, node1
                    );
                    record_success(state, &winning_node, latency_ms).await;
                    Ok(response)
                }
                Err(_) => {
                    // Second request failed, try the first one
                    debug!("Second HTTP request to {} failed, trying first request to {}", node2, node1);
                    let fallback_result = make_single_http_request(state, &client, node1.clone(), endpoint).await;
                    match fallback_result {
                        Ok((response, winning_node, latency_ms)) => {
                            debug!(
                                "HTTP response from {} ({}ms) for {} - fallback success after {} failed",
                                winning_node, latency_ms, endpoint, node2
                            );
                            record_success(state, &winning_node, latency_ms).await;
                            Ok(response)
                        }
                        Err(_) => {
                            error!("Both parallel HTTP requests failed for {}: {} and {}", endpoint, node1, node2);
                            Err(HandlerError::AllRequestsFailed)
                        }
                    }
                }
            }
        }
    }
}

#[axum::debug_handler]
pub async fn simple_rpc_handler(
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> Response {
    // Add detailed logging of incoming requests
    debug!(
        "Received JSON-RPC request: {}",
        serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "Invalid JSON".to_string())
    );

    match handle_rpc_request(&state, &payload).await {
        Ok(response) => {
            debug!("Sending JSON-RPC response");
            response
        }
        Err(error) => {
            let error_response = error.to_json_response(payload.get("id"));
            debug!(
                "Sending JSON-RPC error response: {}",
                serde_json::to_string_pretty(&error_response)
                    .unwrap_or_else(|_| "Invalid JSON".to_string())
            );
            Json(error_response).into_response()
        }
    }
}

#[axum::debug_handler]
pub async fn simple_http_handler(
    State(state): State<AppState>,
    Path(endpoint): Path<String>,
) -> Result<Response, StatusCode> {
    debug!("Received HTTP request for endpoint: /{}", endpoint);

    match handle_http_request(&state, &format!("/{}", endpoint)).await {
        Ok(response) => {
            debug!("Sending HTTP response");
            Ok(response)
        }
        Err(error) => {
            let error_response = error.to_json_response(None);
            debug!(
                "Sending HTTP error response: {}",
                serde_json::to_string_pretty(&error_response)
                    .unwrap_or_else(|_| "Invalid JSON".to_string())
            );
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[axum::debug_handler]
pub async fn simple_stats_handler(State(state): State<AppState>) -> Json<Value> {
    let smart_pool_guard = state.smart_pool.read().await;

    match smart_pool_guard.get_pool_stats().await {
        Ok(stats) => Json(serde_json::json!({
            "status": "healthy",
            "total_nodes": stats.total_nodes,
            "reachable_nodes": stats.reachable_nodes,
            "reliable_nodes": stats.reliable_nodes,
            "health_percentage": stats.health_percentage(),
            "avg_reliable_latency_ms": stats.avg_reliable_latency_ms
        })),
        Err(e) => {
            error!("Failed to get pool stats: {}", e);
            Json(serde_json::json!({
                "status": "error",
                "message": "Failed to get pool stats"
            }))
        }
    }
}
