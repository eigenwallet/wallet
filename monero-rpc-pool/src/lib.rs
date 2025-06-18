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
pub mod smart_pool;

use config::Config;
use database::Database;
use discovery::NodeDiscovery;
use simple_handlers::{simple_http_handler, simple_rpc_handler, simple_stats_handler};
use smart_pool::SmartNodePool;

#[derive(Clone)]
pub struct AppState {
    pub smart_pool: Arc<RwLock<SmartNodePool>>,
}

/// Information about a running RPC pool server
#[derive(Debug, Clone)]
pub struct ServerInfo {
    pub port: u16,
    pub host: String,
}

pub async fn create_app(config: Config, network: String) -> Result<Router> {
    // Initialize database
    let db = Database::new().await?;

    // Initialize smart pool with network
    let smart_pool = Arc::new(RwLock::new(SmartNodePool::new(db.clone(), network.clone())));

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
    tokio::spawn(async move {
        info!("Performing initial health check for network: {}...", network_health_check);
        if let Err(e) = discovery_health_check.health_check_all_nodes(&network_health_check).await {
            error!("Failed initial health check for network {}: {}", network_health_check, e);
        }
    });

    // Start periodic discovery task
    let discovery_clone = discovery.clone();
    let network_clone = network.clone();
    tokio::spawn(async move {
        if let Err(e) = discovery_clone.periodic_discovery_task(&network_clone).await {
            error!("Periodic discovery task failed for network {}: {}", network_clone, e);
        }
    });

    let app_state = AppState { smart_pool };

    // Build the app
    let app = Router::new()
        .route("/json_rpc", post(simple_rpc_handler))
        .route("/stats", get(simple_stats_handler))
        .route("/*path", get(simple_http_handler))
        .route("/*path", post(simple_http_handler))
        .layer(CorsLayer::permissive())
        .with_state(app_state);

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
/// Returns the server info with the actual port used
/// TODO: Network should be part of the config and have a proper type
pub async fn start_server_with_random_port(config: Config, network: String) -> Result<ServerInfo> {
    // Clone the host before moving config
    let host = config.host.clone();
    
    // If port is 0, the system will assign a random available port
    let config_with_random_port = Config {
        port: 0,
        ..config
    };

    let app = create_app(config_with_random_port, network).await?;

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

    Ok(server_info)
}
