use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::{State, Query},
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;
use serde::Deserialize;

use eris_core::{Protocol, log};
use crate::state::{AppState, Peer};

#[derive(Deserialize)]
pub struct WsQuery {
    token: Option<String>,
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    query: Query<WsQuery>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    if let Some(token) = &query.token {
        if token == &state.invite_token {
            return ws.on_upgrade(|socket| handle_socket(socket, state)).into_response();
        }
    }
    
    log("SERVER", "Unauthorized connection attempt rejected (invalid/missing token)");
    axum::http::Response::builder()
        .status(axum::http::StatusCode::UNAUTHORIZED)
        .body(axum::body::Body::empty())
        .unwrap()
        .into_response()
}

pub async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();
    let my_id = Uuid::new_v4();
    let (peer_tx, mut peer_rx) = broadcast::channel(100);
    
    let mut chat_rx = state.broadcast_tx.subscribe();
    let mut my_username = String::new();
    let my_channel = Arc::new(RwLock::new("lobby".to_string()));

    // --- LOGIN HANDSHAKE ---
    while let Some(msg_res) = receiver.next().await {
        let Ok(msg) = msg_res else { break; };
        let Message::Text(text) = msg else { continue; };
        
        if let Ok(Protocol::Login { username }) = serde_json::from_str::<Protocol>(&text) {
            let name = username.trim();
            if name.is_empty() { continue; }
            
            let is_taken = state.peers.iter().any(|p| p.username.to_lowercase() == name.to_lowercase());
            if is_taken {
                let err = Protocol::System { message: "Username taken".to_string(), severity: "error".to_string() };
                if let Ok(txt) = serde_json::to_string(&err) { let _ = sender.send(Message::Text(txt)).await; }
                continue;
            }

            my_username = name.to_string();
            log("SERVER", &format!("Peer '{}' ({}) logged in", my_username, my_id));
            
            state.peers.insert(my_id, Peer { 
                username: my_username.clone(), 
                channel: "lobby".to_string(),
                is_sharing: false,
                tx: peer_tx.clone() 
            });
            
            let ident = Protocol::Identify { id: my_id, username: my_username.clone() };
            if let Ok(msg_json) = serde_json::to_string(&ident) { let _ = sender.send(Message::Text(msg_json)).await; }
            
            let history_snapshot = {
                let history = state.history.lock().unwrap();
                history.clone()
            };
            for msg in history_snapshot {
                if let Protocol::ChatMessage { ref channel, .. } = msg {
                    if channel == "lobby" {
                        if let Ok(txt) = serde_json::to_string(&msg) { let _ = sender.send(Message::Text(txt)).await; }
                    }
                }
            }
            
            let _ = state.broadcast_tx.send(Protocol::System { 
                message: format!("{} joined lobby", my_username), 
                severity: "info".to_string() 
            });
            state.broadcast_peer_list();
            break;
        }
    }

    if my_username.is_empty() { return; }

    let my_channel_send = my_channel.clone();
    let mut send_task = tokio::spawn(async move {
        loop {
            tokio::select! {
                Ok(msg) = chat_rx.recv() => {
                    let current = my_channel_send.read().await;
                    let should_send = match &msg {
                        Protocol::ChatMessage { channel, .. } => channel == &*current,
                        _ => true,
                    };
                    if should_send {
                        if let Ok(txt) = serde_json::to_string(&msg) {
                            if sender.send(Message::Text(txt)).await.is_err() { break; }
                        }
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
    let my_channel_recv = my_channel.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = receiver.next().await {
            if let Ok(protocol_msg) = serde_json::from_str::<Protocol>(&text) {
                match protocol_msg {
                    Protocol::JoinChannel { channel } => {
                        log("SERVER", &format!("Peer '{}' moving to channel: {}", my_username_clone, channel));
                        {
                            let mut current = my_channel_recv.write().await;
                            *current = channel.clone();
                        }
                        if let Some(mut peer) = state_clone.peers.get_mut(&my_id) {
                            peer.channel = channel.clone();
                        }
                        let _ = state_clone.broadcast_tx.send(Protocol::System { 
                            message: format!("{} moved to {}", my_username_clone, channel), 
                            severity: "info".to_string() 
                        });
                        state_clone.broadcast_peer_list();
                    }
                    Protocol::ChatMessage { content, .. } => {
                        let current = my_channel_recv.read().await;
                        let msg = Protocol::ChatMessage { 
                            channel: current.clone(),
                            author: my_username_clone.clone(), 
                            content 
                        };
                        {
                            let mut history = state_clone.history.lock().unwrap();
                            history.push(msg.clone());
                            if history.len() > 100 { history.remove(0); }
                        }
                        let _ = state_clone.broadcast_tx.send(msg);
                    }
                    Protocol::Signal { target_id, data } => {
                        if let Some(target) = state_clone.peers.get(&target_id) {
                            let _ = target.tx.send(Protocol::Signal { target_id: my_id, data });
                        }
                    }
                    Protocol::ScreenState { is_sharing, .. } => {
                        if let Some(mut p) = state_clone.peers.get_mut(&my_id) {
                            p.is_sharing = is_sharing;
                        }
                        let _ = state_clone.broadcast_tx.send(Protocol::ScreenState { peer_id: my_id, is_sharing });
                        state_clone.broadcast_peer_list();
                    }
                    Protocol::RequestStream { target_id } => {
                        if let Some(target) = state_clone.peers.get(&target_id) {
                            let _ = target.tx.send(Protocol::RequestStream { target_id: my_id });
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

    log("SERVER", &format!("Peer '{}' ({}) disconnected", my_username, my_id));
    state.peers.remove(&my_id);
    state.broadcast_peer_list();
}
