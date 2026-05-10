# Track T24: RAPTOR and CRAG

## Context
Phase 10 focuses on transforming session-level summaries into long-term knowledge. We use a hierarchical clustering approach (inspired by RAPTOR) to group related sessions and synthesize "Knowledge Nodes" that represent cross-session insights.

## Goals
- Group session summaries by semantic similarity.
- Use LLMs to synthesize high-level summaries for clusters.
- Persist synthesized knowledge as events and project them into the graph.
- Support hierarchical retrieval (Session -> Cluster -> Knowledge Node).

## Implementation Plan

### Phase 1: Event and Domain Definition
- [x] Define `MemorySynthesized` event payload in `ai-brains-events`.
- [x] Add `MemorySynthesized` to `EventKind`.
- [x] Add `level` property and `SYNTHESIZED_FROM` relations to `ai-brains-graph` schema.

### Phase 2: Synthesis Engine
- [x] Create `MemorySynthesizer` in `ai-brains-brain`.
- [x] Implement Level-1 clustering (heuristic) and recursive synthesis.
- [x] Implement CRAG factual verification gate.

### Phase 3: Projections
- [x] Update `MemoryProjection` in `ai-brains-store` for hierarchy and levels.
- [x] Update `GraphProjector` to handle `MemorySynthesized` events.

### Phase 4: Verification
- [x] Test: Synthesis event correctly updates both SQL and Graph projections.
- [x] Test: CRAG rejects unsupported synthesis.
- [x] CI Gate: `raptor_synthesis` integration tests pass.

## Progress
- [x] Phase 1
- [x] Phase 2
- [x] Phase 3
- [x] Phase 4
