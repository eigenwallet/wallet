use anyhow::Result;
use rand::prelude::*;
use tracing::{debug, warn};

use crate::database::Database;

pub struct SmartNodePool {
    db: Database,
    network: String,
}

impl SmartNodePool {
    pub fn new(db: Database, network: String) -> Self {
        Self { db, network }
    }

    /// Get next node using 70/30 strategy:
    /// - 70% from reliable nodes (top 4)
    /// - 30% from random reachable nodes
    pub async fn get_next_node(&self) -> Result<Option<String>> {
        // Use 70% chance for reliable nodes, 30% for random exploration
        let use_reliable = {
            let mut rng = thread_rng();
            rng.gen_bool(0.7)
        };

        if use_reliable {
            self.get_reliable_node().await
        } else {
            self.get_exploration_node().await
        }
    }

    async fn get_reliable_node(&self) -> Result<Option<String>> {
        let reliable_nodes = self.db.get_reliable_nodes(&self.network).await?;

        if reliable_nodes.is_empty() {
            debug!("No reliable nodes available for network {}, falling back to random selection", self.network);
            return self.get_exploration_node().await;
        }

        // Weight reliable nodes by inverse latency (lower latency = higher weight)
        let weighted_nodes: Vec<(String, f64)> = reliable_nodes
            .iter()
            .map(|node| {
                let weight = if let Some(latency) = node.avg_latency_ms {
                    1.0 / (latency + 1.0) // +1 to avoid division by zero
                } else {
                    1.0
                };
                (node.full_url.clone(), weight)
            })
            .collect();

        let selected = Self::weighted_random_selection(&weighted_nodes);
        debug!("Selected reliable node for network {}: {}", self.network, selected);
        Ok(Some(selected))
    }

    async fn get_exploration_node(&self) -> Result<Option<String>> {
        // Get a random node that's not in the reliable pool
        let random_nodes = self.db.get_random_nodes(10, &self.network).await?;

        if random_nodes.is_empty() {
            warn!("No random nodes available for exploration in network {}", self.network);
            return Ok(None);
        }

        let selected_node = {
            let mut rng = thread_rng();
            random_nodes.choose(&mut rng).unwrap()
        };
        debug!("Selected exploration node for network {}: {}", self.network, selected_node.full_url);
        Ok(Some(selected_node.full_url.clone()))
    }

    fn weighted_random_selection(weighted_items: &[(String, f64)]) -> String {
        let total_weight: f64 = weighted_items.iter().map(|(_, weight)| weight).sum();
        let mut random_value = {
            let mut rng = thread_rng();
            rng.gen::<f64>() * total_weight
        };

        for (item, weight) in weighted_items {
            random_value -= weight;
            if random_value <= 0.0 {
                return item.clone();
            }
        }

        // Fallback to first item if rounding errors occur
        weighted_items[0].0.clone()
    }

    pub async fn record_success(&self, url: &str, latency_ms: f64) -> Result<()> {
        self.db.update_node_success(url, latency_ms).await?;
        tracing::trace!("Recorded success for {} in network {}: {}ms", url, self.network, latency_ms);
        Ok(())
    }

    pub async fn record_failure(&self, url: &str) -> Result<()> {
        self.db.update_node_failure(url).await?;
        tracing::trace!("Recorded failure for {} in network {}", url, self.network);
        Ok(())
    }

    pub async fn get_pool_stats(&self) -> Result<PoolStats> {
        let (total, reachable, reliable) = self.db.get_node_stats(&self.network).await?;
        let reliable_nodes = self.db.get_reliable_nodes(&self.network).await?;

        let avg_reliable_latency = if reliable_nodes.is_empty() {
            None
        } else {
            let total_latency: f64 = reliable_nodes
                .iter()
                .filter_map(|node| node.avg_latency_ms)
                .sum();
            let count = reliable_nodes
                .iter()
                .filter(|node| node.avg_latency_ms.is_some())
                .count();

            if count > 0 {
                Some(total_latency / count as f64)
            } else {
                None
            }
        };

        Ok(PoolStats {
            total_nodes: total,
            reachable_nodes: reachable,
            reliable_nodes: reliable,
            avg_reliable_latency_ms: avg_reliable_latency,
        })
    }
}

#[derive(Debug)]
pub struct PoolStats {
    pub total_nodes: i64,
    pub reachable_nodes: i64,
    pub reliable_nodes: i64,
    pub avg_reliable_latency_ms: Option<f64>,
}

impl PoolStats {
    pub fn health_percentage(&self) -> f64 {
        if self.total_nodes == 0 {
            0.0
        } else {
            (self.reachable_nodes as f64 / self.total_nodes as f64) * 100.0
        }
    }
}
