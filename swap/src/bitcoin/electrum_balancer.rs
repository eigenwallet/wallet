use std::sync::{Arc, Mutex};
use std::time::Instant;
use futures::future::join_all;
use tokio::task::spawn_blocking;
use bdk_electrum::electrum_client::{Client, ConfigBuilder, ElectrumApi, Error};
use bdk_electrum::BdkElectrumClient;
use bitcoin::Transaction;
use tracing::{debug, error, info, instrument, trace, warn};

/// Configuration for the Electrum balancer
#[derive(Clone, Debug)]
pub struct ElectrumBalancerConfig {
    /// Timeout for individual requests in seconds
    pub request_timeout: u8,
    /// Minimum number of retry attempts across all nodes
    pub min_retries: usize,
}

impl Default for ElectrumBalancerConfig {
    fn default() -> Self {
        Self {
            request_timeout: 10,
            min_retries: 3,
        }
    }
}

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
    config: ElectrumBalancerConfig,
}

impl ElectrumBalancer {
    /// Create a single electrum client with timeout
    fn create_client_with_timeout(
        url: &str,
        config: &ElectrumBalancerConfig,
    ) -> Result<BdkElectrumClient<Client>, Error> {
        // Configure client with configurable timeout to prevent hanging on unresponsive servers
        let client_config = ConfigBuilder::new()
            .timeout(Some(config.request_timeout))
            .retry(0)
            .build();

        let client = Client::from_config(url, client_config)?;
        let bdk_client = BdkElectrumClient::new(client);

        Ok(bdk_client)
    }

    /// Create a new balancer from a list of Electrum URLs with default configuration.
    pub async fn new(urls: Vec<String>) -> Result<Self, Error> {
        Self::new_with_config(urls, ElectrumBalancerConfig::default()).await
    }

    /// Create a new balancer from a list of Electrum URLs with custom configuration.
    /// All clients are created at startup - this may take time but eliminates delays during use.
    pub async fn new_with_config(
        urls: Vec<String>,
        config: ElectrumBalancerConfig,
    ) -> Result<Self, Error> {
        if urls.is_empty() {
            return Err(Error::Protocol("No Electrum URLs provided".into()));
        }

        let start_time = Instant::now();
        info!(
            servers = ?urls,
            server_count = urls.len(),
            timeout_seconds = config.request_timeout,
            min_retries = config.min_retries,
            "Initializing Electrum load balancer"
        );

        // Create all clients at startup
        let futures: Vec<_> = urls
            .iter()
            .enumerate()
            .map(|(idx, url)| {
                let url = url.clone();
                let config = config.clone();
                spawn_blocking(
                    move || match Self::create_client_with_timeout(&url, &config) {
                        Ok(client) => {
                            Some(Arc::new(client))
                        }
                        Err(e) => {
                            warn!(url = %url, index = idx, error = ?e, "Failed to create client");
                            None
                        }
                    },
                )
            })
            .collect();

        let results = join_all(futures).await;

        let mut clients: Vec<Arc<BdkElectrumClient<Client>>> = Vec::new();

        for (idx, result) in results.into_iter().enumerate() {
            match result {
                Ok(client_opt) => {
                    if let Some(client) = client_opt {
                        clients.push(client);
                    } else {
                        warn!(url = %urls[idx], index = idx, "Failed to create client");
                    }
                }
                Err(e) => {
                    warn!(url = %urls[idx], index = idx, error = ?e, "Failed to spawn client creation task");
                }
            }
        }

        if clients.is_empty() {
            error!("Failed to create any working Electrum clients");
            return Err(Error::Protocol(
                "No working Electrum servers available".into(),
            ));
        }

        info!(
            successful_clients = clients.len(),
            total_urls = urls.len(),
            initialization_duration_ms = start_time.elapsed().as_millis(),
            "Electrum load balancer initialized successfully"
        );

        if clients.len() < urls.len() {
            warn!(
                working_clients = clients.len(),
                total_urls = urls.len(),
                "Some Electrum clients failed to initialize - continuing with available clients"
            );
        }

        Ok(Self {
            urls,
            clients: Arc::new(Mutex::new(clients)),
            next: Arc::new(Mutex::new(0)),
            config,
        })
    }

    /// Get a client for the given index.
    pub async fn get_client(&self, idx: usize) -> Result<Arc<BdkElectrumClient<Client>>, Error> {
        let clients = self.clients.lock().expect("mutex poisoned");

        if idx >= clients.len() {
            return Err(Error::IOError(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Index {} out of bounds for {} clients", idx, clients.len()),
            )));
        }

        Ok(clients[idx].clone())
    }

    /// Helper function to determine if an error should trigger failover
    fn should_retry_on_error(error: &Error) -> bool {
        // Check if this is a transaction not found error - these should NOT be retried
        let error_str = format!("{:?}", error);
        if error_str.contains("\"code\": Number(-5)")
            || error_str.contains("No such mempool or blockchain transaction")
            || error_str.contains("missing transaction")
        {
            trace!("Non-retryable error detected: transaction not found");
            return false;
        }

        // For all other errors, retry by default
        true
    }

    /// Get the number of clients
    pub fn client_count(&self) -> usize {
        self.clients.lock().expect("mutex poisoned").len()
    }

    /// Execute the given closure using one of the Electrum clients asynchronously.
    ///
    ///
    /// If the closure returns an I/O error or certificate error the balancer will try the next
    /// node until all nodes have been exhausted. The last encountered error
    /// is returned in that case.
    #[instrument(level = "debug", skip(self, f), fields(operation = kind, total_urls = self.urls.len(), total_clients = self.client_count()))]
    pub async fn call_async<F, T>(&self, kind: &str, f: F) -> Result<T, Error>
    where
        F: Fn(&BdkElectrumClient<Client>) -> Result<T, Error> + Send + Sync + Clone + 'static,
        T: Send + 'static,
    {
        let balancer = self.clone();
        let kind = kind.to_string();

        spawn_blocking(move || balancer.call(&kind, f))
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
    #[instrument(level = "debug", skip(self, f), fields(operation = kind, total_clients = self.client_count(), min_retries = self.config.min_retries))]
    pub fn call<F, T>(&self, kind: &str, mut f: F) -> Result<T, Error>
    where
        F: FnMut(&BdkElectrumClient<Client>) -> Result<T, Error>,
    {
        let num_clients = self.client_count();
        let mut last_error = None;
        let mut attempts = 0;

        // Try all electrum clients at least once, or min_retries (whichever is higher)
        let total_attempts = std::cmp::max(self.config.min_retries, num_clients);

        for attempt in 0..total_attempts {
            attempts += 1;
            let idx = {
                let mut next = self.next.lock().expect("mutex poisoned");
                let idx = *next;
                *next = (*next + 1) % num_clients;
                idx
            };

            // Get client for this index
            let client = {
                let clients = self.clients.lock().expect("mutex poisoned");
                clients[idx].clone()
            };

            // Execute the request synchronously
            let start = Instant::now();
            match f(&client) {
                Ok(res) => {
                    trace!(
                        client_index = idx,
                        attempt = attempt + 1,
                        duration_ms = start.elapsed().as_millis(),
                        "Electrum operation successful"
                    );
                    return Ok(res);
                }
                Err(e) => {
                    if Self::should_retry_on_error(&e) {
                        warn!(
                            client_index = idx,
                            attempt = attempt + 1,
                            duration_ms = start.elapsed().as_millis(),
                            error = ?e,
                            "Electrum operation failed, trying next client"
                        );
                        last_error = Some(e);
                        continue;
                    } else {
                        debug!(
                            client_index = idx,
                            attempt = attempt + 1,
                            error = ?e,
                            "Electrum operation failed with non-retryable error"
                        );
                        return Err(e);
                    }
                }
            }
        }

        error!(
            attempts = attempts,
            total_attempts = total_attempts,
            total_clients = self.client_count(),
            "All Electrum clients failed after exhausting retry attempts"
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
    #[instrument(level = "debug", skip(self, f), fields(total_clients = self.client_count()))]
    pub async fn join_all<F, T>(&self, f: F) -> Vec<Result<T, Error>>
    where
        F: Fn(Arc<BdkElectrumClient<Client>>) -> Result<T, Error> + Send + Sync + Clone + 'static,
        T: Send + 'static,
    {
        let start_time = Instant::now();
        info!(
            total_clients = self.client_count(),
            "Executing parallel requests on electrum clients"
        );

        // Create a thread for each Electrum client
        let tasks = self
            .clients
            .lock()
            .expect("mutex poisoned")
            .iter()
            .enumerate()
            .map(|(idx, client)| {
                let f = f.clone();
                let client = client.clone();

                spawn_blocking(move || {
                    let start = Instant::now();
                    let result = f(client);
                    trace!(
                        client_index = idx,
                        duration_ms = start.elapsed().as_millis(),
                        success = result.is_ok(),
                        "Parallel request completed"
                    );
                    result
                })
            })
            .collect::<Vec<_>>();

        // Spawn the threads and wait until they all finish
        let spawn_results = join_all(tasks).await;

        let results: Vec<Result<T, Error>> = spawn_results
            .into_iter()
            .enumerate()
            .filter_map(|(task_idx, res)| match res {
                Ok(r) => Some(r),
                Err(e) => {
                    warn!(task_index = task_idx, error = ?e, "Failed to spawn thread for parallel request");
                    None
                }
            })
            .collect();

        let success_count = results.iter().filter(|r| r.is_ok()).count();
        let failure_count = results.len() - success_count;

        info!(
            total_duration_ms = start_time.elapsed().as_millis(),
            successful_requests = success_count,
            failed_requests = failure_count,
            total_requests = results.len(),
            "Parallel execution completed"
        );

        results
    }

    /// Broadcast the given transaction to all Electrum nodes in parallel.
    ///
    /// The method returns a list of results in the same order as the
    /// configured nodes. Errors for individual nodes do not abort the
    /// others.
    #[instrument(level = "info", skip(self, tx), fields(txid = %tx.compute_txid(), total_clients = self.client_count()))]
    pub async fn broadcast_all(&self, tx: Transaction) -> Vec<Result<bitcoin::Txid, Error>> {
        let txid = tx.compute_txid();
        let start_time = Instant::now();

        info!(
            txid = %txid,
            total_clients = self.client_count(),
            "Broadcasting transaction to electrum clients"
        );

        let results = self
            .join_all(move |client| client.inner.transaction_broadcast(&tx))
            .await;

        let success_count = results.iter().filter(|r| r.is_ok()).count();

        if success_count > 0 {
            info!(
                txid = %txid,
                successful_broadcasts = success_count,
                total_attempts = results.len(),
                duration_ms = start_time.elapsed().as_millis(),
                "Transaction broadcast completed successfully"
            );
        } else {
            error!(
                txid = %txid,
                total_attempts = results.len(),
                duration_ms = start_time.elapsed().as_millis(),
                "Transaction broadcast failed on all servers"
            );
        }

        results
    }

    /// Get the URLs used by this balancer
    pub fn urls(&self) -> &Vec<String> {
        &self.urls
    }

    /// Get the current configuration
    pub fn config(&self) -> &ElectrumBalancerConfig {
        &self.config
    }

    /// Populate the transaction cache for all clients.
    pub fn populate_tx_cache(&self, txs: impl IntoIterator<Item = impl Into<Arc<Transaction>>>) {
        // Convert transactions to Arc<Transaction> and collect them since we'll use them for each client
        let transactions: Vec<Arc<Transaction>> = txs.into_iter().map(|tx| tx.into()).collect();
        let clients = self.clients.lock().expect("mutex poisoned");

        // Iterate through all BdkElectrumClient and populate their caches
        for client in clients.iter() {
            client.populate_tx_cache(transactions.iter().cloned());
        }

        trace!(
            transaction_count = transactions.len(),
            client_count = clients.len(),
            "Populated transaction cache for all clients"
        );
    }
}
