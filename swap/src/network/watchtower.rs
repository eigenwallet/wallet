use libp2p::{
    request_response::{json, Config, Event, Message, ProtocolSupport},
    StreamProtocol,
};
use serde::{Deserialize, Serialize};

const PROTOCOL: &str = "/unstoppableswap/xmr/btc/watchtower/0.1.0";

pub type WatchtowerBehaviour = json::Behaviour<WatchtowerRequest, WatchtowerResponse>;
pub type WatchtowerEvent = Event<WatchtowerRequest, WatchtowerResponse>;
pub type WatchtowerMessage = Message<WatchtowerRequest, WatchtowerResponse>;

#[derive(Debug, Clone, Copy, Default)]
pub struct WatchtowerProtocol;

impl AsRef<str> for WatchtowerProtocol {
    fn as_ref(&self) -> &str {
        PROTOCOL
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct WatchtowerRequest {
    pub raw_tx: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct WatchtowerResponse {
    accepted: bool,
}

impl WatchtowerResponse {
    pub fn accepted() -> Self {
        Self { accepted: true }
    }

    pub fn denied() -> Self {
        Self { accepted: false }
    }
}

pub fn master() -> WatchtowerBehaviour {
    json::Behaviour::new(
        vec![(StreamProtocol::new(PROTOCOL), ProtocolSupport::Inbound)],
        Config::default(),
    )
}

pub fn slave() -> WatchtowerBehaviour {
    json::Behaviour::new(
        vec![(StreamProtocol::new(PROTOCOL), ProtocolSupport::Outbound)],
        Config::default(),
    )
}
