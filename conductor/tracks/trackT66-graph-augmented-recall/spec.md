# Track T66: Graph-Augmented Recall + Graph Query CLI

**Status:** 🔄 **In Progress**
**Started:** 2026-05-30
**Owner:** Hermes Agent
**Parent:** T65 (Repo Alias Resolution)
**Priority:** **HIGH** — unlocks graph value for all recall operations

---

## Problem Statement

The AI-Brains graph is populated (**8,754 nodes, 8,624 edges**) with rich relational data: sessions, turns, memories, conflicts, projects. But `recall()` only performs **flat** semantic + lexical search. It never traverses graph edges to find related memories, synthesized children, or conflict-related content.

**Example of missed value:**
- User recalls "GPU driver fix"
- Semantic search finds one memory about the fix
- But that memory was **synthesized from** 12 session turns — all related context is ignored
- A graph neighbor query would surface the full discussion chain

---

## Acceptance Criteria

**AC1:** `recall --graph-neighbors` augments semantic/lexical results with related memories  
**AC2:** CLI exposes graph traversal: `ai-brains graph neighbors <memory_id>`  
**AC3:** CLI exposes hierarchy: `ai-brains graph hierarchy <memory_id>`  
**AC4:** Graph search integrates into `RecallHit` ranking (not just appended)  
**AC5:** Performance: graph augmentation adds < 50ms per query on current vault size

---

## Verification Results (2026-05-30)

### What Works ✅

| Component | Status | Evidence |
|-----------|--------|----------|
| Graph rebuild | ✅ | `ai-brains graph rebuild` → "Rebuild complete" in 2.7s |
| `graph neighbors` | ✅ | Returns 21 `CONFLICTS_WITH` edges for memory `bbad90b0...` |
| `graph hierarchy` | ✅ | Returns empty JSON (no `SYNTHESIZED_FROM` edges in data) |
| `graph session` | ✅ | Returns empty memories (no `RECALLS`/`SOURCE_FOR` edges in data) |
| Graph-augmented recall | ✅ | Code wired; executes without error; no graph hits because no graph edges for semantic hits |
| Build | ✅ | Compiles with `--features graph` |

### What Doesn’t Work ❌ (Data Issue, Not Code)

| Query | Expected | Actual | Root Cause |
|-------|----------|--------|-----------|
| `graph session <session_id>` | Returns memories in session | `[]` | No `RECALLS`/`SOURCE_FOR` edges exist in graph |
| `graph hierarchy <memory_id>` | Returns synthesized children | `[]` | No `SYNTHESIZED_FROM` edges exist in graph |
| Graph-augmented recall | Neighbors boost semantic hits | No boost applied | Semantic hits are `turn`/`memory` nodes with no outgoing edges |

### Root Cause Analysis

The **graph is correctly built** but the **vault data lacks the events** that create rich edges:

| Event Type | Count | Creates Edges |
|-----------|-------|---------------|
| `AssistantFinalRecorded` | 6,903 | → `turn` node only (no edges to memories) |
| `UserPromptRecorded` | 1,440 | → `turn` node + `IN_SESSION` edge |
| `SessionStarted` | 177 | → `session` node + `IN_PROJECT` edge |
| `SessionSummaryCreated` | 110 | Summary (not synthesis) |
| `RecipePromoted` | 51 | → `recipe` node + `PART_OF_RECIPE` edge |
| `ConflictDetected` | 21 | → `CONFLICTS_WITH` edges ✓ |
| `MemoryPinned` | **0** | → Would create `RECALLS`/`SOURCE_FOR` edges |
| `MemorySynthesized` | **0** | → Would create `SYNTHESIZED_FROM` edges |

**Conclusion:** The projector code is correct. The vault simply has no `MemoryPinned` or `MemorySynthesized` events. These would need to be produced by the nightly pipeline or memory system. Once they exist, the graph will automatically have the edges, and all graph queries will return results.

---

## Current State

- `graph_node` / `graph_edge` tables: **populated and ACID-compliant** ✅
- `GraphSearch` queries written in `crates/ai-brains-graph/src/queries.rs` ✅
- `recall()` accepts `graph_boost` and `_graph_hop_depth` parameters ✅
- Graph augmentation code wired and executing ✅
- `GraphRebuilder`: works, 2.7s for 8,864 events
- Graph is behind `--features graph` flag — binary currently compiled **with** graph support
- **Missing:** `MemoryPinned`/`MemorySynthesized` events in vault data prevent graph edges

---

## Implementation Plan

### Phase 1: Graph-Augmented Recall (recall.rs) — ✅ COMPLETE
**Goal:** Make graph traversal part of recall scoring

**Changes made:**
1. `recall()` now accepts `graph_boost: f64` and `_graph_hop_depth: usize`
2. When `graph` is `Some(searcher)`:
   - Collects top-K hits from semantic/lexical
   - For each hit, queries `get_neighbors()` 
   - Neighbors get score boost (`parent_score + graph_boost`, default 0.1)
   - Unseen neighbors merged into `blended` results
3. Deduplication by `seen_ids` HashSet
4. Final resort by score descending

**File:** `crates/ai-brains-retrieval/src/recall.rs` ✅

---

### Phase 2: Graph CLI Commands (main.rs + graph.rs) — ✅ COMPLETE
**Goal:** Surface graph queries to users and agents

**Changes made:**
1. `ai-brains graph` subcommands:
   - `neighbors <memory_id>` — 1-hop neighbors (all edge labels, incoming+outgoing)
   - `hierarchy <memory_id>` — recursive `SYNTHESIZED_FROM` children
   - `session <session_id>` — all memories via `get_related_memories()` CTE
2. Output as JSON

**Files:**
- `crates/ai-brains-cli/src/commands/graph.rs` ✅
- `crates/ai-brains-cli/src/main.rs` ✅ (enum + dispatch)

---

### Phase 3: Graph-Neighbor Boost Config (recall.rs CLI) — ✅ COMPLETE
**Goal:** Control graph augmentation behavior

**Changes made:**
1. `ai-brains recall` now accepts:
   - `--graph-boost <float>` (default 0.1)
   - `--graph-hop-depth <usize>` (default 1, reserved for future)
2. `run_recall()` passes these to `recall()`
3. `sync.rs` updated to pass defaults (0.1, 1)

**Files:**
- `crates/ai-brains-cli/src/commands/recall.rs` ✅
- `crates/ai-brains-cli/src/commands/sync.rs` ✅

---

## Dependencies

| Track | Status | How This Depends On It |
|-------|--------|--------------------------|
| T65   | ✅ Complete | Alias resolution for project-scoped graph queries |
| T64   | ✅ Complete | WAL checkpoint ensures graph is durable |
| T63   | ✅ Complete | Embeddings populate semantic search that graph augments |

---

## Risks

| Risk | Mitigation | Status |
|------|-----------|--------|
| Graph query slows recall (>50ms) | Add graph query timeout; benchmark before/after | Not yet measured — needs real graph hits |
| Self-referential edges in SYNTHESIZED_FROM cause infinite recursion | CTE depth limit (already 10 in queries.rs) | ✅ Handled |
| Memory IDs in graph don't have embeddings for semantic neighbor scoring | Gracefully skip; graph neighbors are lexical-ranked | ✅ Handled |
| No graph edges for semantic hits | Vault needs `MemoryPinned`/`MemorySynthesized` events | ⚠️ Data gap identified |

---

## Files Touched

| File | Change |
|------|--------|
| `crates/ai-brains-retrieval/src/recall.rs` | Wire graph neighbors into result ranking; add `graph_boost`, `_graph_hop_depth` params; `RecallHit::graph()` constructor |
| `crates/ai-brains-cli/src/commands/graph.rs` | Add `neighbors`, `hierarchy`, `session` subcommands |
| `crates/ai-brains-cli/src/main.rs` | Extend `GraphCommands` enum (Neighbors/Hierarchy/Session); dispatch |
| `crates/ai-brains-cli/src/commands/recall.rs` | Add `--graph-boost`, `--graph-hop-depth` CLI args |
| `crates/ai-brains-cli/src/commands/sync.rs` | Pass graph params to `recall::run()` |
| `crates/ai-brains-graph/src/queries.rs` | Add `get_neighbors()`, `get_session_memories()`, `NeighborHit` struct |
| `crates/ai-brains-graph/src/lib.rs` | Export `NeighborHit`, `GraphSearch` |

---

## Verification Steps

```powershell
# 1. Rebuild graph (works)
ai-brains graph rebuild

# 2. Graph neighbors (works — returns edges where they exist)
# Find a memory with edges via SQL, then:
ai-brains graph neighbors <memory_id_with_edges>
# Example output: {"memory_id":"...","neighbors":[{"direction":"incoming","external_id":"...","label":"CONFLICTS_WITH"}]}

# 3. Graph hierarchy (works — empty because no SYNTHESIZED_FROM edges in data)
ai-brains graph hierarchy <memory_id>

# 4. Graph session (works — empty because no RECALLS/SOURCE_FOR edges in data)
ai-brains graph session <session_id>

# 5. Graph recall (works — code executes, but no graph augmentation because semantic hits are turn nodes with no edges)
ai-brains recall "test" --limit 5 --graph-boost 0.1

# 6. Performance (needs real graph hits to measure)
Measure-Command { ai-brains recall "test" --limit 5 }
```

## Next Steps to Complete T66

1. **Close T66 as "Code Complete, Pending Data"** — All 3 phases implemented and compiling
2. **New Track T67: Memory Pinning Events** — Add `MemoryPinned` event emission to the nightly pipeline so `RECALLS`/`SOURCE_FOR` edges get created
3. **New Track T68: Memory Synthesis Events** — Add `MemorySynthesized` event emission so `SYNTHESIZED_FROM` edges get created

Once T67+T68 are done, re-run `ai-brains graph rebuild` and T66 graph queries will return full results.
