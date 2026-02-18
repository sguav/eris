# Eris Project Context

Eris is a self-contained hub that functions as both a WebRTC signaling server and a native client platform.

## Project Mandates

- **Architectural Integrity**: Any significant structural change (modularization, workspace restructuring) **MUST** be immediately reflected in this document.
- **Documentation Parity**: Every architectural change must be accompanied by a dedicated commit that updates `GEMINI.md`.
- **Multi-Client Support**: The frontend in `www/` must remain compatible with both browser and native (Tauri) contexts.
- **Security First**: Features requiring hardware access (mic/camera) must be served over HTTPS/WSS to ensure a Secure Context.
- **Dual Port Support**: 
    - **Port 8080**: Plain HTTP/WS for native clients bypassing TLS certificate issues.
    - **Port 8443**: Secure HTTPS/WSS for browsers requiring Secure Context.
- **Message Buffering**: The client Rust backend must buffer incoming protocol messages until the frontend signals it is `ready` to prevent race conditions during initialization.
- **Local Development Resilience**: The native client includes logic to bypass TLS certificate validation for local network addresses.
- **Observability**: All significant events and errors **MUST** be logged to `stdout` with a human-friendly timestamp.
- **Strict Quality**: Zero warning policy. All workspace tests must pass strictly before any feature commit.

## Project Overview

- **Architecture**: Cargo Workspace
    - `crates/server`: Rust/Axum signaling server with dual-port support and embedded UI.
    - `crates/client`: Native desktop application using Tauri 2.0.
    - `crates/eris-core`: Shared library for protocol and common utilities (logging, types).
    - `www/`: Shared vanilla JS frontend assets.
- **Capabilities**: Real-time text chat, P2P voice channels, and P2P screen sharing.

## Building and Running

- **Server**: `cargo run -p eris`
- **Native Client**: `cargo run -p eris-client`
- **Web UI**: `https://localhost:8443` (Secure) or `http://localhost:8080` (Plain)

## Development Conventions

- **Shared Logic**: Centralized in `crates/eris-core`. New protocol variants must be added there.
- **Client Logic**: Signaling and connection management are handled in Rust (`crates/client/src/connection.rs`) to ensure robustness.
- **Frontend**: Thin view layer in `www/`. Uses Tauri `invoke` when available, falling back to standard WebSockets in browsers.
