# Eris Project Context

Eris is a self-contained hub that functions as both a WebRTC signaling server and a web UI server. It is designed to facilitate real-time communication and provide a management interface in a single binary.

## Project Overview

- **Purpose:** A unified core for signaling (WebSocket relay) and serving a web-based UI.
- **Primary Technologies:**
    - **Backend:** [Rust](https://www.rust-lang.org/) with the [Axum](https://github.com/tokio-rs/axum) web framework and [Tokio](https://tokio.rs/) runtime.
    - **Concurrency:** Uses `DashMap` for concurrent peer tracking and `tokio::sync::broadcast` for message distribution.
    - **Frontend:** A React-based UI served from the `www` directory, utilizing Tailwind CSS and Font Awesome via CDN/ESM.
- **Architecture:** 
    - The server binds to `0.0.0.0:8080` by default.
    - `/ws`: WebSocket endpoint for the signaling and chat protocol.
    - `/`: Static file server serving the `./www` directory.

## Building and Running

- **Run the server:**
  ```bash
  cargo run
  ```
  Once running, the UI is accessible at `http://localhost:8080` and the signaling endpoint at `ws://localhost:8080/ws`.

- **Build the project:**
  ```bash
  cargo build --release
  ```

- **Run tests:**
  ```bash
  cargo test
  ```

## Development Conventions

- **Language:** Rust (2021 Edition).
- **Project Structure:**
    - `src/main.rs`: Contains the entire backend logic, including the Axum router, WebSocket handler, and state management.
    - `www/`: Contains the frontend static assets. `index.html` is the entry point.
- **WebSocket Protocol:** Messages are JSON-encoded and follow a tagged enum structure:
    - `Identify`: Sent by the server upon connection to assign a UUID.
    - `ChatMessage`: Broadcasted to all connected peers.
    - `Signal`: Routed to a specific `target_id`.
    - `System`: General system-level notifications.

## Key Files

- `Cargo.toml`: Backend dependency management (Axum, Tokio, Serde, etc.).
- `src/main.rs`: Core server implementation and WebSocket relay logic.
- `www/index.html`: The main frontend entry point using React and Tailwind.
