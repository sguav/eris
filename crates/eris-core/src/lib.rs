use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(tag = "type", content = "payload")]
pub enum Protocol {
    Login { username: String },
    JoinChannel { channel: String },
    Identify { id: Uuid, username: String },
    PeerList { peers: Vec<PeerInfo> },
    ChatMessage { channel: String, author: String, content: String },
    ScreenState { peer_id: Uuid, is_sharing: bool },
    RequestStream { target_id: Uuid },
    Signal { target_id: Uuid, data: serde_json::Value },
    System { message: String, severity: String },
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct PeerInfo {
    pub id: Uuid,
    pub username: String,
    pub channel: String,
    pub is_sharing: bool,
}
