// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod connection;

use std::sync::Arc;
use tauri::{State, Emitter};
use eris_core::Protocol;
use connection::ConnectionManager;

struct AppState {
    connection: ConnectionManager,
}

#[tauri::command]
async fn connect_server(
    url: String,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    println!("Rust: Connecting to {}", url);
    match state.connection.connect(url.clone()).await {
        Ok(_) => {
            println!("Rust: Connected successfully to {}", url);
            Ok(())
        },
        Err(e) => {
            eprintln!("Rust: Connection failed: {}", e);
            Err(e)
        }
    }
}

#[tauri::command]
async fn send_protocol(
    msg: Protocol,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    state.connection.send(msg).await
}

fn main() {
    let (connection, mut rx) = ConnectionManager::new();
    let app_state = Arc::new(AppState { connection });
    let state_clone = app_state.clone();

    tauri::Builder::default()
        .manage(state_clone)
        .setup(move |app| {
            // For Linux WebKitGTK - help with some sandboxing issues if any
            std::env::set_var("WEBKIT_FORCE_SANDBOX", "0");
            
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                while let Ok(msg) = rx.recv().await {
                    let _ = handle.emit("protocol-msg", msg);
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![connect_server, send_protocol])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_tauri_context() {
        let _context: tauri::Context<tauri::Wry> = tauri::generate_context!();
    }
}
