# Conductor Master Track Board

## Project Status
**Status:** Completed
**Phase:** Phase 15 - Cross-Agent Memory Synthesis [COMPLETED]
**Current Track:** Finalized
**Verification:** Full Windows workspace gate is green as of 2026-04-30. Relational graph verified in workspace; cross-agent synthesis and harness hooks verified.

## Track Registry

| ID | Name | Status | Owner | Description |
|---|---|---|---|---|
| T00 | Foundation and Workspace | **Completed** | Orchestrator | Workspace structure, CI gates, and global tools. |
| T01 | Core Domain | **Completed** | architecture-planner | Pure domain model (ids, privacy, session). |
| T02 | Event Contracts | **Completed** | architecture-planner | Immutable event definitions. |
| T03 | JSON Contracts | **Completed** | architecture-planner | API DTOs for CLI/Daemon. |
| T04 | Crypto Recovery | **Completed** | architecture-planner | SQLCipher key and recovery kit logic. |
| T05 | Store Event Log | **Completed** | architecture-planner | Append-only event store on SQLCipher. |
| T06 | Store Projections | **Completed** | architecture-planner | Read-optimized views derived from events. |
| T07 | Path Normalization | **Completed** | architecture-planner | Canonical Windows/WSL/UNC project identity. |
| T08 | Git Metadata | **Completed** | architecture-planner | Bounded repository identity and diff statistics. |
| T09 | Security Scanning | **Completed** | architecture-planner | Secret detection, privacy escalation, and redaction. |
| T10 | Capture Core | **Completed** | architecture-planner | Daemon-free capture command handling. |
| T11 | Daemon Writer | **Completed** | architecture-planner | Single-writer queue and spool replay. |
| T12 | CLI Foundation | **Completed** | architecture-planner | JSON-only CLI routing for local capture. |
| T13 | CLI Capture | **Completed** | architecture-planner | CLI integration with capture core. |
| T18 | Retrieval Lexical | **Completed** | architecture-planner | Lexical recall over the store read side. |
| T19 | Preflight Recall | **Completed** | architecture-planner | Word-budgeted preflight assembly with privacy filtering. |
| T20 | Graph Projection | **Completed** | architecture-planner | Rebuildable graph projection layer with default deterministic backend and opt-in LadybugDB backend. |
| T21 | Model Providers | **Completed** | architecture-planner | Provider-agnostic model abstraction and local providers. |
| T22 | Nightly Summaries | **Completed** | architecture-planner | Nightly service with unsummarized session discovery. |
| T23 | Conflicts Recipes | **Completed** | architecture-planner | Contradictory session detection and recipe promotion. |
| T24 | RAPTOR CRAG | **Completed** | architecture-planner | Hierarchical clustering and synthesis (Memory Synthesis). |
| T25 | Scheduler Backups | **Completed** | architecture-planner | Windows task scheduling and automated vault backups. |
| T26 | Retention Forget | **Completed** | architecture-planner | Policy-based turn expiration and soft-delete. |
| T27 | E2E Hardening | **Completed** | architecture-planner | Smoke testing and fault isolation. |
| T28 | Cross-Agent Harness | **Completed** | Orchestrator | Cross-agent synthesis and standardized hook implementation. |
| T29 | Relational Graph | **Completed** | Orchestrator | Migrate graph projection to SQLite (ADR-0009). |

## Current Verification Snapshot
- `cargo fmt`: passes.
- `cargo check --workspace --all-targets`: passes.
- `cargo clippy --workspace --all-targets -- -D warnings`: passes.
- `cargo test --workspace`: passes.
- Relational graph: native SQLite backend; no C++ dependencies.
- `cargo-nextest`: not installed in this shell; `cargo test` was used for local verification.
- `changeguard scan --impact`: HIGH risk because of changed public symbols and high changed-file volume.
- `changeguard ledger status`: no pending transactions and no unaudited drift.

## Completed Track: T27 - E2E Hardening
- [x] Implemented `init` command in CLI for clean vault creation and migration.
- [x] Decoupled graph database compilation via Cargo features (`--no-default-features`).
- [x] Authored automated E2E smoke tests for `init` and `ingest`.
- [x] CI Gate: historical smoke tests existed; current workspace gate must be re-run after reconciliation.

## Completed Track: T26 - Retention Forget
- [x] `last_accessed_at` added to `turn_projection` via migration 0012.
- [x] `RetentionService` implemented for automated 90-day turn expiration.
- [x] `forget` command added to CLI with `MemoryForgotten` event support.
- [x] SQL and Graph search queries updated to exclude forgotten memories.
- [x] Integration tests exist for retention and forget behavior.

## Completed Track: T25 - Scheduler Backups
- [x] `ai-brains-scheduler` crate implemented for Windows task management.
- [x] `ai-brains nightly --schedule` command added to CLI.
- [x] `BackupService` implemented for timestamped vault copies.
- [x] `ai-brains backup` command added to CLI.
- [x] Unit tests exist for scheduler and backup behavior.

## Completed Track: T24 - RAPTOR CRAG
- [x] RAPTOR-style hierarchical clustering for session summaries.
- [x] `MemorySynthesized` event defined and implemented.
- [x] Hierarchical knowledge node synthesis.
- [x] CRAG factual verification gate implemented.
- [x] Tests exist for RAPTOR synthesis behavior.

## Completed Track: T23 - Conflicts Recipes
- [x] `ConflictDetected` event defined and implemented.
- [x] `RecipePromoted` event defined and implemented.
- [x] `ConflictDetectionService` implemented in `ai-brains-brain`.
- [x] `RecipePromotionService` implemented in `ai-brains-brain`.
- [x] Tests exist for conflict detection and recipe promotion behavior.

## Implemented Track: T22 - Nightly Summaries
- [x] `NightlyService` implemented in `ai-brains-brain`.
- [x] Unsummarized session discovery via `QueryStore`.
- [x] `SessionSummaryCreated` event integration.
- [x] Verification reconciliation: brain tests updated for the current `NightlyService::new` signature and pass in degraded workspace verification.
- [x] ChangeGuard reconciliation: no pending transaction for `crates/ai-brains-brain`.

## Implemented Track: T21 - Model Providers
- [x] `ai-brains-models` crate scaffolded and added to workspace.
- [x] `ModelProvider` trait and common DTOs defined.
- [x] `OllamaProvider` implemented with `reqwest`.
- [x] `ProviderRegistry` implemented with privacy-aware routing.
- [x] `MockProvider` implemented for testing.
- [x] Verification reconciliation: model provider tests pass in degraded workspace verification.
- [x] ChangeGuard reconciliation: no pending transaction for `crates/ai-brains-models`.

## Completed Track: T20 - Graph Projection (Superseded by T29)
- [x] `ai-brains-graph` crate scaffolded and added to workspace.
- [x] Initial schema with Project/Session/Turn/Memory nodes.
- [x] `GraphProjector` implemented for event-driven updates.
- [x] `GraphRebuilder` implemented for event-log reconstruction.
- [x] Graph traversal implemented for related memory discovery.
- [x] Verification reconciliation: all-target workspace check passes.

## Completed Track: T19/T18 - Retrieval and Preflight
- [x] Single writer queue supports concurrent ingest.
- [x] Spool replays after restart.

## Completed Track: T13/T12/T10 - Capture Without Daemon
- [x] Capture works without daemon.
- [x] Capture works without graph/models.
- [x] Hook-facing CLI ingest produces JSON only.
- [x] Raw tool/thinking fields ignored.

## Completed Track: T09 - Security Scanning
- [x] Phase 1: Crate Setup
- [x] Phase 2: Detection and Policy
- [x] Phase 3: Verification

## Completed Track: T08 - Git Metadata
- [x] Phase 1: Crate Setup
- [x] Phase 2: Repository Discovery
- [x] Phase 3: Metadata Readers
- [x] Phase 4: Verification

## Completed Track: T07 - Path Normalization
- [x] Phase 1: Crate Setup
- [x] Phase 2: Canonical Windows Identity
- [x] Phase 3: Cross-Environment Mapping
- [x] Phase 4: Best-Effort Resolution and Display
- [x] Phase 5: Verification

## Completed Track: T06 - Store Projections
- [x] Phase 1: Projection Migrations and Schema
- [x] Phase 2: Projection Handlers
- [x] Phase 3: FTS Integration
- [x] Phase 4: Event Replay and Rebuild
- [x] Phase 5: Concurrency and Review

## Completed Track: T05 - Store Event Log
- [x] Phase 1: Cargo Setup and Errors
- [x] Phase 2: Schema and Migrations
- [x] Phase 3: Vault Connection and Config
- [x] Phase 4: Event Store and Transaction
- [x] Phase 5: Final Review
