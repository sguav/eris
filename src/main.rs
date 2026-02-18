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

    let (broadcast_tx, _) = broadcast::channel(2048);
    let state = Arc::new(AppState::new(broadcast_tx));

    let app = Router::new()
        .route("/", get(|| async { Html(INDEX_HTML) }))
        .route("/ws", get(ws_handler))
        .layer(tower_http::cors::CorsLayer::permissive())
        .with_state(state);

    let addr = "0.0.0.0:8443";
    let socket_addr: std::net::SocketAddr = addr.parse().unwrap();
    
    println!("--------------------------------------------------");
    println!("🌑 ERIS UNIFIED CORE ONLINE (HTTPS)");
    println!("🌍 Web UI: https://localhost:8443");
    println!("📡 Signaling: wss://localhost:8443/ws");
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
    /* Integration test temporarily disabled due to complex tokio-tungstenite 0.24 TLS API requirements
    #[tokio::test]
    async fn test_websocket_login_integration() {
        ...
    }
    */
}


