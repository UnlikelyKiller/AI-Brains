# Track T51: Daemon Auto-Launch & Bridge Silence

## Objective
Reduce integration noise and improve UX by allowing the AI-Brains CLI to automatically start the background daemon (`ai-brainsd`) when needed, especially during ChangeGuard bridge operations.

## Problem Statement
Currently, if the AI-Brains daemon is not running, commands that depend on it (like `sync query` used by ChangeGuard) fail immediately with a loud error message: "AI-Brains daemon is not running or unreachable." This causes constant noise in ChangeGuard's logs when the user hasn't manually started the daemon.

## Requirements
- **Auto-Start**: The CLI should attempt to launch `ai-brainsd` in the background if a daemon-dependent command is invoked and the daemon is unreachable.
- **Fast-Fail Preservation**: The ultra-fast handshake (<10ms) must still be the primary path. Auto-start should only happen if the initial probe fails.
- **Silence Option**: Commands used by the bridge should have a way to suppress "daemon down" errors to avoid log pollution.
- **Capture Independence**: Ensure this doesn't introduce dependencies on the daemon for commands that don't strictly need it (e.g., `ingest`).

## Technical Design

### Daemon Client Enhancements
- Add `DaemonClient::spawn_daemon()`: Uses `std::process::Command` to start `ai-brainsd`. On Windows, it should be started with `CREATE_NO_WINDOW` or similar to ensure it stays in the background.
- Add `DaemonClient::ensure_running()`: 
    1. Probe with 10ms timeout.
    2. If fails, call `spawn_daemon()`.
    3. Wait ~100-200ms and probe again.
    4. Return success if running.

### CLI Command Updates
- `sync query`: Use `ensure_running()`.
- `recall`: Optionally use `ensure_running()` or provide a clear "starting daemon..." message.
- Add `--quiet` flag to `sync query` and `recall` to suppress error output if daemon cannot be started.

## Verification Plan
- **Manual Test**:
    1. Ensure `ai-brainsd` is NOT running.
    2. Run `ai-brains sync query "test"`.
    3. Verify that `ai-brainsd` starts automatically and the command eventually succeeds.
- **Automated Test**: Add a test case in `ai-brains-cli` that mocks the probe failure and verifies the spawn attempt.
