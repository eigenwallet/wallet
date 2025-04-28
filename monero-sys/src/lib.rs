mod bridge;

use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
    pin::Pin,
    sync::{Arc, OnceLock},
};

use cxx::let_cxx_string;
use tokio::sync::Mutex;

use bridge::ffi;

pub use bridge::ffi::NetworkType;

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

    /// Create a new wallet.
    pub fn create_wallet(
        &mut self,
        path: &str,
        password: &str,
        language: &str,
        network_type: ffi::NetworkType,
        kdf_rounds: u64,
    ) -> Wallet {
        let_cxx_string!(path = path);
        let_cxx_string!(password = password);
        let_cxx_string!(language = language);

        let wallet_pointer =
            self.inner
                .pinned()
                .createWallet(&path, &password, &language, network_type, kdf_rounds);

        if wallet_pointer.is_null() {
            panic!("Failed to create wallet");
        }

        Wallet::new(wallet_pointer)
    }

    /// Recover a wallet from a mnemonic seed (electrum seed).
    ///
    /// # Arguments
    ///
    /// * `path` - Name of wallet file to be created
    /// * `password` - Password of wallet file
    /// * `mnemonic` - Mnemonic seed (25 words electrum seed)
    /// * `network_type` - Network type (MAINNET, TESTNET, STAGENET)
    /// * `restore_height` - Restore from start height (0 to start from the beginning)
    /// * `kdf_rounds` - Number of rounds for key derivation function
    /// * `seed_offset` - Optional passphrase used to derive the seed
    ///
    /// # Returns
    ///
    /// A new Wallet instance. Call wallet.status() to check if recovered successfully.
    #[allow(clippy::too_many_arguments)]
    pub fn recover_wallet(
        &mut self,
        path: &str,
        password: &str,
        mnemonic: &str,
        network_type: ffi::NetworkType,
        restore_height: u64,
        kdf_rounds: Option<u64>,
        seed_offset: Option<&str>,
    ) -> Wallet {
        let_cxx_string!(path = path);
        let_cxx_string!(password = password);
        let_cxx_string!(mnemonic = mnemonic);
        let_cxx_string!(seed_offset = seed_offset.unwrap_or(""));

        let wallet_pointer = self.inner.pinned().recoveryWallet(
            &path,
            &password,
            &mnemonic,
            network_type,
            restore_height,
            kdf_rounds.unwrap_or(Self::DEFAULT_KDF_ROUNDS),
            &seed_offset,
        );

        Wallet::new(wallet_pointer)
    }

    /// Set the address of the remote node ("daemon").
    pub fn set_daemon_address(&mut self, address: &str) {
        tracing::debug!(%address, "Connecting wallet manager to remote node");

        let_cxx_string!(address = address);

        self.inner.pinned().setDaemonAddress(&address);
    }

    /// Check if the wallet manager is connected to the configured daemon.
    pub fn connected(&mut self) -> bool {
        let mut version = 0;
        unsafe { self.inner.pinned().connected(&mut version) }
    }

    /// Get the error of the wallet, if there is one.
    pub fn get_error(&mut self) -> Option<WalletError> {
        let mut status = 0;
        let_cxx_string!(error_string = "");
        self.inner
            .pinned()
            .statusWithErrorString(&mut status, &mut error_string);

        // If the status is ok, we return None
        if status == ffi::Status::STATUS_OK as i32 {
            return None;
        }

        // Otherwise we return the error
        Some(WalletError {
            code: status,
            message: error_string.to_string(),
        })
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

pub struct RawWallet(*mut ffi::Wallet);

unsafe impl Send for RawWallet {}

impl Wallet {
    /// Create a new wallet from a raw C++ wallet pointer.
    fn new(inner: *mut ffi::Wallet) -> Self {
        Self {
            inner: RawWallet(inner),
        }
    }

    /// Get the address for the given account and address index.
    /// address(0, 0) is the main address.
    pub fn address(&self, account_index: u32, address_index: u32) -> String {
        let address = ffi::address(&self.inner, account_index, address_index);
        address.to_string()
    }

    /// Initialize the wallet by connecting to the specified remote node (daemon).
    pub fn init(&mut self, daemon_address: &str, ssl: bool) -> bool {
        let_cxx_string!(daemon_address = daemon_address);
        let_cxx_string!(daemon_username = "");
        let_cxx_string!(daemon_password = "");
        let_cxx_string!(proxy_address = "");

        self.inner.pinned().init(
            &daemon_address,
            0,
            &daemon_username,
            &daemon_password,
            ssl,
            false,
            &proxy_address,
        )
    }

    /// Start the background refresh thread (refreshes every 10 seconds).
    pub fn start_refresh(&mut self) {
        self.inner.pinned().startRefresh();
    }

    /// Refresh the wallet asynchronously.
    pub fn refresh_async(&mut self) {
        self.inner.pinned().refreshAsync();
    }

    /// Refresh the wallet once.
    pub fn refresh(&mut self) -> bool {
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

    /// Get the total balance across all accounts in atomic units.
    pub fn balance_all(&self) -> u64 {
        self.inner.balanceAll()
    }

    /// Get the total unlocked balance across all accounts in atomic units.
    pub fn unlocked_balance_all(&self) -> u64 {
        self.inner.unlockedBalanceAll()
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
