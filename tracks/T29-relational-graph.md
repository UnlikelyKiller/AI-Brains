# Track T29 - Relational Graph Implementation

## Owner
Orchestrator

## Status
Completed

## Objective
Migrate the graph projection layer from LadybugDB to a relational model within SQLite (ADR-0009). This eliminates C++ build friction on Windows and leverages the existing encrypted SQLCipher foundation.

## Scope
- Define relational graph schema (nodes and edges) in `ai-brains-store`.
- Implement SQL migrations for the new tables.
- Refactor `ai-brains-graph` to use `rusqlite` and recursive CTEs.
- Maintain compatibility with existing `GraphProjector` and `GraphSearch` interfaces where possible.
- Update tests to verify the new SQL-based traversal logic.

## Out of Scope
- Performance optimization for multi-million node graphs (current scale is thousands).
- Weighted shortest path algorithms (not currently required).

## Implementation Steps
1. [x] Add migration 0013 to `ai-brains-store` for `graph_node` and `graph_edge`.
2. [x] Refactor `ai-brains-graph` to use `VaultConnection` instead of `LadybugVault`.
3. [x] Implement `GraphProjector` using SQL `INSERT` / `REPLACE`.
4. [x] Implement `GraphSearch` using recursive CTEs.
5. [x] Update `GraphRebuilder` to work with the new backend.
6. [x] Reconcile tests and CI gate.

## Acceptance Criteria
- Full workspace verification passes on Windows MSVC Debug.
- Graph traversals (Session Recall, RAPTOR) return correct results.
- `ai-brains-graph` no longer depends on the `lbug` crate.
- ChangeGuard ledger is reconciled.
