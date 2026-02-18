# Eris Unified Core

Eris is a self-contained WebRTC signaling server and static web host implemented in Rust. It provides a unified platform for real-time communication, including persistent text chat and peer-to-peer voice channels.

## Architecture

The project is designed with a modular structure to ensure maintainability and clear separation of concerns:

- **Signaling Server**: An Axum-based WebSocket relay that facilitates WebRTC handshakes and chat message broadcasting.
- **State Management**: Concurrent peer tracking and message history handled via thread-safe primitives (`DashMap`, `broadcast`, `Mutex`).
- **Unified Frontend**: A modern, dark-themed vanilla JavaScript UI served directly by the backend, requiring no external build tools.
- **Mesh Networking**: Uses a P2P mesh architecture for audio streams, reducing server-side bandwidth requirements.

## Key Features

- **Identity Management**: Simple username-based login with uniqueness validation and session persistence via `localStorage`.
- **Global Chat**: Real-time text communication with a server-side history buffer (last 50 messages).
- **Voice Channels**: Peer-to-peer audio communication using WebRTC.
- **High Performance**: Built on the Tokio asynchronous runtime for low-latency signaling.

## Project Structure

```text
src/
├── main.rs          # Server entry point and router configuration
├── protocol.rs      # JSON communication protocol definitions
├── state.rs         # Global application state and history logic
└── handlers/        # WebSocket and business logic handlers
www/
└── index.html       # Unified frontend UI
```

## Getting Started

### Prerequisites

- Rust (latest stable version)
- Cargo

### Building and Running

1. Clone the repository.
2. Build and run the server:
   ```bash
   cargo run
   ```
3. Open your browser and navigate to `http://localhost:8080`.

### Testing

The project includes a suite of unit and integration tests covering protocol serialization, state logic, and WebSocket handshakes.

```bash
cargo test
```

## Development

The communication protocol uses a tagged JSON format. New message types should be added to `src/protocol.rs` and handled in `src/handlers/websocket.rs`. The frontend logic in `www/index.html` is self-contained to facilitate easy deployment.
