## Plan: Differential agy Ingestion (Delta Sync)

### Phase 1: Query Store Extension
- [ ] Task 1.1: Verify or create SQLite indices on `session_id` and `turn_index` in the relevant projection tables (`turn_projection` / `session_projection`).
- [ ] Task 1.2: Add the `get_max_turn_index(session_id: &Uuid) -> Result<Option<i32>>` signature to the `QueryStore` trait in `ai-brains-store`.
- [ ] Task 1.3: Implement `get_max_turn_index` for the SQLite backend and add corresponding unit tests to verify the max index is retrieved correctly.

### Phase 2: Core Ingestion Logic
- [ ] Task 2.1: Implement the delta filter in the parsing layer of `antigravity-import` and `agy-hook`.
- [ ] Task 2.2: During transcript parsing, query `get_max_turn_index` for the session and skip processing for any turn where the index is `<= max_turn_index`.

### Phase 3: Integration and Testing
- [ ] Task 3.1: Write a test where an Antigravity log is imported twice. Verify that the second import skips all turns and generates 0 new events.
- [ ] Task 3.2: Append a new turn to the log in the test and verify that only the single new turn generates an event.
- [ ] Task 3.3: Verify CLI output remains clean (no excessive warnings) when skipping existing turns.
