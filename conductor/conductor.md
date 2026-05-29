# Conductor Master Track Board

## Project Status
**Status:** In Progress
**Phase:** Phase 19 - Refinement and Expansion
**Current Track:** T44
**Verification:** Full Windows workspace gate is green as of 2026-05-19.

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
| T36 | CLI Polish & Workflow | **Completed** | Orchestrator | [T36](./tracks/track036-cli-polish-workflow.md) | Manual audit of 9 commands; fix FTS5, ChangeGuard JSON, session safety, cross-command workflow. |
| T37 | Transaction Linking & Project Discovery | **Completed** | Orchestrator | [T37](./tracks/trackT37/spec.md) | Accept --tx-id in context/pin; auto-discover project_id from .changeguard. |
| T38 | Structured Bridge (NDJSON Fallback) | **Completed** | Orchestrator | [T38](./tracks/trackT38/spec.md) | Define BridgeRecord schema and implement sync pull from NDJSON. |
| T39 | Real-Time Bridge (IPC) | **Completed** | Orchestrator | [T39](./tracks/trackT39/spec.md) | Named-pipe/Local-HTTP handoff between ai-brainsd and ChangeGuard. |
| T40 | Unified Retrieval & Feedback Loop | **Completed** | Orchestrator | [T40](./tracks/trackT40/spec.md) | Blended preflight/ask results; nightly sweep accuracy checks. |
| T41 | Risk-Weighted Preflight & MADR Ingestion | **Completed** | Orchestrator | [T41](./tracks/trackT41/spec.md) | Contextual risk preflight + structured MADR ingestion from ChangeGuard. |
| T42 | Shared Knowledge Graph & Unified Search | **Completed** | Orchestrator | [T42](./tracks/trackT42/spec.md) | CozoProxyBackend + unified IPC recall via ChangeGuard Tantivy/CozoDB. |
| T43 | Predictive Verification Gating & Intervention | **Completed** | Orchestrator | [T43](./tracks/trackT43/spec.md) | Ingest-final verification gate + RiskReviewAgent for proactive intervention. |
| T44 | System Hardening & Fast-Fail | **Completed** | Orchestrator | [T44](./tracks/trackT44/spec.md) | Ultra-fast handshake, CLI fast-fail, structured JSON errors, signal handlers. |
| T45 | Antigravity CLI (agy) Integration | **Completed** | Orchestrator | [T45](./tracks/trackT45/spec.md) | `agy` hooks and JSONL transcript capture adapter. |
| T46 | Multi-Path Antigravity Discovery | **Completed** | Orchestrator | [T46](./tracks/trackT46/spec.md) | Expand discovery to scan all tool-specific and project-specific chat locations. |
| T47 | Safety Sync Hardening & Skill Alignment | **Completed** | Orchestrator | [T47](./tracks/trackT47/spec.md) | Fix floating-point parsing in safety sync and update skill instructions. |
| T48 | Automated Project Mapping | **Pending** | Orchestrator | [T48](./tracks/trackT48/spec.md) | Map Antigravity projectHash to project_id for auto-discovery. |
| T49 | Differential agy Ingestion | **Pending** | Orchestrator | [T49](./tracks/trackT49/spec.md) | Delta sync for agy conversations to skip existing turns. |
| T50 | CozoDB Bridge Hardening & Unification | Pending | Orchestrator | [T50](./tracks/trackT50/spec.md) | Batch ingestion, unified projection, fast-fail bridge handshake, and Datalog escaping. |
| T51 | Daemon Auto-Launch & Bridge Silence | **Completed** | Orchestrator | [T51](./tracks/trackT51/spec.md) | Auto-start daemon on demand and suppress bridge noise. |
| T52 | Nightly Resilience & Async Alignment | **Completed** | Orchestrator | [T52](./tracks/trackT52/spec.md) | Fix nightly runtime panic and ensure auto-start for background tasks. |
| T53 | Daemon Lifecycle & Global Install UX | **Completed** | Orchestrator | [T53](./tracks/trackT53/spec.md) | Implement graceful stop and solve binary locking during upgrades. |
| T54 | Bridge Stderr Hardening | **Completed** | Orchestrator | [T54](./tracks/trackT54/spec.md) | Suppress transient ChangeGuard stderr in quiet mode. |
|| T55 | Nightly Performance (Incremental Scan) | **Completed** | Orchestrator | [T55](./tracks/trackT55/spec.md) | Skip parsing unchanged Antigravity sessions to speed up nightly. |
|| T56 | CLI Output Polish (Smart Preflight) | **Completed** | Orchestrator | [T56](./tracks/trackT56/spec.md) | Default to human-readable output and add summary mode to preflight. |
||| **T57** | **Nightly --status Subcommand** | **Completed** | Orchestrator | [T57](./tracks/trackT57-nightly-status/spec.md) | Read-only status report for nightly runs. |
||| **T58** | **Fix Unix Absolute Path Normalization** | **In Progress** | Orchestrator | [T58](./tracks/trackT58-unix-path-normalization/spec.md) | Handle Unix absolute paths in `normalize_project_path()`. |

## Completed Phase: Phase 19 - Refinement and Expansion
- **Track T51**: Implemented daemon auto-launch and bridge silence mechanism. `ai-brains sync query` now auto-starts `ai-brainsd` if unreachable, supports a `--quiet` flag, preserves vault settings (path/key) in spawned processes, and mitigates startup race conditions. Codex review findings addressed.
- [x] **Track T44** - System Hardening & Fast-Fail: Ultra-fast handshake (<10ms), CLI fast-fail, structured JSON errors, and signal handlers.
- [x] **Track T45** - Antigravity CLI (agy) Integration: `agy` hooks configuration and JSONL transcript capture adapter with privacy filtering.
- [x] **Track T46** - Multi-Path Antigravity Discovery: Expanded discovery to scan tool-specific brain dirs and project-specific tmp folders, supporting new JSONL formats.
Cross-repo tracks in ChangeGuard: Milestone C (C1, C2, C3). See `C:\dev\ChangeGuard\conductor\conductor.md`.
