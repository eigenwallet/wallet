use std::path::PathBuf;

use anyhow::Result;
use dirs::data_dir;
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use tracing::{debug, info};

// TODO: This needs to be split up into multiple tables
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct MoneroNode {
    pub id: Option<i64>,
    pub scheme: String,   // http or https
    pub protocol: String, // ssl or tcp
    pub host: String,
    pub port: i64,
    pub full_url: String,
    pub network: String,  // mainnet or stagenet
    pub is_reachable: bool,
    pub success_count: i64,
    pub failure_count: i64,
    pub last_success: Option<String>, // ISO 8601 timestamp
    pub last_failure: Option<String>, // ISO 8601 timestamp
    pub last_checked: Option<String>, // ISO 8601 timestamp
    pub is_reliable: bool,            // Top 4 most reliable nodes
    pub avg_latency_ms: Option<f64>,  // Average latency in milliseconds
    pub min_latency_ms: Option<f64>,  // Minimum recorded latency
    pub max_latency_ms: Option<f64>,  // Maximum recorded latency
    pub last_latency_ms: Option<f64>, // Most recent latency measurement
}

impl MoneroNode {
    pub fn new(scheme: String, protocol: String, host: String, port: i64, network: String) -> Self {
        let full_url = format!("{}://{}:{}", scheme, host, port);
        Self {
            id: None,
            scheme,
            protocol,
            host,
            port,
            full_url,
            network,
            is_reachable: false,
            success_count: 0,
            failure_count: 0,
            last_success: None,
            last_failure: None,
            last_checked: None,
            is_reliable: false,
            avg_latency_ms: None,
            min_latency_ms: None,
            max_latency_ms: None,
            last_latency_ms: None,
        }
    }

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
        let request_weight = (total_requests as f64).min(100.0) / 100.0;
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

#[derive(Clone)]
pub struct Database {
    pub pool: SqlitePool,
}

impl Database {
    // TODO: Make this configurable for when using as a library
    pub async fn new() -> Result<Self> {
        let app_data_dir = get_app_data_dir()?;
        let db_path = app_data_dir.join("nodes.db");

        info!("Using database at: {}", db_path.display());

        let database_url = format!("sqlite:{}?mode=rwc", db_path.display());
        let pool = SqlitePool::connect(&database_url).await?;

        let db = Self { pool };
        db.migrate().await?;

        Ok(db)
    }

    // TODO: Use sqlx migrations
    async fn migrate(&self) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS monero_nodes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                scheme TEXT NOT NULL,
                protocol TEXT NOT NULL,
                host TEXT NOT NULL,
                port INTEGER NOT NULL,
                full_url TEXT NOT NULL UNIQUE,
                network TEXT NOT NULL,
                is_reachable BOOLEAN NOT NULL DEFAULT 0,
                success_count INTEGER NOT NULL DEFAULT 0,
                failure_count INTEGER NOT NULL DEFAULT 0,
                last_success TEXT,
                last_failure TEXT,
                last_checked TEXT,
                is_reliable BOOLEAN NOT NULL DEFAULT 0,
                avg_latency_ms REAL,
                min_latency_ms REAL,
                max_latency_ms REAL,
                last_latency_ms REAL,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create index on full_url for faster lookups
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_full_url ON monero_nodes(full_url)")
            .execute(&self.pool)
            .await?;

        // Create index on network for filtering
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_network ON monero_nodes(network)")
            .execute(&self.pool)
            .await?;

        // Create index on reliability metrics (include network for better performance)
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_reliability ON monero_nodes(network, is_reliable, is_reachable, success_count, avg_latency_ms)")
            .execute(&self.pool)
            .await?;

        info!("Database migration completed");
        Ok(())
    }

    pub async fn upsert_node(&self, node: &MoneroNode) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();

        sqlx::query(
            r#"
            INSERT INTO monero_nodes 
            (scheme, protocol, host, port, full_url, network, is_reachable, success_count, failure_count, 
             last_success, last_failure, last_checked, is_reliable, avg_latency_ms, min_latency_ms, 
             max_latency_ms, last_latency_ms, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(full_url) DO UPDATE SET
                scheme = excluded.scheme,
                protocol = excluded.protocol,
                host = excluded.host,
                port = excluded.port,
                network = excluded.network,
                is_reachable = excluded.is_reachable,
                success_count = excluded.success_count,
                failure_count = excluded.failure_count,
                last_success = excluded.last_success,
                last_failure = excluded.last_failure,
                last_checked = excluded.last_checked,
                is_reliable = excluded.is_reliable,
                avg_latency_ms = excluded.avg_latency_ms,
                min_latency_ms = excluded.min_latency_ms,
                max_latency_ms = excluded.max_latency_ms,
                last_latency_ms = excluded.last_latency_ms,
                updated_at = excluded.updated_at
            "#,
        )
        .bind(&node.scheme)
        .bind(&node.protocol)
        .bind(&node.host)
        .bind(node.port)
        .bind(&node.full_url)
        .bind(&node.network)
        .bind(node.is_reachable)
        .bind(node.success_count)
        .bind(node.failure_count)
        .bind(&node.last_success)
        .bind(&node.last_failure)
        .bind(&node.last_checked)
        .bind(node.is_reliable)
        .bind(node.avg_latency_ms)
        .bind(node.min_latency_ms)
        .bind(node.max_latency_ms)
        .bind(node.last_latency_ms)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        debug!("Upserted node: {}", node.full_url);
        Ok(())
    }

    pub async fn update_node_success(&self, url: &str, latency_ms: f64) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();

        // First get current stats to calculate new average
        let current_node =
            sqlx::query_as::<_, MoneroNode>("SELECT * FROM monero_nodes WHERE full_url = ?")
                .bind(url)
                .fetch_optional(&self.pool)
                .await?;

        let (new_avg, new_min, new_max) = if let Some(node) = current_node {
            let new_avg = if node.success_count == 0 {
                latency_ms
            } else {
                (node.avg_latency_ms.unwrap_or(latency_ms) * node.success_count as f64 + latency_ms)
                    / (node.success_count + 1) as f64
            };

            let new_min = Some(
                node.min_latency_ms
                    .map_or(latency_ms, |min| min.min(latency_ms)),
            );
            let new_max = Some(
                node.max_latency_ms
                    .map_or(latency_ms, |max| max.max(latency_ms)),
            );

            (Some(new_avg), new_min, new_max)
        } else {
            (Some(latency_ms), Some(latency_ms), Some(latency_ms))
        };

        sqlx::query(
            r#"
            UPDATE monero_nodes 
            SET success_count = success_count + 1,
                last_success = ?,
                last_checked = ?,
                is_reachable = 1,
                avg_latency_ms = ?,
                min_latency_ms = ?,
                max_latency_ms = ?,
                last_latency_ms = ?,
                updated_at = ?
            WHERE full_url = ?
            "#,
        )
        .bind(&now)
        .bind(&now)
        .bind(new_avg)
        .bind(new_min)
        .bind(new_max)
        .bind(latency_ms)
        .bind(&now)
        .bind(url)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn update_node_failure(&self, url: &str) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();

        sqlx::query(
            r#"
            UPDATE monero_nodes 
            SET failure_count = failure_count + 1,
                last_failure = ?,
                last_checked = ?,
                is_reachable = 0,
                updated_at = ?
            WHERE full_url = ?
            "#,
        )
        .bind(&now)
        .bind(&now)
        .bind(&now)
        .bind(url)
        .execute(&self.pool)
        .await?;

        tracing::trace!("Updated failure for node: {}", url);
        Ok(())
    }

    pub async fn get_reliable_nodes(&self, network: &str) -> Result<Vec<MoneroNode>> {
        let nodes = sqlx::query_as::<_, MoneroNode>(
            r#"
            SELECT * FROM monero_nodes 
            WHERE is_reliable = 1 AND is_reachable = 1 AND network = ?
            ORDER BY avg_latency_ms ASC, success_count DESC
            "#,
        )
        .bind(network)
        .fetch_all(&self.pool)
        .await?;

        debug!("Retrieved {} reliable nodes for network {}", nodes.len(), network);
        Ok(nodes)
    }

    pub async fn get_random_nodes(&self, limit: i64, network: &str) -> Result<Vec<MoneroNode>> {
        let nodes = sqlx::query_as::<_, MoneroNode>(
            r#"
            SELECT * FROM monero_nodes 
            WHERE is_reachable = 1 AND is_reliable = 0 AND network = ?
            ORDER BY RANDOM() 
            LIMIT ?
            "#,
        )
        .bind(network)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        debug!("Retrieved {} random nodes for network {}", nodes.len(), network);
        Ok(nodes)
    }

    pub async fn get_all_reachable_nodes(&self, network: &str) -> Result<Vec<MoneroNode>> {
        let nodes = sqlx::query_as::<_, MoneroNode>(
            r#"
            SELECT * FROM monero_nodes 
            WHERE is_reachable = 1 AND network = ?
            ORDER BY avg_latency_ms ASC, success_count DESC
            "#,
        )
        .bind(network)
        .fetch_all(&self.pool)
        .await?;

        debug!("Retrieved {} reachable nodes for network {}", nodes.len(), network);
        Ok(nodes)
    }

    pub async fn update_reliable_nodes(&self, network: &str) -> Result<()> {
        // First, mark all nodes as not reliable for this network
        sqlx::query("UPDATE monero_nodes SET is_reliable = 0 WHERE network = ?")
            .bind(network)
            .execute(&self.pool)
            .await?;

        // Then mark the top 4 nodes with highest reliability scores as reliable for this network
        sqlx::query(
            r#"
            UPDATE monero_nodes 
            SET is_reliable = 1 
            WHERE id IN (
                SELECT id FROM monero_nodes 
                WHERE is_reachable = 1 AND success_count > 0 AND network = ?
                ORDER BY 
                    (CAST(success_count AS REAL) / CAST(success_count + failure_count AS REAL)) * 
                    (MIN(success_count + failure_count, 100) / 100.0) * 0.8 +
                    CASE 
                        WHEN avg_latency_ms IS NOT NULL THEN (1.0 - (MIN(avg_latency_ms, 2000) / 2000.0)) * 0.2
                        ELSE 0.0 
                    END DESC
                LIMIT 4
            )
            "#,
        )
        .bind(network)
        .execute(&self.pool)
        .await?;

        let reliable_count = sqlx::query("SELECT COUNT(*) FROM monero_nodes WHERE is_reliable = 1 AND network = ?")
            .bind(network)
            .fetch_one(&self.pool)
            .await?
            .get::<i64, _>(0);

        info!(
            "Updated reliable nodes pool for network {}: {} nodes marked as reliable",
            network, reliable_count
        );
        Ok(())
    }

    pub async fn get_node_stats(&self, network: &str) -> Result<(i64, i64, i64)> {
        let row = sqlx::query(
            r#"
            SELECT 
                COUNT(*) as total,
                SUM(CASE WHEN is_reachable = 1 THEN 1 ELSE 0 END) as reachable,
                SUM(CASE WHEN is_reliable = 1 THEN 1 ELSE 0 END) as reliable
            FROM monero_nodes
            WHERE network = ?
            "#,
        )
        .bind(network)
        .fetch_one(&self.pool)
        .await?;

        let total = row.get::<i64, _>("total");
        let reachable = row.get::<i64, _>("reachable");
        let reliable = row.get::<i64, _>("reliable");

        Ok((total, reachable, reliable))
    }
}

pub fn get_app_data_dir() -> Result<PathBuf> {
    let base_dir =
        data_dir().ok_or_else(|| anyhow::anyhow!("Could not determine system data directory"))?;

    let app_dir = base_dir.join("monero-rpc-pool");

    if !app_dir.exists() {
        std::fs::create_dir_all(&app_dir)?;
        info!("Created application data directory: {}", app_dir.display());
    }

    Ok(app_dir)
}
