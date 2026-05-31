# Audit 2: Conductor Tracks T55-T70

Date: 2026-05-31
Scope: `conductor/tracks/trackT55` through `trackT70-changeguard-symbol-bridge`, current workspace implementation, conductor registry, and local verification.

## Executive Summary

T55-T65 are mostly implemented or at least have direct code evidence, with caveats around missing verification artifacts and some incomplete status reporting. T66-T70 are not ready to be marked fully implemented. The largest blocker is that the default CLI build is broken because graph-only code is compiled without the `graph` feature. Several later tracks also rely on unverified assumptions or intentionally non-fatal fallbacks that can silently report success while doing no useful work.

Overall verdict: do not close T66-T70 as complete. Do not treat the current tree as meeting conductor standards until the default build compiles, `cargo fmt --check` passes, and the graph/symbol flows have end-to-end tests proving non-empty behavior.

## Verification Performed

- `changeguard ledger status`: no pending transactions and no unaudited drift.
- `changeguard scan --impact`: high risk workspace; many track and implementation files are dirty.
- `changeguard index`: symbol/call graph extraction completed, but text indexing failed with an index-writer/channel error.
- `cargo fmt --check`: failed. Many files report newline-style issues; several new/changed files also need rustfmt formatting.
- `cargo check --workspace --all-targets`: failed. Default non-graph build cannot compile `ai-brains-cli`.
- `cargo check --features graph -p ai-brains-cli`: passed.
- `cargo test -p ai-brains-path`: passed.

## Blocking Findings

### F1 - Default CLI build is broken

Severity: Critical

`crates/ai-brains-cli/src/commands/mod.rs:7` unconditionally exports `pub mod graph;`, while `crates/ai-brains-cli/src/commands/graph.rs:2` imports optional crate `ai_brains_graph`. `ai-brains-graph` is optional in `crates/ai-brains-cli/Cargo.toml:22`, enabled only by the `graph` feature at line 49. This breaks the default build:

`cargo check --workspace --all-targets` fails with `E0432: unresolved import ai_brains_graph`.

Impact: the project no longer meets the CI gate or capture-first requirement for the default binary. T66/T69 cannot be considered properly integrated until graph modules are feature-gated correctly.

### F2 - `cargo fmt --check` fails

Severity: Critical

The formatting gate fails across the workspace. The output includes widespread newline-style failures and rustfmt diffs in new/changed files such as:

- `crates/ai-brains-brain/src/embeddings.rs`
- `crates/ai-brains-brain/src/lib.rs`
- `crates/ai-brains-cli/src/commands/graph.rs`
- `crates/ai-brains-cli/src/commands/symbol_bridge.rs`
- `crates/ai-brains-store/src/lib.rs`
- `crates/ai-brains-store/src/query_store.rs`

Impact: conductor standards and the stated CI gate are not met.

### F3 - T70 uses an unsupported ChangeGuard command

Severity: High

T70 requires `changeguard index --auto-index`. The local ChangeGuard binary rejects that flag:

`error: unexpected argument '--auto-index' found`

The implementation calls the same invalid command in `crates/ai-brains-cli/src/commands/symbol_bridge.rs:94-96`. Because the code treats failure as non-fatal, nightly can print that symbol ingestion failed and continue, but AC1 is not met.

Impact: the symbol bridge refresh path is known-broken in this environment.

### F4 - T70 symbol ingestion has no proven recall path and can duplicate events

Severity: High

`symbol_bridge.rs` emits `MemoryPinned` events for up to 500 symbols at lines 43-85, but it does not check existing events before appending. A stable UUID is computed at lines 51-55, but the event log is append-only, so rerunning nightly appends another event with the same aggregate id rather than upserting.

It also does not ingest HTTP route fields required by AC4, does not scope the ChangeGuard query by project root, and does not prove symbols are searchable by `ai-brains recall`. The query at lines 130-132 only fetches `file_path`, `qualified_name`, `symbol_name`, `symbol_kind`, and `line_start`.

Impact: T70 fails AC4, AC5, and AC6, and AC3 is unverified.

### F5 - T67 graph edges require `session_id`, but recall often emits none

Severity: High

`recall.rs` emits `MemoryPinned` events at lines 49-67, but the payload uses the optional CLI `session_id` argument directly at line 61. If `AI_BRAINS_SESSION_ID` is not set or the user does not pass `--session-id`, the event has no session.

The graph projector only creates a `RECALLS` edge when `session_id` is present, as shown in `crates/ai-brains-graph/src/projector.rs:106-114`.

Impact: T67 AC3/AC4 are only conditionally met. The default recall path can append `MemoryPinned` events that create memory nodes but no session-to-memory graph edge, so `graph session <id>` can still be empty.

### F6 - T68 synthesis events may contain empty or misleading sources

Severity: High

Session summary follow-up emits `MemorySynthesized` at `crates/ai-brains-brain/src/lib.rs:468-488`, but it gathers source ids through `get_session_memory_ids(session_id_str)` at lines 469-472. That query reads `memory_projection WHERE session_id = ?`, not the turns that actually produced the summary. For many sessions this can be empty, which creates synthesized memories without useful `SYNTHESIZED_FROM` edges.

The older hierarchical synthesizer does emit `MemorySynthesized` with source memory ids in `crates/ai-brains-brain/src/memory_synthesis.rs:69-88`, but the T68 session-summary acceptance criteria are not fully proven.

Impact: T68 AC2, AC4, and AC5 remain unverified and likely incomplete for session summaries.

### F7 - T57 omits required nightly error reporting

Severity: Medium

T57 requires `last_nightly_errors` to be written and shown. `nightly.rs` status output reads only `last_nightly_run` and `last_nightly_count` at lines 16-31. `run_nightly` stores `last_nightly_run` and `last_nightly_count`, but there is no implementation of `last_nightly_errors`.

Impact: T57 is partially implemented, not complete against its own spec.

### F8 - T69 `graph update` hides database errors

Severity: Medium

`graph update` uses `unwrap_or(0)` for node and edge count queries at `crates/ai-brains-cli/src/commands/graph.rs:65-75`. If the graph tables are missing or the query fails, it reports zero counts and `status: "live"` instead of surfacing the error.

Impact: T69 AC6 can produce a false healthy report.

### F9 - Production-code `unwrap_or_else` in T55 violates the strict no-unwrap mandate

Severity: Medium

`crates/ai-brains-adapters/src/antigravity.rs:488-490` uses `Uuid::parse_str(...).unwrap_or_else(...)` in production import code. The project rule prohibits `unwrap()`/`expect()`/`panic()` in production code. While `unwrap_or_else` is not a direct panic, it is still an unwrap-family API and silently replaces malformed persisted session ids with random ids.

Impact: import idempotency and provenance can break for malformed session identifiers, and the code does not meet the strict local mandate.

## Track-by-Track Assessment

### T55 - Nightly Performance / Incremental Scan

Status: Mostly implemented, needs hardening and tests.

Evidence: `antigravity.rs:448-467` reads file metadata, builds `source_meta:<session_id>`, compares stored metadata, and skips unchanged files. It updates metadata after empty/no-new-turn/success paths at lines 500-580.

Gaps: no focused test proves unchanged files are skipped or that the no-op nightly target is under 5 seconds. Malformed non-UUID session ids are replaced with random UUIDs at lines 488-490, which can defeat stable incremental behavior.

### T56 - CLI Output Polish / Smart Preflight

Status: Implemented with caveats.

Evidence: `preflight.rs:47-69` uses `is_terminal()` to default TTY output to human mode and piped output to JSON. `preflight.rs:42-44` handles `--summary`, and lines 73-82 begin the summary calculation.

Gaps: summary counts are heuristic text-marker counts, not structured counts of pinned memories/hotspots/active sessions. Manual TTY/pipe verification was not run during this audit.

### T57 - Nightly Status

Status: Partial.

Evidence: `nightly.rs:16-32` implements a read-only status branch for last run, unsummarized count, and last count.

Gaps: missing `last_nightly_errors` write/read/display. No test proves `--status` avoids model calls or event writes.

### T58 - Unix Absolute Path Normalization

Status: Implemented and locally verified.

Evidence: `canonical.rs:41-42` accepts Unix absolute paths. `cargo test -p ai-brains-path` passed.

Gaps: none found in this audit.

### T59 - Preexisting Test Fixes

Status: Partially verified.

Evidence: the path test suite passed, covering the path-related pieces. CLI smoke fixes were not verified because the default CLI build currently fails.

Gaps: cannot confirm CLI smoke tests until F1 is fixed.

### T60 - MinGW-w64 Toolchain

Status: Environment partially satisfies the spirit, but spec details are not met.

Evidence: `x86_64-w64-mingw32-gcc.exe` exists on PATH through WinGet. `~/.cargo/config.toml` does not exist. `rustup target list --installed` did not show `x86_64-pc-windows-gnu` in this PowerShell environment.

Gaps: R2/R3/R4/R5 were not verified. This is an environment track, not a repo implementation track.

### T61 - Nightly Synthesis Batch Limit

Status: Implemented.

Evidence: `memory_synthesis.rs:37-41` reads `AI_BRAINS_SYNTHESIS_BATCH`, defaults to 50, and passes the limit into `get_memories_by_level`.

Gaps: no local runtime timing verification was performed.

### T62 - Semantic Search / Stored Embeddings

Status: Partially implemented.

Evidence: migration files `0018_memory_embedding.sql` and `0019_embedding_timestamp.sql` exist; `EmbeddingService` stores f32 vectors as little-endian bytes; semantic search code exists in `ai-brains-retrieval`.

Gaps: no successful local semantic recall was run in this audit. The claimed one-time script from the spec is not present under the exact named path, and T62's verification is mostly documented rather than test-backed.

### T63 - Nightly Embedding Integration

Status: Implemented with external dependency caveats.

Evidence: `NightlyService::run_nightly` runs `EmbeddingService::backfill_recent(50, Some(7))` and records counts. This depends on the embedding provider configured by `AI_BRAINS_EMBEDDING_URL` / `AI_BRAINS_EMBEDDING_MODEL`.

Gaps: no test uses a mock embedding provider to prove nightly stores embeddings. Runtime success depends on the local embedding server.

### T64 - Stale Refresh + WAL Checkpointing

Status: Mostly implemented.

Evidence: `get_stale_memories`, timestamped `store_embedding`, `refresh_stale`, and `ctx.conn.wal_checkpoint()` in `nightly.rs:166-169`.

Gaps: `wal_checkpoint()` result is not checked in `nightly.rs`, so checkpoint failure can be silently reported as success.

### T65 - Repo Alias Resolution

Status: Implemented, with product limitations.

Evidence: project commands are wired in `main.rs`, and `project.rs` implements list/resolve/detect flows. `QueryStore::list_projects` exists.

Gaps: alias population is manual/content-derived and not automatically maintained. No local CLI verification was possible because the default CLI build fails.

### T66 - Graph-Augmented Recall + Graph Query CLI

Status: Code exists, but default build is broken and behavior is not fully proven.

Evidence: recall accepts graph parameters and graph expansion code exists at `recall.rs:132-180`; graph CLI handlers exist in `commands/graph.rs`.

Gaps: default build fails due to feature gating. Non-graph builds warn that graph arguments are unused. Performance target was not measured. Real graph-augmented non-empty recall was not verified.

### T67 - Memory Pinning Events

Status: Partial.

Evidence: recall emits `MemoryPinned` events in `recall.rs:43-72`; projector creates a `RECALLS` edge if `session_id` exists.

Gaps: no `session_id` fallback means default recall may not create graph edges. No test proves event emission, graph projection, and `graph session` behavior end to end.

### T68 - Memory Synthesis Events

Status: Partial.

Evidence: hierarchical synthesis emits `MemorySynthesized` from real source memory ids; session summaries also emit a `MemorySynthesized` event.

Gaps: session summary source ids are inferred from `memory_projection.session_id`, likely empty for many sessions. No backfill for existing summaries. No test proves non-empty `SYNTHESIZED_FROM` edges after nightly.

### T69 - Live Graph Hook

Status: Partial and feature-gated build only.

Evidence: `live_graph.rs` wraps `SqliteEventStore`; recall/nightly use `GraphAwareEventStore` under `feature = "graph"`; capture sink paths include a graph hook.

Gaps: default build is broken by graph module export. `graph update` can mask missing tables as zero counts. No test proves immediate graph visibility after recall/nightly/capture append.

### T70 - ChangeGuard Symbol Bridge

Status: Not complete.

Evidence: `symbol_bridge.rs` exists and nightly invokes it at `nightly.rs:188-193`.

Gaps: invalid ChangeGuard flag, no project-root scoping, no route ingestion, no duplicate prevention, no proven recall output, no tests, and conductor still marks T70 pending.

## Placeholder / TODO / Stub Review

No `TODO`, `todo!`, or `unimplemented!` markers were found in the T55-T70 implementation surface. However, there are functional placeholders:

- T70's route query is described in the spec but not implemented.
- T70's project scoping is described but not implemented.
- T69's `graph update` reports `"status": "live"` without validating graph freshness or last-event timestamp.
- T66's hop depth is documented as reserved and only depth 1 is implemented.

## Repository Hygiene

The working tree contains dirty project files and large/generated local artifacts reported by ChangeGuard, including logs and vault backups. This conflicts with the "No Repository Pollution" mandate if those artifacts are not intentionally ignored or externalized.

Notable examples from scan output: `vault.db`, `vault.db.backup`, `vault.db.bak`, `t70-output.log`, `t66-output.log`, `fixes-output.log`, `ingest.json`, and ad hoc batch/PowerShell scripts.

## Recommended Remediation Order

1. Fix default feature gating: gate `commands::graph` and `mod live_graph` usage so default `cargo check --workspace --all-targets` passes.
2. Run rustfmt or normalize line endings so `cargo fmt --check` passes.
3. Replace T70 `changeguard index --auto-index` with the supported local command, or version-detect before using the flag.
4. Add end-to-end tests for T67/T68/T69 graph behavior with a temporary vault.
5. Add T70 tests with a mocked ChangeGuard bridge export and duplicate-run assertions.
6. Complete T57 error-state sync and status display.
7. Re-run the full CI gate: `cargo fmt --check ; cargo clippy --workspace --all-targets -- -D warnings ; cargo nextest run --workspace ; cargo deny check ; cargo audit`.

## Remediation Pass - 2026-05-31

Resolved in code:

- F1: default workspace build now compiles with graph-only CLI code gated behind the `graph` feature.
- F2: `cargo fmt --check` passes after rustfmt normalization.
- F3: T70 no longer calls unsupported `changeguard index --auto-index`; it uses the locally supported `changeguard index`.
- F4: T70 symbol ingestion now skips already-ingested symbol aggregate IDs, adds best-effort project-root filtering, reads ChangeGuard's local indexed SQLite state directly, joins HTTP route fields from `api_routes`, and has tests proving route content, duplicate prevention, and recallability.
- F5: recall emits graph provenance with an effective session ID even when the caller did not provide one, and the graph projector creates a session node for `RECALLS` edges.
- F6: session summary synthesis now gathers source memory IDs before appending the summary itself, avoiding self-sourcing and preserving source edges.
- F7: nightly status records and displays `last_nightly_errors`.
- F8: `graph update` now surfaces graph table/count failures instead of reporting zero counts as healthy.
- F9: Antigravity import now maps malformed session IDs to deterministic UUIDv5 values instead of generating random IDs through an unwrap-family fallback.

Verification:

- `cargo fmt --check`: passed.
- `cargo check --workspace --all-targets`: passed.
- `cargo check --features graph -p ai-brains-cli`: passed.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `cargo clippy --features graph -p ai-brains-cli --all-targets -- -D warnings`: passed.
- `cargo test --workspace`: passed.
- `changeguard verify`: passed its configured formatting, clippy, and test plan.
- `cargo audit`: passed with no reported vulnerabilities.
- `cargo build --target x86_64-pc-windows-gnu -p ai-brains-cli`: passed after installing the target and writing the user Cargo linker config.
- `changeguard verify`: passed its configured plan again after the T70 direct-index and T69 test additions.
- `changeguard ledger atomic ai-brains:T55-T70-audit-remediation --category CHORE ...`: recorded the remediation provenance; `changeguard ledger status` reports no pending transactions and no unaudited drift.
- `changeguard scan --impact`: completed and wrote `.changeguard/reports/latest-impact.json`; the workspace remains high risk because the change set is broad and includes pre-existing/ignored conductor and script additions.
- Focused T67/T69/T70 coverage added:
  - `cargo test -p ai-brains-graph test_projector_links_pinned_recall_memory_to_session -- --nocapture`: passed.
  - `cargo test --features graph -p ai-brains-cli graph_aware_store_makes_recall_edge_visible_on_append -- --nocapture`: passed.
  - `cargo test -p ai-brains-cli symbol_bridge -- --nocapture`: passed.

Environment/tooling gaps:

- `cargo nextest run --workspace` could not run because `cargo-nextest` is not installed.
- `cargo deny check` could not run because Windows Application Control blocked `cargo-deny.exe`.
- Drafted T71 (`conductor/tracks/trackT71-ci-tooling-reproducibility/spec.md`) to close those local CI tooling reproducibility blockers.
