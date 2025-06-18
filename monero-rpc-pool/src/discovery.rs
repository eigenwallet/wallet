use std::time::{Duration, Instant};

use anyhow::Result;
use reqwest::Client;
use serde_json::Value;
use tracing::{debug, error, info, warn};
use url;

// TODO: Have a set of hardcoded nodes for bootstrapping
// and if we cant reach monero.fail

use crate::database::{Database, MoneroNode};

#[derive(Debug)]
pub struct HealthCheckOutcome {
    pub was_successful: bool,
    pub latency: Duration,
    pub discovered_network: Option<String>,
}

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

    /// Centralized node fetching from various sources
    pub async fn discover_nodes_from_sources(&self, network: &str) -> Result<()> {
        match network {
            "mainnet" => {
                info!("Fetching nodes from monero.fail API for mainnet");
                
                // Use the JSON API for mainnet
                let response = self
                    .client
                    .get("https://monero.fail/nodes.json?chain=monero&network=mainnet")
                    .send()
                    .await?;

                if !response.status().is_success() {
                    return Err(anyhow::anyhow!(
                        "Failed to fetch nodes: HTTP {}",
                        response.status()
                    ));
                }

                let nodes_data: Value = response.json().await?;
                self.process_node_data(&nodes_data, "mainnet").await?;
            }
            "stagenet" => {
                info!("Using hardcoded stagenet nodes (monero.fail doesn't support stagenet)");
                
                // Create a JSON structure matching monero.fail format for stagenet nodes
                let stagenet_nodes_json = serde_json::json!([
                    {"host": "node2.monerodevs.org", "port": 38089},
                    {"host": "stagenet.xmr-tw.org", "port": 38081},
                    {"host": "stagenet.xmr.ditatompel.com", "port": 443},
                    {"host": "3.10.182.182", "port": 38081},
                    {"host": "xmr-lux.boldsuck.org", "port": 38081},
                    {"host": "ykqlrp7lumcik3ubzz3nfsahkbplfgqshavmgbxb4fauexqzat6idjad.onion", "port": 38081},
                    {"host": "node.monerodevs.org", "port": 38089},
                    {"host": "ct36dsbe3oubpbebpxmiqz4uqk6zb6nhmkhoekileo4fts23rvuse2qd.onion", "port": 38081},
                    {"host": "125.229.105.12", "port": 38081},
                    {"host": "node3.monerodevs.org", "port": 38089}
                ]);
                
                self.process_node_data(&stagenet_nodes_json, "stagenet").await?;
            }
            "testnet" => {
                info!("Testnet node discovery not supported, skipping");
            }
            _ => {
                warn!("Unknown network '{}', skipping discovery", network);
            }
        }
        Ok(())
    }

    /// Process node data and insert into database
    async fn process_node_data(&self, nodes_data: &Value, source_network: &str) -> Result<()> {
        let mut success_count = 0;

        if let Some(nodes_array) = nodes_data.as_array() {
            for node_value in nodes_array {
                if let Some(node_obj) = node_value.as_object() {
                    if let (Some(host), Some(port)) = (
                        node_obj.get("host").and_then(|v| v.as_str()),
                        node_obj.get("port").and_then(|v| v.as_u64()),
                    ) {
                        let scheme = if port == 18089 || port == 443 { "https" } else { "http" };

                        match self.db.upsert_node(scheme, host, port as i64).await {
                            Ok(_) => success_count += 1,
                            Err(e) => error!("Failed to insert node {}:{}: {}", host, port, e),
                        }
                    }
                }
            }
        }

        info!(
            "Discovered and inserted {} nodes from {} source",
            success_count, source_network
        );
        Ok(())
    }

    /// Enhanced health check that detects network and validates node identity
    pub async fn check_node_health(&self, url: &str) -> Result<HealthCheckOutcome> {
        let start_time = Instant::now();

        let rpc_request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "0",
            "method": "get_info"
        });

        let full_url = format!("{}/json_rpc", url);
        let response = self.client.post(&full_url).json(&rpc_request).send().await;

        let latency = start_time.elapsed();

        match response {
            Ok(resp) => {
                if resp.status().is_success() {
                    match resp.json::<Value>().await {
                        Ok(json) => {
                            if let Some(result) = json.get("result") {
                                // Extract network information from get_info response
                                let discovered_network = self.extract_network_from_info(result);


                                Ok(HealthCheckOutcome {
                                    was_successful: true,
                                    latency,
                                    discovered_network,
                                })
                            } else {
                                Ok(HealthCheckOutcome {
                                    was_successful: false,
                                    latency,
                                    discovered_network: None,
                                })
                            }
                        }
                        Err(e) => {
                            Ok(HealthCheckOutcome {
                                was_successful: false,
                                latency,
                                discovered_network: None,
                            })
                        }
                    }
                } else {
                    Ok(HealthCheckOutcome {
                        was_successful: false,
                        latency,
                        discovered_network: None,
                    })
                }
            }
            Err(e) => {
                Ok(HealthCheckOutcome {
                    was_successful: false,
                    latency,
                    discovered_network: None,
                })
            }
        }
    }

    /// Extract network type from get_info response
    fn extract_network_from_info(&self, info_result: &Value) -> Option<String> {
        // Check nettype field (0 = mainnet, 1 = testnet, 2 = stagenet)
        if let Some(nettype) = info_result.get("nettype").and_then(|v| v.as_u64()) {
            return match nettype {
                0 => Some("mainnet".to_string()),
                1 => Some("testnet".to_string()),
                2 => Some("stagenet".to_string()),
                _ => None,
            };
        }

        // Fallback: check if testnet or stagenet is mentioned in fields
        if let Some(testnet) = info_result.get("testnet").and_then(|v| v.as_bool()) {
            return if testnet {
                Some("testnet".to_string())
            } else {
                Some("mainnet".to_string())
            };
        }

        // Additional heuristics could be added here
        None
    }

    /// Updated health check workflow with identification and validation logic
    pub async fn health_check_all_nodes(&self, target_network: &str) -> Result<()> {
        info!(
            "Starting health check for all nodes targeting network: {}",
            target_network
        );

        // Get all nodes from database (both identified and unidentified)
        let all_nodes = sqlx::query_as::<_, MoneroNode>(
            "SELECT *, 0 as success_count, 0 as failure_count, NULL as last_success, NULL as last_failure, NULL as last_checked, 0 as is_reliable, NULL as avg_latency_ms, NULL as min_latency_ms, NULL as max_latency_ms, NULL as last_latency_ms FROM monero_nodes ORDER BY id",
        )
        .fetch_all(&self.db.pool)
        .await?;

        let mut checked_count = 0;
        let mut healthy_count = 0;
        let mut identified_count = 0;
        let mut corrected_count = 0;

        for node in all_nodes {
            match self.check_node_health(&node.full_url).await {
                Ok(outcome) => {
                    // Always record the health check
                    self.db
                        .record_health_check(
                            &node.full_url,
                            outcome.was_successful,
                            if outcome.was_successful {
                                Some(outcome.latency.as_millis() as f64)
                            } else {
                                None
                            },
                        )
                        .await?;

                    if outcome.was_successful {
                        healthy_count += 1;

                        // Handle network identification and validation
                        if let Some(discovered_network) = outcome.discovered_network {
                            match &node.network {
                                None => {
                                    // Node is unidentified - identify it
                                    info!(
                                        "Identifying node {} as network: {}",
                                        node.full_url, discovered_network
                                    );
                                    self.db
                                        .update_node_network(&node.full_url, &discovered_network)
                                        .await?;
                                    identified_count += 1;
                                }
                                Some(stored_network) => {
                                    // Node is already identified - validate it
                                    if stored_network != &discovered_network {
                                        warn!("Network mismatch detected for node {}: stored={}, discovered={}. Correcting...", 
                                              node.full_url, stored_network, discovered_network);
                                        self.db
                                            .update_node_network(
                                                &node.full_url,
                                                &discovered_network,
                                            )
                                            .await?;
                                        corrected_count += 1;
                                    }
                                }
                            }
                        }
                    }
                    checked_count += 1;
                }
                Err(e) => {
                    self.db
                        .record_health_check(&node.full_url, false, None)
                        .await?;
                }
            }

            // Small delay to avoid hammering nodes
            tokio::time::sleep(Duration::from_secs(2)).await;
        }

        info!(
            "Health check completed: {}/{} nodes healthy, {} newly identified, {} corrected",
            healthy_count, checked_count, identified_count, corrected_count
        );

        Ok(())
    }

    /// Periodic discovery task with improved error handling
    pub async fn periodic_discovery_task(&self, target_network: &str) -> Result<()> {
        let mut interval = tokio::time::interval(Duration::from_secs(3600)); // Every hour

        loop {
            interval.tick().await;

            info!("Running periodic node discovery for network: {}", target_network);

            // Discover new nodes from sources
            if let Err(e) = self.discover_nodes_from_sources(target_network).await {
                error!("Failed to discover nodes: {}", e);
            }

            // Health check all nodes (will identify networks automatically)
            if let Err(e) = self.health_check_all_nodes(target_network).await {
                error!("Failed to perform health check: {}", e);
            }

            // Log stats for all networks
            for network in &["mainnet", "stagenet", "testnet"] {
                if let Ok((total, reachable, reliable)) = self.db.get_node_stats(network).await {
                    if total > 0 {
                        info!(
                            "Node stats for {}: {} total, {} reachable, {} reliable",
                            network, total, reachable, reliable
                        );
                    }
                }
            }
        }
    }

    /// Insert configured nodes for a specific network
    pub async fn discover_and_insert_nodes(
        &self,
        target_network: &str,
        nodes: Vec<String>,
    ) -> Result<()> {
        info!("Inserting {} configured nodes", nodes.len());

        let mut success_count = 0;
        let mut error_count = 0;

        for (i, node_url) in nodes.iter().enumerate() {
            if let Ok(url) = url::Url::parse(node_url) {
                let scheme = url.scheme();
                let host = url.host_str().unwrap_or("");
                let port = url
                    .port()
                    .unwrap_or(if scheme == "https" { 18089 } else { 18081 })
                    as i64;

                debug!(
                    "Inserting configured node {}/{}: {}://{}:{}",
                    i + 1,
                    nodes.len(),
                    scheme,
                    host,
                    port
                );

                match self.db.upsert_node(scheme, host, port).await {
                    Ok(_) => {
                        success_count += 1;

                        // For configured nodes, we can immediately set the target network
                        // This is safe because these are explicitly configured by the user
                        let full_url = format!("{}://{}:{}", scheme, host, port);
                        if let Err(e) = self.db.update_node_network(&full_url, target_network).await
                        {
                            warn!(
                                "Failed to set network for configured node {}: {}",
                                full_url, e
                            );
                        }
                    }
                    Err(e) => {
                        error_count += 1;
                        error!(
                            "Failed to insert configured node {}://{}:{}: {}",
                            scheme, host, port, e
                        );
                    }
                }
            } else {
                error_count += 1;
                error!("Failed to parse node URL: {}", node_url);
            }
        }

        info!(
            "Configured node insertion complete: {} successful, {} errors",
            success_count, error_count
        );
        Ok(())
    }
}
