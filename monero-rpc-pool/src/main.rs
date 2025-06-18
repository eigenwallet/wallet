use clap::Parser;
use serde::Deserialize;
use tracing::{error, info, warn};
use tracing_subscriber::{self, EnvFilter};

use monero_rpc_pool::database::Database;
use monero_rpc_pool::{config::Config, run_server};
use url;

// TODO: use the type from monero-rs here
#[derive(Debug, Clone, clap::ValueEnum)]
enum Network {
    Mainnet,
    Stagenet,
}

impl std::fmt::Display for Network {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Network::Mainnet => write!(f, "mainnet"),
            Network::Stagenet => write!(f, "stagenet"),
        }
    }
}

#[derive(Debug, Deserialize)]
struct MoneroFailResponse {
    monero: MoneroNodes,
}

#[derive(Debug, Deserialize)]
struct MoneroNodes {
    clear: Vec<String>,
    #[serde(default)]
    web_compatible: Vec<String>,
}

#[derive(Parser)]
#[command(name = "monero-rpc-pool")]
#[command(about = "A load-balancing HTTP proxy for Monero RPC nodes")]
#[command(version)]
struct Args {
    #[arg(long, default_value = "127.0.0.1")]
    #[arg(help = "Host address to bind the server to")]
    host: String,

    #[arg(short, long, default_value = "18081")]
    #[arg(help = "Port to bind the server to")]
    port: u16,

    #[arg(long, value_delimiter = ',')]
    #[arg(help = "Comma-separated list of Monero node URLs (overrides network-based discovery)")]
    nodes: Option<Vec<String>>,

    #[arg(short, long, default_value = "mainnet")]
    #[arg(help = "Network to use for automatic node discovery")]
    network: Network,

    #[arg(short, long)]
    #[arg(help = "Enable verbose logging")]
    verbose: bool,
}

// TODO: This needs to be moved into the discovery module
async fn fetch_nodes_from_network(
    network: &Network,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let url = format!(
        "https://monero.fail/nodes.json?chain=monero&network={}",
        network
    );

    info!("Fetching nodes from: {}", url);

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(format!("HTTP error: {}", response.status()).into());
    }

    let monero_fail_response: MoneroFailResponse = response.json().await?;

    // Combine clear and web_compatible nodes, preferring web_compatible (HTTPS) when available
    let mut nodes = monero_fail_response.monero.web_compatible;
    nodes.extend(monero_fail_response.monero.clear);

    // Remove duplicates while preserving order (web_compatible first)
    let mut unique_nodes = Vec::new();
    for node in nodes {
        if !unique_nodes.contains(&node) {
            unique_nodes.push(node);
        }
    }

    if unique_nodes.is_empty() {
        return Err("No nodes found in response".into());
    }

    info!(
        "Fetched {} nodes for {} network",
        unique_nodes.len(),
        network
    );
    Ok(unique_nodes)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Create a filter that only shows logs from our application
    let filter = if args.verbose {
        // In verbose mode, show DEBUG from our crate and WARN from everything else
        EnvFilter::new("monero_rpc_pool=debug,warn")
    } else {
        // In normal mode, show INFO from our crate and ERROR from everything else
        EnvFilter::new("monero_rpc_pool=info,error")
    };

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_file(true)
        .with_line_number(true)
        .init();

    // Determine nodes to use
    let nodes = if let Some(manual_nodes) = args.nodes {
        info!(
            "Using manually specified nodes for network: {}",
            args.network
        );

        // Insert manual nodes into database with network information
        let db = Database::new().await?;
        let mut parsed_nodes = Vec::new();

        for node_url in &manual_nodes {
            // Parse the URL to extract components
            if let Ok(url) = url::Url::parse(node_url) {
                let scheme = url.scheme().to_string();
                let _protocol = if scheme == "https" { "ssl" } else { "tcp" };
                let host = url.host_str().unwrap_or("").to_string();
                let port = url
                    .port()
                    .unwrap_or(if scheme == "https" { 443 } else { 80 })
                    as i64;

                let full_url = format!("{}://{}:{}", scheme, host, port);

                // Insert into database
                if let Err(e) = db.upsert_node(&scheme, &host, port).await {
                    warn!("Failed to insert manual node {}: {}", node_url, e);
                } else {
                    parsed_nodes.push(full_url);
                }
            } else {
                warn!("Failed to parse manual node URL: {}", node_url);
            }
        }

        parsed_nodes
    } else {
        info!("Fetching nodes for {} network", args.network);
        match fetch_nodes_from_network(&args.network).await {
            Ok(fetched_nodes) => {
                info!("Successfully fetched {} nodes", fetched_nodes.len());

                // Insert fetched nodes into database
                let db = Database::new().await?;
                let mut inserted_nodes = Vec::new();

                for node_url in &fetched_nodes {
                    // Parse the URL to extract components
                    if let Ok(url) = url::Url::parse(node_url) {
                        let scheme = url.scheme().to_string();
                        let _protocol = if scheme == "https" { "ssl" } else { "tcp" };
                        let host = url.host_str().unwrap_or("").to_string();
                        let port =
                            url.port()
                                .unwrap_or(if scheme == "https" { 18089 } else { 18081 })
                                as i64;

                        let full_url = format!("{}://{}:{}", scheme, host, port);

                        // Insert into database
                        if let Err(e) = db.upsert_node(&scheme, &host, port).await {
                            warn!("Failed to insert fetched node {}: {}", node_url, e);
                        } else {
                            inserted_nodes.push(full_url);
                        }
                    } else {
                        warn!("Failed to parse fetched node URL: {}", node_url);
                    }
                }

                inserted_nodes
            }
            Err(e) => {
                error!("Failed to fetch nodes from monero.fail: {}", e);
                warn!("Falling back to empty node list - discovery may be required");
                vec![]
            }
        }
    };

    let config = Config::from_args(Some(args.host), Some(args.port), Some(nodes));

    // TODO: Put this into a single log message
    info!("Starting Monero RPC Pool");
    info!("Configuration:");
    info!("  Host: {}", config.host);
    info!("  Port: {}", config.port);
    info!("  Network: {}", args.network);
    info!("  Nodes: {} configured", config.nodes.len());
    if args.verbose && !config.nodes.is_empty() {
        for (i, node) in config.nodes.iter().enumerate() {
            info!("    {}: {}", i + 1, node);
        }
    }

    if let Err(e) = run_server(config, args.network.to_string()).await {
        eprintln!("Server error: {}", e);
        std::process::exit(1);
    }

    Ok(())
}
