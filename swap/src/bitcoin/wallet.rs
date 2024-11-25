use crate::bitcoin::{Address, Amount, Transaction};
use anyhow::Ok;
use anyhow::{anyhow, bail, Context, Result};
use bdk_electrum::electrum_client::{ElectrumApi, GetHistoryRes};
use bdk_wallet::bitcoin::FeeRate;
use bdk_wallet::bitcoin::Network;
use bdk_wallet::descriptor::IntoWalletDescriptor;
use bdk_wallet::export::FullyNodedExport;
use bdk_wallet::psbt::PsbtUtils;
use bdk_wallet::{rusqlite, KeychainKind};
use bdk_wallet::PersistedWallet;
use bdk_wallet::SignOptions;
use bdk_wallet::WalletPersister;
use bitcoin::ScriptBuf;
use bitcoin::{psbt::Psbt as PartiallySignedTransaction, Txid};
use rust_decimal::prelude::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Debug;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;
use tokio::sync::{watch, Mutex};
use tracing::{debug_span, Instrument};

use super::BlockHeight;

/// Assuming we add a spread of 3% we don't want to pay more than 3% of the
/// amount for tx fees.
const MAX_RELATIVE_TX_FEE: Decimal = dec!(0.03);
const MAX_ABSOLUTE_TX_FEE: Decimal = dec!(100_000);
const DUST_AMOUNT: Amount = Amount::from_sat(546);

const WALLET_NEW: &str = "wallet-new";

/// This is our wrapper around a bdk wallet and a corresponding
/// bdk electrum client.
/// It unifies all the functionality we need when interacting
/// with the bitcoin network.
///
/// This wallet is generic over the persister, which may be a
/// rusqlite connection, or an in-memory database, or something else.
#[derive(Clone)]
pub struct Wallet<Persister = rusqlite::Connection> {
    /// The wallet, which is persisted to the disk.
    wallet: Arc<Mutex<PersistedWallet<Persister>>>,
    /// The database connection used to persist the wallet.
    persister: Arc<Mutex<Persister>>,
    /// The electrum client.
    client: Arc<Mutex<Client>>,
    /// The network this wallet is on.
    network: Network,
    /// The number of confirmations (blocks) we require for a transaction
    /// to be considered final.
    ///
    /// Usually set to 1.
    finality_confirmations: u32,
    /// We want our transactions to be confirmed after this many blocks
    /// (used for fee estimation).
    target_block: usize,
}

/// This is our wrapper around a bdk electrum client.
pub struct Client {
    /// The underlying bdk electrum client.
    electrum: bdk_electrum::electrum_client::Client,
    /// The history of transactions for each script.
    script_history: BTreeMap<ScriptBuf, Vec<GetHistoryRes>>,
    /// The subscriptions to the status of transactions.
    subscriptions: HashMap<(Txid, ScriptBuf), Subscription>,
    /// The time of the last sync.
    last_sync: Instant,
    /// How often we sync with the server.
    sync_interval: Duration,
    /// The height of the latest block we know about.
    latest_block_height: BlockHeight,
}

/// A subscription to the status of a given transaction
/// that can be used to wait for the transaction to be confirmed.
#[derive(Debug, Clone)]
pub struct Subscription {
    /// A receiver used to await updates to the status of the transaction.
    receiver: watch::Receiver<ScriptStatus>,
    /// The number of confirmations we require for a transaction to be considered final.
    finality_confirmations: u32,
    /// The transaction ID we are subscribing to.
    txid: Txid,
}

/// The possible statuses of a script.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ScriptStatus {
    Unseen,
    InMempool,
    Confirmed(Confirmed),
    Retrying,
}

/// The status of a confirmed transaction.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Confirmed {
    /// The depth of this transaction within the blockchain.
    ///
    /// Zero if the transaction is included in the latest block.
    depth: u32,
}

/// Defines a watchable transaction.
///
/// For a transaction to be watchable, we need to know two things: Its
/// transaction ID and the specific output script that is going to change.
/// A transaction can obviously have multiple outputs but our protocol purposes,
/// we are usually interested in a specific one.
pub trait Watchable {
    /// The transaction ID.
    fn id(&self) -> Txid;
    /// The script of the output we are interested in.
    fn script(&self) -> ScriptBuf;
    /// Convenience method to get both the script and the txid.
    fn script_and_txid(&self) -> (ScriptBuf, Txid) {
        (self.script(), self.id())
    }
}

/// An object that can estimate fee rates and minimum relay fees.
pub trait EstimateFeeRate {
    /// Estimate the fee rate for a given target block.
    fn estimate_feerate(&self, target_block: usize) -> Result<FeeRate>;
    /// Get the minimum relay fee.
    fn min_relay_fee(&self) -> Result<bitcoin::Amount>;
}

impl<Persister> Wallet<Persister>
where
    Persister: WalletPersister + Sized,
    <Persister as WalletPersister>::Error: std::error::Error + Send + Sync + 'static,
{
    /// Create a new wallet. The descriptor is the wallet descriptor for the external and internal keys.
    /// 
    /// A persistor is the database connection used to persist the wallet,
    /// mostly a sqlite connection.
    pub async fn new(
        descriptor: impl IntoWalletDescriptor + Send + Clone + 'static,
        network: Network,
        electrum_rpc_url: &str,
        mut persister: Persister,
        finality_confirmations: u32,
        target_block: usize,
        sync_interval: Duration,
    ) -> Result<Wallet<Persister>> {
        let wallet = bdk_wallet::Wallet::create(descriptor.clone(), descriptor.clone())
            .network(network)
            .create_wallet(&mut persister)
            .context("Failed to create wallet")?;

        let client = Client::new(electrum_rpc_url, sync_interval)?;
        
        Ok(Self {
            wallet: Arc::new(Mutex::new(wallet)),
            client: Arc::new(Mutex::new(client)),
            network,
            finality_confirmations,
            target_block,
            persister: Arc::new(Mutex::new(persister)),
        })
    }

    /// Broadcast the given transaction to the network and emit a tracing statement
    /// if done so successfully.
    ///
    /// Returns the transaction ID and a future for when the transaction meets
    /// the configured finality confirmations.
    pub async fn broadcast(
        &self,
        transaction: Transaction,
        kind: &str,
    ) -> Result<(Txid, Subscription)> {
        let txid = transaction.compute_txid();

        // to watch for confirmations, watching a single output is enough
        let subscription = self
            .subscribe_to((txid, transaction.output[0].script_pubkey.clone()))
            .await;

        let client = self.client.lock().await;
        client
            .transaction_broadcast(&transaction)
            .with_context(|| {
                format!("Failed to broadcast Bitcoin {} transaction {}", kind, txid)
            })?;

        tracing::info!(%txid, %kind, "Published Bitcoin transaction");

        Ok((txid, subscription))
    }

    pub async fn get_raw_transaction(&self, txid: Txid) -> Result<Transaction> {
        self.get_tx(txid)
            .await?
            .with_context(|| format!("Could not get raw tx with id: {}", txid))
    }

    pub async fn status_of_script<T>(&self, tx: &T) -> Result<ScriptStatus>
    where
        T: Watchable,
    {
        self.client.lock().await.status_of_script(tx)
    }

    pub async fn subscribe_to(&self, tx: impl Watchable + Send + 'static) -> Subscription {
        let txid = tx.id();
        let script = tx.script();

        let sub = self
            .client
            .lock()
            .await
            .subscriptions
            .entry((txid, script.clone()))
            .or_insert_with(|| {
                let (sender, receiver) = watch::channel(ScriptStatus::Unseen);
                let client = self.client.clone();

                tokio::spawn(async move {
                    let mut last_status = None;

                    loop {
                        let new_status = client.lock()
                            .await
                            .status_of_script(&tx)
                            .unwrap_or_else(|error| {
                                tracing::warn!(%txid, "Failed to get status of script: {:#}", error);
                                ScriptStatus::Retrying
                            });

                        if new_status != ScriptStatus::Retrying
                        {
                            last_status = Some(trace_status_change(txid, last_status, new_status));

                            let all_receivers_gone = sender.send(new_status).is_err();

                            if all_receivers_gone {
                                tracing::debug!(%txid, "All receivers gone, removing subscription");
                                client.lock().await.subscriptions.remove(&(txid, script));
                                return;
                            }
                        }

                        tokio::time::sleep(Duration::from_secs(5)).await;
                    }
                }.instrument(debug_span!("BitcoinWalletSubscription")));

                Subscription {
                    receiver,
                    finality_confirmations: self.finality_confirmations,
                    txid,
                }
            })
            .clone();

        sub
    }

    pub async fn wallet_export(&self, role: &str) -> Result<FullyNodedExport> {
        let wallet = self.wallet.lock().await;
        match bdk_wallet::export::FullyNodedExport::export_wallet(
            &*wallet,
            &format!("{}-{}", role, self.network),
            true,
        ) {
            Result::Ok(wallet_export) => Ok(wallet_export),
            Err(err_msg) => Err(anyhow::Error::msg(err_msg)),
        }
    }

    pub async fn sign_and_finalize(&self, mut psbt: bitcoin::psbt::Psbt) -> Result<Transaction> {
        let finalized = self
            .wallet
            .lock()
            .await
            .sign(&mut psbt, SignOptions::default())?;

        if !finalized {
            bail!("PSBT is not finalized")
        }

        let tx = psbt.extract_tx();

        Ok(tx?)
    }

    /// Returns the total Bitcoin balance, which includes pending funds
    pub async fn balance(&self) -> Result<Amount> {
        Ok(self
            .wallet
            .lock()
            .await
            .balance()
            .total())
    }

    /// Reveals the next address from the wallet.
    pub async fn new_address(&self) -> Result<Address> {
        let mut wallet = self.wallet.lock().await;

        let address = wallet
            .reveal_next_address(KeychainKind::External)
            .address;

        // Important: persist that we revealed a new address. 
        // Otherwise the wallet might reuse it (bad).
        let mut persister = self.persister.lock().await;
        wallet.persist(&mut persister)?;

        Ok(address)
    }

    /// Calculate the fee for a given transaction.
    /// 
    /// Will fail if the transaction inputs are not owned by this wallet.
    pub async fn transaction_fee(&self, txid: Txid) -> Result<Amount> {
        let transaction = self.get_tx(txid).await?
            .context("Could not find tx in bdk wallet when trying to determine fees")?;
        let fee = self.wallet.lock().await.calculate_fee(&transaction)?;

        Ok(fee)
    }

    /// Builds a partially signed transaction
    ///
    /// Ensures that the address script is at output index `0`
    /// for the partially signed transaction.
    pub async fn send_to_address(
        &self,
        address: Address,
        amount: Amount,
        change_override: Option<Address>,
    ) -> Result<PartiallySignedTransaction> {
        if self.network != address.network {
            bail!("Cannot build PSBT because network of given address is {} but wallet is on network {}", address.network, self.network);
        }

        if let Some(change) = change_override.as_ref() {
            if self.network != change.network {
                bail!("Cannot build PSBT because network of given address is {} but wallet is on network {}", change.network, self.network);
            }
        }

        let wallet = self.wallet.lock().await;
        let client = self.client.lock().await;
        let fee_rate = client.estimate_feerate(self.target_block)?;
        let script = address.script_pubkey();

        let mut tx_builder = wallet.build_tx();
        tx_builder.add_recipient(script.clone(), amount);
        tx_builder.fee_rate(fee_rate);
        let psbt = tx_builder.finish()?;
        let mut psbt: PartiallySignedTransaction = psbt;

        match psbt.unsigned_tx.output.as_mut_slice() {
            // our primary output is the 2nd one? reverse the vectors
            [_, second_txout] if second_txout.script_pubkey == script => {
                psbt.outputs.reverse();
                psbt.unsigned_tx.output.reverse();
            }
            [first_txout, _] if first_txout.script_pubkey == script => {
                // no need to do anything
            }
            [_] => {
                // single output, no need do anything
            }
            _ => bail!("Unexpected transaction layout"),
        }

        if let ([_, change], [_, psbt_output], Some(change_override)) = (
            &mut psbt.unsigned_tx.output.as_mut_slice(),
            &mut psbt.outputs.as_mut_slice(),
            change_override,
        ) {
            change.script_pubkey = change_override.script_pubkey();
            // Might be populated based on the previously set change address, but for the
            // overwrite we don't know unless we ask the user for more information.
            psbt_output.bip32_derivation.clear();
        }

        Ok(psbt)
    }

    /// Calculates the maximum "giveable" amount of this wallet.
    ///
    /// We define this as the maximum amount we can pay to a single output,
    /// already accounting for the fees we need to spend to get the
    /// transaction confirmed.
    pub async fn max_giveable(&self, locking_script_size: usize) -> Result<Amount> {
        let wallet = self.wallet.lock().await;
        let balance = wallet.balance();
        if balance.total() < DUST_AMOUNT {
            return Ok(Amount::ZERO);
        }
        let client = self.client.lock().await;
        let min_relay_fee = client.min_relay_fee()?;

        if balance.total() < min_relay_fee {
            return Ok(Amount::ZERO);
        }

        let fee_rate = client.estimate_feerate(self.target_block)?;

        let mut tx_builder = wallet.build_tx();

        let dummy_script = ScriptBuf::from(vec![0u8; locking_script_size]);
        tx_builder.drain_to(dummy_script);
        tx_builder.fee_rate(fee_rate);
        tx_builder.drain_wallet();

        let response = tx_builder.finish();
        match response {
            Result::Ok(psbt) => {
                let max_giveable = psbt.unsigned_tx.output.iter().map(|o| o.value).sum()
                    - psbt
                        .fee_amount()
                        .expect("fees are always present with Electrum backend");
                Ok(Amount::from_sat(max_giveable))
            }
            Err(e) => bail!("Failed to build transaction. {:#}", e),
        }
    }

    /// Estimate total tx fee for a pre-defined target block based on the
    /// transaction weight. The max fee cannot be more than MAX_PERCENTAGE_FEE
    /// of amount
    pub async fn estimate_fee(
        &self,
        weight: usize,
        transfer_amount: bitcoin::Amount,
    ) -> Result<bitcoin::Amount> {
        let client = self.client.lock().await;
        let fee_rate = client.estimate_feerate(self.target_block)?;
        let min_relay_fee = client.min_relay_fee()?;

        estimate_fee(weight, transfer_amount, fee_rate, min_relay_fee)
    }

    pub async fn get_tx(&self, txid: Txid) -> Result<Option<Transaction>> {
        let client = self.client.lock().await;
        let tx = client.get_tx(&txid)
            .context("Failed to get transaction from cache or Electrum server")?;

        Ok(tx)
    }

    pub async fn sync(&self) -> Result<()> {
        let client = self.client.lock().await;
        let blockchain = client.blockchain();
        let sync_opts = SyncOptions::default();
        self.wallet
            .lock()
            .await
            .sync(blockchain, sync_opts)
            .context("Failed to sync balance of Bitcoin wallet")?;

        Ok(())
    }
}

impl Client {
    /// Create a new client to this electrum server.
    fn new(electrum_rpc_url: &str, sync_interval: Duration) -> Result<Self> {
        let client = bdk_electrum::electrum_client::Client::new(electrum_rpc_url)?;
        Ok(Self {
            electrum: client,
            script_history: Default::default(),
            last_sync: Instant::now()
                .checked_sub(sync_interval)
                .ok_or(anyhow!("failed to set last sync time"))?,
            sync_interval,
            latest_block_height: BlockHeight::from(0),
            subscriptions: Default::default(),
        })
    }

    /// Broadcast a transaction to the network.
    pub fn transaction_broadcast(&self, transaction: &Transaction) -> Result<Txid> {
        self.electrum
            .transaction_broadcast(transaction)
            .context("Failed to broadcast transaction")
    }

    /// Get the status of a script.
    pub fn status_of_script(&self, script: &impl Watchable) -> Result<ScriptStatus> {
        let (script, txid) = script.script_and_txid();

        if !self.script_history.contains_key(&script) {
            self.script_history.insert(script.clone(), vec![]);

            // Immediately refetch the status of the script
            // when we first subscribe to it.
            self.update_status(true)?;
        } else {
            // Otherwise, don't force a refetch.
            self.update_status(false)?;
        }

        let history = self.script_history.entry(script).or_default();

        let history_of_tx: Vec<&GetHistoryRes> = history
            .iter()
            .filter(|entry| entry.tx_hash == txid)
            .collect();

        // Destructure history_of_tx into the last entry and the rest.
        let [rest @ .., last] = history_of_tx.as_slice() else {
            // If there is no history of the transaction, it is unseen.
            return Ok(ScriptStatus::Unseen);
        };
            
        // There should only be one entry per txid, we will ignore the rest
        if !rest.is_empty() {
            tracing::warn!(%txid, "Found multiple history entries for the same txid. Ignoring all but the last one.");
        }

        match last.height {
            // If the height is 0 or less, the transaction is still in the mempool.
            ..=0 => Ok(ScriptStatus::InMempool),
            // Otherwise, the transaction has been included in a block.
            height => Ok(ScriptStatus::Confirmed(
                Confirmed::from_inclusion_and_latest_block(
                    u32::try_from(last.height)?,
                    u32::from(self.latest_block_height)
                )
            ))
        }
    }
}

impl EstimateFeeRate for Client {
    fn estimate_feerate(&self, target_block: usize) -> Result<FeeRate> {
        // Get the fee rate in BTC/kvB
        let fee_rate_btc_kvb = self.electrum.estimate_fee(target_block)?;
        // Convert to sat/vb
        let fee_rate_sat_vb = Amount::from_btc(fee_rate_btc_kvb / 1_000.)?;

        Ok(FeeRate::from_sat_per_vb(fee_rate_sat_vb.to_sat())?)
    }

    fn min_relay_fee(&self) -> Result<bitcoin::Amount> {
        let relay_fee_btc = self.electrum.relay_fee()?;

        Ok(Amount::from_btc(relay_fee_btc))
    }
}

fn trace_status_change(txid: Txid, old: Option<ScriptStatus>, new: ScriptStatus) -> ScriptStatus {
    match (old, new) {
        (None, new_status) => {
            tracing::debug!(%txid, status = %new_status, "Found relevant Bitcoin transaction");
        }
        (Some(old_status), new_status) if old_status != new_status => {
            tracing::debug!(%txid, %new_status, %old_status, "Bitcoin transaction status changed");
        }
        _ => {}
    }

    new
}

impl Subscription {
    pub async fn wait_until_final(&self) -> Result<()> {
        let conf_target = self.finality_confirmations;
        let txid = self.txid;

        tracing::info!(%txid, required_confirmation=%conf_target, "Waiting for Bitcoin transaction finality");

        let mut seen_confirmations = 0;

        self.wait_until(|status| match status {
            ScriptStatus::Confirmed(inner) => {
                let confirmations = inner.confirmations();

                if confirmations > seen_confirmations {
                    tracing::info!(%txid,
                        seen_confirmations = %confirmations,
                        needed_confirmations = %conf_target,
                        "Waiting for Bitcoin transaction finality");
                    seen_confirmations = confirmations;
                }

                inner.meets_target(conf_target)
            }
            _ => false,
        })
        .await
    }

    pub async fn wait_until_seen(&self) -> Result<()> {
        self.wait_until(ScriptStatus::has_been_seen).await
    }

    pub async fn wait_until_confirmed_with<T>(&self, target: T) -> Result<()>
    where
        T: Into<u32>,
        T: Copy,
    {
        self.wait_until(|status| status.is_confirmed_with(target))
            .await
    }

    pub async fn wait_until(&self, mut predicate: impl FnMut(&ScriptStatus) -> bool) -> Result<()> {
        let mut receiver = self.receiver.clone();

        while !predicate(&receiver.borrow()) {
            receiver
                .changed()
                .await
                .context("Failed while waiting for next status update")?;
        }

        Ok(())
    }
}

pub mod old {
    //! This module contains some code for creating a bdk wallet from before the update.
    //! We need to keep this around to be able to migrate the wallet.

    use std::path::Path;
    use std::sync::Arc;

    use anyhow::{anyhow, bail, Result};
    use bdk::bitcoin::{util::bip32::ExtendedPrivKey, Network};
    use bdk::sled::Tree;
    use bdk::KeychainKind;
    use tokio::sync::Mutex;

    use crate::env;

    const WALLET: &str = "wallet";
    const WALLET_OLD: &str = "wallet-old";

    const SLED_TREE_NAME: &str = "default_tree";

    /// The is the old bdk wallet before the migration.
    /// We need to contruct it before migration to get the keys and revelation indeces.
    pub struct OldWallet<D = Tree> {
        wallet: Arc<Mutex<bdk::Wallet<D>>>,
        finality_confirmations: u32,
        network: Network,
        target_block: u16,
    }

    impl OldWallet {
        /// Create a new old wallet.
        pub async fn new(
            data_dir: impl AsRef<Path>,
            xprivkey: ExtendedPrivKey,
            env_config: env::Config,
            target_block: u16,
        ) -> Result<Self> {
            let data_dir = data_dir.as_ref();
            let wallet_dir = data_dir.join(WALLET);
            let database = bdk::sled::open(wallet_dir)?.open_tree(SLED_TREE_NAME)?;
            let network = env_config.bitcoin_network;

            // Convert bitcoin network to the bdk network type...
            let network = match network {
                bitcoin::Network::Bitcoin => bdk::bitcoin::Network::Bitcoin,
                bitcoin::Network::Testnet => bdk::bitcoin::Network::Testnet,
                bitcoin::Network::Regtest => bdk::bitcoin::Network::Regtest,
                bitcoin::Network::Signet => bdk::bitcoin::Network::Signet,
                _ => bail!("Unsupported network"),
            };

            let wallet = match bdk::Wallet::new(
                bdk::template::Bip84(xprivkey, KeychainKind::External),
                Some(bdk::template::Bip84(xprivkey, KeychainKind::Internal)),
                network,
                database,
            ) {
                Ok(w) => w,
                Err(bdk::Error::ChecksumMismatch) => Self::old_migrate(data_dir, xprivkey, network)?,
                err => err?,
            };

            let network = wallet.network();

            Ok(Self {
                wallet: Arc::new(Mutex::new(wallet)),
                finality_confirmations: env_config.bitcoin_finality_confirmations,
                network,
                target_block,
            })
        }

        /// Get a full export of the wallet including descriptors and blockheight.
        /// 
        /// TODO: Add internal and external derivation indices to the export.
        pub async fn export(&self, role: &str) -> Result<bdk_wallet::export::FullyNodedExport> {
            let wallet = self.wallet.lock().await;
            let export = bdk::wallet::export::FullyNodedExport::export_wallet(
                &wallet,
                &format!("{}-{}", role, self.network),
                true,
            )
            .map_err(|_| anyhow!("Failed to export old wallet descriptor"))?;

            // Because we upgraded bdk, the type id changed.
            // Thus, we serialize to json and then deserialize to the new type.
            let json = serde_json::to_string(&export)?;
            let export = serde_json::from_str::<bdk_wallet::export::FullyNodedExport>(&json)?;

            Ok(export)
        }

        /// Create a new old database for the wallet and rename the old old one.
        /// 
        /// Create a new database for the wallet and rename the old one.
        /// This is necessary when getting a ChecksumMismatch from a wallet
        /// created with an older version of BDK. Only affected Testnet wallets.
        // https://github.com/comit-network/xmr-btc-swap/issues/1182
        fn old_migrate(
            data_dir: &Path,
            xprivkey: ExtendedPrivKey,
            network: bdk::bitcoin::Network,
        ) -> Result<bdk::Wallet<Tree>> {
            let from = data_dir.join(WALLET);
            let to = data_dir.join(WALLET_OLD);
            std::fs::rename(from, to)?;

            let wallet_dir = data_dir.join(WALLET);
            let database = bdk::sled::open(wallet_dir)?.open_tree(SLED_TREE_NAME)?;

            let wallet = bdk::Wallet::new(
                bdk::template::Bip84(xprivkey, KeychainKind::External),
                Some(bdk::template::Bip84(xprivkey, KeychainKind::Internal)),
                network,
                database,
            )?;

            Ok(wallet)
        }
    }
}

fn estimate_fee(
    weight: usize,
    transfer_amount: Amount,
    fee_rate: FeeRate,
    min_relay_fee: Amount,
) -> Result<Amount> {
    if transfer_amount.to_sat() <= 546 {
        bail!("Amounts needs to be greater than Bitcoin dust amount.")
    }
    let fee_rate_svb = fee_rate.to_sat_per_vb_ceil();
    if fee_rate_svb <= 0 {
        bail!("Fee rate needs to be > 0")
    }
    if fee_rate_svb > 100_000_000 || min_relay_fee.to_sat() > 100_000_000 {
        bail!("A fee_rate or min_relay_fee of > 1BTC does not make sense")
    }

    let min_relay_fee = if min_relay_fee.to_sat() == 0 {
        // if min_relay_fee is 0 we don't fail, we just set it to 1 satoshi;
        Amount::ONE_SAT
    } else {
        min_relay_fee
    };

    let weight = Decimal::from(weight);
    let weight_factor = dec!(4.0);
    let fee_rate = Decimal::from_u64(fee_rate_svb).context("Failed to parse fee rate")?;

    let sats_per_vbyte = weight / weight_factor * fee_rate;

    tracing::debug!(
        %weight,
        %fee_rate,
        %sats_per_vbyte,
        "Estimated fee for transaction",
    );

    let transfer_amount = Decimal::from(transfer_amount.to_sat());
    let max_allowed_fee = transfer_amount * MAX_RELATIVE_TX_FEE;
    let min_relay_fee = Decimal::from(min_relay_fee.to_sat());

    let recommended_fee = if sats_per_vbyte < min_relay_fee {
        tracing::warn!(
            "Estimated fee of {} is smaller than the min relay fee, defaulting to min relay fee {}",
            sats_per_vbyte,
            min_relay_fee
        );
        min_relay_fee.to_u64()
    } else if sats_per_vbyte > max_allowed_fee && sats_per_vbyte > MAX_ABSOLUTE_TX_FEE {
        tracing::warn!(
            "Hard bound of transaction fees reached. Falling back to: {} sats",
            MAX_ABSOLUTE_TX_FEE
        );
        MAX_ABSOLUTE_TX_FEE.to_u64()
    } else if sats_per_vbyte > max_allowed_fee {
        tracing::warn!(
            "Relative bound of transaction fees reached. Falling back to: {} sats",
            max_allowed_fee
        );
        max_allowed_fee.to_u64()
    } else {
        sats_per_vbyte.to_u64()
    };
    let amount = recommended_fee
        .map(bitcoin::Amount::from_sat)
        .context("Could not estimate tranasction fee.")?;

    Ok(amount)
}

impl Watchable for (Txid, ScriptBuf) {
    fn id(&self) -> Txid {
        self.0
    }

    fn script(&self) -> ScriptBuf {
        self.1.clone()
    }
}

impl ScriptStatus {
    pub fn from_confirmations(confirmations: u32) -> Self {
        match confirmations {
            0 => Self::InMempool,
            confirmations => Self::Confirmed(Confirmed::new(confirmations - 1)),
        }
    }
}

impl Confirmed {
    pub fn new(depth: u32) -> Self {
        Self { depth }
    }

    /// Compute the depth of a transaction based on its inclusion height and the
    /// latest known block.
    ///
    /// Our information about the latest block might be outdated. To avoid an
    /// overflow, we make sure the depth is 0 in case the inclusion height
    /// exceeds our latest known block,
    pub fn from_inclusion_and_latest_block(inclusion_height: u32, latest_block: u32) -> Self {
        let depth = latest_block.saturating_sub(inclusion_height);

        Self { depth }
    }

    pub fn confirmations(&self) -> u32 {
        self.depth + 1
    }

    pub fn meets_target<T>(&self, target: T) -> bool
    where
        T: Into<u32>,
    {
        self.confirmations() >= target.into()
    }

    pub fn blocks_left_until<T>(&self, target: T) -> u32
    where
        T: Into<u32> + Copy,
    {
        if self.meets_target(target) {
            0
        } else {
            target.into() - self.confirmations()
        }
    }
}

impl ScriptStatus {
    /// Check if the script has any confirmations.
    pub fn is_confirmed(&self) -> bool {
        matches!(self, ScriptStatus::Confirmed(_))
    }

    /// Check if the script has met the given confirmation target.
    pub fn is_confirmed_with<T>(&self, target: T) -> bool
    where
        T: Into<u32>,
    {
        match self {
            ScriptStatus::Confirmed(inner) => inner.meets_target(target),
            _ => false,
        }
    }

    // Calculate the number of blocks left until the target is met.
    pub fn blocks_left_until<T>(&self, target: T) -> u32
    where
        T: Into<u32> + Copy,
    {
        match self {
            ScriptStatus::Confirmed(inner) => inner.blocks_left_until(target),
            _ => target.into(),
        }
    }

    pub fn has_been_seen(&self) -> bool {
        matches!(self, ScriptStatus::InMempool | ScriptStatus::Confirmed(_))
    }
}

impl fmt::Display for ScriptStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScriptStatus::Unseen => write!(f, "unseen"),
            ScriptStatus::InMempool => write!(f, "in mempool"),
            ScriptStatus::Retrying => write!(f, "retrying"),
            ScriptStatus::Confirmed(inner) => {
                write!(f, "confirmed with {} blocks", inner.confirmations())
            }
        }
    }
}

#[cfg(test)]
pub struct StaticFeeRate {
    fee_rate: FeeRate,
    min_relay_fee: bitcoin::Amount,
}

#[cfg(test)]
impl EstimateFeeRate for StaticFeeRate {
    fn estimate_feerate(&self, _target_block: u16) -> Result<FeeRate> {
        Ok(self.fee_rate)
    }

    fn min_relay_fee(&self) -> Result<bitcoin::Amount> {
        Ok(self.min_relay_fee)
    }
}

#[cfg(test)]
#[derive(Debug)]
pub struct WalletBuilder {
    utxo_amount: u64,
    sats_per_vb: f32,
    min_relay_fee_sats: u64,
    key: bdk::bitcoin::util::bip32::ExtendedPrivKey,
    num_utxos: u8,
}

#[cfg(test)]
impl WalletBuilder {
    /// Creates a new, funded wallet with sane default fees.
    ///
    /// Unless you are testing things related to fees, this is likely what you
    /// want.
    pub fn new(amount: u64) -> Self {
        WalletBuilder {
            utxo_amount: amount,
            sats_per_vb: 1.0,
            min_relay_fee_sats: 1000,
            key: "tprv8ZgxMBicQKsPeZRHk4rTG6orPS2CRNFX3njhUXx5vj9qGog5ZMH4uGReDWN5kCkY3jmWEtWause41CDvBRXD1shKknAMKxT99o9qUTRVC6m".parse().unwrap(),
            num_utxos: 1,
        }
    }

    pub fn with_zero_fees(self) -> Self {
        Self {
            sats_per_vb: 0.0,
            min_relay_fee_sats: 0,
            ..self
        }
    }

    pub fn with_fees(self, sats_per_vb: f32, min_relay_fee_sats: u64) -> Self {
        Self {
            sats_per_vb,
            min_relay_fee_sats,
            ..self
        }
    }

    pub fn with_key(self, key: bitcoin::util::bip32::ExtendedPrivKey) -> Self {
        Self { key, ..self }
    }

    pub fn with_num_utxos(self, number: u8) -> Self {
        Self {
            num_utxos: number,
            ..self
        }
    }

    pub fn build(self) -> Wallet<bdk::database::MemoryDatabase, StaticFeeRate> {
        use bdk::database::{BatchOperations, MemoryDatabase, SyncTime};
        use bdk::{testutils, BlockTime};

        let descriptors = testutils!(@descriptors (&format!("wpkh({}/*)", self.key)));

        let mut database = MemoryDatabase::new();

        for index in 0..self.num_utxos {
            bdk::populate_test_db!(
                &mut database,
                testutils! {
                    @tx ( (@external descriptors, index as u32) => self.utxo_amount ) (@confirmations 1)
                },
                Some(100)
            );
        }
        let block_time = bdk::BlockTime {
            height: 100,
            timestamp: 0,
        };
        let sync_time = SyncTime { block_time };
        database.set_sync_time(sync_time).unwrap();

        let wallet = bdk::Wallet::new(&descriptors.0, None, Network::Regtest, database).unwrap();

        Wallet {
            client: Arc::new(Mutex::new(StaticFeeRate {
                fee_rate: FeeRate::from_sat_per_vb(self.sats_per_vb),
                min_relay_fee: bitcoin::Amount::from_sat(self.min_relay_fee_sats),
            })),
            wallet: Arc::new(Mutex::new(wallet)),
            finality_confirmations: 1,
            network: Network::Regtest,
            target_block: 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bitcoin::{PublicKey, TxLock};
    use crate::tracing_ext::capture_logs;
    use bitcoin::hashes::Hash;
    use proptest::prelude::*;
    use tracing::level_filters::LevelFilter;

    #[test]
    fn given_depth_0_should_meet_confirmation_target_one() {
        let script = ScriptStatus::Confirmed(Confirmed { depth: 0 });

        let confirmed = script.is_confirmed_with(1_u32);

        assert!(confirmed)
    }

    #[test]
    fn given_confirmations_1_should_meet_confirmation_target_one() {
        let script = ScriptStatus::from_confirmations(1);

        let confirmed = script.is_confirmed_with(1_u32);

        assert!(confirmed)
    }

    #[test]
    fn given_inclusion_after_lastest_known_block_at_least_depth_0() {
        let included_in = 10;
        let latest_block = 9;

        let confirmed = Confirmed::from_inclusion_and_latest_block(included_in, latest_block);

        assert_eq!(confirmed.depth, 0)
    }

    #[test]
    fn given_depth_0_should_return_0_blocks_left_until_1() {
        let script = ScriptStatus::Confirmed(Confirmed { depth: 0 });

        let blocks_left = script.blocks_left_until(1_u32);

        assert_eq!(blocks_left, 0)
    }

    #[test]
    fn given_depth_1_should_return_0_blocks_left_until_1() {
        let script = ScriptStatus::Confirmed(Confirmed { depth: 1 });

        let blocks_left = script.blocks_left_until(1_u32);

        assert_eq!(blocks_left, 0)
    }

    #[test]
    fn given_depth_0_should_return_1_blocks_left_until_2() {
        let script = ScriptStatus::Confirmed(Confirmed { depth: 0 });

        let blocks_left = script.blocks_left_until(2_u32);

        assert_eq!(blocks_left, 1)
    }

    #[test]
    fn given_one_BTC_and_100k_sats_per_vb_fees_should_not_hit_max() {
        // 400 weight = 100 vbyte
        let weight = 400;
        let amount = bitcoin::Amount::from_sat(100_000_000);

        let sat_per_vb = 100.0;
        let fee_rate = FeeRate::from_sat_per_vb(sat_per_vb);

        let relay_fee = bitcoin::Amount::ONE_SAT;
        let is_fee = estimate_fee(weight, amount, fee_rate, relay_fee).unwrap();

        // weight / 4.0 *  sat_per_vb
        let should_fee = bitcoin::Amount::from_sat(10_000);
        assert_eq!(is_fee, should_fee);
    }

    #[test]
    fn given_1BTC_and_1_sat_per_vb_fees_and_100ksat_min_relay_fee_should_hit_min() {
        // 400 weight = 100 vbyte
        let weight = 400;
        let amount = bitcoin::Amount::from_sat(100_000_000);

        let sat_per_vb = 1.0;
        let fee_rate = FeeRate::from_sat_per_vb(sat_per_vb);

        let relay_fee = bitcoin::Amount::from_sat(100_000);
        let is_fee = estimate_fee(weight, amount, fee_rate, relay_fee).unwrap();

        // weight / 4.0 *  sat_per_vb would be smaller than relay fee hence we take min
        // relay fee
        let should_fee = bitcoin::Amount::from_sat(100_000);
        assert_eq!(is_fee, should_fee);
    }

    #[test]
    fn given_1mio_sat_and_1k_sats_per_vb_fees_should_hit_relative_max() {
        // 400 weight = 100 vbyte
        let weight = 400;
        let amount = bitcoin::Amount::from_sat(1_000_000);

        let sat_per_vb = 1_000.0;
        let fee_rate = FeeRate::from_sat_per_vb(sat_per_vb);

        let relay_fee = bitcoin::Amount::ONE_SAT;
        let is_fee = estimate_fee(weight, amount, fee_rate, relay_fee).unwrap();

        // weight / 4.0 *  sat_per_vb would be greater than 3% hence we take max
        // relative fee.
        let should_fee = bitcoin::Amount::from_sat(30_000);
        assert_eq!(is_fee, should_fee);
    }

    #[test]
    fn given_1BTC_and_4mio_sats_per_vb_fees_should_hit_total_max() {
        // even if we send 1BTC we don't want to pay 0.3BTC in fees. This would be
        // $1,650 at the moment.
        let weight = 400;
        let amount = bitcoin::Amount::from_sat(100_000_000);

        let sat_per_vb = 4_000_000.0;
        let fee_rate = FeeRate::from_sat_per_vb(sat_per_vb);

        let relay_fee = bitcoin::Amount::ONE_SAT;
        let is_fee = estimate_fee(weight, amount, fee_rate, relay_fee).unwrap();

        // weight / 4.0 *  sat_per_vb would be greater than 3% hence we take total
        // max allowed fee.
        assert_eq!(is_fee.to_sat(), MAX_ABSOLUTE_TX_FEE.to_u64().unwrap());
    }

    proptest! {
        #[test]
        fn given_randon_amount_random_fee_and_random_relay_rate_but_fix_weight_does_not_error(
            amount in 547u64..,
            sat_per_vb in 1.0f32..100_000_000.0f32,
            relay_fee in 0u64..100_000_000u64
        ) {
            let weight = 400;
            let amount = bitcoin::Amount::from_sat(amount);

            let fee_rate = FeeRate::from_sat_per_vb(sat_per_vb);

            let relay_fee = bitcoin::Amount::from_sat(relay_fee);
            let _is_fee = estimate_fee(weight, amount, fee_rate, relay_fee).unwrap();

        }
    }

    proptest! {
        #[test]
        fn given_amount_in_range_fix_fee_fix_relay_rate_fix_weight_fee_always_smaller_max(
            amount in 1u64..100_000_000,
        ) {
            let weight = 400;
            let amount = bitcoin::Amount::from_sat(amount);

            let sat_per_vb = 100.0;
            let fee_rate = FeeRate::from_sat_per_vb(sat_per_vb);

            let relay_fee = bitcoin::Amount::ONE_SAT;
            let is_fee = estimate_fee(weight, amount, fee_rate, relay_fee).unwrap();

            // weight / 4 * 1_000 is always lower than MAX_ABSOLUTE_TX_FEE
            assert!(is_fee.to_sat() < MAX_ABSOLUTE_TX_FEE.to_u64().unwrap());
        }
    }

    proptest! {
        #[test]
        fn given_amount_high_fix_fee_fix_relay_rate_fix_weight_fee_always_max(
            amount in 100_000_000u64..,
        ) {
            let weight = 400;
            let amount = bitcoin::Amount::from_sat(amount);

            let sat_per_vb = 1_000.0;
            let fee_rate = FeeRate::from_sat_per_vb(sat_per_vb);

            let relay_fee = bitcoin::Amount::ONE_SAT;
            let is_fee = estimate_fee(weight, amount, fee_rate, relay_fee).unwrap();

            // weight / 4 * 1_000  is always higher than MAX_ABSOLUTE_TX_FEE
            assert!(is_fee.to_sat() >= MAX_ABSOLUTE_TX_FEE.to_u64().unwrap());
        }
    }

    proptest! {
        #[test]
        fn given_fee_above_max_should_always_errors(
            sat_per_vb in 100_000_000.0f32..,
        ) {
            let weight = 400;
            let amount = bitcoin::Amount::from_sat(547u64);

            let fee_rate = FeeRate::from_sat_per_vb(sat_per_vb);

            let relay_fee = bitcoin::Amount::from_sat(1);
            assert!(estimate_fee(weight, amount, fee_rate, relay_fee).is_err());

        }
    }

    proptest! {
        #[test]
        fn given_relay_fee_above_max_should_always_errors(
            relay_fee in 100_000_000u64..
        ) {
            let weight = 400;
            let amount = bitcoin::Amount::from_sat(547u64);

            let fee_rate = FeeRate::from_sat_per_vb(1.0);

            let relay_fee = bitcoin::Amount::from_sat(relay_fee);
            assert!(estimate_fee(weight, amount, fee_rate, relay_fee).is_err());
        }
    }

    #[tokio::test]
    async fn given_no_balance_returns_amount_0() {
        let wallet = WalletBuilder::new(0).with_fees(1.0, 1).build();
        let amount = wallet.max_giveable(TxLock::script_size()).await.unwrap();

        assert_eq!(amount, Amount::ZERO);
    }

    #[tokio::test]
    async fn given_balance_below_min_relay_fee_returns_amount_0() {
        let wallet = WalletBuilder::new(1000).with_fees(1.0, 1001).build();
        let amount = wallet.max_giveable(TxLock::script_size()).await.unwrap();

        assert_eq!(amount, Amount::ZERO);
    }

    #[tokio::test]
    async fn given_balance_above_relay_fee_returns_amount_greater_0() {
        let wallet = WalletBuilder::new(10_000).build();
        let amount = wallet.max_giveable(TxLock::script_size()).await.unwrap();

        assert!(amount.to_sat() > 0);
    }

    /// This test ensures that the relevant script output of the transaction
    /// created out of the PSBT is at index 0. This is important because
    /// subscriptions to the transaction are on index `0` when broadcasting the
    /// transaction.
    #[tokio::test]
    async fn given_amounts_with_change_outputs_when_signing_tx_then_output_index_0_is_ensured_for_script(
    ) {
        // This value is somewhat arbitrary but the indexation problem usually occurred
        // on the first or second value (i.e. 547, 548) We keep the test
        // iterations relatively low because these tests are expensive.
        let above_dust = 547;
        let balance = 2000;

        // We don't care about fees in this test, thus use a zero fee rate
        let wallet = WalletBuilder::new(balance).with_zero_fees().build();

        // sorting is only relevant for amounts that have a change output
        // if the change output is below dust it will be dropped by the BDK
        for amount in above_dust..(balance - (above_dust - 1)) {
            let (A, B) = (PublicKey::random(), PublicKey::random());
            let change = wallet.new_address().await.unwrap();
            let txlock = TxLock::new(&wallet, bitcoin::Amount::from_sat(amount), A, B, change)
                .await
                .unwrap();
            let txlock_output = txlock.script_pubkey();

            let tx = wallet.sign_and_finalize(txlock.into()).await.unwrap();
            let tx_output = tx.output[0].script_pubkey.clone();

            assert_eq!(
                tx_output, txlock_output,
                "Output {:?} index mismatch for amount {} and balance {}",
                tx.output, amount, balance
            );
        }
    }

    #[tokio::test]
    async fn can_override_change_address() {
        let wallet = WalletBuilder::new(50_000).build();
        let custom_change = "bcrt1q08pfqpsyrt7acllzyjm8q5qsz5capvyahm49rw"
            .parse::<Address>()
            .unwrap();

        let psbt = wallet
            .send_to_address(
                wallet.new_address().await.unwrap(),
                Amount::from_sat(10_000),
                Some(custom_change.clone()),
            )
            .await
            .unwrap();
        let transaction = wallet.sign_and_finalize(psbt).await.unwrap();

        match transaction.output.as_slice() {
            [first, change] => {
                assert_eq!(first.value, 10_000);
                assert_eq!(change.script_pubkey, custom_change.script_pubkey());
            }
            _ => panic!("expected exactly two outputs"),
        }
    }

    #[test]
    fn printing_status_change_doesnt_spam_on_same_status() {
        let writer = capture_logs(LevelFilter::DEBUG);

        let inner = bitcoin::hashes::sha256d::Hash::all_zeros();
        let tx = Txid::from_hash(inner);
        let mut old = None;
        old = Some(trace_status_change(tx, old, ScriptStatus::Unseen));
        old = Some(trace_status_change(tx, old, ScriptStatus::InMempool));
        old = Some(trace_status_change(tx, old, ScriptStatus::InMempool));
        old = Some(trace_status_change(tx, old, ScriptStatus::InMempool));
        old = Some(trace_status_change(tx, old, ScriptStatus::InMempool));
        old = Some(trace_status_change(tx, old, ScriptStatus::InMempool));
        old = Some(trace_status_change(tx, old, ScriptStatus::InMempool));
        old = Some(trace_status_change(tx, old, confs(1)));
        old = Some(trace_status_change(tx, old, confs(2)));
        old = Some(trace_status_change(tx, old, confs(3)));
        old = Some(trace_status_change(tx, old, confs(3)));
        trace_status_change(tx, old, confs(3));

        assert_eq!(
            writer.captured(),
            r"DEBUG swap::bitcoin::wallet: Found relevant Bitcoin transaction txid=0000000000000000000000000000000000000000000000000000000000000000 status=unseen
DEBUG swap::bitcoin::wallet: Bitcoin transaction status changed txid=0000000000000000000000000000000000000000000000000000000000000000 new_status=in mempool old_status=unseen
DEBUG swap::bitcoin::wallet: Bitcoin transaction status changed txid=0000000000000000000000000000000000000000000000000000000000000000 new_status=confirmed with 1 blocks old_status=in mempool
DEBUG swap::bitcoin::wallet: Bitcoin transaction status changed txid=0000000000000000000000000000000000000000000000000000000000000000 new_status=confirmed with 2 blocks old_status=confirmed with 1 blocks
DEBUG swap::bitcoin::wallet: Bitcoin transaction status changed txid=0000000000000000000000000000000000000000000000000000000000000000 new_status=confirmed with 3 blocks old_status=confirmed with 2 blocks
"
        )
    }

    fn confs(confirmations: u32) -> ScriptStatus {
        ScriptStatus::from_confirmations(confirmations)
    }

    proptest::proptest! {
        #[test]
        fn funding_never_fails_with_insufficient_funds(funding_amount in 3000u32.., num_utxos in 1..5u8, sats_per_vb in 1.0..500.0f32, key in crate::proptest::bitcoin::extended_priv_key(), alice in crate::proptest::ecdsa_fun::point(), bob in crate::proptest::ecdsa_fun::point()) {
            proptest::prop_assume!(alice != bob);

            tokio::runtime::Runtime::new().unwrap().block_on(async move {
                let wallet = WalletBuilder::new(funding_amount as u64).with_key(key).with_num_utxos(num_utxos).with_fees(sats_per_vb, 1000).build();

                let amount = wallet.max_giveable(TxLock::script_size()).await.unwrap();
                let psbt: PartiallySignedTransaction = TxLock::new(&wallet, amount, PublicKey::from(alice), PublicKey::from(bob), wallet.new_address().await.unwrap()).await.unwrap().into();
                let result = wallet.sign_and_finalize(psbt).await;

                result.expect("transaction to be signed");
            });
        }
    }
}
