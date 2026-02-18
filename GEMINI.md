# Eris Project Context

Eris is a self-contained hub that functions as both a WebRTC signaling server and a secure web UI server.

## Project Mandates

- **Architectural Integrity**: Any significant structural or architectural change (e.g., modularization, switching protocols, changing ports) **MUST** be immediately reflected in this document.
- **Documentation Parity**: Every architectural change must be accompanied by a dedicated commit that updates `GEMINI.md`.
- **Security First**: Features requiring hardware access (mic/camera) must be served over HTTPS to ensure a Secure Context.
- **Invite Token**: Access to the WebSocket signaling is protected by a server-generated random token.
- **Multi-Channel**: The system supports multiple logical channels. Messages, peer lists, and voice signaling are isolated by the current channel of the user.
- **Screen Sharing**: Users can share their screen within a channel. Signaling and streams are isolated per channel. Watching a shared stream is optional for receivers.

## Project Overview

- **Purpose**: A unified core for signaling (WebSocket relay) and serving a web-based UI over HTTPS.
- **Primary Technologies**:
    - **Backend**: Rust, Axum, Axum-server (TLS), rcgen (Self-signed certs).
    - **Architecture**: Modular design with separate `protocol`, `state`, and `handlers`.
    - **Capabilities**: Real-time text chat, P2P voice channels, and P2P screen sharing.
- **Security**: Serves over HTTPS by default (port 8443) using automatically generated self-signed certificates.

## Architecture & Standalone Nature

- **Binary Embedding**: The frontend `www/index.html` is baked into the executable at compile time. This allows the application to run as a single standalone file without external dependencies.

## Building and Running

- **Run the server**:
  ```bash
  cargo run
  ```
  - Access UI at `https://[IP]:8443`.
  - The server generates `cert.pem` and `key.pem` on first run.
  - **Browser Warning**: Since the certificate is self-signed, you must click **Advanced -> Proceed** (or similar) to allow the connection.

- **Build/Test**:
  ```bash
  cargo build
  cargo test
  ```
- **Testing Documentation**: Detailed test coverage and manual procedures are documented in `docs/testing.md`.

## Development Conventions

- **Module Structure**:
    - `src/protocol.rs`: Message definitions (The "Interface").
    - `src/state.rs`: Shared state and history (The "Memory").
    - `src/handlers/`: Route and WebSocket logic (The "Logic").
- **Protocol**: JSON-tagged enum (`type`/`payload`).

## Key Files

- `src/main.rs`: Entry point and HTTPS server setup.
- `src/protocol.rs`: Communication protocol.
- `src/state.rs`: Application state and history.
- `src/handlers/websocket.rs`: Signaling logic.
- `www/index.html`: Self-contained frontend.
