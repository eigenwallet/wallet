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
        include!("wallet/api/wallet2_api.h");
        include!("bridge.h");

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

        /// A wallet listener.
        ///
        /// Can be attached to a wallet and will get notified upon specific events.
        type WalletListener;

        /// Get the wallet manager.
        fn getWalletManager() -> *mut WalletManager;

        /// Create a new wallet.
        fn createWallet(
            self: Pin<&mut WalletManager>,
            path: &CxxString,
            password: &CxxString,
            language: &CxxString,
            network_type: NetworkType,
            kdf_rounds: u64,
        ) -> *mut Wallet;

        /// Recover a wallet from a mnemonic seed (electrum seed).
        #[allow(clippy::too_many_arguments)]
        fn recoveryWallet(
            self: Pin<&mut WalletManager>,
            path: &CxxString,
            password: &CxxString,
            mnemonic: &CxxString,
            network_type: NetworkType,
            restore_height: u64,
            kdf_rounds: u64,
            seed_offset: &CxxString,
        ) -> *mut Wallet;

        ///virtual Wallet * openWallet(const std::string &path, const std::string &password, NetworkType nettype, uint64_t kdf_rounds = 1, WalletListener * listener = nullptr) = 0;
        unsafe fn openWallet(
            self: Pin<&mut WalletManager>,
            path: &CxxString,
            password: &CxxString,
            network_type: NetworkType,
            kdf_rounds: u64,
            listener: *mut WalletListener,
        ) -> *mut Wallet;

        /// Check whether a wallet exists at the given path.
        fn walletExists(self: Pin<&mut WalletManager>, path: &CxxString) -> bool;

        /// Get the current blockchain height.
        fn blockchainHeight(self: Pin<&mut WalletManager>) -> u64;

        /// Get the current error string of the wallet manager.
        fn walletManagerErrorString(manager: Pin<&mut WalletManager>) -> UniquePtr<CxxString>;

        /// Set the address of the remote node ("daemon").
        fn setDaemonAddress(self: Pin<&mut WalletManager>, address: &CxxString);

        /// Check if the wallet manager is connected to the configured daemon.
        ///
        /// # Safety
        ///
        /// - `version` must be a valid pointer to a `u32` or null.
        unsafe fn connected(self: Pin<&mut WalletManager>, version: *mut u32) -> bool;

        /// Get the status of the wallet and an error string if there is one.
        fn statusWithErrorString(
            self: &Wallet,
            status: &mut i32,
            error_string: Pin<&mut CxxString>,
        );

        /// Address for the given account and address index.
        /// address(0, 0) is the main address.
        fn address(wallet: &Wallet, account_index: u32, address_index: u32)
            -> UniquePtr<CxxString>;

        /// Initialize the wallet by connecting to the specified remote node (daemon).
        #[allow(clippy::too_many_arguments)]
        fn init(
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
        fn refresh(self: Pin<&mut Wallet>) -> bool;

        /// Start the background refresh thread (refreshes every 10 seconds).
        fn startRefresh(self: Pin<&mut Wallet>);

        /// Refresh the wallet asynchronously.
        fn refreshAsync(self: Pin<&mut Wallet>);

        /// Get the current blockchain height.
        fn blockChainHeight(self: &Wallet) -> u64;

        /// Get the daemon's blockchain height.
        fn daemonBlockChainTargetHeight(self: &Wallet) -> u64;

        /// Check if wallet was ever synchronized.
        fn synchronized(self: &Wallet) -> bool;

        /// Get the status of a pending transaction.
        fn status(self: &PendingTransaction) -> i32;

        /// Get the total balance across all accounts in atomic units (piconero).
        fn balanceAll(self: &Wallet) -> u64;

        /// Get the total unlocked balance across all accounts in atomic units (piconero).
        fn unlockedBalanceAll(self: &Wallet) -> u64;

        /// Set whether to allow mismatched daemon versions.
        fn setAllowMismatchedDaemonVersion(self: Pin<&mut Wallet>, allow_mismatch: bool);
    }
}

impl From<monero::Network> for ffi::NetworkType {
    fn from(network: monero::Network) -> Self {
        match network {
            monero::Network::Mainnet => ffi::NetworkType::Mainnet,
            monero::Network::Testnet => ffi::NetworkType::Testnet,
            monero::Network::Stagenet => ffi::NetworkType::Stagenet,
        }
    }
}

impl From<ffi::NetworkType> for monero::Network {
    fn from(network: ffi::NetworkType) -> Self {
        match network {
            ffi::NetworkType::Mainnet => monero::Network::Mainnet,
            ffi::NetworkType::Testnet => monero::Network::Testnet,
            ffi::NetworkType::Stagenet => monero::Network::Stagenet,
            // We have to include this path due to the way C++ translates the enum.
            // The enum only has these 3 values.
            _ => unreachable!(
                "There should be no other network type besides Mainnet, Testnet, and Stagenet"
            ),
        }
    }
}
