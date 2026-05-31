# Track T67: Memory Pinning Events

**Status:** ✅ **Complete**  
**Started:** 2026-05-30  
**Owner:** Hermes  
**Parent:** T66 (Graph-Augmented Recall)  
**Priority:** **HIGH** — required for graph session/memory edges

---

## Problem Statement

The AI-Brains graph has **8,754 nodes and 8,624 edges** after `graph rebuild`. But the edges are almost entirely `IN_SESSION` (turn → session) and `IN_PROJECT` (session → project). **Zero `RECALLS` or `SOURCE_FOR` edges exist** because no `MemoryPinned` events have ever been emitted into the vault.

**Why this matters:**
- `ai-brains graph session <session_id>` returns `[]` — no way to traverse from session to its recalled memories
- Graph-augmented recall (T66) can’t boost neighbors because semantic hits are `turn` nodes with no outgoing memory edges
- The `get_related_memories()` CTE in `queries.rs` depends on `RECALLS`/`SOURCE_FOR` edges — currently dead code

---

## Acceptance Criteria

**AC1:** When `recall()` retrieves a memory, a `MemoryPinned` event is emitted to the event store  
**AC2:** `MemoryPinned` payload includes `memory_id`, `session_id`, `project_id`, `query_text`, `rank`  
**AC3:** `GraphProjector` maps `MemoryPinned` → `memory` node + `turn --RECALLS/SOURCE_FOR--> memory` edge  
**AC4:** After `graph rebuild`, `graph session <session_id>` returns the session’s pinned memories  
**AC5:** Graph-augmented recall (T66) now boosts memory neighbors because semantic hits have graph edges  

---

## Current State

- `GraphProjector` already handles `MemoryPinned` in `projector.rs` ✅
- `GraphRebuilder` replays all events including `MemoryPinned` ✅
- Event store schema supports arbitrary event types ✅
- **Missing:** `MemoryPinned` event emission in `recall.rs` or `recall` CLI command

---

## Implementation Plan

### Phase 1: Emit `MemoryPinned` Event in Recall
**Goal:** Every time a memory is returned by `recall()`, persist a `MemoryPinned` event

**Changes:**
1. In `crates/ai-brains-cli/src/commands/recall.rs`, after `recall()` returns hits:
   - Iterate over returned `RecallHit`s
   - For each hit, emit `MemoryPinned` event via `EventStore::append()`
2. Event payload:
   ```json
   {
     "memory_id": "uuid",
     "session_id": "uuid or null",
     "project_id": "uuid or null", 
     "query_text": "the original query string",
     "rank": 0,
     "source": "semantic|fts|bridge"
   }
   ```
3. Aggregate type: `memory`, aggregate id: `memory_id`

**File:** `crates/ai-brains-cli/src/commands/recall.rs`
**Estimated effort:** 1-2 hrs
**Blocked by:** None

---

### Phase 2: Batch MemoryPinned Events (Optional)
**Goal:** Avoid event storm for large `limit` values

**Changes:**
1. Instead of one event per hit, emit one `MemoryPinned` event with a `memory_ids` array
2. Payload:
   ```json
   {
     "memory_ids": ["uuid1", "uuid2"],
     "session_id": "uuid",
     "project_id": "uuid",
     "query_text": "...",
     "sources": ["semantic", "fts"]
   }
   ```

**File:** `crates/ai-brains-cli/src/commands/recall.rs`
**Estimated effort:** 1 hr
**Blocked by:** Phase 1

---

### Phase 3: Verify Graph Edges
**Goal:** Confirm rebuild creates `RECALLS`/`SOURCE_FOR` edges

**Steps:**
1. Run `ai-brains recall "test" --limit 5`
2. Run `ai-brains graph rebuild`
3. Query SQL: `SELECT COUNT(*) FROM graph_edge WHERE label IN ('RECALLS', 'SOURCE_FOR')`
4. Run `ai-brains graph session <session_id>` — should return memory IDs

**Estimated effort:** 30 min
**Blocked by:** Phase 1

---

## Dependencies

| Track | Status | How This Depends On It |
|-------|--------|--------------------------|
| T66   | ✅ Code Complete | Graph code exists; this provides the data |

---

## Risks

| Risk | Mitigation |
|------|-----------|
| Event storm (1 event per recalled memory) | Use batching in Phase 2; or rate-limit by session |
| Privacy leakage in `query_text` payload | Mark `query_text` as `privacy: "private"` in event |
| Event store bloat | `MemoryPinned` events are small; retention policy cleans old ones |

---

## Files to Touch

| File | Change |
|------|--------|
| `crates/ai-brains-cli/src/commands/recall.rs` | Emit `MemoryPinned` event after recall results |
| `crates/ai-brains-cli/src/context.rs` | Ensure `EventStore` is available in `AppContext` |
| `crates/ai-brains-store/src/event_store.rs` | Verify `append()` accepts `MemoryPinned` payload |
| `crates/ai-brains-core/src/events.rs` | Add `MemoryPinned` payload struct (if not exists) |

---

## Verification Steps

```powershell
# 1. Run recall
ai-brains recall "test" --limit 3

# 2. Check event was emitted
ai-brains events list --limit 5
# Expected: MemoryPinned events at top

# 3. Rebuild graph
ai-brains graph rebuild

# 4. Check edges exist
# (Python query: SELECT COUNT(*) FROM graph_edge WHERE label IN ('RECALLS', 'SOURCE_FOR'))

# 5. Test graph session
ai-brains graph session <session_id_from_recall>
# Expected: JSON with memory_ids array
```

## Success Criteria

- [ ] `ai-brains graph session <id>` returns non-empty memories after recall + rebuild
- [ ] Graph-augmented recall (T66) includes memory neighbors in results
- [ ] Event store has `MemoryPinned` events with correct payloads
