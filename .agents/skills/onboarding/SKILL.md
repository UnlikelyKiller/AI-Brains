---
name: onboarding
description: "Trigger this skill when starting a new session on the AI-Brains repo, when an agent needs orientation, or when asked 'where do I start?', 'what's the project state?', 'how does work get done here?', or 'onboard me'. Loads once per session to establish context."
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
- **`ai-brains-graph`**: Graph projection layer (SQLite + CozoDB via ChangeGuard bridge).
- **`ai-brains-brain`**: High-level intelligence services (Nightly, Summaries).
- **`ai-brains-scheduler`**: Windows Task Scheduler integration for background tasks.

## Current State

- **Plan**: `Docs/Implementation-Plan.md` v2 (Track-based execution).
- **Status**: Tracks T61–T71 all complete. All core features stable.
- **Infrastructure**: CI/CD ready, ChangeGuard ledger active, graph feature-gated (`--features graph`), full CI gate reproducible on Windows (T71).
- **Active tracks**: See `conductor/conductor.md` for the full registry (T61–T71).
- **Deviations**: See `Docs/Deviations.md` for architectural departures (e.g., SQLite fallback, graph decoupling).

## What the System Can Do — Recall & Graph

AI-Brains has two complementary recall systems and a live graph. Use them together:

### 1. AI-Brains Recall (session memory, decisions, synthesized knowledge)
```bash
# Keyword recall (FTS5)
ai-brains recall "GPU driver fix" --limit 5

# Semantic recall (embedding-based)
ai-brains recall "authentication flow" --semantic --limit 5

# Graph-boosted recall (boosts neighbors of top hits)
ai-brains recall "login handler" --semantic --graph-boost 0.1 --limit 5

# Scoped to a project
ai-brains recall "query" --project-id <id>
```

### 2. ChangeGuard Search (live code symbols, functions, routes, call graph)
```bash
# Refresh the symbol index first (always do this before code queries)
changeguard index --auto-index

# Find functions, structs, routes by name
changeguard search "handleGetUser"
changeguard search "POST /auth"

# Natural language code queries
changeguard ask "find all GET route handlers"
changeguard ask "what calls validateToken"

# See blast radius before editing
changeguard scan --impact
```

### 3. Graph Queries (relational memory structure)
```bash
# 1-hop neighbors of a memory
ai-brains graph neighbors <memory_id>

# Synthesis chain (what was this summary built from?)
ai-brains graph hierarchy <memory_id>

# All memories recalled in a session
ai-brains graph session <session_id>

# Graph health check (node/edge counts)
ai-brains graph update

# Full resync (only needed after schema changes or corruption)
ai-brains graph rebuild
```

### Which system to use for what

| Question | Use |
|----------|-----|
| "What did we decide about X?" | `ai-brains recall` |
| "What does function X do / which endpoints exist?" | `ai-brains recall --semantic` (T70: symbols are indexed) |
| "Live code query (not yet in nightly)" | `changeguard search` / `changeguard ask` |
| "What calls this function?" | `changeguard ask "what calls <fn>"` |
| "Find memories related to X" | `ai-brains recall --semantic --graph-boost 0.1` |
| "What was synthesized from this session?" | `ai-brains graph hierarchy <id>` |

> **As of T70:** `ai-brains recall` also returns code symbols (functions, routes) ingested from ChangeGuard during nightly — a single recall query is sufficient for most questions about decisions and code structure.

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

1. **Track Lead**: Follow the `conductor/conductor.md` phase by phase.
2. **Ledger**: Record all architectural decisions using `changeguard ledger`.
3. **Verify**: Use `changeguard verify` to ensure structural and behavioral integrity.
4. **Research-Strategy-Execution**: Follow the standard agent lifecycle for every track.

## Key Reference Documents

| Document | Purpose |
|----------|---------|
| `Docs/PRD.md` | Product vision and core requirements |
| `Docs/Implementation-Plan.md` | Master execution plan (Tracks) |
| `AGENTS.md` | Unified project rules and mandates |
| `conductor/conductor.md` | Track registry (T61–T71, all complete) |

## Quick Start

1. **Activate and read the `changeguard` skill** — it handles impact analysis, symbol search, and provenance.
2. **Run `changeguard index --auto-index`** to refresh the live code symbol index.
3. **Run `ai-brains recall "what is this project"` `--semantic`** to surface existing knowledge.
4. **Read `conductor/conductor.md`** to see the current track state (T61–T71, all complete).
5. **Run `changeguard ledger status`** to check recent architectural provenance.
6. **Run `ai-brains graph update`** to verify the graph is populated (should show >8,000 nodes).
