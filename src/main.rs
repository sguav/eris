//! ERIS UNIFIED CORE
//! Purpose: Server Signaling + UI Web Server

mod protocol;
mod state;
mod handlers;

use axum::{
    routing::get,
    Router,
};
use handlers::websocket::ws_handler;
use state::AppState;
use std::sync::Arc;
use tokio::sync::broadcast;
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let (broadcast_tx, _) = broadcast::channel(2048);
    let state = Arc::new(AppState::new(broadcast_tx));

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

#[cfg(test)]
mod tests {
    use super::*;
    use protocol::Protocol;
    use serde_json::json;

    #[tokio::test]
    async fn test_websocket_login_integration() {
        use tokio_tungstenite::{connect_async, tungstenite::protocol::Message as WsMessage};
        use futures_util::{SinkExt, StreamExt};

        // 1. Setup server on random port
        let (broadcast_tx, _) = broadcast::channel(100);
        let state = Arc::new(AppState::new(broadcast_tx));

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
