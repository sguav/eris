use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// The core signaling protocol for Eris.
///
/// # Examples
/// ```
/// use eris_core::Protocol;
/// let msg = Protocol::System { message: "test".into(), severity: "info".into() };
/// let json = serde_json::to_string(&msg).unwrap();
/// assert!(json.contains("test"));
/// ```
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

pub fn log(component: &str, message: &str) {
    let now = chrono::Local::now();
    println!("[{}] [{}] {}", now.format("%Y-%m-%d %H:%M:%S"), component, message);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_variants() {
        // Simple test to ensure variants exist and are usable
        let login = Protocol::Login { username: "Tester".to_string() };
        if let Protocol::Login { username } = login {
            assert_eq!(username, "Tester");
        }
    }
}
