# Track T53: Daemon Lifecycle & Global Install UX

## Objective
Implement graceful daemon termination and solve the "Access is denied" issue during global binary upgrades.

## Problem Statement
The `ai-brainsd` daemon runs as a persistent background process. When a user runs `cargo install` to upgrade AI-Brains, the installation fails because the running daemon locks the `ai-brainsd.exe` binary. Users currently have to manually kill the process via Task Manager or PowerShell.

## Requirements
- **Graceful Stop**: Implement `ai-brains daemon stop` which sends a shutdown signal to the running daemon.
- **Forced Kill**: Add an optional `--force` flag to the stop command to kill the process if the graceful shutdown hangs.
- **Install Helper**: (Optional) Provide a recommended workflow or script that automates "stop -> install -> start".

## Technical Design
- **IPC Signal**: Add a `DaemonRequest::Shutdown` variant to the daemon API.
- **Shutdown Handler**: Update the daemon loop to flush the event log and exit when the shutdown request is received.
- **CLI Command**: Implement `Commands::Daemon { command: DaemonCommands::Stop { force: bool } }`.

## Verification Plan
- Start the daemon.
- Run `ai-brains daemon stop`.
- Verify the process exits and `ai-brainsd.exe` is no longer locked.
- Verify a subsequent `cargo install` succeeds without manual intervention.
