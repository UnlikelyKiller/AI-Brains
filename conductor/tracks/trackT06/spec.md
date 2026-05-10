# Specification: T06 Store Projections

## 1. Overview
This specification covers the implementation of read-optimized projections in the `ai-brains-store` crate. Projections are purely derived from the event log (the single source of truth). The objective is to efficiently query aggregated state like Sessions, Turns, and Memories without repeatedly folding the entire event log. Additionally, we integrate FTS5 for full-text search over memory content.

## 2. Dependencies
- `ai-brains-core` (domain primitives)
- `ai-brains-events` (event definitions)
- `rusqlite` (FTS5 feature required)
- `serde_json` (payload parsing if necessary)

## 3. Database Schema Migrations
We will introduce migrations for projection tables.
- `0002_identity_projection.sql`
- `0003_project_projection.sql`
- `0004_session_projection.sql`
- `0005_turn_projection.sql`
- `0006_memory_projection.sql`
- `0007_fts_setup.sql` (FTS5 virtual tables)
- `0008_fts_triggers.sql` (triggers to keep FTS in sync with `memory_projection`)

Tables like `session_projection` will contain read-optimized columns: `session_id`, `status`, `created_at`, `updated_at`, etc.

## 4. Components

### 4.1 Projections Module (`src/projections/mod.rs`)
Defines the `Projection` trait or handler interface. Sub-modules:
- `project.rs`
- `session.rs`
- `turn.rs`
- `memory.rs`

These modules provide SQL execution blocks that take an `EventEnvelope` and `rusqlite::Transaction` and perform `INSERT`/`UPDATE`/`DELETE` on the corresponding projection tables.

### 4.2 Replay Mechanism (`src/replay.rs`)
A robust `Replay` service that can:
1. Clear all projection tables.
2. Read the entire event log sequentially.
3. Pass each event through the projection handlers to rebuild the current state.
This ensures eventual consistency and recoverability if projection schemas change.

### 4.3 FTS Integration (`src/fts.rs`)
Manages querying the FTS5 virtual table. SQLite triggers in `0008_fts_triggers.sql` should automatically populate and delete rows in the FTS table when the underlying `memory_projection` table changes. This module provides convenient Rust wrappers around `MATCH` queries.

## 5. Concurrency
SQLite with WAL mode allows concurrent reads and a single writer. Projection updates run in the same transaction as the event append.
