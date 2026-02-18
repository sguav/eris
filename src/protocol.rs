use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(tag = "type", content = "payload")]
pub enum Protocol {
    Login { username: String },
    Identify { id: Uuid, username: String },
    PeerList { peers: Vec<PeerInfo> },
    ChatMessage { author: String, content: String },
    Signal { target_id: Uuid, data: serde_json::Value },
    System { message: String, severity: String },
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct PeerInfo {
    pub id: Uuid,
    pub username: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_protocol_serialization() {
        let msg = Protocol::ChatMessage { 
            author: "Alice".to_string(), 
            content: "Hello".to_string() 
        };
        let json = serde_json::to_value(&msg).unwrap();
        assert_eq!(json, json!({
            "type": "ChatMessage",
            "payload": {
                "author": "Alice",
                "content": "Hello"
            }
        }));

        let login = Protocol::Login { username: "Bob".to_string() };
        let json = serde_json::to_value(&login).unwrap();
        assert_eq!(json, json!({
            "type": "Login",
            "payload": {
                "username": "Bob"
            }
        }));
    }
}
