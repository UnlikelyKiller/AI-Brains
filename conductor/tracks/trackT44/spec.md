# Specification: Track T44 - System Hardening & Fast-Fail

## Objective
Improve the reliability and responsiveness of the CLI-Daemon interaction by introducing an ultra-fast handshake, CLI fast-fail mechanics, structured JSON error contracts, and robust Windows signal handlers.

## Architecture & Scope
1. **Ultra-Fast Handshake (`ai-brainsd` & `ai-brains-cli`)**: Implement a lightweight ping endpoint or dedicated named pipe check in `ai-brainsd`. The CLI must use this to confirm daemon presence in under 5-10ms.
2. **CLI Fast-Fail (`ai-brains-cli`)**: Update commands that depend on the daemon (e.g., `sync query`) to check for daemon presence at the entrypoint. If the daemon is unreachable, exit immediately with status 1 rather than blocking or timing out.
3. **Structured JSON Error Contracts (`ai-brains-contracts` & `ai-brains-cli`)**: Transition error outputs from unstructured stderr strings to structured NDJSON error objects (e.g., `{"status": "error", "code": "DAEMON_UNREACHABLE", "message": "..."}`).
4. **Windows Signal Handlers (`ai-brainsd` & `ai-brains-cli`)**: Catch Windows termination signals (e.g., `ctrlc`). Ensure SQLite connections, named pipes, and temporary files are closed and deleted cleanly during shutdown.

## Technical Constraints & Mandates
- **Rust Safety**: PROHIBITED use of `unwrap()`, `expect()`, or `panic()`. Explicit error handling via `thiserror` and `anyhow` is mandatory.
- **Performance**: The fast-fail handshake mechanism must complete in under 10ms.
- **PowerShell Consistency**: Any necessary shell scripts or integrations must use PowerShell and avoid `&&`.
