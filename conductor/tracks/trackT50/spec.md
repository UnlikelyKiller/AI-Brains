# Track T50 - CozoDB Bridge Hardening & Unification

## 1. Overview
AI-Brains currently uses a hybrid graph system with a local SQLite-based relational graph and a proxy bridge to ChangeGuard's CozoDB instance. To resolve performance friction, drift, and reliability issues, this track introduces batch ingestion for graph backends, unifies the graph projection pipeline, and enhances the CozoDB bridge with fast-fail capabilities and structured error handling.

## 2. Goals
- Eliminate the performance bottleneck of spawning per-node/edge `changeguard` processes.
- Prevent graph drift by synchronously projecting to both SQLite and CozoDB.
- Increase bridge reliability with ultra-fast health checks and structured JSON errors.
- Ensure robust Datalog escaping for all mutated strings.

## 3. Architecture Changes

### 3.1. Batch Mutation API
Modify the `GraphBackend` trait to support bulk operations:
- `add_nodes(nodes: &[Node])`
- `add_edges(edges: &[Edge])`
- `CozoProxyBackend` will implement these by buffering operations and constructing a single Datalog `:put` block, invoking the `changeguard` IPC exactly once per batch.

### 3.2. Projector Unification
Introduce a `MultiplexGraphBackend` or modify `GraphProjector` to hold multiple backends.
- `GraphProjector` pushes `Node`/`Edge` events to both `SqliteGraphBackend` and `CozoProxyBackend`.
- Batching must be utilized when replaying the event log to rebuild projections.

### 3.3. Handshake & Fast-Fail
- Extend `CozoProxyBackend` to execute an ultra-fast handshake (from Track T44) upon initialization or before large batches.
- Fail fast (<10ms) if the ChangeGuard bridge is unavailable, falling back gracefully if necessary.

### 3.4. Structured Error Handling
- Transition `send_datalog_mutation` and `run_datalog_query` to parse `ApiResult` JSON formats from the bridge.
- Map bridge-specific errors to internal `ai_brains_graph::Error` variants.

### 3.5. Datalog Escaping
- Implement an explicit `escape_datalog_string` function.
- Handle Unicode, control characters, and Datalog string delimiter escapes to prevent injection and parse errors.

## 4. Test Strategy
- **Unit Tests:** Verify Datalog string escaping across boundary cases.
- **Integration Tests:** 
  - Mock `changeguard` bridge returning `ApiResult` JSON.
  - Verify `GraphProjector` writes to both backends during an event replay.
  - Verify batch size and IPC call counts.

## 5. Constraints
- **Zero Panic Policy:** All escaping and JSON parsing must return explicit `Result`s. No `unwrap` or `expect`.
- **PowerShell First:** Subprocess invocation for `changeguard` must use standard Windows process spawning compatible with the ecosystem.
