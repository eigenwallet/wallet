use clap::Parser;
use tracing::info;
use tracing_subscriber::{self, EnvFilter};

use monero_rpc_pool::{config::Config, run_server};

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
    #[arg(help = "Comma-separated list of Monero node URLs")]
    nodes: Option<Vec<String>>,

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

    let config = Config::from_args(Some(args.host), Some(args.port), args.nodes);

    info!("Starting Monero RPC Pool");
    info!("Configuration:");
    info!("  Host: {}", config.host);
    info!("  Port: {}", config.port);
    info!("  Nodes: {:?}", config.nodes);

    if let Err(e) = run_server(config).await {
        eprintln!("Server error: {}", e);
        std::process::exit(1);
    }

    Ok(())
}
