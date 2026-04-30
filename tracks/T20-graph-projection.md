# Track T20 - Graph Projection

## Owner
architecture-planner

## Status
Completed

## Objective
Build a rebuildable graph projection layer using LadybugDB (SQLite-based graph) to represent relationships between projects, sessions, turns, and memories.

## Scope
- Scaffold `ai-brains-graph` crate.
- Implement graph schema and node/edge types.
- Implement a `GraphProjector` that listens to events and updates the graph.
- Implement `rebuild` logic to reconstruct the graph from the event store.
- Implement relationship queries, such as "related memories via session".

## Out of Scope
- Native graph database (Neo4j, etc.) - sticking to LadybugDB/SQLite for portability.
- Advanced graph algorithms (PageRank, etc.) - focus on adjacency and traversal.

## Files Owned
- `crates/ai-brains-graph/**`

## Files Allowed To Touch
- `Cargo.toml` (workspace members)
- `Docs/conductor/conductor.md`
- `Docs/status.md`
- `crates/ai-brains-store/src/lib.rs` (if integration needed)

## Files Forbidden To Touch
- `crates/ai-brains-core/**` (Core domain is locked)
- `crates/ai-brains-events/**`

## Public Contracts Consumed
- `ai_brains_events::Envelope`
- `ai_brains_store::EventStore`

## Public Contracts Produced
- `ai_brains_graph::GraphSearch`
- `ai_brains_graph::GraphProjector`

## Required Tests First
- `tests/schema_initializes.rs`
- `tests/projector_creates_project_session_memory_nodes.rs`
- `tests/rebuild_is_idempotent.rs`

## Implementation Steps
1. [x] Scaffold `ai-brains-graph` crate and add to workspace.
2. [x] Implement `schema.rs` and initialize the graph backend.
3. [x] Implement graph node/edge persistence through the graph adapter.
4. [x] Implement `projector.rs` for event-driven updates.
5. [x] Implement `rebuild.rs` for event-log replay.
6. [x] Implement `queries.rs` for graph traversal.
7. [x] Verification and CI gate reconciliation.

## Failure Modes To Handle
- Database locked (retry logic).
- Missing events (rebuild requirement).
- Schema drift.

## Security Requirements
- No plaintext secrets in graph labels.
- Privacy levels respected during traversal.

## Acceptance Criteria
- Graph rebuilds correctly from the event log.
- Queries return nodes related by at least one hop.
- CI pass with clippy and workspace tests.

## Handoff Notes
- Code artifacts and tests are present under `crates/ai-brains-graph`.
- Default graph verification is green on Windows with `cargo check --workspace --all-targets`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace`.
- Native LadybugDB/lbug remains available as the opt-in `ai-brains-graph/ladybug` feature for suitable toolchains.
- ChangeGuard provenance is reconciled locally: no pending transaction remains for `crates/ai-brains-graph`.
