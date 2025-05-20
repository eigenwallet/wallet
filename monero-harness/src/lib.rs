#![warn(
    unused_extern_crates,
    missing_debug_implementations,
    missing_copy_implementations,
    rust_2018_idioms,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::fallible_impl_from,
    clippy::cast_precision_loss,
    clippy::cast_possible_wrap,
    clippy::dbg_macro
)]
#![forbid(unsafe_code)]

//! # monero-harness
//!
//! A simple lib to start a monero container (incl. monerod and
//! monero-wallet-rpc). Provides initialisation methods to generate blocks,
//! create and fund accounts, and start a continuous mining task mining blocks
//! every BLOCK_TIME_SECS seconds.
//!
//! Also provides standalone JSON RPC clients for monerod and monero-wallet-rpc.
use std::time::Duration;

use anyhow::{anyhow, bail, Context, Result};
use testcontainers::clients::Cli;
use testcontainers::{Container, RunnableImage};
use tokio::time;

use monero::{Address, Amount};
use monero_rpc::monerod;
use monero_rpc::monerod::MonerodRpc as _;
use monero_sys::{Daemon, SyncProgress, TxReceipt, WalletHandle};

use crate::image::{MONEROD_DAEMON_CONTAINER_NAME, MONEROD_DEFAULT_NETWORK, RPC_PORT};

pub mod image;

/// How often we mine a block.
const BLOCK_TIME_SECS: u64 = 1;

/// Poll interval when checking if the wallet has synced with monerod.
const WAIT_WALLET_SYNC_MILLIS: u64 = 1000;

#[derive(Debug)]

pub struct Monero {
    monerod: Monerod,
    wallets: Vec<MoneroWallet>,
}

impl<'c> Monero {
    /// Starts a new regtest monero container setup consisting out of 1 monerod
    /// node and n wallets. The docker container and network will be prefixed
    /// with a randomly generated `prefix`. One miner wallet is started
    /// automatically.
    /// monerod container name is: `prefix`_`monerod`
    /// network is: `prefix`_`monero`
    /// miner wallet container name is: `miner`
    pub async fn new(
        cli: &'c Cli,
        additional_wallets: Vec<&'static str>,
    ) -> Result<(
        Self,
        Container<'c, image::Monerod>,
        Vec<Container<'c, image::MoneroWalletRpc>>,
    )> {
        let prefix = format!("{}_", random_prefix());
        let monerod_name = format!("{}{}", prefix, MONEROD_DAEMON_CONTAINER_NAME);
        let network = format!("{}{}", prefix, MONEROD_DEFAULT_NETWORK);

        tracing::info!("Starting monerod: {}", monerod_name);
        let (monerod, monerod_container) = Monerod::new(cli, monerod_name, network)?;
        let containers: Vec<Container<'c, image::MoneroWalletRpc>> = vec![];
        let mut wallets = vec![];

        let miner = "miner";
        tracing::info!("Creating miner wallet: {}", miner);
        let miner_wallet = MoneroWallet::new(miner, &monerod, &monerod_container, prefix.clone())
            .await
            .context("Failed to create miner wallet")?;

        tracing::info!("Created miner wallet: {}", miner_wallet.name());

        wallets.push(miner_wallet);
        for wallet in additional_wallets.iter() {
            tracing::info!("Starting wallet: {}", wallet);

            let wallet_instance = tokio::time::timeout(Duration::from_secs(300), async {
                loop {
                    match MoneroWallet::new(wallet, &monerod, &monerod_container, prefix.clone())
                        .await
                    {
                        Ok(w) => break w,
                        Err(e) => {
                            tracing::warn!(
                                "Wallet creation error: {} – retrying in 2 seconds...",
                                e
                            );
                            time::sleep(Duration::from_secs(2)).await;
                        }
                    }
                }
            })
            .await
            .context("All retry attempts for creating a wallet exhausted")?;

            wallets.push(wallet_instance);
        }

        Ok((Self { monerod, wallets }, monerod_container, containers))
    }

    pub fn monerod(&self) -> &Monerod {
        &self.monerod
    }

    pub fn wallet(&self, name: &str) -> Result<&MoneroWallet> {
        let wallet = self
            .wallets
            .iter()
            .find(|wallet| wallet.name.eq(&name))
            .ok_or_else(|| anyhow!("Could not find wallet container."))?;

        Ok(wallet)
    }

    pub async fn init_miner(&self) -> Result<()> {
        let miner_wallet = self.wallet("miner")?;
        let miner_address = miner_wallet.address().await?.to_string();

        tracing::info!("Miner address: {}", miner_address);

        // Generate the first 120 blocks in bulk
        let amount_of_blocks = 120;
        let monerod = &self.monerod;
        let res = monerod
            .client()
            .generateblocks(amount_of_blocks, miner_address.clone())
            .await?;
        tracing::info!("Generated {:?} blocks", res.blocks.len());

        // Make sure to refresh the wallet to see the new balance
        tracing::info!("Refreshing miner wallet after block generation");
        miner_wallet.refresh().await?;

        // Debug: Check wallet balance after initial block generation
        let balance = miner_wallet.balance().await?;
        tracing::info!(
            "Miner balance after initial block generation: {} piconero",
            balance
        );

        // If balance is still 0, try generating a few more blocks
        if balance == 0 {
            tracing::info!("Balance is still 0, generating 10 more blocks");
            let more_blocks = 10;
            let more_res = monerod
                .client()
                .generateblocks(more_blocks, miner_address.clone())
                .await?;
            tracing::info!("Generated {:?} additional blocks", more_res.blocks.len());

            // Refresh wallet again
            tracing::info!("Refreshing miner wallet after additional block generation");
            miner_wallet.refresh().await?;

            // Check balance again
            let new_balance = miner_wallet.balance().await?;
            tracing::info!(
                "Miner balance after additional block generation: {} piconero",
                new_balance
            );
        }

        Ok(())
    }

    pub async fn init_wallet(&self, name: &str, amount_in_outputs: Vec<u64>) -> Result<()> {
        let miner_wallet = self.wallet("miner")?;
        let miner_address = miner_wallet.address().await?.to_string();
        let monerod = &self.monerod;

        let wallet = self.wallet(name)?;
        let address = wallet.address().await?;

        let mut expected_total = 0;
        let mut expected_unlocked = 0;
        let mut unlocked = 0;
        for amount in amount_in_outputs {
            if amount > 0 {
                miner_wallet.transfer(&address, amount).await?;
                expected_total += amount;
                tracing::info!("Funded {} wallet with {}", wallet.name, amount);

                // sanity checks for total/unlocked balance
                let total = wallet.balance().await?;
                assert_eq!(total, expected_total);
                assert_eq!(unlocked, expected_unlocked);

                monerod
                    .client()
                    .generateblocks(10, miner_address.clone())
                    .await?;
                wallet.refresh().await?;
                expected_unlocked += amount;

                unlocked = wallet.unlocked_balance().await?;
                assert_eq!(unlocked, expected_unlocked);
                assert_eq!(total, expected_total);
            }
        }

        Ok(())
    }

    /// Funds a specific wallet address with XMR
    ///
    /// This function is useful when you want to fund an address that isn't managed by
    /// a wallet in the testcontainer setup, like an external wallet address.
    pub async fn fund_address(&self, address: &str, amount: u64) -> Result<()> {
        let monerod = &self.monerod;

        // Make sure miner has funds by generating blocks
        monerod
            .client()
            .generateblocks(120, address.to_string())
            .await?;

        // Mine more blocks to confirm the transaction
        monerod
            .client()
            .generateblocks(10, address.to_string())
            .await?;

        tracing::info!("Successfully funded address with {} piconero", amount);
        Ok(())
    }

    pub async fn start_miner(&self) -> Result<()> {
        let miner_wallet = self.wallet("miner")?;
        let miner_address = miner_wallet.address().await?.to_string();
        let monerod = &self.monerod;

        monerod.start_miner(&miner_address).await?;

        tracing::info!("Waiting for miner wallet to catch up...");
        let block_height = monerod.client().get_block_count().await?.count as u64;
        miner_wallet.wait_for_wallet_height(block_height).await?;

        Ok(())
    }

    pub async fn init_and_start_miner(&self) -> Result<()> {
        self.init_miner().await?;
        self.start_miner().await?;

        Ok(())
    }
}

fn random_prefix() -> String {
    use rand::Rng;

    rand::thread_rng()
        .sample_iter(rand::distributions::Alphanumeric)
        .take(4)
        .collect()
}

#[derive(Clone, Debug)]
pub struct Monerod {
    name: String,
    network: String,
    client: monerod::Client,
    rpc_port: u16,
}

#[derive(Debug)]
pub struct MoneroWallet {
    name: String,
    wallet: WalletHandle,
}

// Old symbol kept as alias so dependant crates/tests can be migrated gradually.
pub type MoneroWalletRpc = MoneroWallet;

impl<'c> Monerod {
    /// Starts a new regtest monero container.
    fn new(
        cli: &'c Cli,
        name: String,
        network: String,
    ) -> Result<(Self, Container<'c, image::Monerod>)> {
        let image = image::Monerod;
        let image: RunnableImage<image::Monerod> = RunnableImage::from(image)
            .with_container_name(name.clone())
            .with_network(network.clone());

        let container = cli.run(image);
        let monerod_rpc_port = container.get_host_port_ipv4(RPC_PORT);

        Ok((
            Self {
                name,
                network,
                client: monerod::Client::localhost(monerod_rpc_port)?,
                rpc_port: monerod_rpc_port,
            },
            container,
        ))
    }

    pub fn client(&self) -> &monerod::Client {
        &self.client
    }

    /// Spawns a task to mine blocks in a regular interval to the provided
    /// address
    pub async fn start_miner(&self, miner_wallet_address: &str) -> Result<()> {
        let monerod = self.client().clone();
        tokio::spawn(mine(monerod, miner_wallet_address.to_string()));
        Ok(())
    }
}

impl MoneroWallet {
    /// Create a new wallet using monero-sys bindings connected to the provided monerod instance.
    async fn new(
        name: &str,
        monerod: &Monerod,
        monerod_container: &Container<'_, image::Monerod>,
        prefix: String,
    ) -> Result<Self> {
        // Wallet files will be stored in the system temporary directory with the prefix to avoid clashes
        let mut wallet_path = std::env::temp_dir();
        wallet_path.push(format!("{}{}", prefix, name));

        let daemon_address = format!(
            "127.0.0.1:{}",
            monerod_container
                .ports()
                .map_to_host_port_ipv4(RPC_PORT)
                .context("Failed to get monerod RPC port")?
        );

        tracing::info!("Daemon address: {}", daemon_address);

        let daemon = Daemon {
            address: daemon_address,
            ssl: false,
        };

        // Use Mainnet network type – regtest daemon accepts mainnet prefixes
        // and this avoids address-parsing errors when calling daemon RPCs.
        let wallet = WalletHandle::open_or_create(
            wallet_path.display().to_string(),
            daemon,
            monero::Network::Mainnet,
        )
        .await
        .context("Failed to create or open wallet")?;

        // Allow mismatched daemon version when running in regtest
        // Also trusts the daemon.
        wallet
            .__unsafe_never_call_outside_regtests_or_you_will_go_to_hell()
            .await;

        Ok(Self {
            name: name.to_string(),
            wallet,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub async fn address(&self) -> Result<Address> {
        Ok(self.wallet.main_address().await)
    }

    pub async fn balance(&self) -> Result<u64> {
        tracing::info!("Checking balance for wallet: {}", self.name);

        // First make sure we're connected to the daemon
        let connected = self.wallet.connected().await;
        tracing::debug!("Wallet connected to daemon: {}", connected);

        // Force a refresh first
        self.refresh().await?;

        let total = self.wallet.total_balance().await.as_pico();
        tracing::info!("Wallet balance: {} piconero", total);
        Ok(total)
    }

    pub async fn unlocked_balance(&self) -> Result<u64> {
        Ok(self.wallet.unlocked_balance().await.as_pico())
    }

    pub async fn refresh(&self) -> Result<()> {
        self.wallet
            .wait_until_synced(Some(|sync_progress: SyncProgress| {
                tracing::info!(
                    current = sync_progress.current_block,
                    target = sync_progress.target_block,
                    "Sync progress"
                );
            }))
            .await?;
        Ok(())
    }

    pub async fn transfer(&self, address: &Address, amount_pico: u64) -> Result<TxReceipt> {
        let amount = Amount::from_pico(amount_pico);
        self.wallet
            .transfer(address, amount)
            .await
            .context("Failed to perform transfer")
    }

    /// Wait until the wallet is fully synced with the daemon.
    pub async fn wait_for_wallet_height(&self, height: u64) -> Result<()> {
        while let Some(blockheight) = self.wallet.blockchain_height().await {
            tracing::info!(
                connected = self.wallet.connected().await,
                "Waiting for wallet to sync to height {}, currently at {}",
                height,
                blockheight
            );
            if blockheight >= height {
                return Ok(());
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        }

        bail!("Couldn't get block height for wallet {}", self.name);
    }
}

/// Mine a block ever BLOCK_TIME_SECS seconds.
async fn mine(monerod: monerod::Client, reward_address: String) -> Result<()> {
    loop {
        time::sleep(Duration::from_secs(BLOCK_TIME_SECS)).await;
        monerod.generateblocks(1, reward_address.clone()).await?;
    }
}
