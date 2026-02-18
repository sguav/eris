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
- **Strict Quality**: Zero warning policy. All workspace tests must pass strictly before any feature commit.
- **Local Development Resilience**: The native client includes logic to bypass TLS certificate validation when connecting to `localhost` or local network addresses.
- **Observability**: All significant events, state changes, and errors in both server and client **MUST** be logged to `stdout` with a human-friendly timestamp (e.g., `[2026-02-18 15:00:00] [SERVER] Peer Alice joined`).

## Project Overview

- **Architecture**: Cargo Workspace containing:
    - `crates/server`: Rust/Axum signaling server with embedded UI.
    - `crates/client`: Native desktop application using Tauri.
    - `crates/eris-core`: Shared protocol and common types.
    - `www/`: Shared vanilla JS frontend assets.
- **Capabilities**: Real-time text chat, P2P voice channels, and P2P screen sharing.

## Building and Running

- **Run Server**: `cargo run -p eris`
- **Run Client**: `cargo run -p eris-client`

## Development Conventions

- **Server Structure (`crates/server/src/`)**:
    - `state.rs`: Shared state and history.
    - `handlers/`: Route and WebSocket logic.
- **Client Structure (`crates/client/src/`)**:
    - `connection.rs`: Rust-side WebSocket management and message buffering.
- **Shared Protocol (`crates/eris-core/src/lib.rs`)**:
    - Centralized `Protocol` enum used by all crates.
