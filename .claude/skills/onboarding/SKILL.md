---
name: onboarding
description: Trigger this skill when starting a new session on the AI-Brains repo, when an agent needs orientation, or when asked "where do I start?", "what's the project state?", "how does work get done here?", or "onboard me". Loads once per session to establish context.
---

# AI-Brains Onboarding

You are working on **AI-Brains** — a Windows-first, local-first memory system for AI coding harnesses. It captures clean conversation history without the noise of tool logs or hidden thinking.

## Core Pillars

1.  **Capture First**: Capture must be fast, durable, and independent of advanced features (models, graph, etc.).
2.  **Canonical SSOT**: A SQLCipher-backed append-only event log is the single source of truth.
3.  **Privacy & Security**: Encrypted storage, secret scanning, and strict privacy inheritance (local_only, sealed).
4.  **CQRS**: Commands append events; Queries read projections. Never mix them.

## Architecture: Rust Workspace

The project is organized into specialized crates to maintain strict boundaries:

- **`ai-brains-core`**: Pure domain model (ids, privacy, session, memory).
- **`ai-brains-events`**: Immutable event definitions and envelope.
- **`ai-brains-store`**: SQLCipher event log and read-optimized projections.
- **`ai-brainsd`**: Local daemon with a single-writer queue for concurrency safety.
- **`ai-brains-cli`**: Primary user/harness interface (the `ai-brains` command).
- **`ai-brains-capture`**: Logic for converting harness IO into domain events.
- **`ai-brains-models`**: Local AI provider routing (Ollama, etc.).
- **`ai-brains-retrieval`**: Semantic and FTS search engines (Vector + Keyword).
- **`ai-brains-graph`**: (Optional) LadybugDB integration for structural intelligence.
- **`ai-brains-brain`**: High-level intelligence services (Nightly, Summaries).
- **`ai-brains-scheduler`**: Windows Task Scheduler integration for background tasks.

## Current State

- **Plan**: `Docs/Implementation-Plan.md` v2 (Track-based execution).
- **Status**: Completed **Phase 15 — Cross-Agent Memory Synthesis**. All core features are stable.
- **Infrastructure**: CI/CD ready, ChangeGuard ledger active, LadybugDB feature-gated for MSVC compatibility.
- **Deviations**: See `Docs/Deviations.md` for architectural departures (e.g., SQLite fallback, graph decoupling).

## Engineering Principles (Non-Negotiable)

- **Rust Safety**: No `unwrap`, `expect`, or `panic` in production code. Use `thiserror` and `anyhow` for errors.
- **Event Sourcing**: Never update/delete raw events. Use compensating events.
- **No Thinking Capture**: Do not store hidden chain-of-thought or raw tool logs.
- **TDD (Tracks)**: Implementation follows the Conductor/Track system. Verify behavior via tests before implementation where possible.
- **Windows-First**: Paths must handle UNC, WSL, and drive-case normalization correctly.

## CI Gate (Must Pass Before Every Commit)

```powershell
cargo fmt --check ; cargo clippy --workspace --all-targets -- -D warnings ; cargo nextest run --workspace ; cargo deny check ; cargo audit
```

## Workflows

1. **Track Lead**: Follow the `conductor/conductor.md` phase by phase (T00 -> T34).
2. **Ledger**: Record all architectural decisions using `changeguard ledger`.
3. **Verify**: Use `changeguard verify` to ensure structural and behavioral integrity.
4. **Research-Strategy-Execution**: Follow the standard agent lifecycle for every track.

## Key Reference Documents

| Document | Purpose |
|----------|---------|
| `Docs/PRD.md` | Product vision and core requirements |
| `Docs/Implementation-Plan.md` | Master execution plan (Tracks) |
| `AGENTS.md` | Unified project rules and mandates |
| `conductor/` | Track management and review checklists |

## Quick Start

1. **Activate and read the `changeguard` skill** to fully understand the project's change management, risk analysis, and provenance protocols.
2. **Read `Docs/status.md`** to see the current completion roadmap.
3. **Review `Docs/Deviations.md`** to understand Windows-specific build constraints.
4. **Run `ai-brains --help`** to explore the hardened CLI toolchain.
5. **Run `changeguard ledger`** to check recent architectural provenance.
