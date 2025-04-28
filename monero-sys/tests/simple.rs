use monero_wallet_sys::WalletManager;

const KDF_ROUNDS: u64 = 1;
const PASSWORD: &str = "test";

// No seed offset for now. This is the default
// TODO: Let lib.rs take an Option<_> and if None pass in an empty string
const SEED_OFFSET: &str = "";

const STAGENET_REMOTE_NODE: &str = "node.sethforprivacy.com:38089";
const STAGENET_WALLET_SEED: &str = "echo ourselves ruined oven masterful wives enough addicted future cottage illness adopt lucky movement tiger taboo imbalance antics iceberg hobby oval aloof tuesday uttered oval";
const STAGENET_WALLET_RESTORE_HEIGHT: u64 = 1728128;

#[tokio::test]
async fn main() {
    tracing_subscriber::fmt::init();

    let wallet_manager_mutex = WalletManager::get();
    let mut wallet_manager = wallet_manager_mutex.lock().await;

    tracing::info!("Setting daemon address");
    wallet_manager.set_daemon_address(STAGENET_REMOTE_NODE);

    tracing::info!("Connected: {}", wallet_manager.connected().await);

    let temp_dir = tempfile::tempdir().unwrap();

    let wallet_name = "recovered_wallet";
    let wallet_path = temp_dir.path().join(wallet_name);

    tracing::info!("Recovering wallet from seed");
    let wallet = wallet_manager
        .recover_wallet(
            wallet_path.to_str().unwrap(),
            PASSWORD,
            STAGENET_WALLET_SEED,
            monero::Network::Stagenet,
            STAGENET_WALLET_RESTORE_HEIGHT,
            Some(KDF_ROUNDS),
            Some(SEED_OFFSET),
        )
        .await
        .expect("Failed to recover wallet");

    let mut wallet = wallet.lock().await;

    tracing::info!("Primary address: {}", wallet.main_address());

    // Start background refresh
    tracing::info!("Starting background refresh");
    wallet.start_refresh();

    // Wait for a while to let the wallet sync, checking sync status
    tracing::info!("Waiting for wallet to sync...");

    // TODO: lib.rs should provide an async method that does this for us
    while !wallet.synchronized() {
        let wallet_height = wallet.blockchain_height();
        let daemon_height = wallet.daemon_blockchain_height();
        let is_synced = wallet.synchronized();

        // Calculate sync percentage if daemon height is available
        let sync_percentage = if daemon_height > 0 && daemon_height >= wallet_height {
            (wallet_height as f64 / daemon_height as f64 * 100.0).round()
        } else {
            0.0
        };

        tracing::info!(
            "Wallet height: {}, Daemon height: {}, Sync: {}%, Synchronized: {}",
            wallet_height,
            daemon_height,
            sync_percentage,
            is_synced,
        );

        std::thread::sleep(std::time::Duration::from_secs(3));
    }

    tracing::info!("Wallet is synchronized!");

    // Manual refresh one more time
    tracing::info!(result=%wallet.refresh().await, "Manual refresh");

    let balance = wallet.balance_all();
    tracing::info!("Balance: {}", balance.as_pico());

    let unlocked_balance = wallet.unlocked_balance_all();
    tracing::info!("Unlocked balance: {}", unlocked_balance.as_pico());
}
