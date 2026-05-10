# Conductor Master Track Board

## Project Status
**Status:** In Progress
**Phase:** Phase 16 - Technical Debt & Refactoring [IN PROGRESS]
**Current Track:** T35 - CLI Refactor
**Verification:** Full Windows workspace gate is green as of 2026-05-10. Relational graph verified in workspace; cross-agent synthesis and harness hooks verified.

## Track Registry

| ID | Name | Status | Owner | Link | Description |
|---|---|---|---|---|---|
| T00 | Foundation and Workspace | **Completed** | Orchestrator | [T00](./tracks/T00-foundation.md) | Workspace structure, CI gates, and global tools. |
| T01 | Core Domain | **Completed** | architecture-planner | [T01](./tracks/T01-core-domain.md) | Pure domain model (ids, privacy, session). |
| T02 | Event Contracts | **Completed** | architecture-planner | [T02](./tracks/T02-event-contracts.md) | Immutable event definitions. |
| T03 | JSON Contracts | **Completed** | architecture-planner | [T03](./tracks/T03-json-contracts.md) | API DTOs for CLI/Daemon. |
| T04 | Crypto Recovery | **Completed** | architecture-planner | [T04](./tracks/T04-crypto-recovery.md) | SQLCipher key and recovery kit logic. |
| T05 | Store Event Log | **Completed** | architecture-planner | [T05](./tracks/T05-store-event-log.md) | Append-only event store on SQLCipher. |
| T06 | Store Projections | **Completed** | architecture-planner | [T06](./tracks/T06-store-projections.md) | Read-optimized views derived from events. |
| T07 | Path Normalization | **Completed** | architecture-planner | [T07](./tracks/T07-path-normalization.md) | Canonical Windows/WSL/UNC project identity. |
| T08 | Git Metadata | **Completed** | architecture-planner | [T08](./tracks/T08-git-metadata.md) | Bounded repository identity and diff statistics. |
| T09 | Security Scanning | **Completed** | architecture-planner | [T09](./tracks/T09-security-scanning.md) | Secret detection, privacy escalation, and redaction. |
| T10 | Capture Core | **Completed** | architecture-planner | [T10](./tracks/T10-capture-core.md) | Daemon-free capture command handling. |
| T11 | Daemon Writer | **Completed** | architecture-planner | [T11](./tracks/T11-daemon-writer.md) | Single-writer queue and spool replay. |
| T12 | CLI Foundation | **Completed** | architecture-planner | [T12](./tracks/T12-cli-foundation.md) | JSON-only CLI routing for local capture. |
| T13 | CLI Capture | **Completed** | architecture-planner | [T13](./tracks/T13-cli-capture.md) | CLI integration with capture core. |
| T18 | Retrieval Lexical | **Completed** | architecture-planner | | Lexical recall over the store read side. |
| T19 | Preflight Recall | **Completed** | architecture-planner | | Word-budgeted preflight assembly with privacy filtering. |
| T20 | Graph Projection | **Completed** | architecture-planner | [T20](./tracks/T20-graph-projection.md) | Rebuildable graph projection layer with default deterministic backend and opt-in LadybugDB backend. |
| T21 | Model Providers | **Completed** | architecture-planner | [T21](./tracks/T21-model-providers.md) | Provider-agnostic model abstraction and local providers. |
| T22 | Nightly Summaries | **Completed** | architecture-planner | [T22](./tracks/T22-nightly-summaries.md) | Nightly service with unsummarized session discovery. |
| T23 | Conflicts Recipes | **Completed** | architecture-planner | [T23](./tracks/T23-conflicts-recipes.md) | Contradictory session detection and recipe promotion. |
| T24 | RAPTOR CRAG | **Completed** | architecture-planner | [T24](./tracks/T24-raptor-crag.md) | Hierarchical clustering and synthesis (Memory Synthesis). |
| T25 | Scheduler Backups | **Completed** | architecture-planner | [T25](./tracks/T25-scheduler-backups.md) | Windows task scheduling and automated vault backups. |
| T26 | Retention Forget | **Completed** | architecture-planner | [T26](./tracks/T26-retention-forget.md) | Policy-based turn expiration and soft-delete. |
| T27 | E2E Hardening | **Completed** | architecture-planner | [T27](./tracks/T27-e2e-hardening.md) | Smoke testing and fault isolation. |
| T28 | Cross-Agent Harness | **Completed** | Orchestrator | [T28](./tracks/T28-cross-agent-harness.md) | Cross-agent synthesis and standardized hook implementation. |
| T29 | Relational Graph | **Completed** | Orchestrator | [T29](./tracks/T29-relational-graph.md) | Migrate graph projection to SQLite (ADR-0009). |
| T30 | Hardening Protocol | **Completed** | Orchestrator | [T30](./tracks/T30-hardening-protocol.md) | Optimization, encoding fixes, and agentic protocol overhaul. |
| T31 | Onboarding & Observability | **Completed** | Orchestrator | [T31](./tracks/T31-onboarding-observability.md) | Integrate 4-layer repo understanding (ADR-0010). |
| T32 | Preflight ANSI Cleanup & Dedup | **Completed** | Orchestrator | | Strip ANSI codes, deduplicate safety/index, condense hotspots. |
| T33 | Antigravity Conversation Import | **Completed** | Orchestrator | | Parse Antigravity logs, import turns into nightly, CLI subcommand. |
| T34 | Resilient Summarization Truncation | **Completed** | Orchestrator | | Sequential chunking with context carryover for large sessions. |
| T35 | CLI Refactor | **Completed** | Orchestrator | [T35](./tracks/trackT35/spec.md) | Decomposed main.rs, created AppContext, moved Antigravity import. |

## Current Verification Snapshot
- `cargo fmt`: passes.
- `cargo clippy --workspace --all-targets -- -D warnings`: passes.
- `cargo test --workspace`: passes.
- Relational graph: native SQLite backend; no C++ dependencies.
- `changeguard ledger status`: no pending transactions and no unaudited drift.

## Completed Track: T35 - CLI Refactor
- [x] Decomposed `main.rs` God File into modular command handlers in `src/commands/`.
- [x] Introduced `AppContext` to encapsulate shared vault and connection state.
- [x] Migrated Antigravity import core logic to `ai-brains-adapters`.
- [x] Verified zero behavioral regression via CLI integration tests.
- [x] Pass CI gate (formatting, clippy, test, deny, audit).

## Completed Track: T34 - Resilient Summarization Truncation
- [x] Upgraded `ModelProvider` with `tokenize()` and character-based estimation fallback.
- [x] Implemented `NightlyService` sequential chunking loop with context carryover.
- [x] Enforced turn-aware splitting to protect message boundaries.
- [x] Added `AI_BRAINS_CTX_SIZE` environment control for hardware-specific stability (Intel Arc B580).
- [x] Verified via integration test `nightly_summarizes_large_session`.
- [x] Updated `Docs/Deviations.md` with hardware-stable context strategies.

## Completed Track: T31 - Onboarding & Observability
- [x] Hardened `ai-brains` skill with 4-layer protocol (Structural, Behavioral, Observability, Safety).
- [x] Implemented `ai-brains safety sync` for ChangeGuard hotspot ingestion.
- [x] Budget-aware preflight context generation with ANSI stripping and deduplication.
- [x] ChangeGuard ledger transaction `cf1b21f6` recorded for protocol hardening.
- [x] Verified word-budget enforcement in `ai-brains-retrieval`.