# Eris Unified Core

Eris is a self-contained WebRTC signaling server and multi-platform communication hub implemented in Rust. It provides a unified platform for real-time communication, featuring text chat, P2P voice channels, and screen sharing.

## Workspace Architecture

The project is organized as a Cargo Workspace for strict separation of concerns and maximum code reuse:

- **`crates/server`**: An Axum-based WebSocket relay facilitating WebRTC handshakes and secure web hosting.
- **`crates/client`**: A native desktop application built with Tauri 2.0, leveraging Rust for core signaling logic and system webviews for UI.
- **`crates/eris-core`**: Shared library containing the centralized communication protocol, logging utilities, and common types.
- **`www/`**: A shared, vanilla JavaScript frontend compatible with both browsers and the native client.

## Key Features

- **Multi-Platform**: Run as a standalone web server or as a native desktop application.
- **Dual Connectivity**: 
    - **Secure (8443)**: HTTPS/WSS with automatic certificate generation for browser hardware access.
    - **Plain (8080)**: HTTP/WS for easy local development and native client compatibility.
- **Observability**: Standardized, timestamped logging across all components.
- **Local Resilience**: Native client bypasses TLS validation for local/self-signed environments.
- **P2P Mesh**: High-performance audio and screen sharing using WebRTC peer-to-peer mesh.

## Getting Started

### Prerequisites

- Rust (latest stable)
- Linux dependencies: `libgtk-3-dev`, `libwebkit2gtk-4.1-dev`, `libglib2.0-dev`

### Running the Hub (Server)

```bash
cargo run -p eris
```
The server will generate a random **Invite Token** and a **QR Code**. Mobile users can scan the QR code to join instantly.

### Running the Native Client

```bash
cargo run -p eris-client
```
Enter the server address (e.g., `127.0.0.1:8080` or `127.0.0.1:8443`) and the invite token displayed in the server terminal.

## Testing

The workspace enforces a zero-warning policy and includes strict unit, integration, and doc-tests.

```bash
cargo test --workspace
```
Detailed testing documentation can be found in `docs/testing.md`.
