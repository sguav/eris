// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod connection;

use std::sync::Arc;
use tauri::{State, Emitter};
use eris_core::{Protocol, log};
use connection::ConnectionManager;

struct AppState {
    connection: ConnectionManager,
}

#[tauri::command]
fn js_log(level: String, message: String) {
    log(&format!("JS-{}", level), &message);
}

#[tauri::command]
async fn connect_server(
    url: String,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    log("CLIENT", &format!("Attempting to connect to: {}", url));
    match state.connection.connect(url.clone()).await {
        Ok(_) => {
            log("CLIENT", &format!("Connection established: {}", url));
            Ok(())
        },
        Err(e) => {
            log("CLIENT", &format!("ERROR: Connection failed: {}", e));
            Err(e)
        }
    }
}

#[tauri::command]
async fn send_protocol(
    msg: Protocol,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    match state.connection.send(msg).await {
        Ok(_) => Ok(()),
        Err(e) => {
            log("CLIENT", &format!("ERROR sending protocol: {}", e));
            Err(e)
        }
    }
}

#[tauri::command]
async fn client_ready(
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    log("CLIENT", "Frontend reported READY. Flushing buffers.");
    state.connection.set_ready().await;
    Ok(())
}

fn create_app() -> tauri::App {
    let (connection, mut rx) = ConnectionManager::new();
    let app_state = Arc::new(AppState { connection });
    let state_clone = app_state.clone();

    tauri::Builder::default()
        .manage(state_clone)
        .setup(move |app| {
            // AGGRESSIVE LINUX MEDIA FIXES
            // 1. Sandbox and Portal
            std::env::set_var("WEBKIT_FORCE_SANDBOX", "0");
            std::env::set_var("GTK_USE_PORTAL", "1");
            
            // 2. GStreamer / WebRTC specific ranking
            std::env::set_var("GST_PLUGIN_FEATURE_RANK", "webrtcbin:MAX,v4l2src:MAX");
            
            // 3. WebKit WebRTC environment
            std::env::set_var("WEBKIT_WEBRTC_USE_GSTREAMER", "1");
            
            // 4. Critical: Force Software path if hardware is failing
            std::env::set_var("LIBGL_ALWAYS_SOFTWARE", "1"); 

            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                while let Ok(msg) = rx.recv().await {
                    let _ = handle.emit("protocol-msg", msg);
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![connect_server, send_protocol, client_ready, js_log])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
}

fn main() {
    log("CLIENT", "Starting Eris Native Client...");
    create_app().run(|_app_handle, _event| {});
}

#[cfg(test)]
mod tests {
    use super::*;
    use tauri::Manager;
    use tauri::test::{mock_builder, mock_context, noop_assets};

    #[test]
    fn test_tauri_context() {
        let _context: tauri::Context<tauri::Wry> = tauri::generate_context!();
    }

    #[tokio::test]
    async fn test_client_connection_logic() {
        let (connection, _rx) = ConnectionManager::new();
        let app_state = Arc::new(AppState { connection });
        let app = mock_builder()
            .manage(app_state.clone())
            .invoke_handler(tauri::generate_handler![connect_server, send_protocol, client_ready, js_log])
            .build(mock_context(noop_assets()))
            .unwrap();
        let state: State<Arc<AppState>> = app.state();
        let result = send_protocol(Protocol::Login { username: "Test".to_string() }, state).await;
        assert!(result.is_err());
    }
}
