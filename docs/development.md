# Eris Development Guide

This document explains the codebase structure and the workflow for extending Eris.

## Codebase Segmentation

We follow a strict segmentation strategy to maintain a standalone, multi-platform architecture:

1. **`crates/eris-core` (Data/Logic)**:
    - Contains the source of truth for the `Protocol`.
    - Contains shared utilities like the centralized logger.
    - *Dependency constraint*: Must remain minimal and avoid heavy dependencies.

2. **`crates/server` (Signaling/Hub)**:
    - Axum-based server.
    - Manages global state (peer registry, history) in `state.rs`.
    - Handles WebSocket logic in `handlers/websocket.rs`.

3. **`crates/client` (Native Desktop)**:
    - Tauri 2.0 application.
    - Core signaling logic is handled in Rust (`src/connection.rs`) for robustness.
    - Bridges events to the frontend via Tauri Emitter.

4. **`www/` (Universal View)**:
    - Vanilla JS frontend.
    - Must remain agnostic of the execution context (detects `isTauri` at runtime).
    - Communicates via Tauri `invoke` (Native) or standard `WebSockets` (Browser).

## Feature Workflow

When adding a new feature (e.g., File Sharing):

1. **Update Protocol**: Add necessary variants to `crates/eris-core/src/lib.rs`.
2. **Implement Backend Relay**: Update `crates/server/src/handlers/websocket.rs` to handle the new message.
3. **Extend Native Client (Optional)**: If the feature needs Rust-side handling, update `crates/client/src/connection.rs`.
4. **Update Frontend**: Add the UI logic to `www/index.html`.
5. **Verify**: Run `cargo test --workspace` and perform manual verification as per `docs/testing.md`.
6. **Document**: Update `GEMINI.md` and this guide if architectural patterns changed.

## Strict Standards

- **Zero Warnings**: No code with compiler warnings should be committed.
- **Observability**: Always use `eris_core::log` for state changes.
- **Tests**: Every logical branch in the protocol or state management should have a corresponding unit or integration test.
