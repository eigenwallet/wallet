#![doc = include_str!("../README.md")]

mod bridge;

use std::{
    collections::HashMap,
    ops::Deref,
    pin::Pin,
    str::FromStr,
    sync::{Arc, OnceLock},
};

use anyhow::{bail, Context};
use cxx::let_cxx_string;
use tokio::sync::Mutex;

use bridge::ffi;

static WALLET_MANAGER: OnceLock<Arc<Mutex<WalletManager>>> = OnceLock::new();

/// A singleton responsible for managing (creating, opening, ...) wallets.
pub struct WalletManager {
    inner: RawWalletManager,
    wallets: HashMap<String, Arc<Mutex<Wallet>>>,
    daemon: Option<Daemon>,
}

/// This is our own wrapper around a raw C++ wallet manager pointer.
struct RawWalletManager(*mut ffi::WalletManager);

/// A single Monero wallet.
pub struct Wallet {
    inner: RawWallet,
}

/// This is our own wrapper around a raw C++ wallet pointer.
struct RawWallet(*mut ffi::Wallet);

/// The progress of synchronization of a wallet with the remote node.
///
///
pub struct SyncProgress {
    /// The current block height of the wallet.
    pub current_block: u64,
    /// The target block height of the wallet.
    pub target_block: u64,
}

/// The status of a transaction.
pub struct TxStatus {
    /// The amount received in the transaction.
    pub received: u64,
    /// Whether the transaction is in the mempool.
    pub in_pool: bool,
    /// The number of confirmations the transaction has.
    pub confirmations: u64,
}

/// A remote node to connect to.
#[derive(Debug, Clone, Default)]
pub struct Daemon {
    pub address: String,
    pub ssl: bool,
}

impl WalletManager {
    const DEFAULT_KDF_ROUNDS: u64 = 1;

    /// Get the wallet manager instance.
    pub async fn get<'a>(daemon: Option<Daemon>) -> Arc<Mutex<Self>> {
        let manager = WALLET_MANAGER.get_or_init(|| {
            let manager = ffi::getWalletManager();
            let manager = Self {
                inner: RawWalletManager(manager),
                wallets: HashMap::new(),
                daemon,
            };

            Arc::new(Mutex::new(manager))
        });

        {
            let mut lock = manager.lock().await;

            if let Some(daemon) = lock.daemon.clone() {
                lock.set_daemon_address(&daemon.address);
            }
        }

        manager.clone()
    }

    /// Create a new wallet, or open if it already exists.
    pub async fn open_or_create_wallet(
        &mut self,
        path: &str,
        password: Option<&str>,
        language: Option<&str>,
        network: monero::Network,
        kdf_rounds: Option<u64>,
        background_sync: bool,
    ) -> anyhow::Result<Arc<Mutex<Wallet>>> {
        // If we've already loaded this wallet, return it.
        if self.wallets.contains_key(path) {
            let wallet = self.wallets[path].clone();
            if background_sync {
                wallet.lock().await.start_refresh();
            }
            return Ok(wallet);
        }

        // If we haven't loaded the wallet, but it already exists, open it.
        if self.wallet_exists(path).await {
            return Ok(self
                .open_wallet(path, password, network, kdf_rounds, background_sync)
                .await
                .context("Failed to open wallet")?);
        }

        // Otherwise, create (and open) a new wallet.
        let_cxx_string!(path = path);
        let_cxx_string!(password = password.unwrap_or(""));
        let_cxx_string!(language = language.unwrap_or(""));
        let network_type = network.into();
        let kdf_rounds = kdf_rounds.unwrap_or(Self::DEFAULT_KDF_ROUNDS);

        let wallet_pointer =
            self.inner
                .pinned()
                .createWallet(&path, &password, &language, network_type, kdf_rounds);

        if wallet_pointer.is_null() {
            panic!("Failed to create wallet");
        }

        let wallet = Wallet::new(wallet_pointer, background_sync, self.daemon.clone()).await?;

        let wallet = Arc::new(Mutex::new(wallet));
        self.wallets.insert(path.to_string(), wallet.clone());

        Ok(wallet)
    }

    /// Recover a wallet from a mnemonic seed (electrum seed).
    #[allow(clippy::too_many_arguments)]
    pub async fn recover_wallet(
        &mut self,
        path: &str,
        password: &str,
        mnemonic: &str,
        network: monero::Network,
        restore_height: u64,
        kdf_rounds: Option<u64>,
        seed_offset: Option<&str>,
        background_sync: bool,
    ) -> anyhow::Result<Arc<Mutex<Wallet>>> {
        let_cxx_string!(path = path);
        let_cxx_string!(password = password);
        let_cxx_string!(mnemonic = mnemonic);
        let_cxx_string!(seed_offset = seed_offset.unwrap_or(""));
        let network_type = network.into();
        let wallet_pointer = self.inner.pinned().recoveryWallet(
            &path,
            &password,
            &mnemonic,
            network_type,
            restore_height,
            kdf_rounds.unwrap_or(Self::DEFAULT_KDF_ROUNDS),
            &seed_offset,
        );

        let wallet = Wallet::new(wallet_pointer, background_sync, self.daemon.clone()).await?;

        let wallet = Arc::new(Mutex::new(wallet));
        self.wallets.insert(path.to_string(), wallet.clone());

        Ok(wallet)
    }

    /// Close a wallet, optionally storing the wallet state.
    async fn close_wallet(&mut self, wallet: &mut Wallet) -> anyhow::Result<()> {
        let success = unsafe { self.inner.pinned().closeWallet(wallet.inner.0, true) };
        if !success {
            anyhow::bail!("Failed to close wallet");
        }
        Ok(())
    }

    /// Open a wallet. Only used internally. Use [`WalletManager::open_or_create_wallet`] instead.
    ///
    /// Todo: add listener support
    async fn open_wallet(
        &mut self,
        path: &str,
        password: Option<&str>,
        network_type: monero::Network,
        kdf_rounds: Option<u64>,
        background_sync: bool,
    ) -> anyhow::Result<Arc<Mutex<Wallet>>> {
        let_cxx_string!(path = path);
        let_cxx_string!(password = password.unwrap_or(""));
        let network_type = network_type.into();
        let kdf_rounds = kdf_rounds.unwrap_or(Self::DEFAULT_KDF_ROUNDS);

        let wallet_pointer = unsafe {
            self.inner.pinned().openWallet(
                &path,
                &password,
                network_type,
                kdf_rounds,
                std::ptr::null_mut(),
            )
        };

        if wallet_pointer.is_null() {
            anyhow::bail!("Failed to open wallet: got null pointer")
        }

        let wallet = Wallet::new(wallet_pointer, background_sync, self.daemon.clone()).await?;

        // Todo: error checking

        let wallet = Arc::new(Mutex::new(wallet));
        self.wallets.insert(path.to_string(), wallet.clone());

        Ok(wallet)
    }

    /// Set the address of the remote node ("daemon").
    pub fn set_daemon_address(&mut self, address: &str) {
        tracing::debug!(%address, "Updating wallet manager's default remote node");

        let_cxx_string!(address = address);

        self.inner.pinned().setDaemonAddress(&address);
    }

    /// Check if a wallet exists at the given path.
    pub async fn wallet_exists(&mut self, path: &str) -> bool {
        let_cxx_string!(path = path);

        self.inner.pinned().walletExists(&path)
    }

    /// Check if the wallet manager is connected to the configured daemon.
    pub async fn connected(&mut self) -> bool {
        let mut version = 0;
        unsafe { self.inner.pinned().connected(&mut version) }
    }
}

impl RawWalletManager {
    /// Get a pinned reference to the inner (c++) wallet manager.
    /// This is a convenience function necessary because
    /// the ffi interface mostly takes a Pin<&mut T> but
    /// we haven't figured out how to hold that in the struct.
    pub fn pinned(&mut self) -> Pin<&mut ffi::WalletManager> {
        unsafe {
            Pin::new_unchecked(
                self.0
                    .as_mut()
                    .expect("wallet manager pointer not to be null"),
            )
        }
    }
}

/// Safety: Todo
unsafe impl Send for RawWalletManager {}

impl Wallet {
    const MAIN_ACCOUNT_INDEX: u64 = 0;

    /// Create and initialize new wallet from a raw C++ wallet pointer.
    async fn new(
        inner: *mut ffi::Wallet,
        background_sync: bool,
        daemon: Option<Daemon>,
    ) -> anyhow::Result<Self> {
        if inner.is_null() {
            anyhow::bail!("Failed to create wallet: got null pointer");
        }

        let mut wallet = Self {
            inner: RawWallet(inner),
        };

        wallet.check_error()?;

        let daemon = daemon.unwrap_or_default();

        wallet
            .init(&daemon.address, daemon.ssl)
            .await
            .context("Failed to initialize wallet")?;

        if background_sync {
            wallet.start_refresh();
        }

        Ok(wallet)
    }

    /// Get the address for the given account and address index.
    /// address(0, 0) is the main address.
    /// We don't use anything besides the main address so this is a private method (for now).
    fn address(&self, account_index: u32, address_index: u32) -> monero::Address {
        let address = ffi::address(&self.inner, account_index, address_index);
        monero::Address::from_str(&address.to_string()).expect("wallet's own address to be valid")
    }

    pub fn set_daemon_address(&mut self, address: &str) -> anyhow::Result<()> {
        let_cxx_string!(address = address);
        let success = ffi::setWalletDaemon(self.inner.pinned(), &address);

        if !success {
            self.check_error().context("Failed to set daemon address")?;
            anyhow::bail!("Failed to set daemon address");
        }

        Ok(())
    }

    /// Get the main address of the walllet (account 0, address 0).
    pub fn main_address(&self) -> monero::Address {
        self.address(0, 0)
    }

    /// Initialize the wallet and download initial values from the remote node.
    /// Does not actuallyt sync the wallet, use any of the refresh methods to do that.
    async fn init(&mut self, daemon_address: &str, ssl: bool) -> anyhow::Result<()> {
        let_cxx_string!(daemon_address = daemon_address);
        let_cxx_string!(daemon_username = "");
        let_cxx_string!(daemon_password = "");
        let_cxx_string!(proxy_address = "");

        let success = self.inner.pinned().init(
            &daemon_address,
            0,
            &daemon_username,
            &daemon_password,
            ssl,
            true,
            &proxy_address,
        );

        if !success {
            self.check_error().context("Failed to initialize wallet")?;
            anyhow::bail!("Failed to initialize wallet");
        }

        Ok(())
    }

    /// Get the sync progress of the wallet as a percentage.
    pub async fn sync_progress(&mut self) -> SyncProgress {
        let current_block = self.inner.blockChainHeight();
        let target_block = self.daemon_blockchain_height().unwrap_or(0);
        SyncProgress::new(current_block, target_block)
    }

    /// Sync the wallet with the remote node.
    /// Returns when the sync is complete.
    ///
    pub async fn sync(&mut self) -> anyhow::Result<()> {
        // We wait for 100ms before polling the wallet's sync status again.
        // This is ok because this doesn't involve any blocking calls.
        const SLEEP_DURATION_MILLIS: u64 = 100;

        // Initiate the sync
        self.refresh_async().await;

        // Continue polling until the sync is complete
        while !self.inner.synchronized() {
            tokio::time::sleep(std::time::Duration::from_millis(SLEEP_DURATION_MILLIS)).await;
        }

        Ok(())
    }

    /// Start the background refresh thread (refreshes every 10 seconds).
    fn start_refresh(&mut self) {
        self.inner.pinned().startRefresh();
    }

    /// Refresh the wallet asynchronously.
    async fn refresh_async(&mut self) {
        self.inner.pinned().refreshAsync();
    }

    /// Get the current blockchain height.
    pub fn blockchain_height(&self) -> u64 {
        self.inner.blockChainHeight()
    }

    /// Get the daemon's blockchain height.
    ///
    /// Returns the height of the blockchain from the connected daemon.
    /// Returns 0 if there's an error communicating with the daemon.
    pub fn daemon_blockchain_height(&mut self) -> anyhow::Result<u64> {
        // Here we actually use the _target_ height -- incase the remote node is
        // currently catching up we want to work with the height it ends up at.
        let height = self.inner.daemonBlockChainTargetHeight();
        if height == 0 {
            self.check_error()
                .context("Failed to get daemon blockchain height")?;
            anyhow::bail!("Failed to get daemon blockchain height");
        } else {
            Ok(height)
        }
    }

    /// Get the total balance across all accounts.
    pub fn total_balance(&self) -> monero::Amount {
        let balance = self.inner.balanceAll();
        monero::Amount::from_pico(balance)
    }

    /// Get the total unlocked balance across all accounts in atomic units.
    pub fn unlocked_balance(&self) -> monero::Amount {
        let balance = self.inner.unlockedBalanceAll();
        monero::Amount::from_pico(balance)
    }

    /// Check if wallet was ever synchronized.
    ///
    /// Returns true if the wallet has been synchronized at least once,
    /// false otherwise.
    pub fn synchronized(&self) -> bool {
        self.inner.synchronized()
    }

    /// Set the allow mismatched daemon version flag.
    ///
    /// This is needed for regnet compatibility.
    ///
    /// _Do not use for anything besides testing._
    pub fn allow_mismatched_daemon_version(&mut self) {
        self.inner.pinned().setAllowMismatchedDaemonVersion(true);
    }

    /// Check if a transaction is in the mempool/confirmed.
    pub async fn check_tx_key(
        &mut self,
        txid: &str,
        tx_key: &str,
        address: &str,
    ) -> anyhow::Result<TxStatus> {
        let_cxx_string!(txid = txid);
        let_cxx_string!(tx_key = tx_key);
        let_cxx_string!(address = address);

        let mut received = 0;
        let mut in_pool = false;
        let mut confirmations = 0;

        let success = ffi::checkTxKey(
            self.inner.pinned(),
            &txid,
            &tx_key,
            &address,
            &mut received,
            &mut in_pool,
            &mut confirmations,
        );

        if !success {
            self.check_error().context("Failed to check tx key")?;
            anyhow::bail!("Failed to check tx key");
        }

        Ok(TxStatus {
            received,
            in_pool,
            confirmations,
        })
    }

    /// Transfer a specified amount of monero to a specified address.
    pub async fn transfer(
        &mut self,
        address: &monero::Address,
        amount: monero::Amount,
    ) -> anyhow::Result<()> {
        let_cxx_string!(address = address.to_string());
        let amount = amount.as_pico();

        // First we need to create a pending transaction.
        let pending_tx = ffi::createTransaction(self.inner.pinned(), &address, amount);

        let pinned_tx = unsafe {
            Pin::new_unchecked(pending_tx.as_mut().ok_or(anyhow::anyhow!(
                "failed to create transaction, got null pointer"
            ))?)
        };

        // Publish the transaction
        let result = pinned_tx
            .publish()
            .await
            .context("Failed to publish transaction");

        // Dispose of the transaction to avoid leaking memory.
        self.dispose_transaction(pending_tx).await;

        result
    }

    /// Sweep all funds from the wallet to a specified address.
    pub async fn sweep(&mut self, address: &monero::Address) -> anyhow::Result<()> {
        let_cxx_string!(address = address.to_string());

        // Create the sweep transaction
        let pending_tx = ffi::createSweepTransaction(self.inner.pinned(), &address);

        let pinned_tx = unsafe {
            Pin::new_unchecked(pending_tx.as_mut().ok_or(anyhow::anyhow!(
                "Failed to create sweep transaction, got null pointer"
            ))?)
        };

        // Publish the transaction
        let result = pinned_tx
            .publish()
            .await
            .context("Failed to publish transaction");

        // Dispose of the transaction to avoid leaking memory.
        self.dispose_transaction(pending_tx).await;

        result
    }

    /// Dispose (deallocate) a pending transaction object.
    /// Always call this before dropping a pending transaction object,
    /// otherwise we leak memory.
    async fn dispose_transaction(&mut self, tx: *mut ffi::PendingTransaction) {
        unsafe { self.inner.pinned().disposeTransaction(tx) };
    }

    /// Return `Ok` when the wallet is ok, otherwise return the error.
    /// This is a convenience method we use for retrieving errors after
    /// a method call failed.
    fn check_error(&mut self) -> anyhow::Result<()> {
        let mut status = 0;
        let mut error_string = String::new();
        let_cxx_string!(error_string_ref = &mut error_string);

        self.inner
            .statusWithErrorString(&mut status, error_string_ref);

        // If the status is ok, we return None
        if status == 0 {
            return Ok(());
        }

        let error_type = if status == 2 { "critical" } else { "error" };

        // Otherwise we return the error
        bail!(format!(
            "Experienced wallet error ({}): {}",
            error_type, error_string
        ))
    }
}

/// # Safety: Todo
unsafe impl Send for RawWallet {}

impl ffi::PendingTransaction {
    /// Return `Ok` when the pending transaction is ok, otherwise return the error.
    /// This is a convenience method we use for retrieving errors after
    /// a method call failed.
    fn check_error(&self) -> anyhow::Result<()> {
        let status = self.status();
        let error_string = ffi::pendingTransactionErrorString(self);

        if status == 0 {
            return Ok(());
        }

        let error_type = if status == 2 { "critical" } else { "error" };

        bail!(format!(
            "Experienced pending transaction error ({}): {}",
            error_type, error_string
        ))
    }

    /// Publish this transaction to the blockchain or return an error.
    ///
    /// **Important**: you still have to dispose the transaction.
    async fn publish(mut self: Pin<&mut Self>) -> anyhow::Result<()> {
        self.as_mut()
            .check_error()
            .context("Failed to create transaction")?;

        // Then we commit it to the blockchain.
        let_cxx_string!(filename = ""); // Empty filename means we commit to the blockchain
        let success = self.as_mut().commit(&filename, false);

        if success {
            Ok(())
        } else {
            // Get the error from the pending transaction.
            Err(self
                .as_mut()
                .check_error()
                .context("Failed to commit transaction to blockchain")
                .err()
                .unwrap_or(anyhow::anyhow!(
                    "Failed to commit transaction to blockchain"
                )))
        }
    }
}

impl SyncProgress {
    fn new(current_block: u64, target_block: u64) -> Self {
        Self {
            current_block,
            target_block,
        }
    }

    /// Get the sync progress as a fraction.
    pub fn fraction(&self) -> f32 {
        self.current_block as f32 / self.target_block as f32
    }

    /// Get the sync progress as a percentage.
    pub fn percentage(&self) -> f32 {
        100.0 * self.fraction()
    }
}

impl RawWallet {
    /// Convenience method for getting a pinned reference to the inner (c++) wallet.
    fn pinned(&mut self) -> Pin<&mut ffi::Wallet> {
        unsafe { Pin::new_unchecked(self.0.as_mut().expect("wallet pointer not to be null")) }
    }
}

// We implement Deref for RawWallet such that we can use the
// const c++ methods directly on the RawWallet struct.
impl Deref for RawWallet {
    type Target = ffi::Wallet;

    fn deref(&self) -> &ffi::Wallet {
        unsafe { self.0.as_ref().expect("wallet pointer not to be null") }
    }
}
