# Eris Project Context

Eris is a self-contained hub that functions as both a WebRTC signaling server and a secure web UI server.

## Project Overview

- **Purpose:** A unified core for signaling (WebSocket relay) and serving a web-based UI over HTTPS.
- **Primary Technologies:**
    - **Backend:** Rust, Axum, Axum-server (TLS), rcgen (Self-signed certs).
    - **Frontend:** Vanilla JS, served from the `www` directory.
- **Security:** Serves over HTTPS by default (port 8443) to provide a "Secure Context" required for microphone access on mobile browsers.

## Building and Running

- **Run the server:**
  ```bash
  cargo run
  ```
  - Access UI at `https://[IP]:8443`.
  - The server automatically generates `cert.pem` and `key.pem` on first run.
  - You will see a "Your connection is not private" warning in browsers because the certificate is self-signed. Click **Advanced -> Proceed** to enter.

- **Build/Test:** Standard `cargo build` and `cargo test`.

## Development Conventions

- **Protocol:** JSON messages defined in `src/protocol.rs`.
- **State:** Managed in `src/state.rs` with a 50-message history buffer.
- **Handlers:** WebSocket logic isolated in `src/handlers/websocket.rs`.

## Key Files

- `src/main.rs`: Entry point and HTTPS server setup.
- `src/protocol.rs`: Communication protocol.
- `src/state.rs`: Application state and history.
- `src/handlers/websocket.rs`: Signaling logic.
- `www/index.html`: Self-contained frontend.
