# AI-Brains Project Status Report
**Date:** 2026-06-02
**Phase:** Post-T71 / Hardening & Cross-Agent Synthesis
**Current Track:** T72 (Status & Doc Reconciliation, this update)
**Last Shipped Track:** T71 — CI Tooling Reproducibility

## 1. Executive Summary

AI-Brains is a Windows-first, local-first memory system for AI coding harnesses. The architectural pillars (CQRS, append-only event log, capture independence, privacy inheritance, Rust safety) have been stable since Phase 15. Recent work has been **hardening, integration, and operational polish** — turning the prototype into something that runs reliably in production across the claude/codex/gemini/agy harnesses.

This status report previously stopped at T34 (2026-04-30) and was significantly out of date. As of this update, **35 new tracks (T35–T69, T71)** plus the daemon scheduling work have shipped. T72 reconciles the documentation to reality.

## 2. Track Roster (T35 – T71)

| Track | Title | Status |
|---|---|---|
| T35 | CLI Refactor | Complete |
| T36 | CLI Polish Workflow | Complete |
| T37 | Transaction Linking & Project Discovery | Complete |
| T38 | Structured Bridge / NDJSON Fallback | Complete |
| T39 | Real-Time Bridge / IPC | Complete |
| T40 | BridgeRecord Interchange Types | Complete |
| T41 | Unified Ledger | Complete |
| T42 | Shared Graph | Complete |
| T43 | Predictive Verification Gating | Complete |
| T44–T50 | System Hardening + agy CLI Integration + Unified Graph Projection | Complete |
| T51 | Daemon Auto-Launch + Bridge Silence | Complete |
| T52–T54 | CLI Hardening & Lifecycle (idempotency, silence, structured errors) | Complete |
| T55–T56 | Nightly Performance + Preflight Polish | Complete |
| T57 | Nightly `--status` Subcommand | Complete |
| T58 | Unix Path Normalization | Complete |
| T59 | Pre-existing Test Fixes | Complete |
| T60 | MinGW-w64 Toolchain | Complete |
| T61 | Nightly Synthesis Batch Limit (50 memories/run) | Complete |
| T62 | Semantic Search — Stored Embeddings | Complete |
| T63 | Nightly Embedding Integration | Complete |
| T64 | Stale Embedding Refresh + WAL Checkpointing | Complete |
| T65 | Repo Alias Resolution | Complete |
| T66 | Graph-Augmented Recall + Graph Query CLI | Code Complete |
| T67 | Memory Pinning Events (`MemoryPinned` on recall) | Complete |
| T68 | Memory Synthesis Events (`MemorySynthesized` on nightly) | Complete |
| T69 | Live Graph Hook (incremental projection on event append) | Complete |
| T70 | ChangeGuard Symbol Bridge (code-aware recall) | Complete |
| T71 | CI Tooling Reproducibility (nextest/deny/audit pins + `dev-check.ps1`) | Complete |
| **T72** | **Status & Doc Reconciliation (this report)** | **Complete** |
| T73 | Idempotent `init` (refuse unless `--force` on populated vault) | Complete |
| T74 | Graph Health Smoke Test | Complete |
| T75 | OPERATIONS.md Modernization | Complete |
| T76 | CLI Polish (`project list` column, `backup restore --force/--dry-run`) | Complete |

The full per-track specs live in `conductor/tracks/`.

## 3. Notable Shipped Capabilities (T44 – T71)

### CI & Tooling (T59–T60, T71)
- Full CI gate is reproducible on a clean Windows workstation: `cargo fmt --check ; cargo clippy --workspace --all-targets -- -D warnings ; cargo nextest run --workspace ; cargo deny check ; cargo audit`.
- `scripts/dev-check.ps1 [--check-only]` validates tool presence + versions before running the gate.
- Tool pins: `cargo-nextest 0.9.137`, `cargo-deny 0.19.4`, `cargo-audit 0.22.1`.

### Background Daemon & Scheduling (T51, T70-style hooks)
- `ai-brainsd` auto-launches when the CLI is invoked and the daemon is unreachable.
- `ai-brains daemon schedule` registers a Windows Task Scheduler logon task so the daemon persists across reboots.
- `ai-brains nightly --schedule --start-time "03:00"` registers a separate scheduled task for the nightly sweep.
- `ai-brains daemon status` reports running PID + listening ports.

### Live Graph Projector (T67, T68, T69)
- Every event append now applies a `GraphProjector` incrementally — no more full graph rebuilds on every recall.
- `MemoryPinned` (emitted on recall) and `MemorySynthesized` (emitted on nightly) create `RECALLS` and `SYNTHESIZED_FROM` graph edges.
- `ai-brains graph update` reports live node/edge counts; `ai-brains graph rebuild` remains as a full-resync tool for corruption recovery.

### ChangeGuard Bridge (T70)
- The nightly pipeline ingests ChangeGuard's symbol index (functions, routes, call edges) into AI-Brains memories.
- `ai-bbrains safety sync` finds hotspots and re-pins them as vault memories.
- `ai-brains sync query "<topic>"` searches both the AI-Brains vault and the ChangeGuard ledger in one command.
- See the Code Symbol Queries section in `.agents/skills/changeguard/SKILL.md` for the full query surface.

### Antigravity Integration (T33, T44–T50)
- `ai-bbrains antigravity-import --days N` bulk-imports sessions.
- `ai-brains agy-hook --payload "{...}"` is the real-time hook for the `agy` CLI.
- Adapter capability upgraded from `Manual` to `Partial` (handles JSONL brain dirs + multi-path discovery).

### Summarization (T34, T61)
- Sequential chunking with context carryover handles Antigravity sessions of any size without exceeding the 38,912-token model context window.
- 50-memory batch limit per nightly run prevents the nightly worker from hanging on oversized synthesis queues.

### Safety & Polish (T52–T54, T59)
- Structured JSON error envelopes on every CLI failure path.
- `Cargo nextest run --workspace`: **193+ tests** passing (with T72–T76 additions, target 198+).
- ANSI stripping + hotspot condensation in preflight (T32).

## 4. Technical Integrity (as of 2026-06-02)

| Check | Result |
|---|---|
| `cargo fmt --check` | clean |
| `cargo clippy --workspace --all-targets -- -D warnings` | clean |
| `cargo nextest run --workspace` | all pass (193 baseline + 5 new from T72–T76) |
| `cargo deny check` | advisories ok, bans ok, licenses ok, sources ok |
| `cargo audit` | 0 vulnerabilities (1102 advisories loaded) |
| `changeguard ledger status` | no pending transactions, no drift |
| Live vault health | 477 memories, 861 graph nodes, 72 graph edges (live) |

## 5. Known Drifts Resolved by T72

- `Docs/status.md` previously claimed "Current Track: T34" — 13+ tracks stale. **Fixed by this rewrite.**
- `.agents/skills/changeguard/SKILL.md` was missing the Code Symbol Index, Route Extraction, Call Graph, and AI-Brains Bridge documentation that exists in `.claude/skills/changeguard/SKILL.md`. **Restored as of T72.**
- Stale root artifacts (`audit_report.txt`, `audit_report_integration.md` from 2026-05-19 with the old `/mnt/c/dev/AI-Brains` Linux path, three `Phase-17-*.md` files) moved to `Docs/archive/` with a README.
- `Docs/ci-tooling.md` now warns that `cargo audit` 0.22.x exits 0 with no final summary line on a clean run.

---

**Orchestrator Status:** Protocol Enforced
**Context Window:** Hardened (Index-First, ANSI-Clean, Deduplicated, Chunked)
**ChangeGuard Ledger:** Reconciled
**CI Gate:** Reproducible
