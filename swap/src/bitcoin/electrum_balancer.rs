use std::sync::{Arc, Mutex};

use futures::future::join_all;
use tokio::task::spawn_blocking;

use bdk_electrum::electrum_client::{Client, Error};
use bitcoin::Transaction;
use tracing::warn;

/// Round-robin load balancer for Electrum connections.
///
/// The balancer will try each Electrum node until the provided
/// closure succeeds or all nodes have returned an I/O error.
/// Any non I/O error is immediately returned to the caller.
pub struct ElectrumBalancer {
    clients: Vec<Arc<Client>>,
    next: Mutex<usize>,
}

impl ElectrumBalancer {
    /// Create a new balancer from a list of Electrum URLs.
    pub fn new(urls: Vec<String>) -> Result<Self, Error> {
        let clients = urls
            .into_iter()
            .map(|url| Client::new(&url).map(Arc::new))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            clients,
            next: Mutex::new(0),
        })
    }

    /// Execute the given closure using one of the Electrum clients.
    ///
    /// If the closure returns an I/O error the balancer will try the next
    /// node until all nodes have been exhausted. The last encountered error
    /// is returned in that case.
    pub fn call<F, T>(&self, mut f: F) -> Result<T, Error>
    where
        F: FnMut(&Client) -> Result<T, Error>,
    {
        let num_clients = self.clients.len();
        for _ in 0..num_clients {
            let idx = {
                let mut next = self.next.lock().expect("mutex poisoned");
                let idx = *next;
                *next = (*next + 1) % num_clients;
                idx
            };
            let client = &self.clients[idx];
            match f(client) {
                Ok(res) => return Ok(res),
                Err(e) => {
                    if matches!(e, Error::Io(_)) {
                        warn!(?e, "Electrum IO error, trying next node");
                        continue;
                    } else {
                        return Err(e);
                    }
                }
            }
        }
        Err(Error::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            "all electrum nodes failed",
        )))
    }

    /// Execute the given closure on **all** Electrum nodes in parallel.
    ///
    /// The closure is executed in a blocking task for each client.
    /// The resulting `Result`s are collected and returned in the same
    /// order as the nodes were provided during construction.
    pub async fn join_all<F, T>(&self, f: F) -> Vec<Result<T, Error>>
    where
        F: Fn(Arc<Client>) -> Result<T, Error> + Send + Sync + Clone + 'static,
        T: Send + 'static,
    {
        let tasks = self.clients.iter().cloned().map(|client| {
            let f = f.clone();
            spawn_blocking(move || f(client))
        });

        join_all(tasks)
            .await
            .into_iter()
            .map(|res| match res {
                Ok(r) => r,
                Err(e) => Err(Error::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("spawn error: {e}"),
                ))),
            })
            .collect()
    }

    /// Broadcast the given transaction to all Electrum nodes in parallel.
    ///
    /// The method returns a list of results in the same order as the
    /// configured nodes. Errors for individual nodes do not abort the
    /// others.
    pub async fn broadcast_all(
        &self,
        tx: &Transaction,
    ) -> Vec<Result<bitcoin::Txid, Error>> {
        self
            .join_all(|client| client.transaction_broadcast(tx))
            .await
    }
}

