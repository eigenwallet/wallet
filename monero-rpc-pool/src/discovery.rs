use std::error::Error;
use std::time::{Duration, Instant};

use anyhow::Result;
use regex::Regex;
use reqwest::Client;
use serde_json::Value;
use tracing::{debug, error, info, warn};
use url;

use crate::database::{Database, MoneroNode};

#[derive(Clone)]
pub struct NodeDiscovery {
    client: Client,
    db: Database,
}

impl NodeDiscovery {
    pub fn new(db: Database) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent("monero-rpc-pool/1.0")
            .build()
            .unwrap();

        Self { client, db }
    }

    // TODO: Replace this with https://monero.fail/nodes.json?chain=monero&network=mainnet
    pub async fn discover_nodes_from_monero_fail(&self, network: &str) -> Result<()> {
        info!("Fetching nodes from monero.fail/haproxy.cfg for network: {}", network);
        debug!("HTTP client config: timeout=10s, user_agent=default");

        let request = self.client.get("https://monero.fail/haproxy.cfg").build()?;

        debug!(
            "Built request: method={}, url={}, headers={:?}",
            request.method(),
            request.url(),
            request.headers()
        );

        let response = self.client.execute(request).await.map_err(|e| {
            error!(
                "HTTP request failed - error type: {}, error details: {:?}, error source: {:?}",
                std::any::type_name_of_val(&e),
                e,
                e.source()
            );
            e
        })?;

        debug!(
            "Response status: {}, headers: {:?}",
            response.status(),
            response.headers()
        );

        let haproxy_config = response.text().await.map_err(|e| {
            error!("Failed to read response text - error: {:?}", e);
            e
        })?;

        debug!(
            "Downloaded config length: {} bytes, first 200 chars: {:?}",
            haproxy_config.len(),
            haproxy_config.chars().take(200).collect::<String>()
        );

        let nodes = self.parse_haproxy_config(&haproxy_config, network)?;

        info!("Discovered {} nodes from monero.fail for network {}", nodes.len(), network);
        debug!(
            "Sample nodes: {:?}",
            nodes.iter().take(3).collect::<Vec<_>>()
        );

        let mut success_count = 0;
        let mut error_count = 0;

        for (i, node) in nodes.iter().enumerate() {
            debug!("Inserting node {}/{}: {:?}", i + 1, nodes.len(), node);
            match self.db.upsert_node(node).await {
                Ok(_) => {
                    success_count += 1;
                    if i < 5 || i % 100 == 0 {
                        debug!("Successfully inserted node: {}", node.full_url);
                    }
                }
                Err(e) => {
                    error_count += 1;
                    error!(
                        "Failed to insert node {} - error: {:?}, node_data: {:?}",
                        node.full_url, e, node
                    );
                }
            }
        }

        info!(
            "Node insertion complete for network {}: {} successful, {} errors",
            network, success_count, error_count
        );

        Ok(())
    }

    // TODO: Remove this (replaced with .json api)
    fn parse_haproxy_config(&self, config: &str, network: &str) -> Result<Vec<MoneroNode>> {
        debug!("Starting HAProxy config parsing for network: {}", network);
        let mut nodes = Vec::new();

        // Regex to match server lines in HAProxy config
        // Example: server xmr-node-uk02 node.supportxmr.com:18081 check
        let server_regex = Regex::new(r"server\s+\S+\s+([^:\s]+):(\d+)").map_err(|e| {
            error!("Failed to compile regex - error: {:?}", e);
            e
        })?;

        debug!("Regex compiled successfully: {}", server_regex.as_str());

        let lines: Vec<&str> = config.lines().collect();
        debug!("Total lines in config: {}", lines.len());

        for (line_num, line) in lines.iter().enumerate() {
            let line = line.trim();
            if line.starts_with("server ") {
                debug!("Processing server line {}: '{}'", line_num + 1, line);

                if let Some(captures) = server_regex.captures(line) {
                    let host = captures.get(1).unwrap().as_str().to_string();
                    let port_str = captures.get(2).unwrap().as_str();

                    match port_str.parse::<i64>() {
                        Ok(port) => {
                            // Determine protocol based on port (common convention)
                            let (scheme, protocol) = if port == 18089 || line.contains("ssl") {
                                ("https".to_string(), "ssl".to_string())
                            } else {
                                ("http".to_string(), "tcp".to_string())
                            };

                            let node = MoneroNode::new(
                                scheme.clone(),
                                protocol.clone(),
                                host.clone(),
                                port,
                                network.to_string(),
                            );
                            debug!("Created node: scheme={}, protocol={}, host={}, port={}, network={}, full_url={}", 
                                   scheme, protocol, host, port, network, node.full_url);
                            nodes.push(node);
                        }
                        Err(e) => {
                            warn!(
                                "Failed to parse port '{}' on line {}: error={:?}",
                                port_str,
                                line_num + 1,
                                e
                            );
                        }
                    }
                } else {
                    debug!(
                        "Server line {} did not match regex: '{}'",
                        line_num + 1,
                        line
                    );
                }
            }
        }

        info!(
            "Parsed {} nodes from HAProxy config for network {} (total lines: {})",
            nodes.len(),
            network,
            lines.len()
        );
        debug!(
            "First 3 parsed nodes: {:?}",
            nodes.iter().take(3).collect::<Vec<_>>()
        );
        Ok(nodes)
    }

    pub async fn check_node_health(&self, url: &str) -> Result<(bool, f64)> {
        let start_time = Instant::now();

        // Try to make a simple get_info request
        let rpc_request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "0",
            "method": "get_info"
        });

        let full_url = format!("{}/json_rpc", url);
        let response = self.client.post(&full_url).json(&rpc_request).send().await;

        let elapsed = start_time.elapsed();
        let latency_ms = elapsed.as_millis() as f64;

        // TODO: Unnest this crap here
        match response {
            Ok(resp) => {
                if resp.status().is_success() {
                    match resp.json::<Value>().await {
                        Ok(json) => {
                            // Check if we got a valid RPC response
                            if json.get("result").is_some() || json.get("error").is_some() {
                                debug!("Node {} is healthy ({}ms)", url, latency_ms);
                                Ok((true, latency_ms))
                            } else {
                                debug!("Node {} returned invalid JSON structure", url);
                                Ok((false, latency_ms))
                            }
                        }
                        Err(e) => {
                            debug!("Node {} returned invalid JSON: {}", url, e);
                            Ok((false, latency_ms))
                        }
                    }
                } else {
                    debug!("Node {} returned HTTP {}", url, resp.status());
                    Ok((false, latency_ms))
                }
            }
            Err(e) => {
                tracing::trace!("Node {} is unreachable: {}", url, e);
                Ok((false, latency_ms))
            }
        }
    }

    pub async fn health_check_all_nodes(&self, network: &str) -> Result<()> {
        info!("Starting health check for all nodes in network: {}", network);

        // Get all nodes from database for the specified network
        let all_nodes = sqlx::query_as::<_, MoneroNode>(
            "SELECT * FROM monero_nodes WHERE network = ? ORDER BY last_checked ASC NULLS FIRST",
        )
        .bind(network)
        .fetch_all(&self.db.pool)
        .await?;

        let mut checked_count = 0;
        let mut healthy_count = 0;

        for node in all_nodes {
            match self.check_node_health(&node.full_url).await {
                Ok((is_healthy, latency)) => {
                    if is_healthy {
                        self.db.update_node_success(&node.full_url, latency).await?;
                        healthy_count += 1;
                    } else {
                        self.db.update_node_failure(&node.full_url).await?;
                    }
                    checked_count += 1;
                }
                Err(e) => {
                    tracing::trace!("Failed to check node {}: {}", node.full_url, e);
                    self.db.update_node_failure(&node.full_url).await?;
                }
            }

            // Small delay to avoid hammering nodes
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        info!(
            "Health check completed for network {}: {}/{} nodes are healthy",
            network, healthy_count, checked_count
        );

        // Update reliable nodes after health check
        self.db.update_reliable_nodes(network).await?;

        Ok(())
    }

    pub async fn periodic_discovery_task(&self, network: &str) -> Result<()> {
        let mut interval = tokio::time::interval(Duration::from_secs(3600)); // Every hour

        loop {
            interval.tick().await;

            info!("Running periodic node discovery for network: {}", network);

            // Discover new nodes
            if let Err(e) = self.discover_nodes_from_monero_fail(network).await {
                error!("Failed to discover nodes from monero.fail for network {}: {}", network, e);
            }

            // Health check all nodes
            if let Err(e) = self.health_check_all_nodes(network).await {
                error!("Failed to perform health check for network {}: {}", network, e);
            }

            let (total, reachable, reliable) = self.db.get_node_stats(network).await?;
            info!(
                "Node stats for network {}: {} total, {} reachable, {} reliable",
                network, total, reachable, reliable
            );
        }
    }

    pub async fn discover_and_insert_nodes(&self, network: &str, nodes: Vec<String>) -> Result<()> {
        info!("Inserting {} nodes for network: {}", nodes.len(), network);

        let mut success_count = 0;
        let mut error_count = 0;

        for (i, node_url) in nodes.iter().enumerate() {
            // Parse the URL to extract components
            if let Ok(url) = url::Url::parse(node_url) {
                let scheme = url.scheme().to_string();
                let protocol = if scheme == "https" { "ssl" } else { "tcp" };
                let host = url.host_str().unwrap_or("").to_string();
                let port = url.port().unwrap_or(if scheme == "https" { 18089 } else { 18081 }) as i64;
                
                let node = MoneroNode::new(
                    scheme,
                    protocol.to_string(),
                    host,
                    port,
                    network.to_string(),
                );
                
                debug!("Inserting node {}/{}: {:?}", i + 1, nodes.len(), node);
                match self.db.upsert_node(&node).await {
                    Ok(_) => {
                        success_count += 1;
                        if i < 5 || i % 100 == 0 {
                            debug!("Successfully inserted node: {}", node.full_url);
                        }
                    }
                    Err(e) => {
                        error_count += 1;
                        error!(
                            "Failed to insert node {} - error: {:?}",
                            node.full_url, e
                        );
                    }
                }
            } else {
                error_count += 1;
                error!("Failed to parse node URL: {}", node_url);
            }
        }

        info!(
            "Node insertion complete for network {}: {} successful, {} errors",
            network, success_count, error_count
        );

        Ok(())
    }
}
