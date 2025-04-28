mod bridge;

use std::{
    ops::{Deref, DerefMut},
    pin::Pin,
    sync::{Arc, Mutex, OnceLock},
};

use cxx::let_cxx_string;

use bridge::ffi;

pub use bridge::ffi::NetworkType;

static WALLET_MANAGER: OnceLock<Arc<Mutex<WalletManager>>> = OnceLock::new();

pub struct WalletManager {
    inner: RawWalletManager,
}

pub struct RawWalletManager(*mut ffi::WalletManager);

unsafe impl Send for RawWalletManager {}
unsafe impl Sync for RawWalletManager {}

impl WalletManager {
    /// Get the wallet manager instance.
    pub fn get() -> Arc<Mutex<Self>> {
        WALLET_MANAGER
            .get_or_init(|| {
                let manager = unsafe { ffi::getWalletManager() };

                Arc::new(Mutex::new(Self {
                    inner: RawWalletManager(manager),
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

        let wallet_pointer = unsafe {
            self.inner
                .pinned()
                .createWallet(&path, &password, &language, network_type, kdf_rounds)
        };

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
    pub fn recover_wallet(
        &mut self,
        path: &str,
        password: &str,
        mnemonic: &str,
        network_type: ffi::NetworkType,
        restore_height: u64,
        kdf_rounds: u64,
        seed_offset: &str,
    ) -> Wallet {
        let_cxx_string!(path = path);
        let_cxx_string!(password = password);
        let_cxx_string!(mnemonic = mnemonic);
        let_cxx_string!(seed_offset = seed_offset);

        let wallet_pointer = unsafe {
            self.inner
                .pinned()
                .recoveryWallet(&path, &password, &mnemonic, network_type, restore_height, kdf_rounds, &seed_offset)
        };

        Wallet::new(wallet_pointer)
    }

    /// Set the address of the remote node ("daemon").
    pub fn set_daemon_address(&mut self, address: &str) {
        tracing::debug!(%address, "Connecting wallet manager to remote node");

        let_cxx_string!(address = address);
        unsafe {
            self.inner.pinned().setDaemonAddress(&address);
        }
    }

    /// Check if the wallet manager is connected to the configured daemon.
    pub fn connected(&mut self) -> bool {
        unsafe { self.inner.pinned().connected(std::ptr::null_mut()) }
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
unsafe impl Sync for RawWallet {}

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
        let address = unsafe { ffi::address(&self.inner, account_index, address_index) };
        address.to_string()
    }

    /// Initialize the wallet by connecting to the specified remote node (daemon).
    pub fn init(&mut self, daemon_address: &str, ssl: bool) -> bool {
        let_cxx_string!(daemon_address = daemon_address);
        let_cxx_string!(daemon_username = "");
        let_cxx_string!(daemon_password = "");
        let_cxx_string!(proxy_address = "");
        unsafe {
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
    }

    /// Start the background refresh thread (refreshes every 10 seconds).
    pub fn start_refresh(&mut self) {
        unsafe { self.inner.pinned().startRefresh() }
    }

    /// Refresh the wallet asynchronously.
    pub fn refresh_async(&mut self) {
        unsafe { self.inner.pinned().refreshAsync() }
    }

    /// Refresh the wallet once.
    pub fn refresh(&mut self) -> bool {
        unsafe { self.inner.pinned().refresh() }
    }
    
    /// Get the current blockchain height.
    pub fn blockchain_height(&self) -> u64 {
        unsafe { self.inner.blockChainHeight() }
    }
    
    /// Get the daemon's blockchain height.
    /// 
    /// Returns the height of the blockchain from the connected daemon.
    /// Returns 0 if there's an error communicating with the daemon.
    pub fn daemon_blockchain_height(&self) -> u64 {
        unsafe { self.inner.daemonBlockChainHeight() }
    }

    /// Get the total balance across all accounts in atomic units.
    pub fn balance_all(&self) -> u64 {
        unsafe { self.inner.balanceAll() }
    }

    /// Get the total unlocked balance across all accounts in atomic units.
    pub fn unlocked_balance_all(&self) -> u64 {
        unsafe { self.inner.unlockedBalanceAll() }
    }
    
    /// Check if wallet was ever synchronized.
    /// 
    /// Returns true if the wallet has been synchronized at least once,
    /// false otherwise.
    pub fn synchronized(&self) -> bool {
        unsafe { self.inner.synchronized() }
    }

    /// Set the allow mismatched daemon version flag.
    pub fn set_allow_mismatched_daemon_version(&mut self, allow_mismatch: bool) {
        unsafe { self.inner.pinned().setAllowMismatchedDaemonVersion(allow_mismatch) }
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
