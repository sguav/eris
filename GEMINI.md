# Eris Project Context

Eris is a self-contained hub that functions as both a WebRTC signaling server and a native client platform.

## Project Mandates

- **Architectural Integrity**: Any significant structural or architectural change (e.g., modularization, workspace restructuring) **MUST** be immediately reflected in this document.
- **Multi-Client Support**: The frontend in `www/` must remain compatible with both browser and native (Tauri) contexts.
- **Documentation Parity**: Every architectural change must be accompanied by a dedicated commit that updates `GEMINI.md`.
- **Security First**: Features requiring hardware access (mic/camera) must be served over HTTPS/WSS to ensure a Secure Context.
- **Multi-Channel**: The system supports multiple logical channels. Messages, peer lists, and voice signaling are isolated by the current channel of the user.
- **Screen Sharing**: Users can share their screen within a channel. Watching a shared stream is optional.

## Project Overview

- **Architecture**: Cargo Workspace containing:
    - `crates/server`: Rust/Axum signaling server with embedded UI.
    - `crates/client`: Native desktop application using Tauri.
    - `www/`: Shared vanilla JS frontend assets.
- **Capabilities**: Real-time text chat, P2P voice channels, and P2P screen sharing.

## Building and Running

- **Run Server**: `cargo run -p eris` (from root)
- **Run Client**: `cargo run -p eris-client` (from root)

- **Manual Verification**: Detailed procedures are documented in `docs/testing.md`.

## Development Conventions

- **Server Structure (`crates/server/src/`)**:
    - `protocol.rs`: Message definitions.
    - `state.rs`: Shared state and history.
    - `handlers/`: Route and WebSocket logic.
- **Client Structure (`crates/client/`)**:
    - Tauri-based wrapper for the `www/` frontend.
- **Protocol**: JSON-tagged enum (`type`/`payload`).
