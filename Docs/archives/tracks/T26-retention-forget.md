# Track T26: Retention and Forget

## Context
Phase 11 includes data lifecycle management. We need to support user-initiated "forgetting" of specific memories and automated expiration of old, raw session data.

## Goals
- Support `ai-brains forget --memory <ID>` to mark memories as forgotten.
- Implement background cleanup of raw session turns older than 90 days.
- Ensure "forgotten" memories are excluded from all recall/preflight results.

## Implementation Plan

### Phase 1: Schema Updates
- [x] Add `last_accessed_at` to `turn_projection`.
- [x] Add `MemoryForgotten` event to `ai-brains-events`.

### Phase 2: Retention Service
- [x] Implement `RetentionService` in `ai-brains-brain`.
- [x] Add logic to delete turns older than 90 days from `turn_projection` (NOT the event log).
- [x] Integrate into `NightlyService`.

### Phase 3: Forget Command
- [x] Add `forget` command to CLI.
- [x] Emit `MemoryForgotten` event.
- [x] Update `MemoryProjection` handler to set status to `forgotten`.
- [x] Update `QueryStore` (FTS) and Graph search to exclude forgotten memories.

### Phase 4: Verification
- [x] Test: `forget` marks memory as forgotten and excludes it from recall.
- [x] Test: Retention service removes old turns from SQL projection.
- [x] Test: Event log remains intact (retention is projection-only).

## Progress
- [x] Phase 1
- [x] Phase 2
- [x] Phase 3
- [x] Phase 4
