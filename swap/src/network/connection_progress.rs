use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// Represents the current state of a connection progress
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionProgress {
    /// Current attempt number (1-based)
    pub current_attempt: u32,
    /// Total attempts made so far
    pub total_attempts: u32,
    /// Remaining retries (None if unlimited)
    pub retries_left: Option<u32>,
    /// Description of the last error encountered
    pub last_error: String,
    /// Error category for better handling
    pub error_category: ErrorCategory,
    /// Time until next retry attempt
    pub next_retry_in: Option<Duration>,
    /// Time when the connection process started (not serialized)
    #[serde(skip, default = "Instant::now")]
    pub started_at: Instant,
    /// Current connection state
    pub state: ConnectionState,
    /// Target peer/address being connected to
    pub target: String,
}

/// Categories of connection errors for better handling and user messaging
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorCategory {
    /// Network connectivity issues (DNS, routing, etc.)
    Network,
    /// Connection timeout
    Timeout,
    /// Authentication or authorization failures
    Auth,
    /// Protocol-level errors
    Protocol,
    /// Remote peer is unavailable or rejecting connections
    PeerUnavailable,
    /// Resource exhaustion (too many connections, etc.)
    Resource,
    /// Unknown or uncategorized error
    Unknown,
}

/// Current state of the connection process
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionState {
    /// Initial state, no attempts made yet
    Initial,
    /// Currently attempting to connect
    Connecting,
    /// Waiting before next retry attempt
    WaitingToRetry,
    /// Successfully connected
    Connected,
    /// Failed permanently (no more retries)
    Failed,
    /// Connection lost, preparing to reconnect
    Reconnecting,
}

impl ConnectionProgress {
    /// Create a new connection progress tracker
    pub fn new(target: String, max_retries: Option<u32>) -> Self {
        Self {
            current_attempt: 0,
            total_attempts: 0,
            retries_left: max_retries,
            last_error: String::new(),
            error_category: ErrorCategory::Unknown,
            next_retry_in: None,
            started_at: Instant::now(),
            state: ConnectionState::Initial,
            target,
        }
    }

    /// Record a new connection attempt
    pub fn start_attempt(&mut self) {
        self.current_attempt += 1;
        self.total_attempts += 1;
        self.state = ConnectionState::Connecting;
        self.next_retry_in = None;
    }

    /// Record a failed connection attempt
    pub fn record_failure(&mut self, error: String, category: ErrorCategory, retry_in: Option<Duration>) {
        self.last_error = error;
        self.error_category = category;
        self.next_retry_in = retry_in;
        
        if let Some(retries) = &mut self.retries_left {
            if *retries > 0 {
                *retries -= 1;
                self.state = ConnectionState::WaitingToRetry;
            } else {
                self.state = ConnectionState::Failed;
            }
        } else {
            // Unlimited retries
            self.state = ConnectionState::WaitingToRetry;
        }
    }

    /// Record a successful connection
    pub fn record_success(&mut self) {
        self.state = ConnectionState::Connected;
        self.last_error.clear();
        self.next_retry_in = None;
    }

    /// Record connection lost (for reconnection scenarios)
    pub fn record_disconnection(&mut self, error: String, category: ErrorCategory) {
        self.last_error = error;
        self.error_category = category;
        self.state = ConnectionState::Reconnecting;
    }

    /// Get total elapsed time since connection process started
    pub fn elapsed_time(&self) -> Duration {
        self.started_at.elapsed()
    }

    /// Check if the connection process should continue
    pub fn should_continue(&self) -> bool {
        match self.state {
            ConnectionState::Failed => false,
            _ => self.retries_left.map_or(true, |retries| retries > 0),
        }
    }

    /// Format a user-friendly progress message
    pub fn format_message(&self) -> String {
        match self.state {
            ConnectionState::Initial => format!("Preparing to connect to {}", self.target),
            ConnectionState::Connecting => {
                format!("Connecting to {} (attempt {})", self.target, self.current_attempt)
            }
            ConnectionState::WaitingToRetry => {
                let retry_info = if let Some(retries) = self.retries_left {
                    format!("{} retries left", retries)
                } else {
                    "unlimited retries".to_string()
                };
                
                let time_info = if let Some(duration) = self.next_retry_in {
                    format!(" in {}s", duration.as_secs())
                } else {
                    String::new()
                };

                format!(
                    "Trying to reconnect to {} (Last Error: {}, {} times failed, {}){}",
                    self.target,
                    self.format_error_type(),
                    self.total_attempts,
                    retry_info,
                    time_info
                )
            }
            ConnectionState::Connected => format!("Connected to {}", self.target),
            ConnectionState::Failed => {
                format!("Failed to connect to {} after {} attempts", self.target, self.total_attempts)
            }
            ConnectionState::Reconnecting => {
                format!("Connection to {} lost, attempting to reconnect", self.target)
            }
        }
    }

    /// Format error type in a user-friendly way
    fn format_error_type(&self) -> &str {
        match self.error_category {
            ErrorCategory::Network => "Network Error",
            ErrorCategory::Timeout => "Connection Timeout",
            ErrorCategory::Auth => "Authentication Failed",
            ErrorCategory::Protocol => "Protocol Error",
            ErrorCategory::PeerUnavailable => "Peer Unavailable",
            ErrorCategory::Resource => "Resource Exhaustion",
            ErrorCategory::Unknown => "Unknown Error",
        }
    }

    /// Get actionable suggestions for the user based on the error category
    pub fn get_user_suggestions(&self) -> Vec<String> {
        match self.error_category {
            ErrorCategory::Network => vec![
                "Check your internet connection".to_string(),
                "Verify DNS settings".to_string(),
                "Try connecting to a different network".to_string(),
            ],
            ErrorCategory::Timeout => vec![
                "Check if the remote service is running".to_string(),
                "Try again in a few minutes".to_string(),
                "Consider using a different endpoint".to_string(),
            ],
            ErrorCategory::Auth => vec![
                "Check your credentials".to_string(),
                "Verify your account permissions".to_string(),
            ],
            ErrorCategory::Protocol => vec![
                "Update the application".to_string(),
                "Check for compatibility issues".to_string(),
            ],
            ErrorCategory::PeerUnavailable => vec![
                "The peer may be offline".to_string(),
                "Try connecting to a different peer".to_string(),
                "Check the peer's address".to_string(),
            ],
            ErrorCategory::Resource => vec![
                "Wait a moment and try again".to_string(),
                "Close other network-intensive applications".to_string(),
            ],
            ErrorCategory::Unknown => vec![
                "Check application logs for more details".to_string(),
                "Try restarting the application".to_string(),
            ],
        }
    }
}

/// Helper function to categorize errors based on error messages
pub fn categorize_error(error_msg: &str) -> ErrorCategory {
    let error_lower = error_msg.to_lowercase();
    
    if error_lower.contains("timeout") || error_lower.contains("timed out") {
        ErrorCategory::Timeout
    } else if error_lower.contains("dns") || error_lower.contains("network") || error_lower.contains("unreachable") {
        ErrorCategory::Network
    } else if error_lower.contains("auth") || error_lower.contains("unauthorized") || error_lower.contains("forbidden") {
        ErrorCategory::Auth
    } else if error_lower.contains("protocol") || error_lower.contains("handshake") {
        ErrorCategory::Protocol
    } else if error_lower.contains("refused") || error_lower.contains("unavailable") || error_lower.contains("offline") {
        ErrorCategory::PeerUnavailable
    } else if error_lower.contains("resource") || error_lower.contains("limit") || error_lower.contains("exhausted") {
        ErrorCategory::Resource
    } else {
        ErrorCategory::Unknown
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_progress_creation() {
        let progress = ConnectionProgress::new("test-peer".to_string(), Some(5));
        assert_eq!(progress.current_attempt, 0);
        assert_eq!(progress.total_attempts, 0);
        assert_eq!(progress.retries_left, Some(5));
        assert_eq!(progress.state, ConnectionState::Initial);
    }

    #[test]
    fn test_attempt_tracking() {
        let mut progress = ConnectionProgress::new("test-peer".to_string(), Some(3));
        
        progress.start_attempt();
        assert_eq!(progress.current_attempt, 1);
        assert_eq!(progress.total_attempts, 1);
        assert_eq!(progress.state, ConnectionState::Connecting);
    }

    #[test]
    fn test_failure_handling() {
        let mut progress = ConnectionProgress::new("test-peer".to_string(), Some(2));
        
        progress.start_attempt();
        progress.record_failure("timeout".to_string(), ErrorCategory::Timeout, Some(Duration::from_secs(5)));
        
        assert_eq!(progress.retries_left, Some(1));
        assert_eq!(progress.state, ConnectionState::WaitingToRetry);
        assert_eq!(progress.error_category, ErrorCategory::Timeout);
    }

    #[test]
    fn test_error_categorization() {
        assert_eq!(categorize_error("connection timed out"), ErrorCategory::Timeout);
        assert_eq!(categorize_error("DNS resolution failed"), ErrorCategory::Network);
        assert_eq!(categorize_error("authentication failed"), ErrorCategory::Auth);
        assert_eq!(categorize_error("connection refused"), ErrorCategory::PeerUnavailable);
    }

    #[test]
    fn test_message_formatting() {
        let mut progress = ConnectionProgress::new("Alice".to_string(), Some(20));
        
        progress.start_attempt();
        for _ in 0..11 {
            progress.start_attempt();
            progress.record_failure("timeout".to_string(), ErrorCategory::Timeout, Some(Duration::from_secs(30)));
        }
        
        let message = progress.format_message();
        assert!(message.contains("Trying to reconnect"));
        assert!(message.contains("Connection Timeout"));
        assert!(message.contains("12 times failed"));
        assert!(message.contains("retries left"));
    }
}