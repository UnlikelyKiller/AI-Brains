# Track T69: Live Graph Hook — Incremental Graph Updates

**Status:** ✅ **Complete**
**Started:** 2026-05-30
**Owner:** Claude (Cowork)
**Parent:** T66 (Graph-Augmented Recall), T67 (Memory Pinning Events), T68 (Memory Synthesis Events)
**Priority:** **MEDIUM** — performance / operational improvement; correctness is already provided by graph rebuild

---

## Problem Statement

Every time the vault receives new events (recall, nightly, capture), the graph goes stale. Making it current requires a manual `ai-brains graph rebuild`, which:

1. **Reads every event ever written** (O(n) with vault size) — currently ~8,864 events, growing forever
2. **Deletes and re-inserts all graph_node / graph_edge rows** — full table wipe on every run
3. **Adds latency before graph queries are useful** — `graph session <id>` returns stale data until rebuild runs
4. **Requires an explicit manual step** — easy to forget; nightly doesn't run it automatically

The `GraphProjector` already has exactly the API needed for incremental updates:
- `apply(&Envelope) -> Result<()>` — processes a single event into node/edge buffers
- `flush() -> Result<()>` — writes buffered nodes/edges to the SQLite backend

The SQLite backend uses `INSERT OR IGNORE` (via `ensure_node` / `ensure_edge`), so re-applying an already-projected event is idempotent.

---

## Acceptance Criteria

**AC1:** After `ai-brains recall "query"`, graph edges for the returned `MemoryPinned` events are visible immediately — no `graph rebuild` required  
**AC2:** After `ai-brains nightly`, `SYNTHESIZED_FROM` and `SessionSummaryCreated` graph edges are visible immediately  
**AC3:** Capture events (`UserPromptRecorded`, `SessionStarted`, etc.) are projected to graph nodes/edges at append time  
**AC4:** Graph projection failures are non-fatal — primary event append always succeeds  
**AC5:** `ai-brains graph rebuild` continues to work as a full resync tool (for schema changes / corruption recovery)  
**AC6:** New `ai-brains graph update` command reports current graph health (node/edge counts + last-event timestamp)  

---

## Architecture

### The Core Issue: Crate Boundaries

`ai-brains-store` (EventStore) and `ai-brains-graph` (GraphProjector) are separate crates. Adding a graph dep to the store creates a circular dependency. The solution is a thin wrapper layer in `ai-brains-cli` (which already depends on both):

```
ai-brains-store  ──→  [no change]
ai-brains-graph  ──→  [no change]
ai-brains-cli    ──→  NEW: live_graph.rs
                         LiveGraphHook
                         GraphAwareEventStore (implements EventStore)
```

### `LiveGraphHook`

Owns a `GraphProjector<'static>` backed by `SqliteGraphBackend` (+ `CozoProxyBackend` if ChangeGuard is available). Exposes a single method:

```rust
pub fn apply_and_flush(&mut self, envelope: &Envelope)
```

Non-fatal: logs warnings on error, never panics.

### `GraphAwareEventStore`

A newtype wrapper over `SqliteEventStore` that also holds a `Mutex<LiveGraphHook>`. Implements `EventStore`. After each successful `append_event`, applies the envelope to the hook.

Passes through all other `EventStore` methods to the inner store unchanged.

---

## Implementation Plan

### Phase 1: `live_graph.rs` (new file in `ai-brains-cli/src/`)

**`LiveGraphHook`:**
```rust
#[cfg(feature = "graph")]
pub struct LiveGraphHook {
    projector: ai_brains_graph::GraphProjector<'static>,
}

impl LiveGraphHook {
    pub fn new(conn: Arc<VaultConnection>) -> Self { ... }
    pub fn apply_and_flush(&mut self, envelope: &Envelope) { ... }
}
```

**`GraphAwareEventStore`:**
```rust
#[cfg(feature = "graph")]
pub struct GraphAwareEventStore {
    inner: SqliteEventStore,
    hook: std::sync::Mutex<LiveGraphHook>,
}

impl GraphAwareEventStore {
    pub fn new(conn: VaultConnection) -> Self { ... }
}

impl EventStore for GraphAwareEventStore { ... }
```

Non-graph stub (always compiled):
```rust
#[cfg(not(feature = "graph"))]
pub type GraphAwareEventStore = ai_brains_store::SqliteEventStore;
```

### Phase 2: Wire into CLI commands

**`commands/recall.rs`:** Replace inline `SqliteEventStore::new(...)` with `GraphAwareEventStore::new(...)` for MemoryPinned emission.

**`commands/nightly.rs`:** Replace `Arc::new(SqliteEventStore::new(...))` with `Arc::new(GraphAwareEventStore::new(...))` passed to `NightlyService`.

**`context.rs` `StoreSink`:** Add `#[cfg(feature = "graph")] hook: Option<LiveGraphHook>` field. In `CaptureSink::append`, after store append, call `hook.apply_and_flush(&envelope)`.

### Phase 3: `graph update` command

**`main.rs`:** Add to `GraphCommands` enum:
```rust
/// Show current graph health (node/edge counts)
Update,
```

**`commands/graph.rs`:** Add `update()` function that queries `graph_node` and `graph_edge` counts from the vault SQLite and prints a JSON health report:
```json
{"nodes": 8754, "edges": 8624, "status": "live"}
```

### Phase 4: Auto-rebuild in nightly

**`commands/nightly.rs`:** After the nightly sweep completes, run a graph rebuild if the incremental hook is disabled (feature not enabled), or just log that the graph was updated incrementally if the hook is enabled. This removes the need to manually run `graph rebuild` after nightly.

Under `#[cfg(feature = "graph")]`:
```rust
eprintln!("[Nightly] Graph updated incrementally — no rebuild needed.");
```

Under `#[cfg(not(feature = "graph"))]`:
```rust
// No incremental hook; skip rebuild note.
```

---

## Files to Touch

| File | Change |
|------|--------|
| `crates/ai-brains-cli/src/live_graph.rs` | NEW: `LiveGraphHook` + `GraphAwareEventStore` |
| `crates/ai-brains-cli/src/main.rs` | Export `live_graph` module; add `GraphCommands::Update` |
| `crates/ai-brains-cli/src/commands/recall.rs` | Use `GraphAwareEventStore` for MemoryPinned emit |
| `crates/ai-brains-cli/src/commands/nightly.rs` | Use `GraphAwareEventStore` for `NightlyService`; add post-nightly note |
| `crates/ai-brains-cli/src/context.rs` | Add `LiveGraphHook` to `StoreSink` for capture path |
| `crates/ai-brains-cli/src/commands/graph.rs` | Add `update()` function |

---

## Dependencies

| Track | Status | How This Depends On It |
|-------|--------|--------------------------|
| T66   | ✅ Code Complete | Graph CLI + augmented recall in place |
| T67   | ✅ Implemented | MemoryPinned events exist; hook makes them graph-live |
| T68   | ✅ Implemented | MemorySynthesized events exist; hook makes them graph-live |

---

## Risks

| Risk | Mitigation |
|------|-----------|
| `Mutex<LiveGraphHook>` lock contention on high-throughput capture | Single-writer vault already serializes appends; contention is bounded |
| Hook failure cascades into event append failure | All errors from `apply_and_flush` are logged as warnings, never propagated |
| Double-projection on rebuild after incremental updates | `INSERT OR IGNORE` on `graph_node`; `INSERT OR IGNORE` on `graph_edge` — idempotent |
| `GraphProjector` lifetime: `'static` requirement | `SqliteGraphBackend` holds `Arc<VaultConnection>` which is `'static`; no lifetime issues |

---

## Verification Steps

```powershell
# 1. Build with graph feature
cargo build --features graph -p ai-brains-cli

# 2. Run a recall — MemoryPinned events should immediately appear in graph
ai-brains recall "test" --limit 3
ai-brains graph session <session_id_from_output>
# Expected: non-empty memories WITHOUT running graph rebuild first

# 3. Run nightly — synthesis events should immediately appear in graph
ai-brains nightly
ai-brains graph hierarchy <a_summary_memory_id>
# Expected: non-empty synthesized_from WITHOUT graph rebuild

# 4. Verify graph update command
ai-brains graph update
# Expected: {"nodes": N, "edges": M, "status": "live"}

# 5. Verify graph rebuild still works as resync
ai-brains graph rebuild
ai-brains graph update
# Expected: same or higher node/edge counts
```

---

## Success Criteria

- [ ] `graph session <id>` returns memories immediately after `recall`, no rebuild needed
- [ ] `graph hierarchy <id>` returns chain immediately after `nightly`, no rebuild needed
- [ ] `graph update` reports current node/edge counts
- [ ] All existing tests pass
- [ ] No `unwrap()` or `panic!()` in new code
