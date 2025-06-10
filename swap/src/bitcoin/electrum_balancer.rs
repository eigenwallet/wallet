use std::sync::{Arc, Mutex, RwLock};
use std::time::Instant;
use futures::future::join_all;
use tokio::task::spawn_blocking;
use bdk_electrum::electrum_client::{Client, ConfigBuilder, ElectrumApi, Error};
use bdk_electrum::BdkElectrumClient;
use bitcoin::Transaction;
use tracing::{debug, error, info, instrument, trace, warn};
use once_cell::sync::OnceCell;

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
    clients: Arc<RwLock<Vec<Arc<OnceCell<Arc<BdkElectrumClient<Client>>>>>>>,
    next: Arc<Mutex<usize>>,
    config: ElectrumBalancerConfig,
}

impl ElectrumBalancer {
    /// Create a single electrum client with timeout
    fn create_client_with_timeout(
        url: &str,
        config: &ElectrumBalancerConfig,
    ) -> Result<Arc<BdkElectrumClient<Client>>, Error> {
        // Configure client with configurable timeout to prevent hanging on unresponsive servers
        let client_config = ConfigBuilder::new()
            .timeout(Some(config.request_timeout))
            .retry(0)
            .build();

        let client = Client::from_config(url, client_config)?;
        let bdk_client = BdkElectrumClient::new(client);

        Ok(Arc::new(bdk_client))
    }

    /// Helper function to get or initialize a client for a given index
    fn get_or_init_client_sync(
        &self,
        idx: usize,
    ) -> Result<Arc<BdkElectrumClient<Client>>, Error> {
        let clients = self.clients.read().expect("rwlock poisoned");
        if idx >= clients.len() {
            return Err(Error::IOError(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Index {} out of bounds for {} clients", idx, clients.len()),
            )));
        }
        let once_cell = clients[idx].clone();
        let url = self.urls[idx].clone();
        let config = self.config.clone();
        drop(clients); // Release the read lock early

        let client = once_cell.get_or_try_init(|| {
            Self::create_client_with_timeout(&url, &config)
        })?;

        Ok(client.clone())
    }

    async fn get_or_init_client_async(
        &self,
        idx: usize,
    ) -> Result<Arc<BdkElectrumClient<Client>>, Error> {
        let balancer = self.clone();
        spawn_blocking(move || balancer.get_or_init_client_sync(idx))
            .await
            .map_err(|e| Error::IOError(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            )))?
    }

    /// Create a new balancer from a list of Electrum URLs with default configuration.
    pub async fn new(urls: Vec<String>) -> Result<Self, Error> {
        Self::new_with_config(urls, ElectrumBalancerConfig::default()).await
    }

    /// Create a new balancer from a list of Electrum URLs with custom configuration.
    /// Clients are initialized lazily on first use.
    pub async fn new_with_config(
        urls: Vec<String>,
        config: ElectrumBalancerConfig,
    ) -> Result<Self, Error> {
        if urls.is_empty() {
            return Err(Error::Protocol("No Electrum URLs provided".into()));
        }

        debug!(
            servers = ?urls,
            server_count = urls.len(),
            timeout_seconds = config.request_timeout,
            min_retries = config.min_retries,
            "Initializing Electrum load balancer"
        );

        // Create OnceCell containers for each URL - clients will be created on first use
        let clients: Vec<Arc<OnceCell<Arc<BdkElectrumClient<Client>>>>> = urls
            .iter()
            .map(|_| Arc::new(OnceCell::new()))
            .collect();

        Ok(Self {
            urls,
            clients: Arc::new(RwLock::new(clients)),
            next: Arc::new(Mutex::new(0)),
            config,
        })
    }

    /// Helper function to determine if an error should trigger failover
    /// TODO: This should not be in this file?
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

    /// Get the number of URLs (potential clients)
    pub fn client_count(&self) -> usize {
        self.urls.len()
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

            // Get client for this index (will initialize if needed)
            let client = match self.get_or_init_client_sync(idx) {
                Ok(client) => client,
                Err(e) => {
                    if Self::should_retry_on_error(&e) {
                        warn!(
                            client_index = idx,
                            attempt = attempt + 1,
                            error = ?e,
                            "Client initialization failed, trying next client"
                        );
                        last_error = Some(e);
                        continue;
                    } else {
                        debug!(
                            client_index = idx,
                            attempt = attempt + 1,
                            error = ?e,
                            "Client initialization failed with non-retryable error"
                        );
                        return Err(e);
                    }
                }
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
    pub async fn join_all<F, T>(&self, f: F) -> Result<Vec<Result<T, Error>>, Error>
    where
        F: Fn(&BdkElectrumClient<Client>) -> Result<T, Error> + Send + Sync + Clone + 'static,
        T: Send + 'static,
    {
        let start_time = Instant::now();
        info!(
            total_clients = self.client_count(),
            "Executing parallel requests on electrum clients"
        );

        // Create a task for each potential client
        let tasks = {
            (0..self.client_count())
                .map(|idx| {
                    let f = f.clone();
                    let balancer = self.clone();

                    tokio::spawn(async move {
                        let start = Instant::now();
                        
                        let result = match balancer.get_or_init_client_async(idx).await {
                            Ok(client) => {
                                // Now call f with a blocking task since f expects sync operation
                                tokio::task::spawn_blocking(move || f(&client)).await
                                    .map_err(|e| Error::IOError(std::io::Error::new(
                                        std::io::ErrorKind::Other,
                                        e.to_string(),
                                    )))?
                            }
                            Err(e) => {
                                warn!(index = idx, error = ?e, "Failed to create client during join_all");
                                Err(e)
                            }
                        };

                        match result {
                            Ok(r) => {
                                trace!(
                                    client_index = idx,
                                    duration_ms = start.elapsed().as_millis(),
                                    "Parallel request completed"
                                );
                                Ok(r)
                            }
                            Err(e) => {
                                trace!(index = idx, error = ?e, "Failed to execute request during join_all");
                                Err(e)
                            }
                        }
                    })
                })
                .collect::<Vec<_>>()
        };

        // Spawn the threads and wait until they all finish
        let spawn_results = join_all(tasks).await;

        let mut results: Vec<Result<T, Error>> = Vec::new();
        for (task_idx, res) in spawn_results.into_iter().enumerate() {
            match res {
                Ok(r) => results.push(r),
                Err(err) if err.is_cancelled() => {
                    return Err(Error::IOError(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "Task cancelled",
                    )));
                }
                Err(e) => {
                    warn!(task_index = task_idx, error = ?e, "Failed to spawn thread for parallel request");
                }
            }
        }

        let success_count = results.iter().filter(|r| r.is_ok()).count();
        let failure_count = results.len() - success_count;

        info!(
            total_duration_ms = start_time.elapsed().as_millis(),
            successful_requests = success_count,
            failed_requests = failure_count,
            total_requests = results.len(),
            "Parallel execution completed"
        );

        Ok(results)
    }

    /// Broadcast the given transaction to all Electrum nodes in parallel.
    ///
    /// The method returns a list of results in the same order as the
    /// configured nodes. Errors for individual nodes do not abort the
    /// others.
    #[instrument(level = "info", skip(self, tx), fields(txid = %tx.compute_txid(), total_clients = self.client_count()))]
    pub async fn broadcast_all(&self, tx: Transaction) -> Result<Vec<Result<bitcoin::Txid, Error>>, Error> {
        let txid = tx.compute_txid();
        let start_time = Instant::now();

        info!(
            txid = %txid,
            total_clients = self.client_count(),
            "Broadcasting transaction to electrum clients"
        );

        let results = self
            .join_all(move |client| client.inner.transaction_broadcast(&tx))
            .await?;

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

        Ok(results)
    }

    /// Get the URLs used by this balancer
    pub fn urls(&self) -> &Vec<String> {
        &self.urls
    }

    /// Get the current configuration
    pub fn config(&self) -> &ElectrumBalancerConfig {
        &self.config
    }

    /// Populate the transaction cache for all initialized clients.
    pub fn populate_tx_cache(&self, txs: impl IntoIterator<Item = impl Into<Arc<Transaction>>>) {
        // Convert transactions to Arc<Transaction> and collect them since we'll use them for each client
        let transactions: Vec<Arc<Transaction>> = txs.into_iter().map(|tx| tx.into()).collect();
        let clients = self.clients.read().expect("rwlock poisoned");

        let mut initialized_count = 0;
        // Only populate cache for already initialized clients
        for client_once_cell in clients.iter() {
            if let Some(client) = client_once_cell.get() {
                client.populate_tx_cache(transactions.iter().cloned());
                initialized_count += 1;
            }
        }

        trace!(
            transaction_count = transactions.len(),
            initialized_client_count = initialized_count,
            total_client_count = clients.len(),
            "Populated transaction cache for initialized clients"
        );
    }
}
