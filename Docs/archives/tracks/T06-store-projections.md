# Track T06 — Store Projections

## Owner
architecture-planner

## Status
Completed

## Objective
Implement read-optimized projections (Session, Turn, Memory, FTS) derived from events in the `ai-brains-store` crate.

## Scope
- Projections are derive-only; they are NOT the source of truth.
- Implement projection rebuild strategy (replay events to update projections).
- Implement FTS5 virtual table for memory search.
- SQL Migrations for all projection tables and the FTS virtual table.

## Out of Scope
- Domain logic validation (already handled by events).
- Integration with external web services or frontends.

## Files Owned
- `crates/ai-brains-store/src/projections/*`
- `crates/ai-brains-store/src/fts.rs`
- `crates/ai-brains-store/src/replay.rs`
- `crates/ai-brains-store/migrations/0002_identity_projection.sql` through `0008_fts.sql`
- `Docs/conductor/trackT06/spec.md`
- `Docs/conductor/trackT06/plan.md`

## Files Allowed To Touch
- `crates/ai-brains-store/src/lib.rs`
- `crates/ai-brains-store/src/migrations.rs`
- `crates/ai-brains-store/Cargo.toml`
- `Docs/conductor/conductor.md`

## Public Contracts Consumed
- `ai-brains-core`
- `ai-brains-events`
- `rusqlite`
- `serde_json`

## Required Tests First
- `projections_update_from_events`
- `replay_rebuilds_projections`
- `fts_indexes_memory`
- `concurrent_read_single_writer`
