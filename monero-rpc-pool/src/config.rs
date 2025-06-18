use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub data_dir: Option<PathBuf>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 18081,
            data_dir: None, // Use default data directory
        }
    }
}

// TODO: Use a builder library here
impl Config {
    pub fn from_args(host: Option<String>, port: Option<u16>) -> Self {
        let default = Self::default();
        Self {
            host: host.unwrap_or(default.host),
            port: port.unwrap_or(default.port),
            data_dir: None, // Use default data directory
        }
    }

    /// Creates a new config suitable for library usage with automatic discovery
    pub fn for_library(host: Option<String>, port: Option<u16>) -> Self {
        Self {
            host: host.unwrap_or_else(|| "127.0.0.1".to_string()),
            port: port.unwrap_or(0), // 0 for random port
            data_dir: None,          // Use default data directory
        }
    }

    /// Creates a new config for library usage with a custom data directory
    pub fn for_library_with_data_dir(
        host: Option<String>,
        port: Option<u16>,
        data_dir: PathBuf,
    ) -> Self {
        Self {
            host: host.unwrap_or_else(|| "127.0.0.1".to_string()),
            port: port.unwrap_or(0), // 0 for random port
            data_dir: Some(data_dir),
        }
    }
}
