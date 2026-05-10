# Track T23: Conflicts and Recipes

## Context
Phase 10: Memory Intelligence. We need to detect when new sessions contradict past knowledge (Conflicts) or when a specific workaround is repeated enough to be considered an "Execution Recipe".

## Goals
- Detect semantic contradictions between recent session summaries and existing memory.
- Identify "Workarounds" or "Steps" that are repeated and promote them to Recipes.
- Emit events for both, allowing them to be projected into SQL and Graph.

## Implementation Plan

### Phase 1: Event Definitions
- [x] Define `ConflictDetectedPayload` in `ai-brains-events`.
- [x] Define `RecipePromotedPayload` in `ai-brains-events`.
- [x] Add both to `EventKind`.

### Phase 2: Intelligence Services
- [x] Implement `ConflictDetectionService` in `ai-brains-brain`.
- [x] Implement `RecipePromotionService` in `ai-brains-brain`.
- [x] Use `ModelProvider` to analyze recent summaries against related memories (retrieved via Graph/FTS).

### Phase 3: Projections
- [x] Update SQL migrations to include `conflict_projection` and `recipe_projection` (0007).
- [x] Update `GraphProjector` to handle these events.

### Phase 4: Verification
- [x] Test: `conflict_detected_from_contradictory_sessions`
- [x] Test: `recipe_promoted_from_windows_workaround`

## Progress
- [x] Phase 1
- [x] Phase 2
- [x] Phase 3
- [x] Phase 4
