//! ERIS UNIFIED CORE
//! Purpose: Server Signaling + UI Web Server

use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::State,
    response::IntoResponse,
    routing::get,
    Router,
};
use dashmap::DashMap;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;
use tower_http::services::ServeDir;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(tag = "type", content = "payload")]
enum Protocol {
    Login { username: String },
    Identify { id: Uuid, username: String },
    PeerList { peers: Vec<PeerInfo> },
    ChatMessage { author: String, content: String },
    Signal { target_id: Uuid, data: serde_json::Value },
    System { message: String, severity: String },
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
struct PeerInfo {
    id: Uuid,
    username: String,
}

struct Peer {
    username: String,
    tx: broadcast::Sender<Protocol>,
}

struct AppState {
    peers: DashMap<Uuid, Peer>,
    broadcast_tx: broadcast::Sender<Protocol>,
    history: std::sync::Mutex<Vec<Protocol>>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let (broadcast_tx, _) = broadcast::channel(2048);
    let state = Arc::new(AppState {
        peers: DashMap::new(),
        broadcast_tx,
        history: std::sync::Mutex::new(Vec::new()),
    });

    let app = Router::new()
        .route("/ws", get(ws_handler))
        .fallback_service(ServeDir::new("./www"))
        .layer(tower_http::cors::CorsLayer::permissive())
        .with_state(state);

    let addr = "0.0.0.0:8080";
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    
    println!("--------------------------------------------------");
    println!("🌑 ERIS UNIFIED CORE ONLINE");
    println!("🌍 Web UI: http://localhost:8080");
    println!("📡 Signaling: ws://localhost:8080/ws");
    println!("--------------------------------------------------");
    
    axum::serve(listener, app).await.unwrap();
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();
    let my_id = Uuid::new_v4();
    let (peer_tx, mut peer_rx) = broadcast::channel(100);
    
    let mut chat_rx = state.broadcast_tx.subscribe();
    let mut my_username = String::new();

    // --- LOGIN HANDSHAKE ---
    while let Some(msg_res) = receiver.next().await {
        let Ok(msg) = msg_res else { break; };
        
        // Skip non-text messages during handshake (e.g. Ping)
        let Message::Text(text) = msg else { continue; };
        
        if let Ok(Protocol::Login { username }) = serde_json::from_str::<Protocol>(&text) {
            let name = username.trim();
            if name.is_empty() { continue; }
            
            let is_taken = state.peers.iter().any(|p| p.username.to_lowercase() == name.to_lowercase());
            
            if is_taken {
                let err = Protocol::System { 
                    message: "Username already taken".to_string(), 
                    severity: "error".to_string() 
                };
                if let Ok(txt) = serde_json::to_string(&err) {
                    let _ = sender.send(Message::Text(txt)).await;
                }
                continue;
            }

            my_username = name.to_string();
            state.peers.insert(my_id, Peer { username: my_username.clone(), tx: peer_tx.clone() });
            
            let ident = Protocol::Identify { id: my_id, username: my_username.clone() };
            if let Ok(msg_json) = serde_json::to_string(&ident) {
                let _ = sender.send(Message::Text(msg_json)).await;
            }
            
            // Send history
            let history_snapshot = {
                let history = state.history.lock().unwrap();
                history.clone()
            };
            
            for msg in history_snapshot {
                if let Ok(txt) = serde_json::to_string(&msg) {
                    if sender.send(Message::Text(txt)).await.is_err() { break; }
                }
            }
            
            let _ = state.broadcast_tx.send(Protocol::System { 
                message: format!("{} joined", my_username), 
                severity: "info".to_string() 
            });

            broadcast_peer_list(&state);
            break;
        }
    }

    if my_username.is_empty() { return; }

    // --- MAIN MESSAGE LOOP ---
    let mut send_task = tokio::spawn(async move {
        loop {
            tokio::select! {
                Ok(msg) = chat_rx.recv() => {
                    if let Ok(txt) = serde_json::to_string(&msg) {
                        if sender.send(Message::Text(txt)).await.is_err() { break; }
                    }
                }
                Ok(msg) = peer_rx.recv() => {
                    if let Ok(txt) = serde_json::to_string(&msg) {
                        if sender.send(Message::Text(txt)).await.is_err() { break; }
                    }
                }
            }
        }
    });

    let state_clone = state.clone();
    let my_username_clone = my_username.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = receiver.next().await {
            if let Ok(protocol_msg) = serde_json::from_str::<Protocol>(&text) {
                match protocol_msg {
                    Protocol::ChatMessage { content, .. } => {
                        let msg = Protocol::ChatMessage { 
                            author: my_username_clone.clone(), 
                            content 
                        };
                        
                        // Save to history
                        {
                            let mut history = state_clone.history.lock().unwrap();
                            history.push(msg.clone());
                            if history.len() > 50 { history.remove(0); }
                        }

                        let _ = state_clone.broadcast_tx.send(msg);
                    }
                    Protocol::Signal { target_id, data } => {
                        if let Some(target) = state_clone.peers.get(&target_id) {
                            let _ = target.tx.send(Protocol::Signal { target_id: my_id, data });
                        }
                    }
                    _ => {}
                }
            }
        }
    });

    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };

    state.peers.remove(&my_id);
    let _ = state.broadcast_tx.send(Protocol::System { 
        message: format!("{} left", my_username), 
        severity: "info".to_string() 
    });
    broadcast_peer_list(&state);
}

fn broadcast_peer_list(state: &Arc<AppState>) {
    let peers: Vec<PeerInfo> = state.peers.iter().map(|p| PeerInfo {
        id: *p.key(),
        username: p.username.clone(),
    }).collect();
    let _ = state.broadcast_tx.send(Protocol::PeerList { peers });
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_protocol_serialization() {
        // Test ChatMessage serialization
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

        // Test Login serialization
        let login = Protocol::Login { username: "Bob".to_string() };
        let json = serde_json::to_value(&login).unwrap();
        assert_eq!(json, json!({
            "type": "Login",
            "payload": {
                "username": "Bob"
            }
        }));
    }

    #[test]
    fn test_history_buffer_limit() {
        let (tx, _) = broadcast::channel(10);
        let state = AppState {
            peers: DashMap::new(),
            broadcast_tx: tx,
            history: std::sync::Mutex::new(Vec::new()),
        };

        // Fill history beyond limit
        for i in 0..60 {
            let msg = Protocol::ChatMessage { 
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
        let state = AppState {
            peers: DashMap::new(),
            broadcast_tx: tx,
            history: std::sync::Mutex::new(Vec::new()),
        };

        let uid = Uuid::new_v4();
        state.peers.insert(uid, Peer {
            username: "Alice".to_string(),
            tx: broadcast::channel(1).0,
        });

        let name_to_check = "alice"; // Case insensitive check
        let is_taken = state.peers.iter().any(|p| p.username.to_lowercase() == name_to_check.to_lowercase());
        
        assert!(is_taken);
        
        let is_taken_bob = state.peers.iter().any(|p| p.username.to_lowercase() == "bob".to_lowercase());
        assert!(!is_taken_bob);
    }

    #[tokio::test]
    async fn test_websocket_login_integration() {
        use tokio_tungstenite::{connect_async, tungstenite::protocol::Message as WsMessage};
        use futures_util::{SinkExt, StreamExt};

        // 1. Setup server on random port
        let (broadcast_tx, _) = broadcast::channel(100);
        let state = Arc::new(AppState {
            peers: DashMap::new(),
            broadcast_tx,
            history: std::sync::Mutex::new(Vec::new()),
        });

        let app = Router::new()
            .route("/ws", get(ws_handler))
            .with_state(state);

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        // 2. Connect client
        let url = format!("ws://{}/ws", addr);
        let (mut ws_stream, _) = connect_async(url).await.expect("Failed to connect");

        // 3. Send Login
        let login_msg = json!({
            "type": "Login",
            "payload": { "username": "Tester" }
        }).to_string();
        
        ws_stream.send(WsMessage::Text(login_msg)).await.unwrap();

        // 4. Verify Identity response
        let msg = ws_stream.next().await.unwrap().unwrap();
        let text = msg.to_text().unwrap();
        let protocol_res: Protocol = serde_json::from_str(text).unwrap();
        
        if let Protocol::Identify { username, .. } = protocol_res {
            assert_eq!(username, "Tester");
        } else {
            panic!("Expected Identify message, got {:?}", protocol_res);
        }

        // 5. Verify Join message (System)
        let msg = ws_stream.next().await.unwrap().unwrap();
        let text = msg.to_text().unwrap();
        let protocol_res: Protocol = serde_json::from_str(text).unwrap();
        
        if let Protocol::System { message, .. } = protocol_res {
            assert!(message.contains("Tester joined"));
        } else {
            panic!("Expected Join message, got {:?}", protocol_res);
        }
    }
}
