# Track T70: ChangeGuard Symbol Bridge — Code-Aware Recall

**Status:** ✅ **Complete**
**Started:** 2026-05-31
**Owner:** Codex
**Parent:** T69 (Live Graph Hook), T66 (Graph-Augmented Recall)
**Priority:** **MEDIUM** — enables AI agents to recall live code symbols, endpoints, and functions via the AI-Brains vault

---

## Problem Statement

AI-Brains recall is rich with session memory, decisions, and synthesized knowledge — but it has no awareness of the actual codebase: what functions exist, what HTTP routes are registered, what modules call what.

ChangeGuard already solves the code-symbol problem: it indexes every public function, struct, HTTP route, and call edge via tree-sitter (Rust, TypeScript, Python) and SCIP, storing them in its local index (`project_symbols`, `project_files`, and `api_routes` tables). `changeguard search "handleGetUser"` works today.

The gap: an AI starting a session must know to run `changeguard search` separately. It cannot ask `ai-brains recall "GET /users endpoints"` and get code-symbol results. T70 closes that gap by bridging ChangeGuard's symbol index into AI-Brains memories during the nightly pipeline.

**Example of what becomes possible after T70:**
```
ai-brains recall "authentication endpoints" --semantic --limit 5
→ [score=0.91] fn handle_login (src/routes/auth.rs:42) — POST /auth/login
→ [score=0.87] fn handle_refresh (src/routes/auth.rs:89) — POST /auth/refresh
→ [score=0.83] fn validate_token (src/middleware/auth.rs:14) — called by 12 handlers
```

---

## Acceptance Criteria

**AC1:** During nightly, `changeguard index` is called to refresh the symbol index before ingestion  
**AC2:** Public functions, HTTP routes, and module-level symbols are ingested as AI-Brains memories via a new `SymbolIndexed` event type (or `MemoryPinned` with `source_tag: "changeguard:symbol"`)  
**AC3:** `ai-brains recall "GET handlers"` returns code symbols alongside session memories  
**AC4:** Symbol memories include: `qualified_name`, `symbol_kind`, `file_path`, `line_start`, `http_method` (if route), `http_path` (if route)  
**AC5:** Symbols are scoped to the correct project via `project_id`  
**AC6:** Existing symbols are upserted (not duplicated) on subsequent nightly runs  
**AC7:** If ChangeGuard is unavailable, symbol ingestion is skipped non-fatally  

---

## Current State

### What ChangeGuard already has (no changes needed there)
- `project_symbols` SQLite table joined to `project_files`: `id`, `file_path`, `qualified_name`, `symbol_name`, `symbol_kind`, `is_public`, `line_start`, `line_end`
- `api_routes` SQLite table: `method`, `path_pattern`, `handler_symbol_id`, `handler_symbol_name`, `handler_file_id`
- `ExtractedRoute` struct: `method`, `path_pattern`, `handler_name`, `framework`, `confidence`
- Call graph: `CallKind` (Direct, MethodCall, TraitDispatch, Dynamic, External)
- SCIP symbol table with precise LSP-level navigation
- `changeguard index` refreshes all of the above incrementally in the current local binary
- Bridge IPC snapshots exist, but current `changeguard bridge export` does not expose arbitrary `--graph-query`; AI-Brains reads the local indexed SQLite state directly for T70

### What AI-Brains has (the entry points)
- `CozoProxyBackend` — can send arbitrary Datalog queries to ChangeGuard's CozoDB
- `nightly.rs` — already calls MADR ingestion from ChangeGuard; this is where T70 hooks in
- `MemoryPinnedPayload` — has `source_tag` field for labeling external sources (added in T67)
- `SqliteEventStore::append_event` — the target for symbol event emission

---

## Implementation Plan

### Phase 1: Refresh ChangeGuard index in nightly (non-fatal)

**File:** `crates/ai-brains-cli/src/commands/nightly.rs`

Before the MADR ingestion step, add:
```rust
// Refresh ChangeGuard symbol index so symbol bridge has current data
fn refresh_changeguard_index() {
    let result = std::process::Command::new("changeguard")
        .arg("index")
        .output();
    match result {
        Ok(o) if o.status.success() => eprintln!("[Nightly] ChangeGuard symbol index refreshed."),
        Ok(o) => eprintln!("[Nightly] ChangeGuard index refresh non-fatal: {}", 
                           String::from_utf8_lossy(&o.stderr)),
        Err(e) => eprintln!("[Nightly] ChangeGuard not available for indexing: {}", e),
    }
}
```

### Phase 2: Query ChangeGuard symbols via bridge

**New file:** `crates/ai-brains-cli/src/commands/symbol_bridge.rs`

Query `project_symbol` table:
```datalog
?[file_path, qualified_name, symbol_name, symbol_kind, line_start]
:= *project_symbol{file_path, qualified_name, symbol_name, symbol_kind, is_public: true, line_start}
```

Query `ExtractedRoute` data (stored in ChangeGuard's KG under a `route` relation or similar — confirm exact table name by reading ChangeGuard's schema):
```datalog
?[method, path_pattern, handler_name, file_path]
:= *route{method, path_pattern, handler_name, file_path}
```

Parse results and build a `Vec<SymbolRecord>` with fields matching AC4.

### Phase 3: Emit `MemoryPinned` events for each symbol

For each symbol returned:
1. Compute a stable `memory_id` from `sha256(project_id + qualified_name)` mapped to UUID — ensures upsert idempotency
2. Check if event with that aggregate_id already exists via `read_events(memory_id_uuid)`
3. If it doesn't exist (or if the symbol's `line_start` has changed), emit:
```rust
Payload::MemoryPinned(MemoryPinnedPayload {
    memory_id,
    content: format!("{} {} ({}:{})", symbol_kind, qualified_name, file_path, line_start),
    session_id: None,
    project_id: Some(project_id),
    tx_id: None,
    rank: None,
    source_tag: Some("changeguard:symbol".to_string()),
    query_text: None,
})
```

For HTTP routes, include method and path in `content`:
```
"route GET /users/:id → handle_get_user (src/routes/user.rs:42)"
```

### Phase 4: Scope by project

The nightly command already operates under a `project_id`. Symbol ingestion must only query symbols from the current project's source root. Pass the project's resolved root path to the Datalog query as a filter:
```datalog
?[qualified_name, ...] := *project_symbol{qualified_name, file_path, ...},
    starts_with(file_path, <project_root>)
```

### Phase 5: Wire into nightly

In `nightly.rs`, after MADR ingestion:
```rust
eprintln!("[Nightly] Ingesting code symbols from ChangeGuard...");
if let Err(e) = ingest_changeguard_symbols(ctx, project_id, &event_store) {
    eprintln!("[Nightly] Symbol ingestion failed (non-fatal): {}", e);
}
```

---

## Files to Touch

| File | Change |
|------|--------|
| `crates/ai-brains-cli/src/commands/nightly.rs` | Add `refresh_changeguard_index()` call + `ingest_changeguard_symbols()` call |
| `crates/ai-brains-cli/src/commands/symbol_bridge.rs` | NEW: query logic, `SymbolRecord` struct, emission loop |
| `crates/ai-brains-cli/src/main.rs` | `mod symbol_bridge;` |

No changes to ChangeGuard. No new event types needed (reuse `MemoryPinned` with `source_tag`).

---

## Dependencies

| Track | Status | How This Depends On It |
|-------|--------|--------------------------|
| T67   | ✅ Implemented | `MemoryPinnedPayload.source_tag` field exists |
| T69   | ✅ Implemented | Live graph hook means symbol memories are graph-projected immediately |
| T66   | ✅ Implemented | Graph-boosted recall surfaces symbol memories with neighbor context |

---

## Risks

| Risk | Mitigation |
|------|-----------|
| ChangeGuard's Datalog table names differ from what we expect | Read ChangeGuard's `src/state/cozo/queries.rs` for exact `CREATE` statements before writing queries |
| Symbol volume too large (thousands of functions) | Filter to `is_public: true` only; cap at 500 per nightly run with a cursor |
| Duplicate events on re-run | Stable `memory_id` from `sha256(project_id + qualified_name)` ensures idempotency |
| ChangeGuard bridge IPC unavailable | Non-fatal skip with `eprintln!` warning |
| Route table name in ChangeGuard index | Resolved: route metadata is read from the SQLite `api_routes` table and joined to `project_symbols` |

---

## Verification Steps

```powershell
# 1. Run nightly with symbol bridge
ai-brains nightly --vault-path C:\dev\ai-brains\vault.db

# 2. Check for symbol memories
ai-brains recall "authentication" --semantic --limit 10
# Expected: includes entries with source "changeguard:symbol"

# 3. Recall specific endpoint
ai-brains recall "POST /auth" --semantic
# Expected: returns route handler memories

# 4. Graph neighbors of a symbol memory
ai-brains graph neighbors <symbol_memory_id>
# Expected: related functions from graph edges

# 5. Verify no duplicates on re-run
ai-brains nightly  # run again
# Expected: same memory IDs, no duplicates
```

---

## Success Criteria

- [ ] `ai-brains recall "GET endpoints"` returns code-symbol results
- [ ] Symbol memories have `source_tag: "changeguard:symbol"` visible in output
- [ ] Running nightly twice does not duplicate symbol memories
- [ ] ChangeGuard unavailability does not fail nightly
- [ ] All existing tests pass
