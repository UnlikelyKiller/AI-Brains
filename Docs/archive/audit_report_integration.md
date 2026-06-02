# AI-Brains Integration Audit Report
**Date:** 2026-05-19
**Repo:** /mnt/c/dev/AI-Brains
**Audited Against:** integration.md spec (Phases T37–T40 + cross-cutting concerns)

---

## Phase 1 — T37 (Transaction Linking & Project Discovery)

| Item | Status | Evidence |
|---|---|---|
| `--tx-id` in `ai-brains context` and `pin` | **IMPLEMENTED** | `crates/ai-brains-cli/src/commands/context.rs:88` writes `CHANGEGUARD_TX_ID` to `.env`; `crates/ai-brains-cli/src/commands/pin.rs:29` parses `tx_id` and falls back to env var |
| Contextual Auto-Discovery (detect `project_id` without explicit flags) | **IMPLEMENTED** | `crates/ai-brains-path/src/discovery.rs` walks up tree to `.changeguard` and extracts `project_id`; `crates/ai-brains-cli/src/commands/context.rs:38-40` invokes it |
| Privacy Guard (privacy inheritance for bridge records) | **IMPLEMENTED** | `crates/ai-brains-core/src/privacy.rs` defines `Privacy::combine()` which returns the strictest level; tested in `crates/ai-brains-core/tests/privacy_strictest_wins.rs` |

---

## Phase 2 — T38 (Structured Bridge / NDJSON Fallback)

| Item | Status | Evidence |
|---|---|---|
| Define `BridgeRecord` JSON schema + `schema.json` | **PARTIAL** | `crates/ai-brains-contracts/src/bridge.rs` defines `BridgeRecord` struct with all spec fields (bridge_version, direction, timestamp, parent_hash, project_id, session_id, tx_id, record_kind, payload, privacy). **No standalone `schema.json` file exists for BridgeRecord.** The only `schema.json` found is `.changeguard/state/schema.json` (ChangeGuard’s own public-interface schema). |
| `sync pull --from-file` to ingest NDJSON | **IMPLEMENTED** | `crates/ai-brains-cli/src/commands/sync.rs` implements `run_pull()` which reads NDJSON line-by-line, deserializes `BridgeRecord`, filters `Inbound` records, maps to `IngestRequest`, and appends via `CaptureService`. CLI routing in `main.rs:312-315`. |

---

## Phase 3 — T39 (Real-Time Bridge / IPC)

| Item | Status | Evidence |
|---|---|---|
| Named-pipe / Local-HTTP handoff between `ai-brainsd` and ChangeGuard | **IMPLEMENTED** | `crates/ai-brainsd/src/main.rs:39` uses `r"\\.\pipe\aibrains-sync"`. Daemon listens via `tokio::net::windows::named_pipe::ServerOptions`, spawns per-client tasks, and handles `DaemonRequest::Sync(record)` by writing to spool and processing via `process_sync()` in `lib.rs:210`. |
| Trigger `sync pull` automatically on `ai-brains context` | **MISSING** | `crates/ai-brains-cli/src/commands/context.rs` initializes `.env` and project/session IDs but does **not** invoke any sync operation or spawn a daemon request. No auto-trigger logic exists. |

---

## Phase 4 — T40 (Unified Retrieval & Feedback Loop)

| Item | Status | Evidence |
|---|---|---|
| ChangeGuard `verify` outcomes pushed to AI-Brains | **PARTIAL** | `crates/ai-brains-cli/src/commands/safety.rs` runs `changeguard hotspots` (JSON and text fallback) and pins results as a memory. It does **not** ingest `changeguard verify` outcomes or ledger entries. No `verify` subcommand or outcome parser exists. |
| AI-Brains nightly sweep computes prediction accuracy | **IMPLEMENTED** | `crates/ai-brains-brain/src/feedback_loop.rs:21-71` defines `FeedbackLoopService::run_accuracy_check()`. It compares recent hotspot paths against predicted paths extracted from summary memories, records `FeedbackMetric` events for matches, and returns match count. Called from `crates/ai-brains-brain/src/lib.rs:121-124` inside `NightlyService::run_nightly()`. |
| Blended results in `ai-brains preflight` and `changeguard ask` | **MISSING** | `crates/ai-brains-retrieval/src/preflight.rs` queries SQLite projections only (memory_projection, session_projection). There is **no** CozoDB query integration, no ChangeGuard data blending, and no `changeguard ask` command or IPC handler that returns blended results. |

---

## Cross-Cutting Checks

| Item | Status | Evidence |
|---|---|---|
| `sync push` command with `--with-impact` and `--with-verify` flags | **MISSING** | No `SyncCommands::Push` variant exists in CLI. `crates/ai-brains-cli/src/main.rs` only defines `SyncCommands::Pull { from_file }`. |
| `sync pull` command with `--hotspots` and `--ledger` flags | **MISSING** | `sync pull` only accepts `--from-file`. Hotspot ingestion lives under the separate `SafetyCommands::Sync { limit, dry_run }` subcommand, not under `sync pull`. No `--ledger` flag anywhere. |
| `sync query` command that queries CozoDB + Vault with blended results | **MISSING** | No `SyncCommands::Query` variant. No CozoDB dependency or query code exists anywhere in the AI-Brains crates. |
| `BridgeRecord` struct/type in codebase | **IMPLEMENTED** | `crates/ai-brains-contracts/src/bridge.rs:6-17` defines the struct. Exported via `crates/ai-brains-contracts/src/lib.rs:2`. Unit-tested in `crates/ai-brains-contracts/tests/bridge_record_shape.rs`. |
| Named pipe path `\\.\pipe\aibrains-sync` | **IMPLEMENTED** | `crates/ai-brainsd/src/main.rs:39` literal string. Also referenced in conductor docs. |
| Privacy flags: `LocalOnly`, `Sealed`, `ProjectLocal` | **PARTIAL** | `crates/ai-brains-core/src/privacy.rs` defines `CloudOk`, `LocalOnly`, `NeverInject`, `Sealed`. **`ProjectLocal` is NOT present** in the enum. |
| No `unwrap` / `expect` / `panic!` in production code (use `thiserror` / `anyhow`) | **IMPLEMENTED** | Ripgrep across all `crates/*/src/**/*.rs` returned **0** hits for `unwrap()`, `expect(`, or `panic!`. Test files (`tests/`, `src/**/tests/`) contain them, which is acceptable per TDD mandate. All production crates declare `thiserror` in `Cargo.toml` and use custom error enums (e.g., `PathError`, `EventError`, `RetrievalError`, `StoreError`). |
| Windows path handling (UNC, WSL, drive-case) | **IMPLEMENTED** | `crates/ai-brains-path/src/canonical.rs` orchestrates normalization. `windows.rs` strips extended-length prefixes and normalizes drive-case. `wsl.rs` converts `/mnt/c/...` to `C:\...`. `unc.rs` handles UNC paths. Used by `ai-brains-cli` context command for deterministic project IDs. |

---

## Migration & Schema Evidence

- **Migration 0015** (`memory_project_id.sql`): Adds `project_id` to `memory_projection`.
- **Migration 0016** (`provenance_tx_id.sql`): Adds `tx_id` columns to `project_projection`, `session_projection`, `turn_projection`, `memory_projection`, and creates indexes (`idx_project_tx`, `idx_session_tx`, `idx_turn_tx`, `idx_memory_tx`).
- **Event payloads**: `crates/ai-brains-events/src/payload.rs` includes `tx_id: Option<TransactionId>` on `ProjectRegisteredPayload`, `SessionStartedPayload`, `UserPromptRecordedPayload`, `AssistantFinalRecordedPayload`, and `MemoryPinnedPayload`.

---

## Workspace Cargo.toml Summary

- **Members:** 17 crates as listed in spec (ai-brains-core, events, store, ai-brainsd, cli, capture, models, retrieval, graph, brain, scheduler, plus contracts, crypto, path, git, security, adapters, daemon-api).
- **Dependencies:** `anyhow`, `thiserror`, `serde`, `tokio`, `sqlx`, `rusqlite`, `chrono`, `uuid` all present in workspace root `Cargo.toml`.

---

## Summary Counts

| Category | Count |
|---|---|
| IMPLEMENTED | 11 |
| PARTIAL | 3 |
| MISSING | 7 |

**Key Gaps:**
1. No standalone `BridgeRecord` `schema.json`.
2. No auto-trigger of sync on `ai-brains context`.
3. No `changeguard verify` outcome ingestion (only hotspots).
4. No CozoDB integration or blended retrieval.
5. Missing `sync push`, `sync query` CLI commands.
6. Missing `--with-impact`, `--with-verify`, `--hotspots`, `--ledger` flags.
7. Missing `ProjectLocal` privacy variant.
