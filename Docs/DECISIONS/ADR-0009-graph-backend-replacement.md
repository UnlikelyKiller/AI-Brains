# ADR-0009: Graph Backend Replacement Strategy

## Status
Proposed

## Context
AI-Brains initially selected LadybugDB (`lbug`) as its embedded graph database. LadybugDB is a C++ based property graph that provides Cypher query support. However, during development on Windows 11 using the MSVC toolchain, the following critical issues emerged:

1.  **Linker Failure (`LNK1248`):** The LadybugDB C++ core, when compiled in Debug mode, exceeds the maximum allowable image size for PE/COFF files on Windows. This blocks routine development and CI verification.
2.  **Build Complexity:** Compiling the C++ backend requires CMake, a specific MSVC version, and often hits environment-specific issues that conflict with the "boringly reliable" capture principle.
3.  **Decoupling:** To protect the capture path, the graph was already decoupled and made an opt-in feature. RAPTOR and CRAG currently operate over FTS/Lexical projections successfully without the graph.

## Decision
We will replace LadybugDB with a **Relational Graph Model** implemented directly in the existing SQLCipher/SQLite store. 

This model will use:
- **Relationship Tables:** Dedicated tables for edges (e.g., `memory_hierarchy`, `session_turn_edges`).
- **Recursive CTEs:** SQLite's Recursive Common Table Expressions for multi-hop graph traversal.
- **Unified Transactionality:** The graph projection will live in the same database (or a sibling SQLite file) as the other projections, ensuring consistent backups and atomic updates.

## Rationale
1.  **Zero Extra Build Dependencies:** SQLite is already part of the core stack. No C++ compiler or CMake will be required for the graph feature.
2.  **Proven Windows Reliability:** SQLite is the industry standard for local-first storage on Windows.
3.  **Sufficient Performance:** For the scale of a local AI memory (thousands of nodes/edges), SQLite JOINs and CTEs are highly efficient and often faster than cross-language C++ calls for simple traversals.
4.  **Simplified Rebuilds:** The graph can be rebuilt from the append-only event log using standard SQL insert logic.
5.  **SQLCipher Support:** Relationships can be encrypted using the same keys as the primary vault.

## Consequences
- **Cypher Support:** We will lose native Cypher query support. Graph queries must be rewritten as SQL CTEs.
- **Schema Management:** We must manage a relational schema for nodes and edges instead of a schema-less property graph.
- **Migration Path:** The `ai-brains-graph` crate will be refactored to use `rusqlite` instead of `lbug`. The `GraphSearch` and `GraphProjector` traits will remain the primary interfaces.
