use std::path::PathBuf;

use anyhow::Result;
use dirs::data_dir;
use sqlx::SqlitePool;
use tracing::{debug, info, warn};
use crate::types::{NodeAddress, NodeHealthStats, NodeMetadata, NodeRecord};

#[derive(Clone)]
pub struct Database {
    pub pool: SqlitePool,
}

impl Database {
    pub async fn new() -> Result<Self> {
        let app_data_dir = get_app_data_dir()?;
        Self::new_with_data_dir(app_data_dir).await
    }

    pub async fn new_with_data_dir(data_dir: PathBuf) -> Result<Self> {
        if !data_dir.exists() {
            std::fs::create_dir_all(&data_dir)?;
            info!("Created application data directory: {}", data_dir.display());
        }

        let db_path = data_dir.join("nodes.db");

        info!("Using database at {}", db_path.display());

        let database_url = format!("sqlite:{}?mode=rwc", db_path.display());
        let pool = SqlitePool::connect(&database_url).await?;

        let db = Self { pool };
        db.migrate().await?;

        Ok(db)
    }

    async fn migrate(&self) -> Result<()> {
        // Run sqlx migrations
        sqlx::migrate!("./migrations").run(&self.pool).await?;

        info!("Database migration completed");
        Ok(())
    }

    /// Record a health check event
    pub async fn record_health_check(
        &self,
        scheme: &str,
        host: &str,
        port: i64,
        was_successful: bool,
        latency_ms: Option<f64>,
    ) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();

        let result = sqlx::query!(
            r#"
            INSERT INTO health_checks (node_id, timestamp, was_successful, latency_ms)
            SELECT id, ?, ?, ?
            FROM monero_nodes 
            WHERE scheme = ? AND host = ? AND port = ?
            "#,
            now,
            was_successful,
            latency_ms,
            scheme,
            host,
            port
        )
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            warn!(
                "Cannot record health check for unknown node: {}://{}:{}",
                scheme, host, port
            );
        }

        Ok(())
    }

    /// Get nodes that have been identified (have network set)
    pub async fn get_identified_nodes(&self, network: &str) -> Result<Vec<NodeRecord>> {
        let rows = sqlx::query!(
            r#"
            SELECT 
                n.id as "id!: i64",
                n.scheme,
                n.host,
                n.port,
                n.network,
                n.first_seen_at,
                CAST(COALESCE(stats.success_count, 0) AS INTEGER) as "success_count!: i64",
                CAST(COALESCE(stats.failure_count, 0) AS INTEGER) as "failure_count!: i64",
                stats.last_success as "last_success?: String",
                stats.last_failure as "last_failure?: String",
                stats.last_checked as "last_checked?: String",
                CAST(CASE WHEN reliable_nodes.node_id IS NOT NULL THEN 1 ELSE 0 END AS INTEGER) as "is_reliable!: i64",
                stats.avg_latency_ms as "avg_latency_ms?: f64",
                stats.min_latency_ms as "min_latency_ms?: f64",
                stats.max_latency_ms as "max_latency_ms?: f64",
                stats.last_latency_ms as "last_latency_ms?: f64"
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
            network,
            network
        )
        .fetch_all(&self.pool)
        .await?;

        let nodes: Vec<NodeRecord> = rows
            .into_iter()
            .map(|row| {
                let address = NodeAddress::new(row.scheme, row.host, row.port as u16);
                let first_seen_at = row.first_seen_at.parse().unwrap_or_else(|_| chrono::Utc::now());
                let metadata = NodeMetadata::new(row.id, row.network, first_seen_at);
                let health = NodeHealthStats {
                    success_count: row.success_count,
                    failure_count: row.failure_count,
                    last_success: row.last_success.and_then(|s| s.parse().ok()),
                    last_failure: row.last_failure.and_then(|s| s.parse().ok()),
                    last_checked: row.last_checked.and_then(|s| s.parse().ok()),
                    is_reliable: row.is_reliable != 0,
                    avg_latency_ms: row.avg_latency_ms,
                    min_latency_ms: row.min_latency_ms,
                    max_latency_ms: row.max_latency_ms,
                    last_latency_ms: row.last_latency_ms,
                };
                NodeRecord::new(address, metadata, health)
            })
            .collect();

        debug!(
            "Retrieved {} identified nodes for network {}",
            nodes.len(),
            network
        );
        Ok(nodes)
    }

    /// Get reliable nodes (top 4 by reliability score)
    pub async fn get_reliable_nodes(&self, network: &str) -> Result<Vec<NodeRecord>> {
        let rows = sqlx::query!(
            r#"
            SELECT 
                n.id as "id!: i64",
                n.scheme,
                n.host,
                n.port,
                n.network,
                n.first_seen_at,
                CAST(COALESCE(stats.success_count, 0) AS INTEGER) as "success_count!: i64",
                CAST(COALESCE(stats.failure_count, 0) AS INTEGER) as "failure_count!: i64",
                stats.last_success as "last_success?: String",
                stats.last_failure as "last_failure?: String",
                stats.last_checked as "last_checked?: String",
                CAST(1 AS INTEGER) as "is_reliable!: i64",
                stats.avg_latency_ms as "avg_latency_ms?: f64",
                stats.min_latency_ms as "min_latency_ms?: f64",
                stats.max_latency_ms as "max_latency_ms?: f64",
                stats.last_latency_ms as "last_latency_ms?: f64"
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
            network
        )
        .fetch_all(&self.pool)
        .await?;

        let nodes: Vec<NodeRecord> = rows
            .into_iter()
            .map(|row| {
                let address = NodeAddress::new(row.scheme, row.host, row.port as u16);
                let first_seen_at = row.first_seen_at.parse().unwrap_or_else(|_| chrono::Utc::now());
                let metadata = NodeMetadata::new(row.id, row.network, first_seen_at);
                let health = NodeHealthStats {
                    success_count: row.success_count,
                    failure_count: row.failure_count,
                    last_success: row.last_success.and_then(|s| s.parse().ok()),
                    last_failure: row.last_failure.and_then(|s| s.parse().ok()),
                    last_checked: row.last_checked.and_then(|s| s.parse().ok()),
                    is_reliable: true, // For reliable nodes, we explicitly set is_reliable to true
                    avg_latency_ms: row.avg_latency_ms,
                    min_latency_ms: row.min_latency_ms,
                    max_latency_ms: row.max_latency_ms,
                    last_latency_ms: row.last_latency_ms,
                };
                NodeRecord::new(address, metadata, health)
            })
            .collect();

        Ok(nodes)
    }

    /// Get node statistics for a network
    pub async fn get_node_stats(&self, network: &str) -> Result<(i64, i64, i64)> {
        let row = sqlx::query!(
            r#"
            SELECT 
                COUNT(*) as total,
                CAST(SUM(CASE WHEN stats.success_count > 0 THEN 1 ELSE 0 END) AS INTEGER) as "reachable!: i64",
                CAST((SELECT COUNT(*) FROM (
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
                )) AS INTEGER) as "reliable!: i64"
            FROM monero_nodes n
            LEFT JOIN (
                SELECT 
                    node_id,
                    SUM(CASE WHEN was_successful THEN 1 ELSE 0 END) as success_count,
                    SUM(CASE WHEN NOT was_successful THEN 1 ELSE 0 END) as failure_count
                FROM health_checks 
                GROUP BY node_id
            ) stats ON n.id = stats.node_id
            WHERE n.network = ?
            "#,
            network,
            network
        )
        .fetch_one(&self.pool)
        .await?;

        let total = row.total;
        let reachable = row.reachable;
        let reliable = row.reliable;

        Ok((total, reachable, reliable))
    }

    /// Get health check statistics for a network
    pub async fn get_health_check_stats(&self, network: &str) -> Result<(u64, u64)> {
        let row = sqlx::query!(
            r#"
            SELECT 
                CAST(SUM(CASE WHEN hc.was_successful THEN 1 ELSE 0 END) AS INTEGER) as "successful!: i64",
                CAST(SUM(CASE WHEN NOT hc.was_successful THEN 1 ELSE 0 END) AS INTEGER) as "unsuccessful!: i64"
            FROM (
                SELECT hc.was_successful
                FROM health_checks hc
                JOIN monero_nodes n ON hc.node_id = n.id
                WHERE n.network = ?
                ORDER BY hc.timestamp DESC
                LIMIT 100
            ) hc
            "#,
            network
        )
        .fetch_one(&self.pool)
        .await?;

        let successful = row.successful as u64;
        let unsuccessful = row.unsuccessful as u64;

        Ok((successful, unsuccessful))
    }

    /// Get top nodes based on recent success rate and latency
    pub async fn get_top_nodes_by_recent_success(
        &self,
        network: &str,
        _recent_checks_limit: i64,
        limit: i64,
    ) -> Result<Vec<NodeRecord>> {
        let rows = sqlx::query!(
            r#"
            SELECT 
                n.id as "id!: i64",
                n.scheme,
                n.host,
                n.port,
                n.network,
                n.first_seen_at,
                CAST(COALESCE(stats.success_count, 0) AS INTEGER) as "success_count!: i64",
                CAST(COALESCE(stats.failure_count, 0) AS INTEGER) as "failure_count!: i64",
                stats.last_success as "last_success?: String",
                stats.last_failure as "last_failure?: String",
                stats.last_checked as "last_checked?: String",
                CAST(CASE WHEN reliable_nodes.node_id IS NOT NULL THEN 1 ELSE 0 END AS INTEGER) as "is_reliable!: i64",
                stats.avg_latency_ms as "avg_latency_ms?: f64",
                stats.min_latency_ms as "min_latency_ms?: f64",
                stats.max_latency_ms as "max_latency_ms?: f64",
                stats.last_latency_ms as "last_latency_ms?: f64"
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
            WHERE n.network = ? AND (COALESCE(stats.success_count, 0) + COALESCE(stats.failure_count, 0)) > 0
            ORDER BY 
                (CAST(COALESCE(stats.success_count, 0) AS REAL) / CAST(COALESCE(stats.success_count, 0) + COALESCE(stats.failure_count, 0) AS REAL)) DESC,
                stats.avg_latency_ms ASC
            LIMIT ?
            "#,
            network,
            network,
            limit
        )
        .fetch_all(&self.pool)
        .await?;

        let nodes: Vec<NodeRecord> = rows
            .into_iter()
            .map(|row| {
                let address = NodeAddress::new(row.scheme, row.host, row.port as u16);
                let first_seen_at = row.first_seen_at.parse().unwrap_or_else(|_| chrono::Utc::now());
                let metadata = NodeMetadata::new(row.id, row.network, first_seen_at);
                let health = NodeHealthStats {
                    success_count: row.success_count,
                    failure_count: row.failure_count,
                    last_success: row.last_success.and_then(|s| s.parse().ok()),
                    last_failure: row.last_failure.and_then(|s| s.parse().ok()),
                    last_checked: row.last_checked.and_then(|s| s.parse().ok()),
                    is_reliable: row.is_reliable != 0,
                    avg_latency_ms: row.avg_latency_ms,
                    min_latency_ms: row.min_latency_ms,
                    max_latency_ms: row.max_latency_ms,
                    last_latency_ms: row.last_latency_ms,
                };
                NodeRecord::new(address, metadata, health)
            })
            .collect();

        Ok(nodes)
    }

    /// Get identified nodes that have at least one successful health check
    pub async fn get_identified_nodes_with_success(
        &self,
        network: &str,
    ) -> Result<Vec<NodeRecord>> {
        let rows = sqlx::query!(
            r#"
            SELECT 
                n.id as "id!: i64",
                n.scheme,
                n.host,
                n.port,
                n.network,
                n.first_seen_at,
                CAST(COALESCE(stats.success_count, 0) AS INTEGER) as "success_count!: i64",
                CAST(COALESCE(stats.failure_count, 0) AS INTEGER) as "failure_count!: i64",
                stats.last_success as "last_success?: String",
                stats.last_failure as "last_failure?: String",
                stats.last_checked as "last_checked?: String",
                CAST(CASE WHEN reliable_nodes.node_id IS NOT NULL THEN 1 ELSE 0 END AS INTEGER) as "is_reliable!: i64",
                stats.avg_latency_ms as "avg_latency_ms?: f64",
                stats.min_latency_ms as "min_latency_ms?: f64",
                stats.max_latency_ms as "max_latency_ms?: f64",
                stats.last_latency_ms as "last_latency_ms?: f64"
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
            WHERE n.network = ? AND stats.success_count > 0
            ORDER BY stats.avg_latency_ms ASC, stats.success_count DESC
            "#,
            network,
            network
        )
        .fetch_all(&self.pool)
        .await?;

        let nodes: Vec<NodeRecord> = rows
            .into_iter()
            .map(|row| {
                let address = NodeAddress::new(row.scheme, row.host, row.port as u16);
                let first_seen_at = row.first_seen_at.parse().unwrap_or_else(|_| chrono::Utc::now());
                let metadata = NodeMetadata::new(row.id, row.network, first_seen_at);
                let health = NodeHealthStats {
                    success_count: row.success_count,
                    failure_count: row.failure_count,
                    last_success: row.last_success.and_then(|s| s.parse().ok()),
                    last_failure: row.last_failure.and_then(|s| s.parse().ok()),
                    last_checked: row.last_checked.and_then(|s| s.parse().ok()),
                    is_reliable: row.is_reliable != 0,
                    avg_latency_ms: row.avg_latency_ms,
                    min_latency_ms: row.min_latency_ms,
                    max_latency_ms: row.max_latency_ms,
                    last_latency_ms: row.last_latency_ms,
                };
                NodeRecord::new(address, metadata, health)
            })
            .collect();

        debug!(
            "Retrieved {} identified nodes with success for network {}",
            nodes.len(),
            network
        );
        Ok(nodes)
    }

    /// Get random nodes for the specified network, excluding specific IDs
    pub async fn get_random_nodes(
        &self,
        network: &str,
        limit: i64,
        exclude_ids: &[i64],
    ) -> Result<Vec<NodeRecord>> {
        if exclude_ids.is_empty() {
            let rows = sqlx::query!(
                r#"
                SELECT 
                    n.id as "id!: i64",
                    n.scheme,
                    n.host,
                    n.port,
                    n.network,
                    n.first_seen_at,
                    CAST(COALESCE(stats.success_count, 0) AS INTEGER) as "success_count!: i64",
                    CAST(COALESCE(stats.failure_count, 0) AS INTEGER) as "failure_count!: i64",
                    stats.last_success as "last_success?: String",
                    stats.last_failure as "last_failure?: String",
                    stats.last_checked as "last_checked?: String",
                    CAST(CASE WHEN reliable_nodes.node_id IS NOT NULL THEN 1 ELSE 0 END AS INTEGER) as "is_reliable!: i64",
                    stats.avg_latency_ms as "avg_latency_ms?: f64",
                    stats.min_latency_ms as "min_latency_ms?: f64",
                    stats.max_latency_ms as "max_latency_ms?: f64",
                    stats.last_latency_ms as "last_latency_ms?: f64"
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
                ORDER BY RANDOM()
                LIMIT ?
                "#,
                network,
                network,
                limit
            )
            .fetch_all(&self.pool)
            .await?;

            return Ok(rows
                .into_iter()
                .map(|row| {
                    let address = NodeAddress::new(row.scheme, row.host, row.port as u16);
                    let first_seen_at = row.first_seen_at.parse().unwrap_or_else(|_| chrono::Utc::now());
                    let metadata = NodeMetadata::new(row.id, row.network, first_seen_at);
                    let health = NodeHealthStats {
                        success_count: row.success_count,
                        failure_count: row.failure_count,
                        last_success: row.last_success.and_then(|s| s.parse().ok()),
                        last_failure: row.last_failure.and_then(|s| s.parse().ok()),
                        last_checked: row.last_checked.and_then(|s| s.parse().ok()),
                        is_reliable: row.is_reliable != 0,
                        avg_latency_ms: row.avg_latency_ms,
                        min_latency_ms: row.min_latency_ms,
                        max_latency_ms: row.max_latency_ms,
                        last_latency_ms: row.last_latency_ms,
                    };
                    NodeRecord::new(address, metadata, health)
                })
                .collect());
        }

        // If exclude_ids is not empty, we need to handle it differently
        // For now, get all nodes and filter in Rust (can be optimized with dynamic SQL)
        let fetch_limit = limit + exclude_ids.len() as i64 + 10; // Get extra to account for exclusions
        let all_rows = sqlx::query!(
            r#"
            SELECT 
                n.id as "id!: i64",
                n.scheme,
                n.host,
                n.port,
                n.network,
                n.first_seen_at,
                CAST(COALESCE(stats.success_count, 0) AS INTEGER) as "success_count!: i64",
                CAST(COALESCE(stats.failure_count, 0) AS INTEGER) as "failure_count!: i64",
                stats.last_success as "last_success?: String",
                stats.last_failure as "last_failure?: String",
                stats.last_checked as "last_checked?: String",
                CAST(CASE WHEN reliable_nodes.node_id IS NOT NULL THEN 1 ELSE 0 END AS INTEGER) as "is_reliable!: i64",
                stats.avg_latency_ms as "avg_latency_ms?: f64",
                stats.min_latency_ms as "min_latency_ms?: f64",
                stats.max_latency_ms as "max_latency_ms?: f64",
                stats.last_latency_ms as "last_latency_ms?: f64"
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
            ORDER BY RANDOM()
            LIMIT ?
            "#,
            network,
            network,
            fetch_limit
        )
        .fetch_all(&self.pool)
        .await?;

        // Convert exclude_ids to a HashSet for O(1) lookup
        let exclude_set: std::collections::HashSet<i64> = exclude_ids.iter().cloned().collect();

        let nodes: Vec<NodeRecord> = all_rows
            .into_iter()
            .filter(|row| !exclude_set.contains(&row.id))
            .take(limit as usize)
            .map(|row| {
                let address = NodeAddress::new(row.scheme, row.host, row.port as u16);
                let first_seen_at = row.first_seen_at.parse().unwrap_or_else(|_| chrono::Utc::now());
                let metadata = NodeMetadata::new(row.id, row.network, first_seen_at);
                let health = NodeHealthStats {
                    success_count: row.success_count,
                    failure_count: row.failure_count,
                    last_success: row.last_success.and_then(|s| s.parse().ok()),
                    last_failure: row.last_failure.and_then(|s| s.parse().ok()),
                    last_checked: row.last_checked.and_then(|s| s.parse().ok()),
                    is_reliable: row.is_reliable != 0,
                    avg_latency_ms: row.avg_latency_ms,
                    min_latency_ms: row.min_latency_ms,
                    max_latency_ms: row.max_latency_ms,
                    last_latency_ms: row.last_latency_ms,
                };
                NodeRecord::new(address, metadata, health)
            })
            .collect();

        Ok(nodes)
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
