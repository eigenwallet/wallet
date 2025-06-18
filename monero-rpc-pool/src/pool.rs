use std::collections::HashMap;
use std::time::{Duration, Instant};

use tracing::{debug, info, warn};

#[derive(Debug, Clone)]
pub struct NodeInfo {
    pub url: String,
    pub is_healthy: bool,
    pub last_failure: Option<Instant>,
    pub consecutive_failures: u32,
    pub request_count: u64,
}

impl NodeInfo {
    pub fn new(url: String) -> Self {
        Self {
            url,
            is_healthy: true,
            last_failure: None,
            consecutive_failures: 0,
            request_count: 0,
        }
    }

    pub fn mark_failure(&mut self) {
        self.consecutive_failures += 1;
        self.last_failure = Some(Instant::now());

        if self.consecutive_failures >= 3 {
            self.is_healthy = false;
            warn!(
                "Node {} marked as unhealthy after {} consecutive failures",
                self.url, self.consecutive_failures
            );
        }
    }

    pub fn mark_success(&mut self) {
        if !self.is_healthy && self.consecutive_failures > 0 {
            info!("Node {} recovered and marked as healthy", self.url);
        }
        self.consecutive_failures = 0;
        self.is_healthy = true;
        self.last_failure = None;
        self.request_count += 1;
    }

    pub fn should_retry(&self) -> bool {
        if self.is_healthy {
            return true;
        }

        if let Some(last_failure) = self.last_failure {
            let backoff_duration =
                Duration::from_secs(60 * (2_u64.pow(self.consecutive_failures.min(5))));
            return last_failure.elapsed() > backoff_duration;
        }

        true
    }
}

pub struct NodePool {
    nodes: HashMap<String, NodeInfo>,
    current_index: usize,
    node_urls: Vec<String>,
}

impl NodePool {
    pub fn new(node_urls: Vec<String>) -> Self {
        let mut nodes = HashMap::new();

        for url in &node_urls {
            nodes.insert(url.clone(), NodeInfo::new(url.clone()));
        }

        info!("Initialized node pool with {} nodes", node_urls.len());
        for url in &node_urls {
            debug!("  - {}", url);
        }

        Self {
            nodes,
            current_index: 0,
            node_urls,
        }
    }

    // TODO: Use a smarter selection algorithm here
    pub fn get_next_node(&mut self) -> Option<String> {
        if self.node_urls.is_empty() {
            return None;
        }

        let total_nodes = self.node_urls.len();
        let mut attempts = 0;

        while attempts < total_nodes {
            let url = &self.node_urls[self.current_index];
            self.current_index = (self.current_index + 1) % total_nodes;

            if let Some(node_info) = self.nodes.get(url) {
                if node_info.is_healthy || node_info.should_retry() {
                    debug!("Selected node: {}", url);
                    return Some(url.clone());
                }
            }

            attempts += 1;
        }

        warn!("No healthy nodes available, using first node as fallback");
        self.node_urls.first().cloned()
    }

    pub fn mark_node_failed(&mut self, url: &str) {
        if let Some(node_info) = self.nodes.get_mut(url) {
            node_info.mark_failure();
            debug!(
                "Marked node {} as failed (consecutive failures: {})",
                url, node_info.consecutive_failures
            );
        }
    }

    pub fn mark_node_success(&mut self, url: &str) {
        if let Some(node_info) = self.nodes.get_mut(url) {
            node_info.mark_success();
            debug!(
                "Marked node {} as successful (total requests: {})",
                url, node_info.request_count
            );
        }
    }

    pub fn get_node_stats(&self) -> Vec<(&String, &NodeInfo)> {
        self.nodes.iter().collect()
    }

    pub fn get_healthy_node_count(&self) -> usize {
        self.nodes.values().filter(|node| node.is_healthy).count()
    }
}
