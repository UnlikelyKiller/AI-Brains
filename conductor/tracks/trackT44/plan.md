# Plan: Track T44 - System Hardening & Fast-Fail

### Phase 1: Ultra-Fast Handshake
- [ ] Task 1.1: Implement a lightweight `/ping` endpoint or a lightweight named pipe listener in `ai-brainsd` dedicated to fast health checks.
- [ ] Task 1.2: Implement the client-side probe in `ai-brains-cli` to check this endpoint with a strict timeout (<10ms).

### Phase 2: CLI Fast-Fail Mechanics
- [ ] Task 2.1: Identify all CLI commands in `ai-brains-cli` that require a running daemon to function properly.
- [ ] Task 2.2: Inject the fast-fail health check at the entrypoint of these commands, returning status 1 immediately if the daemon is missing.

### Phase 3: Structured JSON Error Contracts
- [ ] Task 3.1: Define structured NDJSON error DTOs in `ai-brains-contracts`.
- [ ] Task 3.2: Refactor error reporting in `ai-brains-cli` and `ai-brainsd` to emit these structured JSON objects on failure instead of plain strings to stderr.

### Phase 4: Graceful Shutdown (Windows Signal Handlers)
- [ ] Task 4.1: Integrate `ctrlc` or comparable Windows-compatible signal handling crates into `ai-brainsd`.
- [ ] Task 4.2: Ensure all critical resources (SQLite connections, open named pipes) are gracefully dropped when a termination signal is received.
- [ ] Task 4.3: Implement the same signal handlers in `ai-brains-cli` to safely clean up temporary resources before exit.
