use std::sync::Arc;
use std::collections::HashMap;
use std::time::Duration;

use anyhow::Result;
use bitcoin::{Script, Txid};
use uuid::Uuid;

use crate::bitcoin::Wallet;
use crate::bitcoin::wallet::ScriptStatus;
use crate::cli::api::tauri_bindings::TauriHandle;

use super::api::tauri_bindings::TauriEmitter;

/// A long running task which watches for changes to timelocks and the number of confirmations.
#[derive(Clone)]
pub struct Watcher {
    wallet: Arc<Wallet>,
    subscriptions: HashMap<(Uuid, Txid, Script), ScriptStatus>,
    tauri: Option<TauriHandle>
}

impl Watcher {
    /// Create a new Watcher
    pub fn new(wallet: Arc<Wallet>, tauri: Option<TauriHandle>) -> Self {
        Self {
            wallet,
            subscriptions: HashMap::new(),
            tauri
        }
    }

    /// Start running the watcher event loop. 
    /// Should be done in a new task using [`tokio::spawn`].
    pub async fn run(mut self) {
        loop {
            // Fetch current transactions and timelocks
            let current_transactions = match self.fetch_current_transactions().await {
                Ok(x) => x,
                Err(e) => {
                    tracing::error!(error=%e, "Failed to fetch current transactions, retrying later");
                    continue;
                }
            };

            for (uuid, txid, script) in current_transactions {
                // Fetch new script status
                let script_status = match self.wallet.status_of_script(&(txid, script.clone())).await {
                    Ok(x) => x,
                    Err(e) => {
                        tracing::error!(error=%e, "Failed to fetch status of script, retrying later");
                        continue;
                    }
                };
                // Check if the status changed
                if let Some(old_status) = self.subscriptions.get(&(uuid, txid, script.clone())) {
                    // And send a tauri event if it did
                    // TODO: distinguish between timelock and confirmation events
                    if *old_status == script_status {
                        self.tauri.emit_timelock_change_event(uuid);
                    }
                }
                // Insert new status
                self.subscriptions.insert((uuid, txid, script), script_status);
            }

            // Check for updated timelocks and confirmations
            tokio::time::sleep(Duration::from_secs(3)).await;
        }
    }

    /// Helper function for fetching the current list of swaps
    async fn fetch_current_transactions(&self) -> Result<Vec<(Uuid, Txid, Script)>> {
        // TODO fetch all relevant TxLock, TxCancel, TxRefund, TxRedeem, here
        Err(std::io::Error::new(std::io::ErrorKind::Other, "TODO").into())
    }
}
