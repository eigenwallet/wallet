use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub nodes: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 18081,
            nodes: vec![
                "https://node.moneroworld.com:18089".to_string(),
                "https://node1.moneroworld.com:18089".to_string(),
                "https://node2.moneroworld.com:18089".to_string(),
                "https://xmr-node.cakewallet.com:18081".to_string(),
                "https://opennode.xmr-tw.org:18089".to_string(),
            ],
        }
    }
}

impl Config {
    pub fn from_args(host: Option<String>, port: Option<u16>, nodes: Option<Vec<String>>) -> Self {
        let default = Self::default();
        Self {
            host: host.unwrap_or(default.host),
            port: port.unwrap_or(default.port),
            nodes: nodes.unwrap_or(default.nodes),
        }
    }
}
