# Eris Testing Documentation

This document outlines the testing strategy, current coverage, and manual verification procedures for the Eris project.

## Automated Tests

We use a combination of unit tests for core logic and integration tests for protocol verification.

### Unit Tests

- **Protocol Serialization (`src/protocol.rs`)**:
    - Verifies that `Protocol` enum variants (Login, ChatMessage, etc.) serialize to and from the expected JSON format.
    - Ensures compatibility with the frontend protocol.
- **State Management (`src/state.rs`)**:
    - `test_history_buffer_limit`: Confirms the in-memory chat history correctly caps at 50 messages and follows FIFO.
    - `test_username_uniqueness_logic`: Verifies the case-insensitive check for existing usernames in the active peer list.

### Integration Tests

- **WebSocket Handshake (`src/main.rs`)**:
    - *Note: Currently partially disabled due to TLS configuration complexity in test environments.*
    - Designed to simulate a full login flow: connecting, sending a `Login` message, and receiving `Identify` and `System` join notifications.

## Test Gaps & Roadmap

Currently, the following areas require further automated coverage:

1. **Signaling Routing**: Verifying that a `Signal` message sent from Peer A to Peer B is correctly routed by the server without reaching Peer C.
2. **Broadcast Integrity**: Ensuring `ChatMessage` and `PeerList` updates reach *all* connected clients.
3. **Concurrency Stress**: Testing the server's stability under a high volume of concurrent connections and rapid message throughput.
4. **Frontend Unit Tests**: Testing the vanilla JS logic (state transitions, message rendering) in a headless environment.

## Manual Verification Procedure

Until full integration coverage is restored for HTTPS, use the following steps for manual verification:

1. **Local Server**: Run `cargo run`.
2. **Multi-Client Test**:
    - Open `https://localhost:8443` in two separate browser tabs.
    - Log in with different usernames.
    - Verify that both receive a "Joined" notification.
    - Send a chat message from one and verify it appears in the other.
3. **Voice Test**:
    - Click "Join Voice" in both tabs.
    - Accept microphone permissions.
    - Verify peer-to-peer audio connectivity (use headphones to avoid feedback).
4. **Screen Sharing Test**:
    - Click "Share Screen" in one tab.
    - Select a window or screen to share in the browser dialog.
    - Verify that a video grid appears in *both* tabs and displays the stream.
    - Click "Stop Sharing" and verify the video grid disappears.
5. **Persistence Test**:
    - Refresh one tab; it should automatically log back in and show chat history.
5. **Logout Test**:
    - Click "Logout" and verify the `localStorage` is cleared and the login screen returns.
