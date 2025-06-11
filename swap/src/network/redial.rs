use backoff::backoff::Backoff;
use backoff::ExponentialBackoff;
use futures::future::FutureExt;
use libp2p::core::Multiaddr;
use libp2p::swarm::dial_opts::{DialOpts, PeerCondition};
use libp2p::swarm::{NetworkBehaviour, ToSwarm};
use libp2p::PeerId;
use std::collections::VecDeque;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;
use tokio::time::{Instant, Sleep};
use void::Void;

use crate::cli;
use super::connection_progress::{ConnectionProgress, ErrorCategory, categorize_error};

/// A [`NetworkBehaviour`] that tracks whether we are connected to the given
/// peer and attempts to re-establish a connection with an exponential backoff
/// if we lose the connection.
pub struct Behaviour {
    /// The peer we are interested in.
    peer: PeerId,
    /// If present, tracks for how long we need to sleep until we dial again.
    sleep: Option<Pin<Box<Sleep>>>,
    /// Tracks the current backoff state.
    backoff: ExponentialBackoff,
    /// Enhanced connection progress tracking
    progress: ConnectionProgress,
    /// Queue of events to emit
    pending_events: VecDeque<RedialEvent>,
}

/// Events that can be emitted by the redial behavior
#[derive(Debug, Clone)]
pub enum RedialEvent {
    /// Connection progress update
    ProgressUpdate(ConnectionProgressUpdate),
    /// Request to dial peer
    Dial(PeerId),
}

impl Behaviour {
    pub fn new(peer: PeerId, interval: Duration, max_interval: Duration) -> Self {
        let target = format!("{}", peer);
        Self {
            peer,
            sleep: None,
            backoff: ExponentialBackoff {
                initial_interval: interval,
                current_interval: interval,
                max_interval,
                max_elapsed_time: None, // We never give up on re-dialling
                ..ExponentialBackoff::default()
            },
            progress: ConnectionProgress::new(target, None), // Unlimited retries
            pending_events: VecDeque::new(),
        }
    }

    pub fn until_next_redial(&self) -> Option<Duration> {
        let until_next_redial = self
            .sleep
            .as_ref()?
            .deadline()
            .checked_duration_since(Instant::now())?;

        Some(until_next_redial)
    }

    /// Get current connection progress information
    pub fn connection_progress(&self) -> &ConnectionProgress {
        &self.progress
    }

    /// Update connection progress with new error information
    fn record_connection_failure(&mut self, error: String) {
        let category = categorize_error(&error);
        let retry_in = self.until_next_redial();
        self.progress.record_failure(error, category, retry_in);
        
        // Queue progress update event
        let progress_update = ConnectionProgressUpdate {
            peer_id: self.peer,
            progress: self.progress.clone(),
        };
        self.pending_events.push_back(RedialEvent::ProgressUpdate(progress_update));
        
        // Log the enhanced progress message
        tracing::info!("{}", self.progress.format_message());
        
        if let Some(duration) = retry_in {
            tracing::info!(
                seconds_until_next_redial = %duration.as_secs(), 
                total_attempts = self.progress.total_attempts,
                error_category = ?self.progress.error_category,
                "Enhanced connection progress tracking"
            );
        }
    }

    /// Record successful connection
    fn record_connection_success(&mut self) {
        self.progress.record_success();
        
        // Queue progress update event
        let progress_update = ConnectionProgressUpdate {
            peer_id: self.peer,
            progress: self.progress.clone(),
        };
        self.pending_events.push_back(RedialEvent::ProgressUpdate(progress_update));
        
        tracing::info!(
            peer_id = %self.peer,
            total_attempts = self.progress.total_attempts,
            elapsed_time = ?self.progress.elapsed_time(),
            "Successfully connected after {} attempts",
            self.progress.total_attempts
        );
    }
}

impl NetworkBehaviour for Behaviour {
    type ConnectionHandler = libp2p::swarm::dummy::ConnectionHandler;
    type ToSwarm = ConnectionProgressUpdate;

    fn handle_established_inbound_connection(
        &mut self,
        _connection_id: libp2p::swarm::ConnectionId,
        peer: PeerId,
        _local_addr: &Multiaddr,
        _remote_addr: &Multiaddr,
    ) -> Result<libp2p::swarm::THandler<Self>, libp2p::swarm::ConnectionDenied> {
        // We establish an inbound connection to the peer we are interested in.
        // We stop re-dialling.
        // Reset the backoff state to start with the initial interval again once we disconnect again
        if peer == self.peer {
            self.backoff.reset();
            self.sleep = None;
            self.record_connection_success();
        }
        Ok(Self::ConnectionHandler {})
    }

    fn handle_established_outbound_connection(
        &mut self,
        _connection_id: libp2p::swarm::ConnectionId,
        peer: PeerId,
        _addr: &Multiaddr,
        _role_override: libp2p::core::Endpoint,
    ) -> Result<libp2p::swarm::THandler<Self>, libp2p::swarm::ConnectionDenied> {
        // We establish an outbound connection to the peer we are interested in.
        // We stop re-dialling.
        // Reset the backoff state to start with the initial interval again once we disconnect again
        if peer == self.peer {
            self.backoff.reset();
            self.sleep = None;
            self.record_connection_success();
        }
        Ok(Self::ConnectionHandler {})
    }

    fn on_swarm_event(&mut self, event: libp2p::swarm::FromSwarm<'_>) {
        let redial = match &event {
            libp2p::swarm::FromSwarm::ConnectionClosed(e) if e.peer_id == self.peer => {
                let error = format!(
                    "Connection closed to peer {} (endpoint: {:?}, remaining: {})",
                    e.peer_id, e.endpoint, e.remaining_established
                );
                self.progress.record_disconnection(error, ErrorCategory::PeerUnavailable);
                
                // Queue progress update event
                let progress_update = ConnectionProgressUpdate {
                    peer_id: self.peer,
                    progress: self.progress.clone(),
                };
                self.pending_events.push_back(RedialEvent::ProgressUpdate(progress_update));
                true
            }
            libp2p::swarm::FromSwarm::DialFailure(e) if e.peer_id == Some(self.peer) => {
                let error = format!("Dial failure: {}", e.error);
                self.record_connection_failure(error);
                true
            }
            _ => false,
        };

        if redial && self.sleep.is_none() {
            self.sleep = Some(Box::pin(tokio::time::sleep(self.backoff.initial_interval)));
        }
    }

    fn poll(&mut self, cx: &mut Context<'_>) -> std::task::Poll<ToSwarm<Self::ToSwarm, Void>> {
        // First, check if we have any pending events to emit
        if let Some(event) = self.pending_events.pop_front() {
            return match event {
                RedialEvent::ProgressUpdate(update) => {
                    Poll::Ready(ToSwarm::GenerateEvent(update))
                }
                RedialEvent::Dial(peer_id) => {
                    Poll::Ready(ToSwarm::Dial {
                        opts: DialOpts::peer_id(peer_id)
                            .condition(PeerCondition::Disconnected)
                            .build(),
                    })
                }
            };
        }

        let sleep = match self.sleep.as_mut() {
            None => return Poll::Pending, // early exit if we shouldn't be re-dialling
            Some(future) => future,
        };

        futures::ready!(sleep.poll_unpin(cx));

        let next_dial_in = match self.backoff.next_backoff() {
            Some(next_dial_in) => next_dial_in,
            None => {
                unreachable!("The backoff should never run out of attempts");
            }
        };

        // Record the new attempt and queue progress update
        self.progress.start_attempt();
        let progress_update = ConnectionProgressUpdate {
            peer_id: self.peer,
            progress: self.progress.clone(),
        };
        self.pending_events.push_back(RedialEvent::ProgressUpdate(progress_update));

        self.sleep = Some(Box::pin(tokio::time::sleep(next_dial_in)));

        // Queue the dial event
        self.pending_events.push_back(RedialEvent::Dial(self.peer));

        // Return the first event (progress update)
        if let Some(event) = self.pending_events.pop_front() {
            match event {
                RedialEvent::ProgressUpdate(update) => {
                    Poll::Ready(ToSwarm::GenerateEvent(update))
                }
                RedialEvent::Dial(peer_id) => {
                    Poll::Ready(ToSwarm::Dial {
                        opts: DialOpts::peer_id(peer_id)
                            .condition(PeerCondition::Disconnected)
                            .build(),
                    })
                }
            }
        } else {
            Poll::Pending
        }
    }

    fn on_connection_handler_event(
        &mut self,
        _peer_id: PeerId,
        _connection_id: libp2p::swarm::ConnectionId,
        _event: libp2p::swarm::THandlerOutEvent<Self>,
    ) {
        unreachable!("The re-dial dummy connection handler does not produce any events");
    }
}

/// Event emitted when connection progress is updated
#[derive(Debug, Clone)]
pub struct ConnectionProgressUpdate {
    pub peer_id: PeerId,
    pub progress: ConnectionProgress,
}

impl From<ConnectionProgressUpdate> for cli::OutEvent {
    fn from(update: ConnectionProgressUpdate) -> Self {
        Self::ConnectionProgress(update)
    }
}

// Update the existing From<()> implementation
impl From<()> for cli::OutEvent {
    fn from(_: ()) -> Self {
        Self::Other
    }
}
