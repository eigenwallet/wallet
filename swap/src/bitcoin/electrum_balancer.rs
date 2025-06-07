use std::sync::{Arc, Mutex};

use futures::future::join_all;
use tokio::task::spawn_blocking;

use bdk_electrum::electrum_client::{
    Batch, Client, ConfigBuilder, ElectrumApi, Error, GetBalanceRes, GetHeadersRes, GetHistoryRes,
    GetMerkleRes, ListUnspentRes, RawHeaderNotification, ScriptStatus, ServerFeaturesRes,
    TxidFromPosRes,
};
use bitcoin::{Script, Transaction, Txid};
use serde_json::Value;
use std::borrow::Borrow;
use tracing::{debug, error, info, instrument, warn};

/// Round-robin load balancer for Electrum connections.
///
/// The balancer will try each Electrum node until the provided
/// closure succeeds or all nodes have returned an I/O error.
/// Any non I/O error is immediately returned to the caller.
///
/// Clients are created lazily on first use to avoid blocking during initialization.
pub struct ElectrumBalancer {
    urls: Vec<String>,
    clients: Mutex<Vec<Option<Arc<Client>>>>,
    next: Mutex<usize>,
}

impl ElectrumBalancer {
    /// Create a new balancer from a list of Electrum URLs.
    /// Clients are created lazily on first use to avoid blocking during initialization.
    #[instrument(level = "info", fields(num_urls = urls.len()))]
    pub fn new(urls: Vec<String>) -> Result<Self, Error> {
        if urls.is_empty() {
            error!("No Electrum URLs provided");
            return Err(Error::Protocol("No Electrum URLs provided".into()));
        }

        info!(
            "Initializing ElectrumBalancer with {} URLs (lazy client creation)",
            urls.len()
        );
        debug!("Electrum URLs: {:?}", urls);

        let num_urls = urls.len();
        let clients = Mutex::new(vec![None; num_urls]);

        info!(
            "ElectrumBalancer initialized with {} URLs, clients will be created on demand",
            num_urls
        );
        Ok(Self {
            urls,
            clients,
            next: Mutex::new(0),
        })
    }

    /// Get or create a client for the given index.
    async fn get_or_create_client(&self, idx: usize) -> Result<Arc<Client>, Error> {
        // First check if client already exists
        {
            let clients = self.clients.lock().expect("mutex poisoned");
            if let Some(ref client) = clients[idx] {
                return Ok(client.clone());
            }
        }

        // Create client on demand with a short timeout to avoid hanging
        let url = self.urls[idx].clone();
        debug!("Creating client on demand for server #{}: {}", idx, url);

        // Run client creation in spawn_blocking to prevent blocking the async runtime
        let client_result = spawn_blocking(move || {
            // Configure client with short timeout to prevent hanging on unresponsive servers
            let config = ConfigBuilder::new()
                .timeout(Some(5)) // 5 second timeout
                .retry(1) // Only 1 retry
                .build();

            Client::from_config(&url, config)
        })
        .await;

        match client_result {
            Ok(Ok(client)) => {
                let client = Arc::new(client);
                info!(
                    "Successfully created client #{} for {}",
                    idx, self.urls[idx]
                );

                // Store the client
                let mut clients = self.clients.lock().expect("mutex poisoned");
                clients[idx] = Some(client.clone());

                Ok(client)
            }
            Ok(Err(e)) => {
                warn!(
                    "Failed to create client #{} for {} (with 5s timeout): {}",
                    idx, self.urls[idx], e
                );
                Err(e)
            }
            Err(join_err) => {
                error!("Spawn_blocking failed for client #{}: {}", idx, join_err);
                Err(Error::IOError(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("spawn_blocking failed: {}", join_err),
                )))
            }
        }
    }

    /// Helper function to determine if an error should trigger failover
    fn should_retry_on_error(error: &Error) -> bool {
        match error {
            // Direct I/O errors should always retry
            Error::IOError(_) => true,

            // AllAttemptsErrored containing I/O errors should retry
            Error::AllAttemptsErrored(attempts) => {
                // Check if any of the attempts contained I/O or certificate errors
                attempts.iter().any(|attempt| match attempt {
                    Error::IOError(io_err) => {
                        // Check for certificate-related errors in the error chain
                        let error_str = format!("{:?}", io_err);
                        error_str.contains("InvalidCertificate")
                            || error_str.contains("UnsupportedCertVersion")
                            || error_str.contains("certificate")
                            || error_str.contains("Certificate")
                            || true // All I/O errors should trigger retry
                    }
                    _ => false,
                })
            }

            // Check for other error types that might contain certificate errors
            _ => {
                let error_str = format!("{:?}", error);
                error_str.contains("InvalidCertificate")
                    || error_str.contains("UnsupportedCertVersion")
                    || error_str.contains("certificate verify failed")
                    || error_str.contains("SSL")
                    || error_str.contains("TLS")
            }
        }
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
        F: Fn(&Client) -> Result<T, Error> + Send + Sync + Clone + 'static,
        T: Send + 'static,
    {
        let num_urls = self.urls.len();
        let mut last_error = None;

        for attempt in 0..num_urls {
            let idx = {
                let mut next = self.next.lock().expect("mutex poisoned");
                let idx = *next;
                *next = (*next + 1) % num_urls;
                idx
            };

            debug!(
                "Attempting request on server #{} (attempt {}/{})",
                idx,
                attempt + 1,
                num_urls
            );

            // Get or create client for this index
            let client = match self.get_or_create_client(idx).await {
                Ok(client) => client,
                Err(e) => {
                    warn!(
                        "Failed to create client #{}: {:?}, trying next server",
                        idx, e
                    );
                    last_error = Some(e);
                    continue;
                }
            };

            // Execute the request in spawn_blocking to prevent blocking the async runtime
            let f_clone = f.clone();
            let request_result = spawn_blocking(move || f_clone(&client)).await;

            match request_result {
                Ok(Ok(res)) => {
                    debug!("Request successful on server #{}", idx);
                    return Ok(res);
                }
                Ok(Err(e)) => {
                    if Self::should_retry_on_error(&e) {
                        warn!(
                            "Retryable error on server #{}: {:?}, trying next server",
                            idx, e
                        );
                        last_error = Some(e);
                        continue;
                    } else {
                        debug!(
                            "Non-retryable error on server #{}: {:?}, returning immediately",
                            idx, e
                        );
                        return Err(e);
                    }
                }
                Err(join_err) => {
                    let error = Error::IOError(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("spawn_blocking failed: {}", join_err),
                    ));
                    warn!(
                        "Spawn_blocking failed for server #{}: {:?}, trying next server",
                        idx, join_err
                    );
                    last_error = Some(error);
                    continue;
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
        F: FnMut(&Client) -> Result<T, Error>,
    {
        let num_urls = self.urls.len();
        let mut last_error = None;

        for attempt in 0..num_urls {
            let idx = {
                let mut next = self.next.lock().expect("mutex poisoned");
                let idx = *next;
                *next = (*next + 1) % num_urls;
                idx
            };

            debug!(
                "Attempting request on server #{} (attempt {}/{})",
                idx,
                attempt + 1,
                num_urls
            );

            // Get or create client for this index - block on async operation
            let client = {
                let rt = tokio::runtime::Handle::current();
                match rt.block_on(self.get_or_create_client(idx)) {
                    Ok(client) => client,
                    Err(e) => {
                        warn!(
                            "Failed to create client #{}: {:?}, trying next server",
                            idx, e
                        );
                        last_error = Some(e);
                        continue;
                    }
                }
            };

            // Execute the request synchronously
            match f(&client) {
                Ok(res) => {
                    debug!("Request successful on server #{}", idx);
                    return Ok(res);
                }
                Err(e) => {
                    if Self::should_retry_on_error(&e) {
                        warn!(
                            "Retryable error on server #{}: {:?}, trying next server",
                            idx, e
                        );
                        last_error = Some(e);
                        continue;
                    } else {
                        debug!(
                            "Non-retryable error on server #{}: {:?}, returning immediately",
                            idx, e
                        );
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
        F: Fn(Arc<Client>) -> Result<T, Error> + Send + Sync + Clone + 'static,
        T: Send + 'static,
    {
        info!(
            "Executing parallel requests on {} electrum servers",
            self.urls.len()
        );

        // Pre-create all clients asynchronously
        let mut all_clients = Vec::new();
        for idx in 0..self.urls.len() {
            match self.get_or_create_client(idx).await {
                Ok(client) => all_clients.push(Some(client)),
                Err(e) => {
                    warn!("Failed to create client #{}: {:?}", idx, e);
                    all_clients.push(None);
                }
            }
        }

        let tasks = all_clients
            .into_iter()
            .enumerate()
            .map(|(idx, client_opt)| {
                let f = f.clone();
                spawn_blocking(move || {
                    debug!("Starting parallel request on server #{}", idx);

                    let client = match client_opt {
                        Some(client) => client,
                        None => {
                            debug!("No client available for server #{}", idx);
                            return Err(Error::IOError(std::io::Error::new(
                                std::io::ErrorKind::Other,
                                format!("client #{} unavailable", idx),
                            )));
                        }
                    };

                    let result = f(client);
                    match &result {
                        Ok(_) => debug!("Parallel request succeeded on server #{}", idx),
                        Err(e) => debug!("Parallel request failed on server #{}: {:?}", idx, e),
                    }
                    result
                })
            });

        let results = join_all(tasks)
            .await
            .into_iter()
            .enumerate()
            .map(|(idx, res)| match res {
                Ok(r) => r,
                Err(e) => {
                    error!("Spawn error for server #{}: {}", idx, e);
                    Err(Error::IOError(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("spawn error: {e}"),
                    )))
                }
            })
            .collect::<Vec<_>>();

        let success_count = results.iter().filter(|r| r.is_ok()).count();
        let failure_count = results.len() - success_count;

        if failure_count > 0 {
            warn!(
                "Parallel execution completed: {}/{} succeeded, {}/{} failed",
                success_count,
                results.len(),
                failure_count,
                results.len()
            );
        } else {
            info!(
                "Parallel execution completed: all {}/{} requests succeeded",
                success_count,
                results.len()
            );
        }

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
            .join_all(move |client| client.transaction_broadcast(&tx))
            .await;

        let success_count = results.iter().filter(|r| r.is_ok()).count();

        if success_count > 0 {
            info!(
                "Transaction {} broadcast successful on {}/{} servers",
                txid,
                success_count,
                results.len()
            );
        } else {
            error!(
                "Transaction {} broadcast failed on all {} servers",
                txid,
                results.len()
            );
        }

        results
    }
}

impl ElectrumApi for ElectrumBalancer {
    #[instrument(level = "debug", skip(self, params), fields(method = method_name))]
    fn raw_call(
        &self,
        method_name: &str,
        params: impl IntoIterator<Item = bdk_electrum::electrum_client::Param>,
    ) -> Result<Value, Error> {
        debug!("Making raw call to method: {}", method_name);
        let method_str = method_name.to_string();
        let params_vec: Vec<_> = params.into_iter().collect();
        self.call(move |client| client.raw_call(&method_str, params_vec.clone()))
    }

    fn batch_call(&self, batch: &Batch) -> Result<Vec<Value>, Error> {
        self.call(|client| client.batch_call(batch))
    }

    fn block_headers_subscribe_raw(&self) -> Result<RawHeaderNotification, Error> {
        self.call(|client| client.block_headers_subscribe_raw())
    }

    fn block_headers_pop_raw(&self) -> Result<Option<RawHeaderNotification>, Error> {
        self.call(|client| client.block_headers_pop_raw())
    }

    fn block_header_raw(&self, height: usize) -> Result<Vec<u8>, Error> {
        self.call(move |client| client.block_header_raw(height))
    }

    fn block_headers(&self, start_height: usize, count: usize) -> Result<GetHeadersRes, Error> {
        self.call(move |client| client.block_headers(start_height, count))
    }

    #[instrument(level = "debug", skip(self), fields(target_blocks = number))]
    fn estimate_fee(&self, number: usize) -> Result<f64, Error> {
        debug!("Estimating fee for {} blocks", number);
        let result = self.call(move |client| client.estimate_fee(number));
        match &result {
            Ok(fee) => debug!("Fee estimation for {} blocks: {} BTC/kB", number, fee),
            Err(e) => debug!("Fee estimation failed for {} blocks: {:?}", number, e),
        }
        result
    }

    fn relay_fee(&self) -> Result<f64, Error> {
        self.call(|client| client.relay_fee())
    }

    fn script_subscribe(&self, script: &Script) -> Result<Option<ScriptStatus>, Error> {
        let script = script.to_owned();
        self.call(move |client| client.script_subscribe(&script))
    }

    fn batch_script_subscribe<'s, I>(&self, scripts: I) -> Result<Vec<Option<ScriptStatus>>, Error>
    where
        I: IntoIterator + Clone,
        I::Item: Borrow<&'s Script>,
    {
        self.call(move |client| client.batch_script_subscribe(scripts.clone()))
    }

    fn script_unsubscribe(&self, script: &Script) -> Result<bool, Error> {
        let script = script.to_owned();
        self.call(move |client| client.script_unsubscribe(&script))
    }

    fn script_pop(&self, script: &Script) -> Result<Option<ScriptStatus>, Error> {
        let script = script.to_owned();
        self.call(move |client| client.script_pop(&script))
    }

    fn script_get_balance(&self, script: &Script) -> Result<GetBalanceRes, Error> {
        let script = script.to_owned();
        self.call(move |client| client.script_get_balance(&script))
    }

    fn batch_script_get_balance<'s, I>(&self, scripts: I) -> Result<Vec<GetBalanceRes>, Error>
    where
        I: IntoIterator + Clone,
        I::Item: Borrow<&'s Script>,
    {
        self.call(move |client| client.batch_script_get_balance(scripts.clone()))
    }

    #[instrument(level = "debug", skip(self, script), fields(script_hash = %script.script_hash()))]
    fn script_get_history(&self, script: &Script) -> Result<Vec<GetHistoryRes>, Error> {
        debug!(
            "Getting script history for script hash: {}",
            script.script_hash()
        );
        let script = script.to_owned();
        let result = self.call(move |client| client.script_get_history(&script));
        match &result {
            Ok(history) => debug!("Script history retrieved: {} transactions", history.len()),
            Err(e) => debug!("Script history request failed: {:?}", e),
        }
        result
    }

    fn batch_script_get_history<'s, I>(&self, scripts: I) -> Result<Vec<Vec<GetHistoryRes>>, Error>
    where
        I: IntoIterator + Clone,
        I::Item: Borrow<&'s Script>,
    {
        self.call(move |client| client.batch_script_get_history(scripts.clone()))
    }

    fn script_list_unspent(&self, script: &Script) -> Result<Vec<ListUnspentRes>, Error> {
        let script = script.to_owned();
        self.call(move |client| client.script_list_unspent(&script))
    }

    fn batch_script_list_unspent<'s, I>(
        &self,
        scripts: I,
    ) -> Result<Vec<Vec<ListUnspentRes>>, Error>
    where
        I: IntoIterator + Clone,
        I::Item: Borrow<&'s Script>,
    {
        self.call(move |client| client.batch_script_list_unspent(scripts.clone()))
    }

    #[instrument(level = "debug", skip(self), fields(txid = %txid))]
    fn transaction_get_raw(&self, txid: &Txid) -> Result<Vec<u8>, Error> {
        debug!("Getting raw transaction: {}", txid);
        let txid = *txid;
        let result = self.call(move |client| client.transaction_get_raw(&txid));
        
        match &result {
            Ok(raw_tx) => {
                debug!("Raw transaction retrieved: {} bytes", raw_tx.len());
                result
            },
            Err(e) => {
                // Check if this is the specific "No such mempool or blockchain transaction" error
                let error_str = format!("{:?}", e);
                if error_str.contains("No such mempool or blockchain transaction") ||
                   error_str.contains("code': -5") {
                    debug!("Transaction {} not found in mempool or blockchain, treating as missing", txid);
                    // Return a specific error that indicates the transaction was not found
                    // This should be handled by the caller to return None appropriately
                    Err(Error::Protocol("Transaction not found".into()))
                } else {
                    debug!("Raw transaction request failed for {}: {:?}", txid, e);
                    result
                }
            }
        }
    }

    fn batch_transaction_get_raw<'t, I>(&self, txids: I) -> Result<Vec<Vec<u8>>, Error>
    where
        I: IntoIterator + Clone,
        I::Item: Borrow<&'t Txid>,
    {
        self.call(move |client| client.batch_transaction_get_raw(txids.clone()))
    }

    fn batch_block_header_raw<I>(&self, heights: I) -> Result<Vec<Vec<u8>>, Error>
    where
        I: IntoIterator + Clone,
        I::Item: Borrow<u32>,
    {
        self.call(move |client| client.batch_block_header_raw(heights.clone()))
    }

    fn batch_estimate_fee<I>(&self, numbers: I) -> Result<Vec<f64>, Error>
    where
        I: IntoIterator + Clone,
        I::Item: Borrow<usize>,
    {
        self.call(move |client| client.batch_estimate_fee(numbers.clone()))
    }

    #[instrument(level = "info", skip(self, raw_tx), fields(tx_size = raw_tx.len()))]
    fn transaction_broadcast_raw(&self, raw_tx: &[u8]) -> Result<Txid, Error> {
        info!("Broadcasting raw transaction ({} bytes)", raw_tx.len());
        let raw_tx = raw_tx.to_vec();
        let result = self.call(move |client| client.transaction_broadcast_raw(&raw_tx));
        match &result {
            Ok(txid) => info!("Raw transaction broadcast successful: {}", txid),
            Err(e) => warn!("Raw transaction broadcast failed: {:?}", e),
        }
        result
    }

    fn transaction_get_merkle(&self, txid: &Txid, height: usize) -> Result<GetMerkleRes, Error> {
        let txid = *txid;
        self.call(move |client| client.transaction_get_merkle(&txid, height))
    }

    fn txid_from_pos(&self, height: usize, tx_pos: usize) -> Result<Txid, Error> {
        self.call(move |client| client.txid_from_pos(height, tx_pos))
    }

    fn txid_from_pos_with_merkle(
        &self,
        height: usize,
        tx_pos: usize,
    ) -> Result<TxidFromPosRes, Error> {
        self.call(move |client| client.txid_from_pos_with_merkle(height, tx_pos))
    }

    fn server_features(&self) -> Result<ServerFeaturesRes, Error> {
        self.call(|client| client.server_features())
    }

    fn ping(&self) -> Result<(), Error> {
        self.call(|client| client.ping())
    }
}
