## Plan: T06 Store Projections

### Phase 1: Projection Migrations and Schema
- [ ] Task 1.1: Create SQL migrations `0002_identity_projection.sql` through `0006_memory_projection.sql`.
- [ ] Task 1.2: Create SQL migrations `0007_fts_setup.sql` and `0008_fts_triggers.sql` for FTS5.
- [ ] Task 1.3: Update `migrations.rs` to include the new embedded SQL files.

### Phase 2: Projection Handlers
- [ ] Task 2.1: Define projection interface in `src/projections/mod.rs`.
- [ ] Task 2.2: Implement `src/projections/project.rs`, `session.rs`, `turn.rs`, and `memory.rs` to map domain events to SQL updates.
- [ ] Task 2.3: Hook projection handlers into the `EventStore::append` transaction so they update atomically with new events.
- [ ] Task 2.4: Write test `projections_update_from_events` to verify atomic updates.

### Phase 3: FTS Integration
- [ ] Task 3.1: Implement `src/fts.rs` with FTS5 search queries.
- [ ] Task 3.2: Write test `fts_indexes_memory` to verify search behavior.

### Phase 4: Event Replay and Rebuild
- [ ] Task 4.1: Implement `src/replay.rs` with logic to truncate projections and stream the event log to rebuild them.
- [ ] Task 4.2: Write test `replay_rebuilds_projections` validating state after a full replay.

### Phase 5: Concurrency and Review
- [ ] Task 5.1: Write test `concurrent_read_single_writer` to ensure WAL mode allows projection reads during long writes.
- [ ] Task 5.2: Final formatting, `cargo clippy`, and `cargo test`.
