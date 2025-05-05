use monero_harness::{image::Monerod, Monero};
use monero_sys::{Daemon, SyncProgress, Wallet, WalletManager};
use std::sync::OnceLock;
use tempfile::{tempdir, TempDir};
use testcontainers::{clients::Cli, Container};
use tracing::info;

const PASSWORD: &str = "test";

// Amount to fund the wallet with (in piconero)
const FUND_AMOUNT: monero::Amount = monero::Amount::ONE_XMR;

// Global temporary directory for all wallet files
static GLOBAL_TEMP_DIR: OnceLock<TempDir> = OnceLock::new();

#[tokio::test]
async fn test_monero_wrapper_with_harness() {
    tracing_subscriber::fmt()
        .with_env_filter("warn,test=debug,monero_harness=debug,monero_rpc=debug,harness_test=debug")
        .with_test_writer()
        .init();

    // Step 1: Create a wallet with monero-wrapper using the global temp directory
    let (wallet_dir, wallet_filename) = temp_path();
    let (address, wallet_seed) = create_wallet(&wallet_filename, None, &wallet_dir).await;

    info!("Created monero-wrapper wallet with address: {}", address);
    info!("Wallet seed: {}", wallet_seed);

    // Step 2: Set up monero-harness and fund the address
    let tc = Cli::default();
    let (monero, monerod_container, _wallet_containers) = Monero::new(&tc, vec![])
        .await
        .expect("Failed to create Monero containers");

    let daemon_address = get_daemon_address(&monerod_container);
    let daemon = Daemon {
        address: daemon_address,
        ssl: false,
    };

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
    info!("Funding the test wallet address: {}", address);
    fund_address(&monero, &address, FUND_AMOUNT)
        .await
        .expect("Failed to fund wallet address");

    // Step 3: Connect the wrapper wallet to the daemon and check balance
    info!("Connecting to daemon at: {}", &daemon.address);

    let wallet_balance = connect_and_check_balance(wallet_seed, daemon).await;

    // Step 4: Verify the balance
    info!("Wallet balance: {}", wallet_balance);
    assert!(
        wallet_balance.as_pico() > 0,
        "Wallet balance should be greater than 0"
    );

    info!("Test passed! Wallet successfully received and detected funds");
}

/// Creates a wallet from a predefined seed and returns the main address and seed.
async fn create_wallet(
    wallet_name: &str,
    daemon: Option<Daemon>,
    wallet_dir: &str,
) -> (monero::Address, String) {
    // Get wallet manager
    let wallet_manager_mutex = WalletManager::get(daemon, wallet_dir).await;
    let mut wallet_manager = wallet_manager_mutex.lock().await;

    // Define a fixed seed to use for reproducible tests
    let seed = "echo ourselves ruined oven masterful wives enough addicted future cottage illness adopt lucky movement tiger taboo imbalance antics iceberg hobby oval aloof tuesday uttered oval";

    // Create wallet from the seed - we'll use 'recover' since we have a seed
    let wallet = wallet_manager
        .recover_wallet(
            wallet_name,
            PASSWORD,
            seed,
            // Regtest uses Mainnet addresses
            monero::Network::Mainnet,
            1,
        )
        .await
        .expect("Failed to recover wallet");

    let wallet = wallet.lock().await;

    // Get the main address
    let address = wallet.main_address();

    (address, seed.to_string())
}

/// Funds an address with a given amount of piconero.
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
fn temp_path() -> (String, String) {
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

    (
        temp_dir.path().to_str().unwrap().to_string(),
        wallet_filename,
    )
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

async fn connect_and_check_balance(seed: String, daemon: Daemon) -> monero::Amount {
    // Get wallet manager
    let (wallet_dir, wallet_filename) = temp_path();
    let wallet_manager_mutex = WalletManager::get(Some(daemon.clone()), &wallet_dir).await;
    let mut wallet_manager = wallet_manager_mutex.lock().await;

    // Get a unique wallet path from the global temp directory
    tracing::info!(
        "Recovering wallet from seed to {}/{}",
        &wallet_dir,
        wallet_filename
    );

    // Recover wallet from seed
    let wallet_mutex = wallet_manager
        .recover_wallet(
            &wallet_filename,
            PASSWORD,
            &seed,
            monero::Network::Mainnet, // Regtest uses Mainnet addresses
            1,                        // Restore height (start from beginning)
        )
        .await
        .expect("Failed to recover wallet");

    let mut wallet = wallet_mutex.lock().await;

    // Explicitly set the daemon address on the wallet instance
    info!(
        "Explicitly setting daemon address for wallet to: {}",
        &daemon.address
    );
    wallet
        .set_daemon_address(&daemon.address)
        .expect("Failed to set daemon address on wallet");

    tracing::info!(
        "Starting testing of wallet with address: {}",
        wallet.main_address()
    );

    // We need to allow mismatched daemon versions for the Regtest network
    // to be accepted by wallet2
    wallet.allow_mismatched_daemon_version();

    // Start background refresh
    let _ = wallet;

    Wallet::wait_until_synced(
        wallet_mutex.clone(),
        Some(|sync_progress: SyncProgress| {
            tracing::info!("Sync progress: {}%", sync_progress.percentage());
        }),
    )
    .await
    .expect("Failed to sync wallet");

    let wallet = wallet_mutex.lock().await;

    // Check balance
    let balance = wallet.total_balance();
    info!("Final balance check: {}", balance);
    balance
}
