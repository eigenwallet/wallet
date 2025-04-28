#[cxx::bridge(namespace = "Monero")]
pub mod ffi {
    /// The type of the network.
    enum NetworkType {
        #[rust_name = "Mainnet"]
        MAINNET,
        #[rust_name = "Testnet"]
        TESTNET,
        #[rust_name = "Stagenet"]
        STAGENET,
    }

    unsafe extern "C++" {
        include!("monero-wallet-sys/monero/src/wallet/api/wallet2_api.h");
        include!("monero-wallet-sys/src/bridge.h");

        /// A manager for multiple wallets.
        type WalletManager;

        /// A single wallet.
        type Wallet;

        /// The type of the network.
        type NetworkType;

        /// An unsigned transaction.
        type UnsignedTransaction;

        /// A pending transaction.
        type PendingTransaction;

        /// Get the wallet manager.
        unsafe fn getWalletManager() -> *mut WalletManager;

        /// Create a new wallet.
        unsafe fn createWallet(
            self: Pin<&mut WalletManager>,
            path: &CxxString,
            password: &CxxString,
            language: &CxxString,
            network_type: NetworkType,
            kdf_rounds: u64,
        ) -> *mut Wallet;

        /// Recover a wallet from a mnemonic seed (electrum seed).
        unsafe fn recoveryWallet(
            self: Pin<&mut WalletManager>,
            path: &CxxString,
            password: &CxxString,
            mnemonic: &CxxString,
            network_type: NetworkType,
            restore_height: u64,
            kdf_rounds: u64,
            seed_offset: &CxxString,
        ) -> *mut Wallet;

        /// Get the current blockchain height.
        unsafe fn blockchainHeight(self: Pin<&mut WalletManager>) -> u64;

        /// Set the address of the remote node ("daemon").
        unsafe fn setDaemonAddress(self: Pin<&mut WalletManager>, address: &CxxString);

        /// Check if the wallet manager is connected to the configured daemon.
        unsafe fn connected(self: Pin<&mut WalletManager>, version: *mut u32) -> bool;

        /// Get the status of the wallet and an error string if there is one.
        unsafe fn statusWithErrorString(
            self: &Wallet,
            status: &mut i32,
            error_string: Pin<&mut CxxString>,
        );

        /// Address for the given account and address index.
        /// address(0, 0) is the main address.
        unsafe fn address(
            wallet: &Wallet,
            account_index: u32,
            address_index: u32,
        ) -> UniquePtr<CxxString>;

        /// Initialize the wallet by connecting to the specified remote node (daemon).
        #[allow(clippy::too_many_arguments)]
        unsafe fn init(
            self: Pin<&mut Wallet>,
            daemon_address: &CxxString,
            upper_transaction_size_limit: u64,
            daemon_username: &CxxString,
            daemon_password: &CxxString,
            use_ssl: bool,
            light_wallet: bool,
            proxy_address: &CxxString,
        ) -> bool;

        /// Refresh the wallet once.
        unsafe fn refresh(self: Pin<&mut Wallet>) -> bool;

        /// Start the background refresh thread (refreshes every 10 seconds).
        unsafe fn startRefresh(self: Pin<&mut Wallet>);

        /// Refresh the wallet asynchronously.
        unsafe fn refreshAsync(self: Pin<&mut Wallet>);

        /// Get the current blockchain height.
        unsafe fn blockChainHeight(self: &Wallet) -> u64;

        /// Get the daemon's blockchain height.
        unsafe fn daemonBlockChainHeight(self: &Wallet) -> u64;

        /// Check if wallet was ever synchronized.
        unsafe fn synchronized(self: &Wallet) -> bool;

        /// Get the status of a pending transaction.
        unsafe fn status(self: &PendingTransaction) -> i32;
        
        /// Get the total balance across all accounts in atomic units.
        unsafe fn balanceAll(self: &Wallet) -> u64;

        /// Get the total unlocked balance across all accounts in atomic units.
        unsafe fn unlockedBalanceAll(self: &Wallet) -> u64;

        unsafe fn setAllowMismatchedDaemonVersion(self: Pin<&mut Wallet>, allow_mismatch: bool);
    }
}
