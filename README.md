# Eris Unified Core

Eris is a self-contained WebRTC signaling server and multi-platform communication hub implemented in Rust. It provides a unified platform for real-time communication, featuring text chat, P2P voice channels, and screen sharing.

## Workspace Architecture

The project is organized as a Cargo Workspace for strict separation of concerns and maximum code reuse:

- **`crates/server`**: An Axum-based WebSocket relay facilitating WebRTC handshakes and secure web hosting.
- **`crates/client`**: A native desktop application built with Tauri, leveraging Rust for core signaling logic and system webviews for UI.
- **`crates/eris-core`**: Shared library containing the centralized communication protocol and common types.
- **`www/`**: A shared, vanilla JavaScript frontend compatible with both browsers and the native client.

## Key Features

- **Multi-Platform**: Run as a standalone web server or as a native desktop application.
- **Identity Management**: Token-protected login with session persistence.
- **Secure Context**: Built-in HTTPS/WSS support with automatic certificate generation for mobile/browser hardware access.
- **Dual Connectivity**: Supports both plain (8080) and secure (8443) ports to accommodate various network environments and client types.
- **P2P Mesh**: High-performance audio and screen sharing using WebRTC peer-to-peer mesh.

## Getting Started

### Prerequisites

- Rust (latest stable)
- System dependencies (Linux): `libgtk-3-dev`, `libwebkit2gtk-4.1-dev`, `libglib2.0-dev`

### Running the Project

1. **Start the Hub (Server)**:
   ```bash
   cargo run -p eris
   ```
   Note the **Invite Token** printed in the terminal.

2. **Start the Native Client**:
   ```bash
   cargo run -p eris-client
   ```
   Enter the server address (e.g., `127.0.0.1:8080`) and the invite token.

3. **Access via Browser**:
   Navigate to `https://localhost:8443` or use the QR code link.

## Testing

The workspace includes comprehensive unit and integration tests.

```bash
cargo test --workspace
```
