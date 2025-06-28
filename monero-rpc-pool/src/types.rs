use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeAddress {
    pub scheme: String, // "http" or "https"
    pub host: String,
    pub port: u16,
}

impl NodeAddress {
    pub fn new(scheme: String, host: String, port: u16) -> Self {
        Self { scheme, host, port }
    }

    pub fn full_url(&self) -> String {
        format!("{}://{}:{}", self.scheme, self.host, self.port)
    }
}

impl fmt::Display for NodeAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}://{}:{}", self.scheme, self.host, self.port)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMetadata {
    pub id: i64,
    pub network: String, // "mainnet", "stagenet", or "testnet"
    pub first_seen_at: DateTime<Utc>,
}

impl NodeMetadata {
    pub fn new(id: i64, network: String, first_seen_at: DateTime<Utc>) -> Self {
        Self {
            id,
            network,
            first_seen_at,
        }
    }
}

/// Health check statistics for a node
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NodeHealthStats {
    pub success_count: i64,
    pub failure_count: i64,
    pub last_success: Option<DateTime<Utc>>,
    pub last_failure: Option<DateTime<Utc>>,
    pub last_checked: Option<DateTime<Utc>>,
    pub is_reliable: bool,
    pub avg_latency_ms: Option<f64>,
    pub min_latency_ms: Option<f64>,
    pub max_latency_ms: Option<f64>,
    pub last_latency_ms: Option<f64>,
}

impl NodeHealthStats {
    pub fn success_rate(&self) -> f64 {
        let total = self.success_count + self.failure_count;
        if total == 0 {
            0.0
        } else {
            self.success_count as f64 / total as f64
        }
    }

    pub fn reliability_score(&self) -> f64 {
        let success_rate = self.success_rate();
        let total_requests = self.success_count + self.failure_count;

        // Weight success rate by total requests (more requests = more reliable data)
        let request_weight = (total_requests as f64).min(200.0) / 200.0;
        let mut score = success_rate * request_weight;

        // Factor in latency - lower latency = higher score
        if let Some(avg_latency) = self.avg_latency_ms {
            // Normalize latency to 0-1 range (assuming 0-2000ms range)
            let latency_factor = 1.0 - (avg_latency.min(2000.0) / 2000.0);
            score = score * 0.8 + latency_factor * 0.2; // 80% success rate, 20% latency
        }

        score
    }
}

/// A complete node record combining address, metadata, and health stats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeRecord {
    #[serde(flatten)]
    pub address: NodeAddress,
    #[serde(flatten)]
    pub metadata: NodeMetadata,
    #[serde(flatten)]
    pub health: NodeHealthStats,
}

impl NodeRecord {
    pub fn new(address: NodeAddress, metadata: NodeMetadata, health: NodeHealthStats) -> Self {
        Self {
            address,
            metadata,
            health,
        }
    }

    pub fn full_url(&self) -> String {
        self.address.full_url()
    }

    pub fn success_rate(&self) -> f64 {
        self.health.success_rate()
    }

    pub fn reliability_score(&self) -> f64 {
        self.health.reliability_score()
    }
}

/// Database row representation matching current SQL queries
/// This struct is used for conversion from sqlx::FromRow to our new types
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DbNodeRow {
    pub id: i64,
    pub scheme: String,
    pub host: String,
    pub port: i64, // Database uses i64
    pub network: String,
    pub first_seen_at: String, // ISO 8601 string
    #[sqlx(default)]
    pub success_count: i64,
    #[sqlx(default)]
    pub failure_count: i64,
    #[sqlx(default)]
    pub last_success: Option<String>,
    #[sqlx(default)]
    pub last_failure: Option<String>,
    #[sqlx(default)]
    pub last_checked: Option<String>,
    #[sqlx(default)]
    pub is_reliable: bool,
    #[sqlx(default)]
    pub avg_latency_ms: Option<f64>,
    #[sqlx(default)]
    pub min_latency_ms: Option<f64>,
    #[sqlx(default)]
    pub max_latency_ms: Option<f64>,
    #[sqlx(default)]
    pub last_latency_ms: Option<f64>,
}

impl From<DbNodeRow> for NodeRecord {
    fn from(row: DbNodeRow) -> Self {
        let address = NodeAddress::new(
            row.scheme,
            row.host,
            row.port as u16, // Convert from i64 to u16
        );

        let first_seen_at = row
            .first_seen_at
            .parse::<DateTime<Utc>>()
            .unwrap_or_else(|_| Utc::now());

        let metadata = NodeMetadata::new(row.id, row.network, first_seen_at);

        let health = NodeHealthStats {
            success_count: row.success_count,
            failure_count: row.failure_count,
            last_success: row
                .last_success
                .and_then(|s| s.parse::<DateTime<Utc>>().ok()),
            last_failure: row
                .last_failure
                .and_then(|s| s.parse::<DateTime<Utc>>().ok()),
            last_checked: row
                .last_checked
                .and_then(|s| s.parse::<DateTime<Utc>>().ok()),
            is_reliable: row.is_reliable,
            avg_latency_ms: row.avg_latency_ms,
            min_latency_ms: row.min_latency_ms,
            max_latency_ms: row.max_latency_ms,
            last_latency_ms: row.last_latency_ms,
        };

        NodeRecord::new(address, metadata, health)
    }
}