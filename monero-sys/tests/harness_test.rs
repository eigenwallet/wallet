use monero_harness::{image::Monerod, Monero};
use monero_wallet_sys::{NetworkType, WalletManager};
use std::{sync::OnceLock, time::Duration};
use tempfile::{tempdir, TempDir};
use testcontainers::{clients::Cli, Container};
use tokio::time::sleep;
use tracing::info;
use uuid::Uuid;

const KDF_ROUNDS: u64 = 1;
const PASSWORD: &str = "test";
const SEED_OFFSET: &str = "";

// Amount to fund the wallet with (in piconero)
const FUND_AMOUNT: u64 = 1_000_000_000_000;

// Global temporary directory for all wallet files
static GLOBAL_TEMP_DIR: OnceLock<TempDir> = OnceLock::new();

#[tokio::test]
async fn test_monero_wrapper_with_harness() {
    tracing_subscriber::fmt()
        .with_env_filter("warn,test=debug,monero_harness=debug,monero_rpc=debug,harness_test=debug")
        .with_test_writer()
        .init();

    // Step 1: Create a wallet with monero-wrapper using the global temp directory
    let wallet_path = get_temp_wallet_path();
    let (address, wallet_seed) = create_wallet(&wallet_path).await;

    info!("Created monero-wrapper wallet with address: {}", address);
    info!("Wallet seed: {}", wallet_seed);

    // Step 2: Set up monero-harness and fund the address
    let tc = Cli::default();
    let (monero, monerod_container, _wallet_containers) = Monero::new(&tc, vec![])
        .await
        .expect("Failed to create Monero containers");

    let daemon_address = get_daemon_address(&monerod_container);

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
    fund_address(&monero, &address, monero::Amount::from_pico(FUND_AMOUNT))
        .await
        .expect("Failed to fund wallet address");

    // Step 3: Connect the wrapper wallet to the daemon and check balance
    info!("Connecting to daemon at: {}", daemon_address);

    let wallet_balance = connect_and_check_balance(wallet_seed, daemon_address).await;

    // Step 4: Verify the balance
    info!("Wallet balance: {}", wallet_balance);
    assert!(
        wallet_balance.as_pico() > 0,
        "Wallet balance should be greater than 0"
    );

    info!("Test passed! Wallet successfully received and detected funds");
}

async fn create_wallet(wallet_path: &str) -> (monero::Address, String) {
    // Get wallet manager
    let wallet_manager_mutex = WalletManager::get();
    let mut wallet_manager = wallet_manager_mutex.lock().await;

    // Define a fixed seed to use for reproducible tests
    let seed = "echo ourselves ruined oven masterful wives enough addicted future cottage illness adopt lucky movement tiger taboo imbalance antics iceberg hobby oval aloof tuesday uttered oval";

    // Create wallet from the seed - we'll use 'recover' since we have a seed
    let wallet = wallet_manager
        .recover_wallet(
            wallet_path,
            PASSWORD,
            seed,
            // Regtest uses Mainnet addresses
            monero::Network::Mainnet,
            1,
            None,
            None,
        )
        .await;

    let wallet = wallet.lock().await;

    // Get the main address
    let address = wallet.main_address();

    (address, seed.to_string())
}

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

fn get_temp_wallet_path() -> String {
    // Get or initialize the global temp directory
    let temp_dir = GLOBAL_TEMP_DIR.get_or_init(|| {
        // Create a directory that won't be deleted until the program exits
        info!("Creating global temporary directory for wallet files");
        tempdir().expect("Failed to create global temporary directory")
    });

    // Generate a unique wallet filename using UUID
    let uuid = uuid::Uuid::new_v4(); // This is the correct method to generate a random UUID
    let wallet_filename = format!("wallet_{}", uuid);
    let wallet_path = temp_dir.path().join(wallet_filename);

    info!("Generated wallet path: {}", wallet_path.display());
    wallet_path.to_str().unwrap().to_string()
}

/// As we are not running the monero-wrapper inside the Docker network, we need to connect to the locally exposed port
/// Docker maps the port from inside the container (18081) to a random port on the host
/// This function extracts the port and constructs the address as "localhost:<port>"
fn get_daemon_address(monerod_container: &Container<'_, Monerod>) -> String {
    let local_daemon_rpc_port = monerod_container
        .ports()
        .map_to_host_port_ipv4(monero_harness::image::RPC_PORT);
    let local_daemon_rpc_port = local_daemon_rpc_port
        .expect("monerod should have a mapping to the host for the default RPC port");

    format!("localhost:{}", local_daemon_rpc_port)
}

async fn connect_and_check_balance(seed: String, daemon_address: String) -> monero::Amount {
    // Get wallet manager
    let wallet_manager_mutex = WalletManager::get();
    let mut wallet_manager = wallet_manager_mutex.lock().await;

    // Set daemon address
    wallet_manager.set_daemon_address(&daemon_address);

    // Check connection
    let connected = wallet_manager.connected().await;
    info!("Connected to daemon: {}", connected);
    assert!(connected, "Should be connected to daemon");

    // Get a unique wallet path from the global temp directory
    let wallet_path = get_temp_wallet_path();
    tracing::info!("Recovering wallet from seed to {}", wallet_path);

    // Recover wallet from seed
    let mut wallet = wallet_manager
        .recover_wallet(
            &wallet_path,
            PASSWORD,
            &seed,
            monero::Network::Mainnet, // Regtest uses Mainnet addresses
            1,                        // Restore height (start from beginning)
            Some(KDF_ROUNDS),
            Some(SEED_OFFSET),
        )
        .await;

    let mut wallet = wallet.lock().await;

    // Initialize wallet
    wallet.init(Some(&daemon_address), false).await;

    tracing::info!(
        "Starting testing of wallet with address: {}",
        wallet.main_address()
    );

    // We need to allow mismatched daemon versions for the Regtest network
    // to be accepted by wallet2
    wallet.set_allow_mismatched_daemon_version(true);

    // Start background refresh
    wallet.start_refresh();

    // Wait for wallet to sync
    info!("Waiting for wallet to sync...");
    while !wallet.synchronized() {
        let wallet_height = wallet.blockchain_height();
        let daemon_height = wallet.daemon_blockchain_height();

        info!(
            "Wallet height: {}, Daemon height: {}",
            wallet_height, daemon_height
        );

        sleep(Duration::from_secs(1)).await;
    }

    // Manual refresh to ensure we have the latest state
    info!("Performing final refresh");
    let refresh_result = wallet.refresh().await;
    info!("Final refresh result: {}", refresh_result);

    // Check balance
    let balance = wallet.balance_all();
    info!("Final balance check: {}", balance);
    balance
}
