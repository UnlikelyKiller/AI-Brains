## Plan: T05 Store Event Log

### Phase 1: Cargo Setup and Errors
- [ ] Task 1.1: Update `crates/ai-brains-store/Cargo.toml` with dependencies: `rusqlite` (with `bundled-sqlcipher` feature), `thiserror`, `serde_json`. Depend on `ai-brains-core`, `ai-brains-events`, and `ai-brains-crypto`.
- [ ] Task 1.2: Create `crates/ai-brains-store/src/errors.rs` to define `StoreError`. Must include variants for locked vault, migration failure, and event append failure.

### Phase 2: Schema and Migrations
- [ ] Task 2.1: Create `crates/ai-brains-store/migrations/0001_event_log.sql` containing the `events` table schema, indexes, and the `prevent_event_update` trigger.
- [ ] Task 2.2: Implement `crates/ai-brains-store/src/migrations.rs`. Include logic to create a `schema_migrations` table and apply embedded SQL scripts sequentially.
- [ ] Task 2.3: Write test `tests/migrations_idempotent.rs` to ensure running migrations multiple times does not fail or corrupt state.

### Phase 3: Vault Connection and Config
- [ ] Task 3.1: Implement `crates/ai-brains-store/src/config.rs` for `StoreConfig` (holds vault path).
- [ ] Task 3.2: Implement `crates/ai-brains-store/src/pragmas.rs` to execute SQLite pragmas securely (key injection, WAL mode, synchronous settings).
- [ ] Task 3.3: Implement `crates/ai-brains-store/src/connection.rs` providing `VaultConnection`. Must require `StoreConfig` and `SqlCipherKey` to open or create the database securely. Ensure it validates the key (e.g., via `SELECT count(*) FROM sqlite_master`).
- [ ] Task 3.4: Write tests `tests/sqlcipher_encrypted_vault.rs` and `tests/wrong_key_cannot_open.rs`.
- [ ] Task 3.5: Write red-team test `tests/plaintext_fallback_forbidden.rs` verifying it's impossible to open a vault without providing a key.

### Phase 4: Event Store and Transaction
- [ ] Task 4.1: Implement `crates/ai-brains-store/src/transaction.rs` to wrap `rusqlite::Transaction`.
- [ ] Task 4.2: Implement `crates/ai-brains-store/src/event_store.rs`. Add `EventStore::append` to write `EventEnvelope` structures into the `events` table.
- [ ] Task 4.3: Implement `EventStore::read_events` to retrieve `EventEnvelope`s for a given aggregate.
- [ ] Task 4.4: Write tests `tests/event_append_atomic.rs` and `tests/event_log_is_append_only.rs` (the latter tests the update trigger).
- [ ] Task 4.5: Write red-team test `tests/direct_event_update_forbidden.rs` to manually execute an `UPDATE` on the connection and verify it aborts.
- [ ] Task 4.6: Write red-team test `tests/raw_tool_call_not_storable.rs` to ensure attempting to append an envelope with forbidden structures fails at the API level or is simply impossible by type design.

### Phase 5: Final Review
- [ ] Task 5.1: Run `cargo clippy -p ai-brains-store -- -D warnings` and fix all warnings.
- [ ] Task 5.2: Run `cargo test -p ai-brains-store` and ensure all tests, especially red-team ones, pass securely.
