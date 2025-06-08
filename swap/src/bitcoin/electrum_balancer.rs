use std::ops::Deref;
use std::sync::{Arc, Mutex};

use futures::future::join_all;
use tokio::task::spawn_blocking;

use bdk_electrum::electrum_client::{Client, ConfigBuilder, ElectrumApi, Error};
use bdk_electrum::BdkElectrumClient;
use bitcoin::Transaction;
use tracing::{debug, error, info, instrument, trace, warn};

/// Round-robin load balancer for Electrum connections.
///
/// The balancer will try each Electrum node until the provided
/// closure succeeds or all nodes have returned an I/O error.
/// Any non I/O error is immediately returned to the caller.
///
/// Clients are created lazily on first use to avoid blocking during initialization.
#[derive(Clone)]
pub struct ElectrumBalancer {
    urls: Vec<String>,
    clients: Arc<Mutex<Vec<Arc<BdkElectrumClient<Client>>>>>,
    next: Arc<Mutex<usize>>,
}

impl ElectrumBalancer {
    /// Create a single electrum client with timeout
    fn create_client_with_timeout(url: &str) -> Result<BdkElectrumClient<Client>, Error> {
        // Configure client with short timeout to prevent hanging on unresponsive servers
        let config = ConfigBuilder::new()
            .timeout(Some(5)) // 5 second timeout
            .retry(0) // Retrying is done at the caller level (ElectrumBalancer)
            .build();

        let client = Client::from_config(url, config)?;
        Ok(BdkElectrumClient::new(client))
    }

    /// Create a new balancer from a list of Electrum URLs.
    /// All clients are created at startup - this may take time but eliminates delays during use.
    #[instrument(level = "info", fields(num_urls = urls.len()))]
    pub async fn new(urls: Vec<String>) -> Result<Self, Error> {
        if urls.is_empty() {
            error!("No Electrum URLs provided");
            return Err(Error::Protocol("No Electrum URLs provided".into()));
        }

        info!(
            "Initializing Electrum load balancer with {} servers",
            urls.len()
        );

        // Create all clients at startup
        let futures: Vec<_> = urls
            .iter()
            .map(|url| {
                let url = url.clone();
                spawn_blocking(move || {
                    Self::create_client_with_timeout(&url)
                        .map(|client| Arc::new(client))
                        .ok()
                })
            })
            .collect();

        let results = join_all(futures).await;

        let clients: Vec<Arc<BdkElectrumClient<Client>>> = results
            .into_iter()
            .filter_map(|res| res.unwrap_or(None))
            .collect();

        Ok(Self {
            urls,
            clients: Arc::new(Mutex::new(clients)),
            next: Arc::new(Mutex::new(0)),
        })
    }

    /// Get a client for the given index.
    pub async fn get_or_create_client(
        &self,
        idx: usize,
    ) -> Result<Arc<BdkElectrumClient<Client>>, Error> {
        let clients = self.clients.lock().expect("mutex poisoned");

        if let Some(ref client) = clients.get(idx) {
            // TODO: Why do we need to double clone here?
            Ok(client.deref().clone())
        } else {
            Err(Error::IOError(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Client {} is not available", self.urls[idx]),
            )))
        }
    }

    /// Helper function to determine if an error should trigger failover
    fn should_retry_on_error(error: &Error) -> bool {
        // Check if this is a transaction not found error - these should NOT be retried
        let error_str = format!("{:?}", error);
        if error_str.contains("\"code\": Number(-5)")
            || error_str.contains("No such mempool or blockchain transaction")
            || error_str.contains("missing transaction")
        {
            return false;
        }

        // For all other errors, retry by default
        true
    }

    /// Execute the given closure using one of the Electrum clients asynchronously.
    ///
    ///
    /// If the closure returns an I/O error or certificate error the balancer will try the next
    /// node until all nodes have been exhausted. The last encountered error
    /// is returned in that case.
    #[instrument(level = "debug", skip(self, f), fields(total_urls = self.urls.len()))]
    pub async fn call_async<F, T>(&self, f: F) -> Result<T, Error>
    where
        F: Fn(&BdkElectrumClient<Client>) -> Result<T, Error> + Send + Sync + Clone + 'static,
        T: Send + 'static,
    {
        let balancer = self.clone();

        spawn_blocking(move || balancer.call(f))
            .await
            .map_err(|e| {
                Error::IOError(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    e.to_string(),
                ))
            })?
    }

    /// Execute the given closure using one of the Electrum clients synchronously.
    ///
    /// This version blocks for client creation if needed but executes the request synchronously.
    /// Used for implementing the ElectrumApi trait.
    ///
    /// If the closure returns an I/O error or certificate error the balancer will try the next
    /// node until all nodes have been exhausted. The last encountered error
    /// is returned in that case.
    #[instrument(level = "debug", skip(self, f), fields(total_urls = self.urls.len()))]
    pub fn call<F, T>(&self, mut f: F) -> Result<T, Error>
    where
        F: FnMut(&BdkElectrumClient<Client>) -> Result<T, Error>,
    {
        let num_urls = self.urls.len();
        let mut last_error = None;

        for _attempt in 0..num_urls {
            let idx = {
                let mut next = self.next.lock().expect("mutex poisoned");
                let idx = *next;
                *next = (*next + 1) % num_urls;
                idx
            };

            // Get client for this index
            let client = {
                let clients = self.clients.lock().expect("mutex poisoned").clone();

                match clients.get(idx) {
                    Some(client) => client.clone(),
                    None => {
                        last_error = Some(Error::IOError(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            format!("Client {} is not available", self.urls[idx]),
                        )));
                        continue;
                    }
                }
            };

            // Execute the request synchronously
            match f(&client) {
                Ok(res) => {
                    trace!("Request successful on {}", self.urls[idx]);
                    return Ok(res);
                }
                Err(e) => {
                    if Self::should_retry_on_error(&e) {
                        warn!(
                            "Request failed on {}: {:?}, trying next server",
                            self.urls[idx], e
                        );
                        last_error = Some(e);
                        continue;
                    } else {
                        debug!("Non-retryable error on {}: {:?}", self.urls[idx], e);
                        return Err(e);
                    }
                }
            }
        }

        error!(
            "All {} electrum servers failed or could not be created",
            num_urls
        );
        Err(last_error.unwrap_or_else(|| {
            Error::IOError(std::io::Error::new(
                std::io::ErrorKind::Other,
                "all electrum nodes failed",
            ))
        }))
    }

    /// Execute the given closure on **all** Electrum nodes in parallel.
    ///
    /// The closure is executed in a blocking task for each client.
    /// The resulting `Result`s are collected and returned in the same
    /// order as the nodes were provided during construction.
    #[instrument(level = "debug", skip(self, f), fields(num_urls = self.urls.len()))]
    pub async fn join_all<F, T>(&self, f: F) -> Vec<Result<T, Error>>
    where
        F: Fn(Arc<BdkElectrumClient<Client>>) -> Result<T, Error> + Send + Sync + Clone + 'static,
        T: Send + 'static,
    {
        info!(
            "Executing parallel requests on {} electrum servers",
            self.urls.len()
        );

        // Create a thread for each Electrum node
        let tasks = self
            .clients
            .lock()
            .expect("mutex poisoned")
            .clone()
            .into_iter()
            .map(|client| {
                let f = f.clone();

                spawn_blocking(move || f(client))
            });

        // Spawn the threads and wait until they all finish
        let results = join_all(tasks)
            .await
            .into_iter()
            .enumerate()
            .map(|(idx, res)| match res {
                Ok(r) => r,
                Err(e) => Err(Error::IOError(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to spawn thread for server {idx}: {e}"),
                ))),
            })
            .collect::<Vec<_>>();

        let success_count = results.iter().filter(|r| r.is_ok()).count();
        let failure_count = results.len() - success_count;

        info!(
            "Parallel execution completed: {}/{} succeeded, {}/{} failed",
            success_count,
            results.len(),
            failure_count,
            results.len()
        );

        results
    }

    /// Broadcast the given transaction to all Electrum nodes in parallel.
    ///
    /// The method returns a list of results in the same order as the
    /// configured nodes. Errors for individual nodes do not abort the
    /// others.
    #[instrument(level = "info", skip(self, tx), fields(txid = %tx.compute_txid(), num_servers = self.urls.len()))]
    pub async fn broadcast_all(&self, tx: Transaction) -> Vec<Result<bitcoin::Txid, Error>> {
        let txid = tx.compute_txid();
        info!(
            "Broadcasting transaction {} to {} electrum servers",
            txid,
            self.urls.len()
        );

        let results = self
            .join_all(move |client| client.inner.transaction_broadcast(&tx))
            .await;

        let success_count = results.iter().filter(|r| r.is_ok()).count();

        if success_count > 0 {
            debug!(
                "Transaction {} broadcast successful on {}/{} servers",
                txid,
                success_count,
                results.len()
            );
        } else {
            warn!(
                "Transaction {} broadcast failed on all {} servers",
                txid,
                results.len()
            );
        }

        results
    }

    /// Get the URLs used by this balancer
    pub fn urls(&self) -> &Vec<String> {
        &self.urls
    }

    /// Populate the transaction cache for all clients.
    /// Note: This is not implemented for the load balancer as the underlying clients
    /// don't support transaction caching.
    pub fn populate_tx_cache(&self, _txs: impl IntoIterator<Item = impl Into<Arc<Transaction>>>) {
        // No-op: The raw electrum clients don't support transaction caching
        tracing::debug!("populate_tx_cache called on ElectrumBalancer - this is a no-op");
    }
}
