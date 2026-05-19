# Plan: Track T37 - Transaction Linking & Project Discovery

### Phase 1: Core Contracts and CLI Updates
- [ ] Task 1.1: Update event payloads in `ai-brains-contracts` to support optional `tx_id` and `project_id` fields.
- [ ] Task 1.2: Add `--tx-id` flag to `context` and `pin` commands in `ai-brains-cli`.
- [ ] Task 1.3: Update `AppContext` struct in `ai-brains-cli` to hold the parsed `tx_id`.

### Phase 2: Project Auto-Discovery
- [ ] Task 2.1: Implement `.changeguard` discovery logic in `ai-brains-path` (walking up the tree from the current working directory).
- [ ] Task 2.2: Implement parsing of `.changeguard` metadata to extract `project_id`.
- [ ] Task 2.3: Integrate auto-discovery into CLI bootstrap to populate `AppContext.project_id` if available.

### Phase 3: Event Ingestion Integration
- [ ] Task 3.1: Update `ai-brains-capture` to pass `tx_id` and `project_id` into event creation.
- [ ] Task 3.2: Write tests in `ai-brains-capture` verifying that commands emit events containing the correct `tx_id` and `project_id`.
