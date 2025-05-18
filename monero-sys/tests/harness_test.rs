use anyhow::Context;
use monero_harness::{image::Monerod, Monero};
use monero_sys::{Daemon, SyncProgress, WalletHandle};
use std::sync::OnceLock;
use tempfile::{tempdir, TempDir};
use testcontainers::{clients::Cli, Container};
use tracing::info;

const PASSWORD: &str = "test";

const SEED: &str = "echo ourselves ruined oven masterful wives enough addicted future cottage illness adopt lucky movement tiger taboo imbalance antics iceberg hobby oval aloof tuesday uttered oval";

// Amount to fund the wallet with (in piconero)
const FUND_AMOUNT: monero::Amount = monero::Amount::ONE_XMR;

// Global temporary directory for all wallet files
static GLOBAL_TEMP_DIR: OnceLock<TempDir> = OnceLock::new();

#[tokio::test]
async fn test_monero_wrapper_with_harness() {
    tracing_subscriber::fmt()
        .with_env_filter("info,test=debug,monero_harness=debug,monero_rpc=debug,harness_test=debug,monero_sys=trace")
        .with_test_writer()
        .init();

    // Step 1: Set up monero-harness and fund the address
    let tc = Cli::default();
    let (monero, monerod_container, _wallet_containers) = Monero::new(&tc, vec![])
        .await
        .expect("Failed to create Monero containers");

    let daemon_address = get_daemon_address(&monerod_container);
    let daemon = Daemon {
        address: daemon_address,
        ssl: false,
    };

    // Step 2: Create a wallet with monero-sys using the global temp directory
    let wallet_path = temp_wallet_path();
    let wallet = WalletHandle::open_or_create_from_seed(
        wallet_path,
        SEED.to_string(),
        monero::Network::Mainnet,
        1,
        daemon.clone(),
    )
    .await
    .expect("Failed to create wallet");

    wallet
        .__unsafe_never_call_outside_regtests_or_you_will_go_to_hell()
        .await;

    info!(
        "Created monero-wrapper wallet with address: {}",
        wallet.main_address().await
    );

    // Initialize miner
    info!("Initializing miner wallet");
    monero
        .init_miner()
        .await
        .expect("Failed to initialize miner");

    // Start mining continuously to generate blocks
    info!("Starting continuous mining");
    monero.start_miner().await.expect("Failed to start miner");

    // Fund the address created by monero-wrapper
    info!(
        "Funding the test wallet address: {}",
        &wallet.main_address().await
    );
    fund_address(&monero, &wallet.main_address().await, FUND_AMOUNT)
        .await
        .expect("Failed to fund wallet address");

    // Step 3: Connect the wrapper wallet to the daemon and check balance
    info!("Connecting to daemon at: {}", &daemon.address);

    wallet
        .wait_until_synced(Some(|sync_progress: SyncProgress| {
            info!("Sync progress: {}%", sync_progress.percentage());
        }))
        .await
        .expect("Failed to sync wallet");

    let wallet_balance = wallet.total_balance().await;

    // Step 4: Verify the balance
    info!("Wallet balance: {}", wallet_balance);
    assert!(
        wallet_balance.as_pico() > 0,
        "Wallet balance should be greater than 0"
    );

    info!("Test passed! Wallet successfully received and detected funds");
}

/// Creates a wallet from a predefined seed and returns/// Funds an address with a given amount of piconero.
async fn fund_address(
    monero: &Monero,
    address: &monero::Address,
    amount: monero::Amount,
) -> anyhow::Result<()> {
    info!(
        "Funding address {} with {} piconero",
        address.to_string(),
        amount.as_pico()
    );

    // Generate some blocks to ensure miner has funds
    monero
        .fund_address(&address.to_string(), amount.as_pico())
        .await?;

    info!("Successfully funded address with {} piconero", amount);
    Ok(())
}

/// Returns a unique path to a temporary wallet dir and wallet file.
fn temp_wallet_path() -> String {
    // Get or initialize the global temp directory
    let temp_dir = GLOBAL_TEMP_DIR.get_or_init(|| {
        // Create a directory that won't be deleted until the program exits
        info!("Creating global temporary directory for wallet files");
        tempdir().expect("Failed to create global temporary directory")
    });

    // Generate a unique wallet filename using UUID
    let uuid = uuid::Uuid::new_v4(); // This is the correct method to generate a random UUID
    let wallet_filename = format!("wallet_{}", uuid);

    info!("Generated wallet dir: {}", temp_dir.path().display());
    info!("Generated wallet filename: {}", wallet_filename);

    temp_dir.path().join(wallet_filename).display().to_string()
}

/// As we are not running the monero-wrapper inside the Docker network, we need to connect to the locally exposed port
/// Docker maps the port from inside the container (18081) to a random port on the host
/// This function extracts the port and constructs the address as "localhost:<port>"
fn get_daemon_address(monerod_container: &Container<'_, Monerod>) -> String {
    let local_daemon_rpc_port = monerod_container
        .ports()
        .map_to_host_port_ipv4(monero_harness::image::RPC_PORT)
        .expect("monerod should have a mapping to the host for the default RPC port");

    format!("127.0.0.1:{}", local_daemon_rpc_port)
}
