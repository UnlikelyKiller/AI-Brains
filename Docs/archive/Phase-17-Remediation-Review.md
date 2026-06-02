**Findings**

High: [payload.rs](C:/dev/ai-brains/crates/ai-brains-events/src/payload.rs:126) does not use internal tagging. `Payload` is currently serde’s default externally tagged enum, producing JSON like `{"SessionStarted": {...}}`, not internal tagging like `{"type":"SessionStarted", ...}`. There is also no `rename_all = "PascalCase"` on `Payload`; PascalCase only happens implicitly from variant names.

High: [0005_turn_projection.sql](C:/dev/ai-brains/crates/ai-brains-store/migrations/0005_turn_projection.sql:2) and [turn.rs](C:/dev/ai-brains/crates/ai-brains-store/src/projections/turn.rs:40) still do not project `project_id` into `turn_projection`. The turn rows only carry `session_id`, `turn_index`, `role`, `content`, `tx_id`, and `occurred_at`. That fails the “project_id projected in all 4 tables” requirement unless turn projection is intentionally excluded.

Medium: [fts.rs](C:/dev/ai-brains/crates/ai-brains-store/src/fts.rs:25) still ignores the `project_id` filter and returns `NULL AS project_id`. Even where `memory_projection.project_id` exists, FTS recall remains cross-project capable through this path.

Medium: Privacy escalation is only partial. [session.rs](C:/dev/ai-brains/crates/ai-brains-store/src/projections/session.rs:120) escalates session privacy for prompt/final events, and [memory.rs](C:/dev/ai-brains/crates/ai-brains-store/src/projections/memory.rs:142) escalates a session when a pinned memory references it. But [memory.rs](C:/dev/ai-brains/crates/ai-brains-store/src/projections/memory.rs:24) does not update `privacy` on memory upsert, and synthesized memories use only the envelope privacy at [memory.rs](C:/dev/ai-brains/crates/ai-brains-store/src/projections/memory.rs:75) rather than deriving strictest privacy from source memories in the projection.

Low: The named pipe uses `.first_pipe_instance(false)` at [main.rs](C:/dev/ai-brains/crates/ai-brainsd/src/main.rs:45), so that specific requirement is met. The loop is still brittle: `server.connect().await?` at [main.rs](C:/dev/ai-brains/crates/ai-brainsd/src/main.rs:48) can terminate the daemon on accept errors, and the read loop at [main.rs](C:/dev/ai-brains/crates/ai-brainsd/src/main.rs:72) has no frame size limit or invalid-JSON response path.

**Verified OK**

`ai-brainsd` now opens a persistent SQLCipher-backed `VaultConnection`, migrates it, wraps it in `SqliteEventStore`, and passes it into `DaemonWriter` at [main.rs](C:/dev/ai-brains/crates/ai-brainsd/src/main.rs:34).

Raw events remain append-only in the inspected code. Writes go through `INSERT INTO events` at [event_store.rs](C:/dev/ai-brains/crates/ai-brains-store/src/event_store.rs:59), and migration triggers reject raw event updates/deletes at [0001_event_log.sql](C:/dev/ai-brains/crates/ai-brains-store/migrations/0001_event_log.sql:32). The `UPDATE`/`DELETE` hits I found are projection maintenance or replay rebuilds, not raw event mutation.

I did not run the test suite because the session is read-only and `changeguard ledger status` was blocked by the local policy wrapper. Static review found the issues above.