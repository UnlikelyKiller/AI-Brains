# Plan: Track T39 - Real-Time Bridge (IPC)

### Phase 1: Daemon IPC Endpoint
- [ ] Task 1.1: Design and set up the IPC server in `ai-brainsd` (e.g., using `tokio` named pipes or axum bound to localhost).
- [ ] Task 1.2: Implement a handler that accepts `BridgeRecord` JSON payloads.

### Phase 2: Event Integration
- [ ] Task 2.1: Wire the IPC handler in `ai-brainsd` to the event ingestion pipeline (similar to `ai-brains-capture`).
- [ ] Task 2.2: Ensure robust error handling for invalid payloads (no panics, logging errors).

### Phase 3: IPC Client implementation
- [ ] Task 3.1: Implement an IPC client utility/module to push data to the daemon.
- [ ] Task 3.2: Write end-to-end tests sending mocked `BridgeRecord` data via IPC and verifying its presence in the store.
