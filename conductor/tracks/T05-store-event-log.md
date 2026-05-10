# Track T05 — Store Event Log

## Owner
architecture-planner

## Status
Completed

## Objective
Implement the SQLCipher-backed encrypted event log, migrations, and basic event append operations in the `ai-brains-store` crate.

## Scope
- SQLCipher integration via `rusqlite`.
- Vault initialization and secure connection management (`connection.rs`, `config.rs`, `pragmas.rs`).
- Embedded SQL migrations using the `include_str!` or similar macro (`migrations.rs`, `0001_event_log.sql`).
- Event store implementation for appending events (`event_store.rs`, `transaction.rs`).
- Strict append-only constraints enforced via SQLite triggers (`prevent_event_update`).

## Out of Scope
- Projections (session, turn, memory, etc.).
- Full Text Search (FTS).
- Backup, restore, and retention operations.
- Health checks and projection rebuilds.
- CLI, daemon, HTTP, or adapter integration.

## Files Owned
`crates/ai-brains-store/*`

## Files Allowed To Touch
`crates/ai-brains-store/src/lib.rs`
`crates/ai-brains-store/src/connection.rs`
`crates/ai-brains-store/src/config.rs`
`crates/ai-brains-store/src/pragmas.rs`
`crates/ai-brains-store/src/migrations.rs`
`crates/ai-brains-store/src/event_store.rs`
`crates/ai-brains-store/src/transaction.rs`
`crates/ai-brains-store/src/errors.rs`
`crates/ai-brains-store/migrations/0001_event_log.sql`
`crates/ai-brains-store/tests/*.rs`
`crates/ai-brains-store/Cargo.toml`
`Docs/conductor/trackT05/spec.md`
`Docs/conductor/trackT05/plan.md`
`Docs/conductor/conductor.md`
`tracks/T05-store-event-log.md`

## Files Forbidden To Touch
Any file outside `crates/ai-brains-store/` and the conductor planning docs.
Must NOT touch `ai-brains-cli`, `ai-brainsd`, `ai-brains-retrieval`, or `ai-brains-adapters`.

## Public Contracts Consumed
- `ai-brains-core` (domain types, ids).
- `ai-brains-events` (event envelope, kinds).
- `ai-brains-crypto` (SQLCipherKey material).
- `rusqlite` (with `sqlcipher` feature enabled).
- `thiserror` (for errors).

## Public Contracts Produced
- `StoreConfig` (database paths, pragma preferences).
- `VaultConnection` (wrapper around rusqlite connection ensuring encryption).
- `EventStore` (appends and reads from the event log).
- `Transaction` (unit of work).

## Required Tests First
- `tests/sqlcipher_encrypted_vault.rs`
- `tests/wrong_key_cannot_open.rs`
- `tests/migrations_idempotent.rs`
- `tests/event_append_atomic.rs`
- `tests/event_log_is_append_only.rs`
- `tests/direct_event_update_forbidden.rs`
- `tests/plaintext_fallback_forbidden.rs`
- `tests/raw_tool_call_not_storable.rs`

## Implementation Steps
1. Define the SQL schema in `migrations/0001_event_log.sql` along with the `prevent_event_update` trigger.
2. Implement vault connection logic using `rusqlite` configured with the SQLCipher key from `ai-brains-crypto`. Apply secure pragmas (e.g., locking mode, synchronous, journal mode, WAL if applicable).
3. Implement `migrations.rs` to run idempotent schema updates.
4. Implement `event_store.rs` to persist `EventEnvelope` structures into the `events` table. Ensure atomicity via `transaction.rs`.
5. Implement error types bridging SQLite errors and domain-specific store errors.
6. Write integration tests enforcing append-only rules, encryption enforcement, and rejection of forbidden data types.

## Failure Modes To Handle
- Vault locked or corrupted (failed SQLCipher decryption).
- Migration failures (must be atomic, not partially applied).
- Attempted updates or deletes on the `events` table (must abort via SQLite trigger).
- Appending invalid event payload data.

## Security Requirements
- **No Plaintext Fallback:** If the SQLCipher key is missing or incorrect, the connection must fail instantly. Under no circumstances should the vault fall back to plaintext SQLite.
- **Append Only:** The core event log must be strictly immutable at the database level.
- **Red Team Blocks:** Must explicitly reject the storage of raw tool calls or chain-of-thought elements.

## Acceptance Criteria
- Connection wrapper enforces SQLCipher keys successfully.
- Migrations apply safely and idempotently.
- `EventStore` can append and retrieve `EventEnvelope` instances.
- SQLite `BEFORE UPDATE` trigger actively prevents modifications to existing events.
- All tests pass (including red-team forbidden behavior tests).

## Commands To Run
`cargo test -p ai-brains-store`
`cargo clippy -p ai-brains-store -- -D warnings`

## Handoff Notes
N/A
