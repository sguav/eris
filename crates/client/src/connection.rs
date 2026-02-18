use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async_tls_with_config, connect_async_with_config, tungstenite::protocol::Message, Connector, WebSocketStream, MaybeTlsStream};
use eris_core::{Protocol, log};
use rustls::client::danger::{ServerCertVerifier, ServerCertVerified};
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use http::Request;
use tokio::net::TcpStream;

pub struct ConnectionManager {
    broadcast_tx: broadcast::Sender<Protocol>,
    ws_sink: Arc<Mutex<Option<futures_util::stream::SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>>>,
    buffer: Arc<Mutex<Vec<Protocol>>>,
    is_ready: Arc<Mutex<bool>>,
}

#[derive(Debug)]
struct NoVerifier;
impl ServerCertVerifier for NoVerifier {
    fn verify_server_cert(&self, _end_entity: &CertificateDer, _intermediates: &[CertificateDer], _server_name: &ServerName, _ocsp_response: &[u8], _now: UnixTime) -> Result<ServerCertVerified, rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }
    fn verify_tls12_signature(&self, _message: &[u8], _cert: &CertificateDer, _dss: &rustls::DigitallySignedStruct) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }
    fn verify_tls13_signature(&self, _message: &[u8], _cert: &CertificateDer, _dss: &rustls::DigitallySignedStruct) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }
    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        vec![
            rustls::SignatureScheme::ECDSA_NISTP256_SHA256,
            rustls::SignatureScheme::ECDSA_NISTP384_SHA384,
            rustls::SignatureScheme::RSA_PSS_SHA256,
            rustls::SignatureScheme::RSA_PSS_SHA384,
            rustls::SignatureScheme::RSA_PSS_SHA512,
            rustls::SignatureScheme::RSA_PKCS1_SHA256,
            rustls::SignatureScheme::RSA_PKCS1_SHA384,
            rustls::SignatureScheme::RSA_PKCS1_SHA512,
            rustls::SignatureScheme::ED25519,
        ]
    }
}

impl ConnectionManager {
    pub fn new() -> (Self, broadcast::Receiver<Protocol>) {
        let _ = rustls::crypto::ring::default_provider().install_default();
        let (tx, rx) = broadcast::channel(100);
        (
            Self {
                broadcast_tx: tx,
                ws_sink: Arc::new(Mutex::new(None)),
                buffer: Arc::new(Mutex::new(Vec::new())),
                is_ready: Arc::new(Mutex::new(false)),
            },
            rx,
        )
    }

    pub async fn set_ready(&self) {
        let mut ready = self.is_ready.lock().await;
        *ready = true;
        let mut buffer = self.buffer.lock().await;
        for msg in buffer.drain(..) {
            let _ = self.broadcast_tx.send(msg);
        }
    }

    pub async fn connect(&self, url_str: String) -> Result<(), String> {
        log("CLIENT", &format!("Connecting to WebSocket: {}", url_str));
        
        let url = url::Url::parse(&url_str).map_err(|e| e.to_string())?;
        let host = url.host_str().ok_or("No host in URL")?.to_string();
        
        let request = Request::builder()
            .uri(url_str.clone())
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .header("Host", &host)
            .header("Origin", format!("{}://{}", url.scheme().replace("ws", "http"), host))
            .header("Connection", "Upgrade")
            .header("Upgrade", "websocket")
            .header("Sec-WebSocket-Version", "13")
            .header("Sec-WebSocket-Key", tokio_tungstenite::tungstenite::handshake::client::generate_key())
            .body(())
            .map_err(|e: http::Error| e.to_string())?;

        let ws_stream: WebSocketStream<MaybeTlsStream<TcpStream>> = if url_str.starts_with("wss://") {
            let is_local = host == "localhost" || host == "127.0.0.1" || host.starts_with("192.168.");
            
            let mut root_store = rustls::RootCertStore::empty();
            root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
            
            let mut config = rustls::ClientConfig::builder()
                .with_root_certificates(root_store)
                .with_no_client_auth();
            
            if is_local {
                log("CLIENT", "Local address detected, bypassing TLS verification");
                config.dangerous().set_certificate_verifier(Arc::new(NoVerifier));
            }
            
            let connector = Connector::Rustls(Arc::new(config));
            let (s, _) = connect_async_tls_with_config(request, None, false, Some(connector)).await.map_err(|e| e.to_string())?;
            s
        } else {
            let (s, _) = connect_async_with_config(request, None, false).await.map_err(|e| e.to_string())?;
            s
        };

        let (sink, mut stream) = ws_stream.split();
        *self.ws_sink.lock().await = Some(sink);
        
        let tx = self.broadcast_tx.clone();
        let buffer = self.buffer.clone();
        let is_ready = self.is_ready.clone();
        
        tokio::spawn(async move {
            while let Some(msg_res) = stream.next().await {
                match msg_res {
                    Ok(Message::Text(text)) => {
                        if let Ok(protocol_msg) = serde_json::from_str::<Protocol>(&text) {
                            let ready = is_ready.lock().await;
                            if *ready {
                                let _ = tx.send(protocol_msg);
                            } else {
                                buffer.lock().await.push(protocol_msg);
                            }
                        }
                    }
                    _ => {}
                }
            }
            log("CLIENT", "WS receive loop terminated");
        });

        Ok(())
    }

    pub async fn send(&self, msg: Protocol) -> Result<(), String> {
        let mut sink_lock = self.ws_sink.lock().await;
        if let Some(sink) = sink_lock.as_mut() {
            let text = serde_json::to_string(&msg).map_err(|e| e.to_string())?;
            sink.send(Message::Text(text)).await.map_err(|e| e.to_string())?;
            Ok(())
        } else {
            Err("Not connected".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use eris_core::Protocol;
    use serde_json::json;

    #[test]
    fn test_protocol_parsing() {
        let json = json!({
            "type": "ChatMessage",
            "payload": {
                "channel": "lobby",
                "author": "System",
                "content": "Test"
            }
        }).to_string();
        
        let msg: Protocol = serde_json::from_str(&json).unwrap();
        if let Protocol::ChatMessage { author, .. } = msg {
            assert_eq!(author, "System");
        } else {
            panic!("Wrong variant");
        }
    }
}
