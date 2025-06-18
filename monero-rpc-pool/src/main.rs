use clap::Parser;
use tracing::{error, info, warn};
use tracing_subscriber::{self, EnvFilter};

use monero_rpc_pool::database::Database;
use monero_rpc_pool::discovery::NodeDiscovery;
use monero_rpc_pool::{config::Config, run_server};
use url;

use monero::Network;

fn parse_network(s: &str) -> Result<Network, String> {
    match s.to_lowercase().as_str() {
        "mainnet" => Ok(Network::Mainnet),
        "stagenet" => Ok(Network::Stagenet),
        "testnet" => Ok(Network::Testnet),
        _ => Err(format!(
            "Invalid network: {}. Must be mainnet, stagenet, or testnet",
            s
        )),
    }
}

fn network_to_string(network: &Network) -> String {
    match network {
        Network::Mainnet => "mainnet".to_string(),
        Network::Stagenet => "stagenet".to_string(),
        Network::Testnet => "testnet".to_string(),
    }
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
    #[arg(value_parser = parse_network)]
    network: Network,

    #[arg(short, long)]
    #[arg(help = "Enable verbose logging")]
    verbose: bool,
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
            network_to_string(&args.network)
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
        info!(
            "Fetching nodes for {} network",
            network_to_string(&args.network)
        );
        let db = Database::new().await?;
        let discovery = NodeDiscovery::new(db.clone());

        match discovery
            .fetch_nodes_from_network(&network_to_string(&args.network))
            .await
        {
            Ok(fetched_nodes) => {
                info!("Successfully fetched {} nodes", fetched_nodes.len());

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

    let node_list = if args.verbose && !config.nodes.is_empty() {
        let nodes_formatted: Vec<String> = config
            .nodes
            .iter()
            .enumerate()
            .map(|(i, node)| format!("    {}: {}", i + 1, node))
            .collect();
        format!("\n{}", nodes_formatted.join("\n"))
    } else {
        String::new()
    };

    info!(
        "Starting Monero RPC Pool\nConfiguration:\n  Host: {}\n  Port: {}\n  Network: {}\n  Nodes: {} configured{}",
        config.host, config.port, network_to_string(&args.network), config.nodes.len(), node_list
    );

    if let Err(e) = run_server(config, network_to_string(&args.network)).await {
        eprintln!("Server error: {}", e);
        std::process::exit(1);
    }

    Ok(())
}
