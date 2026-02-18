use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use eris_core::Protocol;
use serde_json::json;

pub struct ConnectionManager {
    broadcast_tx: broadcast::Sender<Protocol>,
    ws_sink: Arc<Mutex<Option<futures_util::stream::SplitSink<tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>, Message>>>>,
}

impl ConnectionManager {
    pub fn new() -> (Self, broadcast::Receiver<Protocol>) {
        let (tx, rx) = broadcast::channel(100);
        (
            Self {
                broadcast_tx: tx,
                ws_sink: Arc::new(Mutex::new(None)),
            },
            rx,
        )
    }

    pub async fn connect(&self, url: String) -> Result<(), String> {
        let (ws_stream, _) = connect_async(url).await.map_err(|e| e.to_string())?;
        let (sink, mut stream) = ws_stream.split();
        
        *self.ws_sink.lock().await = Some(sink);
        
        let tx = self.broadcast_tx.clone();
        
        tokio::spawn(async move {
            while let Some(Ok(msg)) = stream.next().await {
                if let Message::Text(text) = msg {
                    if let Ok(protocol_msg) = serde_json::from_str::<Protocol>(&text) {
                        let _ = tx.send(protocol_msg);
                    }
                }
            }
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
    use super::*;
    use eris_core::Protocol;

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
