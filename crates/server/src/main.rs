//! ERIS UNIFIED CORE
//! Purpose: Server Signaling + UI Web Server

mod protocol;
mod state;
mod handlers;
mod frontend;

use axum::{
    routing::get,
    response::Html,
    Router,
};
use axum_server::tls_rustls::RustlsConfig;
use handlers::websocket::ws_handler;
use state::AppState;
use frontend::INDEX_HTML;
use std::sync::Arc;
use tokio::sync::broadcast;
use std::path::Path;
use rand::{distributions::Alphanumeric, Rng};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // Ensure certificates exist for HTTPS
    let (cert_path, key_path) = ("cert.pem", "key.pem");
    if !Path::new(cert_path).exists() || !Path::new(key_path).exists() {
        println!("Generating self-signed certificate...");
        generate_self_signed_cert(cert_path, key_path).expect("Failed to generate certs");
    }

    let config = RustlsConfig::from_pem_file(cert_path, key_path)
        .await
        .expect("Failed to load certificates");

    let invite_token: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(6)
        .map(char::from)
        .collect();

    let (broadcast_tx, _) = broadcast::channel(2048);
    let state = Arc::new(AppState::new(broadcast_tx, invite_token.clone()));

    let app = Router::new()
        .route("/", get(|| async { Html(INDEX_HTML) }))
        .route("/ws", get(ws_handler))
        .layer(tower_http::cors::CorsLayer::permissive())
        .with_state(state);

    let addr = "0.0.0.0:8443";
    let socket_addr: std::net::SocketAddr = addr.parse().unwrap();
    
    // Attempt to get local IP for the QR code
    let local_ip = local_ip_address::local_ip().map(|ip| ip.to_string()).unwrap_or_else(|_| "localhost".to_string());
    let invite_url = format!("https://{}:8443/?token={}", local_ip, invite_token);

    println!("--------------------------------------------------");
    println!("🌑 ERIS UNIFIED CORE ONLINE (HTTPS)");
    println!("🌍 Web UI: https://localhost:8443");
    println!("📡 Signaling: wss://localhost:8443/ws");
    println!("🔑 Invite Token: {}", invite_token);
    println!("🔗 Invite URL: {}", invite_url);
    println!("--------------------------------------------------");
    qr2term::print_qr(&invite_url).ok();
    println!("--------------------------------------------------");
    
    axum_server::bind_rustls(socket_addr, config)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

fn generate_self_signed_cert(cert_path: &str, key_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut params = rcgen::CertificateParams::default();
    params.subject_alt_names = vec![
        rcgen::SanType::DnsName(rcgen::Ia5String::try_from("localhost")?),
        rcgen::SanType::IpAddress(std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1))),
    ];
    let key_pair = rcgen::KeyPair::generate()?;
    let cert = params.self_signed(&key_pair)?;
    
    std::fs::write(cert_path, cert.pem())?;
    std::fs::write(key_path, key_pair.serialize_pem())?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tokio_tungstenite::{connect_async, tungstenite::protocol::Message as WsMessage};
    use futures_util::{SinkExt, StreamExt};

    #[tokio::test]
    async fn test_strict_multi_user_flow() {
        let (broadcast_tx, _) = broadcast::channel(100);
        let token = "test_token_123".to_string();
        let state = Arc::new(AppState::new(broadcast_tx, token.clone()));

        let app = Router::new()
            .route("/ws", get(ws_handler))
            .with_state(state);

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        let url = format!("ws://{}/ws?token={}", addr, token);

        // Connect User A
        let (mut ws_a, _) = connect_async(&url).await.unwrap();
        ws_a.send(WsMessage::Text(json!({"type": "Login", "payload": {"username": "Alice"}}).to_string())).await.unwrap();
        
        // Skip Identify
        let _ = ws_a.next().await.unwrap().unwrap();

        // Connect User B
        let (mut ws_b, _) = connect_async(&url).await.unwrap();
        ws_b.send(WsMessage::Text(json!({"type": "Login", "payload": {"username": "Bob"}}).to_string())).await.unwrap();
        
        // Alice should eventually see Bob join
        let mut bob_joined = false;
        for _ in 0..5 { // Check a few messages
            let msg = ws_a.next().await.unwrap().unwrap();
            let text = msg.to_text().unwrap();
            if text.contains("Bob joined lobby") {
                bob_joined = true;
                break;
            }
        }
        assert!(bob_joined, "Alice never saw Bob join");

        // Alice switches to 'gaming'
        ws_a.send(WsMessage::Text(json!({"type": "JoinChannel", "payload": {"channel": "gaming"}}).to_string())).await.unwrap();
        
        // Bob switches to 'gaming'
        ws_b.send(WsMessage::Text(json!({"type": "JoinChannel", "payload": {"channel": "gaming"}}).to_string())).await.unwrap();
        
        // Alice should eventually see Bob move to gaming
        let mut bob_moved = false;
        for _ in 0..5 {
            let msg = ws_a.next().await.unwrap().unwrap();
            let text = msg.to_text().unwrap();
            if text.contains("Bob moved to gaming") {
                bob_moved = true;
                break;
            }
        }
        assert!(bob_moved, "Alice never saw Bob move to gaming");

        // Bob sends message to 'gaming' - Alice SHOULD receive it
        ws_b.send(WsMessage::Text(json!({"type": "ChatMessage", "payload": {"content": "Hello Alice", "author": "Bob", "channel": "gaming"}}).to_string())).await.unwrap();
        
        let mut alice_received = false;
        for _ in 0..5 {
            let msg = ws_a.next().await.unwrap().unwrap();
            let text = msg.to_text().unwrap();
            if text.contains("Hello Alice") {
                alice_received = true;
                break;
            }
        }
        assert!(alice_received, "Alice never received Bob's message");
    }
}
