use monero_sys::{Daemon, WalletManager};

const STAGENET_REMOTE_NODE: &str = "node.sethforprivacy.com:38089";

#[tokio::test(flavor = "multi_thread")]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("info,test=debug,monero_harness=debug,monero_rpc=debug,wallet_closing=trace,monero_sys=trace")
        .with_test_writer()
        .init();

    let temp_dir = tempfile::tempdir().unwrap();
    let daemon = Daemon {
        address: STAGENET_REMOTE_NODE.into(),
        ssl: true,
    };

    let manager = WalletManager::get(Some(daemon)).await.unwrap();

    {
        let mut manager_lock = manager.lock().await;

        while !manager_lock.connected().await {
            tracing::info!("Waiting to connect to daemon...");
            std::thread::sleep(std::time::Duration::from_secs(1));
        }

        let _wallet = manager_lock
            .open_or_create_wallet(
                temp_dir
                    .path()
                    .join("wallet212321")
                    .display()
                    .to_string()
                    .as_str(),
                None,
                monero::Network::Stagenet,
            )
            .await
            .unwrap();

        tracing::debug!("Dropping wallet");
    }

    // Sleep for 2 seconds to allow the wallet to be closed
    tracing::info!("Sleeping for 2 seconds");
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    assert!(manager.lock().await.all_open_wallets().is_empty());
    tracing::info!("Closed wallet automatically");

    {
        // Make sure we can open the wallet again
        let mut manager_lock = manager.lock().await;
        let _wallet = manager_lock
            .open_or_create_wallet(
                temp_dir
                    .path()
                    .join("wallet212321")
                    .display()
                    .to_string()
                    .as_str(),
                None,
                monero::Network::Stagenet,
            )
            .await
            .unwrap();

        std::mem::drop(manager_lock); // To avoid deadlock
        tracing::debug!("Dropping wallet");
    }

    tracing::info!("Sleeping for 2 seconds");
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    // Make sure the wallet is still closed again
    assert!(manager.lock().await.all_open_wallets().is_empty());
}
