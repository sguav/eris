use dashmap::DashMap;
use std::sync::Mutex;
use tokio::sync::broadcast;
use uuid::Uuid;
use crate::protocol::{PeerInfo, Protocol};

pub struct Peer {
    pub username: String,
    pub channel: String,
    pub tx: broadcast::Sender<Protocol>,
}

pub struct AppState {
    pub peers: DashMap<Uuid, Peer>,
    pub broadcast_tx: broadcast::Sender<Protocol>,
    pub history: Mutex<Vec<Protocol>>,
    pub invite_token: String,
}

impl AppState {
    pub fn new(broadcast_tx: broadcast::Sender<Protocol>, invite_token: String) -> Self {
        Self {
            peers: DashMap::new(),
            broadcast_tx,
            history: Mutex::new(Vec::new()),
            invite_token,
        }
    }

    pub fn broadcast_peer_list(&self) {
        let peers: Vec<PeerInfo> = self.peers.iter().map(|p| PeerInfo {
            id: *p.key(),
            username: p.username.clone(),
            channel: p.channel.clone(),
        }).collect();
        let _ = self.broadcast_tx.send(Protocol::PeerList { peers });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_history_buffer_limit() {
        let (tx, _) = broadcast::channel(10);
        let state = AppState::new(tx, "token".to_string());

        for i in 0..60 {
            let msg = Protocol::ChatMessage { 
                channel: "lobby".to_string(),
                author: "System".to_string(), 
                content: format!("Msg {}", i) 
            };
            let mut history = state.history.lock().unwrap();
            history.push(msg);
            if history.len() > 50 {
                history.remove(0);
            }
        }

        let history = state.history.lock().unwrap();
        assert_eq!(history.len(), 50);
        if let Protocol::ChatMessage { content, .. } = &history[0] {
            assert_eq!(content, "Msg 10");
        }
    }

    #[test]
    fn test_username_uniqueness_logic() {
        let (tx, _) = broadcast::channel(10);
        let state = AppState::new(tx, "token".to_string());

        let uid = Uuid::new_v4();
        state.peers.insert(uid, Peer {
            username: "Alice".to_string(),
            channel: "lobby".to_string(),
            tx: broadcast::channel(1).0,
        });

        let name_to_check = "alice";
        let is_taken = state.peers.iter().any(|p| p.username.to_lowercase() == name_to_check.to_lowercase());
        assert!(is_taken);
    }

    #[test]
    fn test_signal_routing_logic() {
        let (tx, _) = broadcast::channel(10);
        let state = AppState::new(tx, "token".to_string());

        let id_a = Uuid::new_v4();
        let (tx_a, mut rx_a) = broadcast::channel(10);
        state.peers.insert(id_a, Peer {
            username: "Alice".to_string(),
            channel: "lobby".to_string(),
            tx: tx_a,
        });

        let id_b = Uuid::new_v4();
        let (tx_b, _) = broadcast::channel(10);
        state.peers.insert(id_b, Peer {
            username: "Bob".to_string(),
            channel: "lobby".to_string(),
            tx: tx_b,
        });

        // Simulate a Signal from Bob to Alice
        let signal_data = serde_json::json!({"sdp": "..."});
        if let Some(target) = state.peers.get(&id_a) {
            let _ = target.tx.send(Protocol::Signal { 
                target_id: id_b, // The sender's ID
                data: signal_data.clone() 
            });
        }

        // Alice should receive the signal
        let received = rx_a.try_recv().unwrap();
        if let Protocol::Signal { target_id, data } = received {
            assert_eq!(target_id, id_b);
            assert_eq!(data, signal_data);
        } else {
            panic!("Expected Signal, got {:?}", received);
        }
    }
}
