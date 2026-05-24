# Track T49: Differential agy Ingestion (Delta Sync)

## 1. Objective
Reduce ingestion overhead by evaluating vault state and only appending new conversation turns. This converts the `agy-hook` and `antigravity-import` pathways from full-sync to delta-sync.

## 2. Rationale
Running continuous capture hooks on every Antigravity update can result in duplicate processing overhead if the system resubmits entire transcripts. To maintain responsiveness, especially during active chat sessions, AI-Brains should ignore turns that have already been persisted to the event log.

## 3. Architecture & Requirements

### 3.1 Status-Aware Ingestion Pre-Check
*   Before calling `ingest_request` (or creating an event for a turn), the ingestion pipeline must evaluate the `session_id` and the proposed `turn_index` (or chronological position).
*   Only turns that exceed the current `max_turn_index` known to the vault for that specific session should be dispatched.

### 3.2 Query Store Extension
*   Extend `ai-brains-store`'s `QueryStore` interface with a fast, read-only method: `get_max_turn_index(session_id: &Uuid) -> Result<Option<i32>>`.
*   Implementation will query the relevant projection table (e.g., `turn_projection` or `session_projection`) directly.
*   **Performance Constraint**: This check must run directly against the local SQLite database. It must be highly optimized and indexed (ensure an index exists on `session_id` and `turn_index`).

### 3.3 Skip Logic in Hooks
*   Update `agy-hook` and `antigravity-import`.
*   For each turn in an Antigravity transcript, compare its index against `max_turn_index`.
*   If `turn_index <= max_turn_index`, gracefully skip (do not generate an event, do not emit noisy warnings to the CLI).
*   Only call `ingest_request` for `turn_index > max_turn_index`.

## 4. Engineering Mandates Adherence
*   **Capture Independence**: This delta-check uses pure local SQL over projections; it requires no models or external dependencies.
*   **Rust Safety**: Graceful error handling using `Result`. No panics if the session does not exist.
*   **Performance**: Fast local lookup guarantees hook responsiveness, respecting the CLI speed mandate.
