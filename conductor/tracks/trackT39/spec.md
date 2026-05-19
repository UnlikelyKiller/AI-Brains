# Specification: Track T39 - Real-Time Bridge (IPC)

## Objective
Establish a real-time IPC bridge (Named Pipes or Local HTTP) for handoff between `ai-brainsd` and ChangeGuard.

## Architecture & Scope
1. **Daemon IPC Endpoint (`ai-brainsd`)**: Add a new local endpoint (HTTP or Named Pipe) to receive `BridgeRecord` streams.
2. **IPC Client (`ai-brains-cli` or specialized client)**: Implement the sender side of the IPC bridge to push records to the daemon.
3. **Event Forwarding**: Convert received IPC payloads into `BridgeRecord`s and append them to the event log.

## Technical Constraints & Mandates
- **Daemon-Preferred**: IPC connection goes directly to the background daemon for zero-blocking performance.
- **CQRS Integrity**: The IPC receiver purely issues append commands to the event store.
- **Windows-first**: Named Pipes are preferred on Windows for security and performance if local HTTP requires firewall prompt bypasses, but local HTTP bound to `127.0.0.1` is acceptable if it respects Windows firewall constraints.
