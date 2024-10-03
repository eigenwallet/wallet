use std::sync::Arc;
use std::collections::HashMap;
use std::time::Duration;

use bitcoin::Txid;
use uuid::Uuid;

use crate::bitcoin::Wallet;
use crate::bitcoin::wallet::Subscription;
use crate::cli::api::tauri_bindings::TauriHandle;

#[derive(Clone)]
pub struct Watcher {
    wallet: Arc<Wallet>,
    subscriptions: HashMap<(Txid, Uuid), Subscription>,
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
    pub async fn run(self) {
        loop {
            tokio::time::sleep(Duration::from_secs(3)).await;
        }
    }
}
