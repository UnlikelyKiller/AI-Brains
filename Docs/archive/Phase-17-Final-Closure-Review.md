Gate result: **not fully passed**.

Resolved:
- Payload enum is correctly tagged: [payload.rs](C:/dev/ai-brains/crates/ai-brains-events/src/payload.rs:126) has `#[serde(tag = "type", rename_all = "PascalCase")]`.
- `ai-brainsd` uses `Arc<dyn EventStore>` in the writer path: [lib.rs](C:/dev/ai-brains/crates/ai-brainsd/src/lib.rs:16), [lib.rs](C:/dev/ai-brains/crates/ai-brainsd/src/lib.rs:44), and opens the SQLCipher vault through `VaultConnection::open` before wrapping `SqliteEventStore`: [main.rs](C:/dev/ai-brains/crates/ai-brainsd/src/main.rs:34).
- Named pipe creation/connect errors log and `continue`, so they do not exit the daemon loop: [main.rs](C:/dev/ai-brains/crates/ai-brainsd/src/main.rs:50), [main.rs](C:/dev/ai-brains/crates/ai-brainsd/src/main.rs:56).
- `turn_projection` carries `project_id` via migration [0016_provenance_tx_id.sql](C:/dev/ai-brains/crates/ai-brains-store/migrations/0016_provenance_tx_id.sql:4) and populates it from `session_projection`: [turn.rs](C:/dev/ai-brains/crates/ai-brains-store/src/projections/turn.rs:33).
- `memory_projection` carries `project_id` via migration [0015_memory_project_id.sql](C:/dev/ai-brains/crates/ai-brains-store/migrations/0015_memory_project_id.sql:1).
- FTS core search filters by `project_id` when supplied: [fts.rs](C:/dev/ai-brains/crates/ai-brains-store/src/fts.rs:22), [fts.rs](C:/dev/ai-brains/crates/ai-brains-store/src/fts.rs:30).
- No raw event updates/deletes found. The store only inserts into `events`: [event_store.rs](C:/dev/ai-brains/crates/ai-brains-store/src/event_store.rs:59), and migration triggers prevent event update/delete: [0001_event_log.sql](C:/dev/ai-brains/crates/ai-brains-store/migrations/0001_event_log.sql:31), [0001_event_log.sql](C:/dev/ai-brains/crates/ai-brains-store/migrations/0001_event_log.sql:38).

Blocking issue:
- `SessionSummaryCreated` still inserts into `memory_projection` without `project_id`: [memory.rs](C:/dev/ai-brains/crates/ai-brains-store/src/projections/memory.rs:55). That means `memory_projection` is not consistently populated with `project_id`.

Additional concern:
- `search_memory()` still calls `fts.search(query, None)`, which bypasses project scoping: [fts.rs](C:/dev/ai-brains/crates/ai-brains-store/src/fts.rs:64). The scoped API exists, but this public helper remains unscoped.

I could not run `changeguard verify`; the command was blocked by the current read-only execution policy.