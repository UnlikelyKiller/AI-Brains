# Plan: Track T38 - Structured Bridge (NDJSON Fallback)

### Phase 1: Bridge Schema Definition
- [ ] Task 1.1: Define the `BridgeRecord` struct in `ai-brains-contracts` with standard JSON serialization.
- [ ] Task 1.2: Write unit tests in `ai-brains-contracts` ensuring accurate serialization/deserialization of `BridgeRecord`.

### Phase 2: CLI Command Implementation
- [ ] Task 2.1: Add `sync pull --from-file <path>` subcommand in `ai-brains-cli`.
- [ ] Task 2.2: Implement file reading stream in `ai-brains-cli` to yield individual NDJSON lines.

### Phase 3: Ingestion and Event Dispatch
- [ ] Task 3.1: Implement NDJSON parsing logic in `ai-brains-capture` to convert lines to `BridgeRecord` instances.
- [ ] Task 3.2: Map `BridgeRecord` fields (especially `privacy`, `tx_id`, `project_id`) into internal event sourcing commands.
- [ ] Task 3.3: Write integration tests verifying NDJSON file ingestion correctly appends events to the SQLite store.
