# Specification: T05 Store Event Log

## 1. Overview
This specification details the design and implementation of the `ai-brains-store` crate, specifically focusing on the initial event storage layer (the "canonical SSOT"). The implementation will use `rusqlite` bundled with `sqlcipher` to ensure that all data is encrypted at rest using keys managed by `ai-brains-crypto`. 

The system relies strictly on an append-only event log.

## 2. Dependencies
**Crates consumed:**
- `ai-brains-core` for core types (Ids, Privacy, etc.).
- `ai-brains-events` for the `EventEnvelope` and structured payloads.
- `ai-brains-crypto` for retrieving the `SqlCipherKey`.
- `rusqlite` (with `bundled-sqlcipher` feature).
- `thiserror` for centralized error typing.
- `serde_json` for serialization/deserialization of event payloads.

## 3. Database Schema

### 3.1 Migration `0001_event_log.sql`
The initial migration script defines the primary `events` table and an append-only trigger.

```sql
CREATE TABLE events (
    event_id TEXT PRIMARY KEY,
    schema_version INTEGER NOT NULL,
    aggregate_type TEXT NOT NULL,
    aggregate_id TEXT NOT NULL,
    event_type TEXT NOT NULL,
    occurred_at TEXT NOT NULL,
    actor_json TEXT NOT NULL,
    causation_id TEXT,
    correlation_id TEXT,
    privacy TEXT NOT NULL,
    payload_json TEXT NOT NULL,
    payload_hash TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_events_aggregate
ON events(aggregate_type, aggregate_id, occurred_at);

CREATE INDEX idx_events_type_time
ON events(event_type, occurred_at);

CREATE INDEX idx_events_correlation
ON events(correlation_id);

CREATE INDEX idx_events_privacy
ON events(privacy);

-- Strict append-only constraint
CREATE TRIGGER prevent_event_update
BEFORE UPDATE ON events
BEGIN
    SELECT RAISE(ABORT, 'events are immutable');
END;
```

## 4. Components

### 4.1 Config and Pragmas
**`config.rs`:** Defines `StoreConfig` handling file paths (e.g., path to the vault).
**`pragmas.rs`:** Ensures secure SQLite defaults are set after the key is provided:
- `PRAGMA key = 'x''<hex_key>''';`
- `PRAGMA cipher_compatibility = 4;`
- `PRAGMA journal_mode = WAL;` (if compatible, or `DELETE`/`TRUNCATE` if WAL is problematic with SQLCipher in some environments, default to what rusqlite supports safely).
- `PRAGMA synchronous = NORMAL;`

### 4.2 Connection Wrapper
**`connection.rs`:** 
Provides `VaultConnection` which encapsulates the `rusqlite::Connection`. It is solely responsible for opening the database, applying the SQLCipher key, running the pragmas, and exposing a safe API to the `EventStore`. 

*Constraint:* Must explicitly check that `sqlite3_key` (via `pragma key`) was successful, typically by reading from the schema (`SELECT count(*) FROM sqlite_master;`). If it fails, it returns a clear "Vault Locked / Incorrect Key" error instead of falling back to plaintext or creating a new empty DB.

### 4.3 Migrations
**`migrations.rs`:**
Applies SQL strings in order. Keeps track of applied migrations in a simple `schema_migrations` table. It ensures idempotent applications (applying the same migration twice is a no-op).

### 4.4 Event Store and Transactions
**`event_store.rs`:**
Exposes methods like `append(envelope: &EventEnvelope)` and `read_events(aggregate_id: &AggregateId) -> Result<Vec<EventEnvelope>>`.
**`transaction.rs`:**
Wraps SQLite transactions to ensure atomicity.

### 4.5 Error Handling
**`errors.rs`:**
Must map `rusqlite::Error` into semantic application errors, such as:
- `StoreError::VaultLocked`
- `StoreError::MigrationFailed`
- `StoreError::EventAppendFailed`
- `StoreError::ImmutableEventModified` (when the trigger fires)

## 5. Security & Invariants
- **No plaintext:** We never open the SQLite file without a `PRAGMA key`.
- **Immutability:** The `events` table cannot be updated (enforced via SQLite Trigger).
- **No tool log storage:** The store implementation or the caller layer must reject any incoming payload containing hidden chain-of-thought or raw tool calls. This will be enforced by ensuring the `ai-brains-events` definitions used do not possess those fields, and an explicit test verifying we cannot shoehorn them in.
