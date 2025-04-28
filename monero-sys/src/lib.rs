mod bridge;

use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
    pin::Pin,
    str::FromStr,
    sync::{Arc, OnceLock},
};

use cxx::let_cxx_string;
use tokio::sync::Mutex;

use bridge::ffi;

static WALLET_MANAGER: OnceLock<Arc<Mutex<WalletManager>>> = OnceLock::new();

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WalletError {
    pub code: i32,
    pub message: String,
}

impl std::error::Error for WalletError {}

/// A singleton responsible for managing (creating, opening, ...) wallets.
pub struct WalletManager {
    inner: RawWalletManager,
    wallets: HashMap<String, Arc<Mutex<Wallet>>>,
}

pub struct RawWalletManager(*mut ffi::WalletManager);

unsafe impl Send for RawWalletManager {}

impl WalletManager {
    const DEFAULT_KDF_ROUNDS: u64 = 1;

    /// Get the wallet manager instance.
    pub fn get() -> Arc<Mutex<Self>> {
        WALLET_MANAGER
            .get_or_init(|| {
                let manager = ffi::getWalletManager();

                Arc::new(Mutex::new(Self {
                    inner: RawWalletManager(manager),
                    wallets: HashMap::new(),
                }))
            })
            .clone()
    }

    /// Create a new wallet, or open if it already exists.
    pub async fn open_or_create_wallet(
        &mut self,
        path: &str,
        password: Option<&str>,
        language: Option<&str>,
        network: monero::Network,
        kdf_rounds: Option<u64>,
    ) -> Result<Arc<Mutex<Wallet>>, WalletError> {
        // If we've already loaded this wallet, return it.
        if self.wallets.contains_key(path) {
            return Ok(self.wallets[path].clone());
        }

        // If we haven't loaded the wallet, but it already exists, open it.
        if self.wallet_exists(path).await {
            return Ok(self
                .open_wallet(path, password, network, kdf_rounds)
                .await?);
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

        let wallet = Wallet::new(wallet_pointer).await?;

        // Todo: error checking

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
    ) -> Result<Arc<Mutex<Wallet>>, WalletError> {
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

        let wallet = Wallet::new(wallet_pointer).await?;

        // Todo: error checking

        let wallet = Arc::new(Mutex::new(wallet));
        self.wallets.insert(path.to_string(), wallet.clone());

        Ok(wallet)
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
    ) -> Result<Arc<Mutex<Wallet>>, WalletError> {
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

        let wallet = Wallet::new(wallet_pointer).await?;

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

/// A single Monero wallet.
pub struct Wallet {
    inner: RawWallet,
}

/// This is our own wrapper around a raw C++ wallet pointer.
pub struct RawWallet(*mut ffi::Wallet);

/// # Safety: Todo
unsafe impl Send for RawWallet {}

impl Wallet {
    /// Create and initialize new wallet from a raw C++ wallet pointer.
    async fn new(inner: *mut ffi::Wallet) -> Result<Self, WalletError> {
        if inner.is_null() {
            return Err(WalletError::critical("wallet pointer is null"));
        }

        let mut wallet = Self {
            inner: RawWallet(inner),
        };

        wallet.check_error()?;

        wallet.init(false).await?;

        Ok(wallet)
    }

    /// Get the address for the given account and address index.
    /// address(0, 0) is the main address.
    pub fn address(&self, account_index: u32, address_index: u32) -> monero::Address {
        let address = ffi::address(&self.inner, account_index, address_index);
        monero::Address::from_str(&address.to_string()).expect("wallet's own address to be valid")
    }

    /// Get the main address of the walllet (account 0, address 0).
    pub fn main_address(&self) -> monero::Address {
        self.address(0, 0)
    }

    /// Initialize the wallet and download initial values from the remote node.
    /// Does not actuallyt sync the wallet, use any of the refresh methods to do that.
    async fn init(&mut self, ssl: bool) -> Result<(), WalletError> {
        let_cxx_string!(daemon_address = "");
        let_cxx_string!(daemon_username = "");
        let_cxx_string!(daemon_password = "");
        let_cxx_string!(proxy_address = "");

        let success = self.inner.pinned().init(
            &daemon_address,
            0,
            &daemon_username,
            &daemon_password,
            ssl,
            false,
            &proxy_address,
        );

        if !success {
            self.check_error()?;
            return Err(WalletError::critical("wallet init failed"));
        }

        Ok(())
    }

    /// Start the background refresh thread (refreshes every 10 seconds).
    pub fn start_refresh(&mut self) {
        self.inner.pinned().startRefresh();
    }

    /// Refresh the wallet asynchronously.
    pub async fn refresh_async(&mut self) {
        self.inner.pinned().refreshAsync();
    }

    /// Refresh the wallet once.
    pub async fn refresh(&mut self) -> bool {
        self.inner.pinned().refresh()
    }

    /// Get the current blockchain height.
    pub fn blockchain_height(&self) -> u64 {
        self.inner.blockChainHeight()
    }

    /// Get the daemon's blockchain height.
    ///
    /// Returns the height of the blockchain from the connected daemon.
    /// Returns 0 if there's an error communicating with the daemon.
    pub fn daemon_blockchain_height(&self) -> u64 {
        self.inner.daemonBlockChainHeight()
    }

    /// Get the total balance across all accounts.
    pub fn balance_all(&self) -> monero::Amount {
        let balance = self.inner.balanceAll();
        monero::Amount::from_pico(balance)
    }

    /// Get the total unlocked balance across all accounts in atomic units.
    pub fn unlocked_balance_all(&self) -> monero::Amount {
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
    pub fn set_allow_mismatched_daemon_version(&mut self, allow_mismatch: bool) {
        self.inner
            .pinned()
            .setAllowMismatchedDaemonVersion(allow_mismatch);
    }

    /// Return `Ok` when the wallet is ok, otherwise return the error.
    pub fn check_error(&mut self) -> Result<(), WalletError> {
        let mut status = 0;
        let mut error_string = String::new();
        let_cxx_string!(error_string_ref = &mut error_string);

        self.inner
            .statusWithErrorString(&mut status, error_string_ref);

        // If the status is ok, we return None
        if status == 0 {
            return Ok(());
        }

        // Otherwise we return the error
        Err(WalletError {
            code: status,
            message: error_string.to_string(),
        })
    }
}

impl WalletError {
    /// Create a new non-critical wallet error.
    fn error(message: impl Into<String>) -> Self {
        Self {
            code: 1,
            message: message.into(),
        }
    }

    /// Create a new critical wallet error.
    fn critical(message: impl Into<String>) -> Self {
        Self {
            code: 2,
            message: message.into(),
        }
    }
}

impl RawWallet {
    /// Get a pinned reference to the inner (c++) wallet.
    pub fn pinned(&mut self) -> Pin<&mut ffi::Wallet> {
        unsafe { Pin::new_unchecked(self.0.as_mut().expect("wallet pointer not to be null")) }
    }
}

impl Deref for Wallet {
    type Target = RawWallet;

    fn deref(&self) -> &RawWallet {
        &self.inner
    }
}

impl DerefMut for Wallet {
    fn deref_mut(&mut self) -> &mut RawWallet {
        &mut self.inner
    }
}

impl Deref for RawWallet {
    type Target = ffi::Wallet;

    fn deref(&self) -> &ffi::Wallet {
        unsafe { self.0.as_ref().expect("wallet pointer not to be null") }
    }
}

impl DerefMut for RawWallet {
    fn deref_mut(&mut self) -> &mut ffi::Wallet {
        unsafe { self.0.as_mut().expect("wallet pointer not to be null") }
    }
}

impl std::fmt::Display for WalletError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Wallet status not ok: `{}` with message `{}`",
            self.code, self.message
        )
    }
}
