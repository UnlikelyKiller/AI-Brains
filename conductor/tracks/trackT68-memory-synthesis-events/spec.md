# Track T68: Memory Synthesis Events

**Status:** ✅ **Complete**  
**Started:** 2026-05-30  
**Owner:** Hermes  
**Parent:** T66 (Graph-Augmented Recall)  
**Priority:** **HIGH** — required for graph synthesis/hierarchy edges

---

## Problem Statement

The AI-Brains nightly pipeline creates **SessionSummary** events (110 in vault) but never emits `MemorySynthesized` events. This means the graph has **zero `SYNTHESIZED_FROM` edges**, so:

- `ai-brains graph hierarchy <memory_id>` returns `[]` for every memory
- The `get_synthesized_hierarchy()` CTE (depth-limited recursive query) finds nothing
- Hierarchical recall — e.g., "find the summary synthesized from these 12 session turns" — is impossible

**What synthesis currently does:**
- Nightly reads batches of memories
- Summarizes them into `SessionSummaryCreated` events
- These summaries are stored as `session_summaries` table rows
- But **no graph edges** link summary → source memories

**What synthesis SHOULD do:**
- When a summary is created, emit `MemorySynthesized` event
- Graph projector maps this to `synthesized` node + `SYNTHESIZED_FROM` edges
- Hierarchy queries work: `memory --SYNTHESIZED_FROM--> source1, source2, source3`

---

## Acceptance Criteria

**AC1:** When nightly synthesis creates a summary, a `MemorySynthesized` event is emitted  
**AC2:** `MemorySynthesized` payload includes `memory_id` (new summary), `source_ids` (parent memories), `session_id`  
**AC3:** `GraphProjector` maps `MemorySynthesized` → `synthesized` node + `synthesized --SYNTHESIZED_FROM--> source` edges  
**AC4:** After `graph rebuild`, `graph hierarchy <memory_id>` returns the synthesis chain  
**AC5:** `get_synthesized_hierarchy()` in `queries.rs` returns non-empty results  

---

## Current State

- `GraphProjector` already handles `MemorySynthesized` in `projector.rs` ✅
- `GraphRebuilder` replays all events including `MemorySynthesized` ✅
- `SessionSummaryCreated` events exist (110) but not mapped to graph edges
- **Missing:** `MemorySynthesized` event emission in nightly pipeline

---

## Implementation Plan

### Phase 1: Add `MemorySynthesized` Event to Nightly
**Goal:** Emit graph-trackable synthesis events during nightly pipeline

**Changes:**
1. In `crates/ai-brains-cli/src/commands/nightly.rs`, during synthesis loop:
   - When `synthesize_memory()` or summary creation succeeds
   - Collect `source_memory_ids` from the batch
   - Emit `MemorySynthesized` event:
     ```json
     {
       "memory_id": "new_summary_uuid",
       "source_ids": ["source1", "source2"],
       "session_id": "session_uuid or null",
       "project_id": "project_uuid",
       "synthesis_type": "session_summary|hierarchical|batch"
     }
     ```
2. Aggregate type: `memory`, aggregate id: `memory_id`

**File:** `crates/ai-brains-cli/src/commands/nightly.rs`
**Estimated effort:** 2-3 hrs
**Blocked by:** None

---

### Phase 2: Map Existing SessionSummaryCreated → MemorySynthesized (Backfill)
**Goal:** Convert existing 110 `SessionSummaryCreated` events into graph edges

**Changes:**
1. Create one-time backfill script or migration
2. Read `session_summaries` table
3. For each summary row, infer source memories from `session_id` + event correlation
4. Emit retroactive `MemorySynthesized` events

**Alternative (simpler):** Skip backfill. Run `graph rebuild` after Phase 1. Only new syntheses get edges.

**File:** New script or inline in `nightly.rs`
**Estimated effort:** 1-2 hrs
**Blocked by:** Phase 1

---

### Phase 3: Verify Graph Hierarchy
**Goal:** Confirm `graph hierarchy` and `get_synthesized_hierarchy()` work

**Steps:**
1. Run nightly pipeline to generate new synthesis
2. Run `ai-brains graph rebuild`
3. Query: `SELECT COUNT(*) FROM graph_edge WHERE label = 'SYNTHESIZED_FROM'`
4. Run: `ai-brains graph hierarchy <summary_memory_id>`

**Estimated effort:** 30 min
**Blocked by:** Phase 1

---

## Dependencies

| Track | Status | How This Depends On It |
|-------|--------|--------------------------|
| T66   | ✅ Code Complete | Graph code exists; this provides the data |
| T67   | ✅ Complete | Memory pinning provides RECALLS edges; synthesis provides SYNTHESIZED_FROM edges |

---

## Risks

| Risk | Mitigation |
|------|-----------|
| Retroactive synthesis attribution is hard | Skip backfill; only new syntheses get edges |
| Synthesis batch size affects event count | One `MemorySynthesized` per batch, with `source_ids` array |
| Event payload too large (many source_ids) | Cap `source_ids` at 50; truncate with warning |

---

## Files to Touch

| File | Change |
|------|--------|
| `crates/ai-brains-cli/src/commands/nightly.rs` | Emit `MemorySynthesized` event after each synthesis batch |
| `crates/ai-brains-core/src/events.rs` | Add `MemorySynthesized` payload struct (if not exists) |
| `crates/ai-brains-store/src/event_store.rs` | Verify `append()` accepts new payload |
| `crates/ai-brains-store/src/session_summary_store.rs` | Link summary rows to emitted events |

---

## Verification Steps

```powershell
# 1. Run nightly
ai-brains nightly

# 2. Check for new MemorySynthesized events
ai-brains events list --limit 10

# 3. Rebuild graph
ai-brains graph rebuild

# 4. Check SYNTHESIZED_FROM edges
# (Python: SELECT COUNT(*) FROM graph_edge WHERE label = 'SYNTHESIZED_FROM')

# 5. Test hierarchy on a newly synthesized memory
ai-brains graph hierarchy <summary_memory_id>
# Expected: {"root":"...","synthesized_from":["source1","source2"]}
```

## Success Criteria

- [ ] `ai-brains graph hierarchy <id>` returns non-empty `synthesized_from` after nightly + rebuild
- [ ] `SYNTHESIZED_FROM` edges exist in graph_edge table
- [ ] Nightly pipeline emits `MemorySynthesized` events without error
- [ ] `get_synthesized_hierarchy()` in `queries.rs` returns real results
