use std::sync::Arc;

use anyhow::Result;
use axum::{
    routing::{get, post},
    Router,
};
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
use tracing::{error, info};

pub mod config;
pub mod database;
pub mod discovery;
// pub mod handlers;
pub mod pool;
pub mod simple_handlers;

use config::Config;
use database::Database;
use discovery::NodeDiscovery;
use pool::{NodePool, PoolStatus};
use simple_handlers::{simple_http_handler, simple_rpc_handler, simple_stats_handler};

#[derive(Clone)]
pub struct AppState {
    pub node_pool: Arc<RwLock<NodePool>>,
}

/// Information about a running RPC pool server
#[derive(Debug, Clone)]
pub struct ServerInfo {
    pub port: u16,
    pub host: String,
}

// TODO: Network should be part of the config and use the same type we use in swap (from monero-rs)
async fn create_app_with_receiver(config: Config, network: String) -> Result<(Router, tokio::sync::broadcast::Receiver<PoolStatus>)> {
    // Initialize database
    let db = Database::new().await?;

    // Initialize node pool with network
    let (node_pool, status_receiver) = NodePool::new(db.clone(), network.clone());
    let node_pool = Arc::new(RwLock::new(node_pool));

    // Initialize discovery service
    let discovery = NodeDiscovery::new(db.clone());

    // Insert configured nodes if any
    if !config.nodes.is_empty() {
        info!("Inserting {} configured nodes for network: {}...", config.nodes.len(), network);
        if let Err(e) = discovery.discover_and_insert_nodes(&network, config.nodes.clone()).await {
            error!("Failed to insert configured nodes for network {}: {}", network, e);
        }
    }

    // Start initial health check in parallel (non-blocking)
    let discovery_health_check = discovery.clone();
    let network_health_check = network.clone();
    let node_pool_for_health_check = node_pool.clone();
    
    // TODO: Store the handles in a struct and drop them when the app is dropped
    tokio::spawn(async move {
        info!("Performing initial health check for network: {}...", network_health_check);
        if let Err(e) = discovery_health_check.health_check_all_nodes(&network_health_check).await {
            error!("Failed initial health check for network {}: {}", network_health_check, e);
        }        
    });

    // Periodically send status updates (ever 10s)
    tokio::spawn(async move {
        loop {
            // Publish status update after health check
            let pool_guard = node_pool_for_health_check.read().await;
            if let Err(e) = pool_guard.publish_status_update().await {
                error!("Failed to publish status update after health check: {}", e);
            }

            tokio::time::sleep(std::time::Duration::from_secs(10)).await;
        }
    });


    // Start periodic discovery task
    let discovery_clone = discovery.clone();
    let network_clone = network.clone();
    let _node_pool_for_periodic = node_pool.clone();
    tokio::spawn(async move {
        if let Err(e) = discovery_clone.periodic_discovery_task(&network_clone).await {
            error!("Periodic discovery task failed for network {}: {}", network_clone, e);
        }
    });

    let app_state = AppState { node_pool };

    // Build the app
    let app = Router::new()
        // TODO: Route ALL types of requests here (POST, GET, etc, no need to differentiate)
        .route("/json_rpc", post(simple_rpc_handler))
        .route("/stats", get(simple_stats_handler))
        .route("/*path", get(simple_http_handler))
        .route("/*path", post(simple_http_handler))
        .layer(CorsLayer::permissive())
        .with_state(app_state);

    Ok((app, status_receiver))
}

pub async fn create_app(config: Config, network: String) -> Result<Router> {
    let (app, _) = create_app_with_receiver(config, network).await?;
    Ok(app)
}

pub async fn run_server(config: Config, network: String) -> Result<()> {
    let app = create_app(config.clone(), network).await?;

    let bind_address = format!("{}:{}", config.host, config.port);
    info!("Starting server on {}", bind_address);

    let listener = tokio::net::TcpListener::bind(&bind_address).await?;
    info!("Server listening on {}", bind_address);

    axum::serve(listener, app).await?;
    Ok(())
}

/// Start a server with a random port for library usage
/// Returns the server info with the actual port used and a receiver for pool status updates
/// TODO: Network should be part of the config and have a proper type
pub async fn start_server_with_random_port(config: Config, network: String) -> Result<(ServerInfo, tokio::sync::broadcast::Receiver<PoolStatus>)> {
    // Clone the host before moving config
    let host = config.host.clone();
    
    // If port is 0, the system will assign a random available port
    let config_with_random_port = Config {
        port: 0,
        ..config
    };

    let (app, status_receiver) = create_app_with_receiver(config_with_random_port, network).await?;

    // Bind to port 0 to get a random available port
    let listener = tokio::net::TcpListener::bind(format!("{}:0", host)).await?;
    let actual_addr = listener.local_addr()?;

    let server_info = ServerInfo {
        port: actual_addr.port(),
        host: host.clone(),
    };

    info!(
        "Started server on {}:{} (random port)",
        server_info.host, server_info.port
    );

    // Start the server in a background task
    tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, app).await {
            error!("Server error: {}", e);
        }
    });

    Ok((server_info, status_receiver))
}
