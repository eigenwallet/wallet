use std::path::PathBuf;

use anyhow::Result;
use dirs::data_dir;
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use tracing::{debug, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct MoneroNode {
    pub id: Option<i64>,
    pub scheme: String, // http or https
    pub host: String,
    pub port: i64,
    pub full_url: String,
    pub network: Option<String>, // mainnet, stagenet, or NULL if unidentified
    pub first_seen_at: String,   // ISO 8601 timestamp when first discovered
    // Computed fields from health_checks (not stored in monero_nodes table)
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

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct HealthCheck {
    pub id: Option<i64>,
    pub node_id: i64,
    pub timestamp: String, // ISO 8601 timestamp
    pub was_successful: bool,
    pub latency_ms: Option<f64>,
}

impl MoneroNode {
    pub fn new(scheme: String, host: String, port: i64) -> Self {
        let full_url = format!("{}://{}:{}", scheme, host, port);
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: None,
            scheme,
            host,
            port,
            full_url,
            network: None, // Unidentified initially
            first_seen_at: now,
            // These are computed from health_checks
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

#[derive(Clone)]
pub struct Database {
    pub pool: SqlitePool,
}

impl Database {
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

    async fn migrate(&self) -> Result<()> {
        // Create monero_nodes table - stores node identity and current state
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS monero_nodes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                scheme TEXT NOT NULL,
                host TEXT NOT NULL,
                port INTEGER NOT NULL,
                full_url TEXT NOT NULL UNIQUE,
                network TEXT,  -- NULL if unidentified, mainnet/stagenet/testnet when identified
                first_seen_at TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create health_checks table - stores raw event data
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS health_checks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                node_id INTEGER NOT NULL,
                timestamp TEXT NOT NULL,
                was_successful BOOLEAN NOT NULL,
                latency_ms REAL,
                FOREIGN KEY (node_id) REFERENCES monero_nodes(id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create indexes for performance
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_nodes_full_url ON monero_nodes(full_url)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_nodes_network ON monero_nodes(network)")
            .execute(&self.pool)
            .await?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_health_checks_node_id ON health_checks(node_id)",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_health_checks_timestamp ON health_checks(timestamp)",
        )
        .execute(&self.pool)
        .await?;

        info!("Database migration completed");
        Ok(())
    }

    /// Insert a node if it doesn't exist, return the node_id
    pub async fn upsert_node(&self, scheme: &str, host: &str, port: i64) -> Result<i64> {
        let full_url = format!("{}://{}:{}", scheme, host, port);
        let now = chrono::Utc::now().to_rfc3339();

        let result = sqlx::query(
            r#"
            INSERT INTO monero_nodes (scheme, host, port, full_url, first_seen_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?)
            ON CONFLICT(full_url) DO UPDATE SET
                updated_at = excluded.updated_at
            RETURNING id
            "#,
        )
        .bind(scheme)
        .bind(host)
        .bind(port)
        .bind(&full_url)
        .bind(&now)
        .bind(&now)
        .fetch_one(&self.pool)
        .await?;

        let node_id = result.get::<i64, _>("id");
        debug!("Upserted node: {} (id: {})", full_url, node_id);
        Ok(node_id)
    }

    /// Update a node's network after it has been identified
    pub async fn update_node_network(&self, url: &str, network: &str) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();

        let rows_affected = sqlx::query(
            r#"
            UPDATE monero_nodes 
            SET network = ?, updated_at = ?
            WHERE full_url = ?
            "#,
        )
        .bind(network)
        .bind(&now)
        .bind(url)
        .execute(&self.pool)
        .await?
        .rows_affected();

        if rows_affected > 0 {
            debug!("Updated network for node {} to {}", url, network);
        } else {
            warn!("Failed to update network for node {}: not found", url);
        }

        Ok(())
    }

    /// Record a health check event
    pub async fn record_health_check(
        &self,
        url: &str,
        was_successful: bool,
        latency_ms: Option<f64>,
    ) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();

        // First get the node_id
        let node_row = sqlx::query("SELECT id FROM monero_nodes WHERE full_url = ?")
            .bind(url)
            .fetch_optional(&self.pool)
            .await?;

        let node_id = match node_row {
            Some(row) => row.get::<i64, _>("id"),
            None => {
                warn!("Cannot record health check for unknown node: {}", url);
                return Ok(());
            }
        };

        sqlx::query(
            r#"
            INSERT INTO health_checks (node_id, timestamp, was_successful, latency_ms)
            VALUES (?, ?, ?, ?)
            "#,
        )
        .bind(node_id)
        .bind(&now)
        .bind(was_successful)
        .bind(latency_ms)
        .execute(&self.pool)
        .await?;

        debug!(
            "Recorded health check for node {} (id: {}): success={}, latency={:?}ms",
            url, node_id, was_successful, latency_ms
        );
        Ok(())
    }

    /// Get nodes that have been identified (have network set)
    pub async fn get_identified_nodes(&self, network: &str) -> Result<Vec<MoneroNode>> {
        let nodes = sqlx::query_as::<_, MoneroNode>(
            r#"
            SELECT 
                n.*,
                COALESCE(stats.success_count, 0) as success_count,
                COALESCE(stats.failure_count, 0) as failure_count,
                stats.last_success,
                stats.last_failure,
                stats.last_checked,
                CASE WHEN reliable_nodes.node_id IS NOT NULL THEN 1 ELSE 0 END as is_reliable,
                stats.avg_latency_ms,
                stats.min_latency_ms,
                stats.max_latency_ms,
                stats.last_latency_ms
            FROM monero_nodes n
            LEFT JOIN (
                SELECT 
                    node_id,
                    SUM(CASE WHEN was_successful THEN 1 ELSE 0 END) as success_count,
                    SUM(CASE WHEN NOT was_successful THEN 1 ELSE 0 END) as failure_count,
                    MAX(CASE WHEN was_successful THEN timestamp END) as last_success,
                    MAX(CASE WHEN NOT was_successful THEN timestamp END) as last_failure,
                    MAX(timestamp) as last_checked,
                    AVG(CASE WHEN was_successful AND latency_ms IS NOT NULL THEN latency_ms END) as avg_latency_ms,
                    MIN(CASE WHEN was_successful AND latency_ms IS NOT NULL THEN latency_ms END) as min_latency_ms,
                    MAX(CASE WHEN was_successful AND latency_ms IS NOT NULL THEN latency_ms END) as max_latency_ms,
                    (SELECT latency_ms FROM health_checks hc2 WHERE hc2.node_id = health_checks.node_id ORDER BY timestamp DESC LIMIT 1) as last_latency_ms
                FROM health_checks 
                GROUP BY node_id
            ) stats ON n.id = stats.node_id
            LEFT JOIN (
                SELECT DISTINCT node_id FROM (
                    SELECT 
                        n2.id as node_id,
                        COALESCE(s2.success_count, 0) as success_count,
                        COALESCE(s2.failure_count, 0) as failure_count,
                        s2.avg_latency_ms,
                        (CAST(COALESCE(s2.success_count, 0) AS REAL) / CAST(COALESCE(s2.success_count, 0) + COALESCE(s2.failure_count, 0) AS REAL)) * 
                        (MIN(COALESCE(s2.success_count, 0) + COALESCE(s2.failure_count, 0), 200) / 200.0) * 0.8 +
                        CASE 
                            WHEN s2.avg_latency_ms IS NOT NULL THEN (1.0 - (MIN(s2.avg_latency_ms, 2000) / 2000.0)) * 0.2
                            ELSE 0.0 
                        END as reliability_score
                    FROM monero_nodes n2
                    LEFT JOIN (
                        SELECT 
                            node_id,
                            SUM(CASE WHEN was_successful THEN 1 ELSE 0 END) as success_count,
                            SUM(CASE WHEN NOT was_successful THEN 1 ELSE 0 END) as failure_count,
                            AVG(CASE WHEN was_successful AND latency_ms IS NOT NULL THEN latency_ms END) as avg_latency_ms
                        FROM health_checks 
                        GROUP BY node_id
                    ) s2 ON n2.id = s2.node_id
                    WHERE n2.network = ? AND (COALESCE(s2.success_count, 0) + COALESCE(s2.failure_count, 0)) > 0
                    ORDER BY reliability_score DESC
                    LIMIT 4
                )
            ) reliable_nodes ON n.id = reliable_nodes.node_id
            WHERE n.network = ?
            ORDER BY stats.avg_latency_ms ASC, stats.success_count DESC
            "#,
        )
        .bind(network)
        .bind(network)
        .fetch_all(&self.pool)
        .await?;

        debug!(
            "Retrieved {} identified nodes for network {}",
            nodes.len(),
            network
        );
        Ok(nodes)
    }

    /// Get reliable nodes (top 4 by reliability score)
    pub async fn get_reliable_nodes(&self, network: &str) -> Result<Vec<MoneroNode>> {
        let nodes = sqlx::query_as::<_, MoneroNode>(
            r#"
            SELECT 
                n.*,
                COALESCE(stats.success_count, 0) as success_count,
                COALESCE(stats.failure_count, 0) as failure_count,
                stats.last_success,
                stats.last_failure,
                stats.last_checked,
                1 as is_reliable,
                stats.avg_latency_ms,
                stats.min_latency_ms,
                stats.max_latency_ms,
                stats.last_latency_ms
            FROM monero_nodes n
            LEFT JOIN (
                SELECT 
                    node_id,
                    SUM(CASE WHEN was_successful THEN 1 ELSE 0 END) as success_count,
                    SUM(CASE WHEN NOT was_successful THEN 1 ELSE 0 END) as failure_count,
                    MAX(CASE WHEN was_successful THEN timestamp END) as last_success,
                    MAX(CASE WHEN NOT was_successful THEN timestamp END) as last_failure,
                    MAX(timestamp) as last_checked,
                    AVG(CASE WHEN was_successful AND latency_ms IS NOT NULL THEN latency_ms END) as avg_latency_ms,
                    MIN(CASE WHEN was_successful AND latency_ms IS NOT NULL THEN latency_ms END) as min_latency_ms,
                    MAX(CASE WHEN was_successful AND latency_ms IS NOT NULL THEN latency_ms END) as max_latency_ms,
                    (SELECT latency_ms FROM health_checks hc2 WHERE hc2.node_id = health_checks.node_id ORDER BY timestamp DESC LIMIT 1) as last_latency_ms
                FROM health_checks 
                GROUP BY node_id
            ) stats ON n.id = stats.node_id
            WHERE n.network = ? AND (COALESCE(stats.success_count, 0) + COALESCE(stats.failure_count, 0)) > 0
            ORDER BY 
                (CAST(COALESCE(stats.success_count, 0) AS REAL) / CAST(COALESCE(stats.success_count, 0) + COALESCE(stats.failure_count, 0) AS REAL)) * 
                (MIN(COALESCE(stats.success_count, 0) + COALESCE(stats.failure_count, 0), 200) / 200.0) * 0.8 +
                CASE 
                    WHEN stats.avg_latency_ms IS NOT NULL THEN (1.0 - (MIN(stats.avg_latency_ms, 2000) / 2000.0)) * 0.2
                    ELSE 0.0 
                END DESC
            LIMIT 4
            "#,
        )
        .bind(network)
        .fetch_all(&self.pool)
        .await?;

        debug!(
            "Retrieved {} reliable nodes for network {}",
            nodes.len(),
            network
        );
        Ok(nodes)
    }

    /// Get node statistics for a network
    pub async fn get_node_stats(&self, network: &str) -> Result<(i64, i64, i64)> {
        let row = sqlx::query(
            r#"
            SELECT 
                COUNT(*) as total,
                SUM(CASE WHEN stats.success_count > 0 THEN 1 ELSE 0 END) as reachable,
                (SELECT COUNT(*) FROM (
                    SELECT n2.id
                    FROM monero_nodes n2
                    LEFT JOIN (
                        SELECT 
                            node_id,
                            SUM(CASE WHEN was_successful THEN 1 ELSE 0 END) as success_count,
                            SUM(CASE WHEN NOT was_successful THEN 1 ELSE 0 END) as failure_count,
                            AVG(CASE WHEN was_successful AND latency_ms IS NOT NULL THEN latency_ms END) as avg_latency_ms
                        FROM health_checks 
                        GROUP BY node_id
                    ) s2 ON n2.id = s2.node_id
                    WHERE n2.network = ? AND (COALESCE(s2.success_count, 0) + COALESCE(s2.failure_count, 0)) > 0
                    ORDER BY 
                        (CAST(COALESCE(s2.success_count, 0) AS REAL) / CAST(COALESCE(s2.success_count, 0) + COALESCE(s2.failure_count, 0) AS REAL)) * 
                        (MIN(COALESCE(s2.success_count, 0) + COALESCE(s2.failure_count, 0), 200) / 200.0) * 0.8 +
                        CASE 
                            WHEN s2.avg_latency_ms IS NOT NULL THEN (1.0 - (MIN(s2.avg_latency_ms, 2000) / 2000.0)) * 0.2
                            ELSE 0.0 
                        END DESC
                    LIMIT 4
                )) as reliable
            FROM monero_nodes n
            LEFT JOIN (
                SELECT 
                    node_id,
                    SUM(CASE WHEN was_successful THEN 1 ELSE 0 END) as success_count
                FROM health_checks 
                GROUP BY node_id
            ) stats ON n.id = stats.node_id
            WHERE n.network = ?
            "#,
        )
        .bind(network)
        .bind(network)
        .fetch_one(&self.pool)
        .await?;

        let total = row.get::<i64, _>("total");
        let reachable = row.get::<i64, _>("reachable");
        let reliable = row.get::<i64, _>("reliable");

        Ok((total, reachable, reliable))
    }

    /// Get health check statistics
    pub async fn get_health_check_stats(&self, network: &str) -> Result<(u64, u64)> {
        let row = sqlx::query(
            r#"
            SELECT 
                SUM(CASE WHEN hc.was_successful THEN 1 ELSE 0 END) as successful,
                SUM(CASE WHEN NOT hc.was_successful THEN 1 ELSE 0 END) as unsuccessful
            FROM health_checks hc
            JOIN monero_nodes n ON hc.node_id = n.id
            WHERE n.network = ?
            "#,
        )
        .bind(network)
        .fetch_one(&self.pool)
        .await?;

        let successful = row.get::<Option<i64>, _>("successful").unwrap_or(0) as u64;
        let unsuccessful = row.get::<Option<i64>, _>("unsuccessful").unwrap_or(0) as u64;

        Ok((successful, unsuccessful))
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
