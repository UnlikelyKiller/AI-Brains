---
name: coding-core
description: Use this skill when writing, modifying, or reviewing Rust code in AI-Brains. Trigger when editing .rs files, making architectural decisions, implementing features, or discussing event sourcing, SQLCipher, crate boundaries, or memory intelligence.
---

# Coding Core - AI-Brains

Load this skill when working on any crate in the AI-Brains Rust workspace.

## Retrieval Precedence

1. **Active File / Context**: Current code and task context.
2. **Local Rules**: `.agents/rules/*.md`.
3. **Documentation**: `Docs/PRD.md`, `Docs/Implementation-Plan.md`.
4. **External**: `context7` for library docs (Tokio, Axum, SQLCipher, Rusqlite), `exa` for web search.

## Engineering Standards

- **Rust Version**: ~1.95.0. 
- **Error Handling**: `thiserror` for library crates, `anyhow` for binary crates. No `unwrap` or `panic` in production.
- **Serialization**: `serde` for all DTOs and events.
- **Database**: `rusqlite` with SQLCipher for the canonical store. `LadybugDB` for graph projections.
- **Async**: `Tokio` runtime for daemon and scheduler coordination.
- **API Framework**: `Axum` for the localhost daemon API.
- **Composition**: Prefer composition and delegation over complex inheritance or trait-based cloning.
- **Immutability**: Canonical events are immutable. Projections are rebuildable.

## Workspace Boundaries

| Crate | Responsibility |
|-------|----------------|
| `ai-brains-core` | Pure domain model (ids, privacy, session, memory). No external IO. |
| `ai-brains-events` | Immutable event definitions and the event envelope (hashing, signing). |
| `ai-brains-contracts` | Shared JSON DTOs for CLI <-> Daemon communication. |
| `ai-brains-store` | SQLCipher event log, migrations, and read projections. |
| `ai-brains-crypto` | Key material management, DPAPI wrappers, and recovery kit logic. |
| `ai-brains-capture` | Converts harness-specific IO into normalized domain events. |
| `ai-brainsd` | Daemon process managing single-writer queue and vault unlock. |
| `ai-brains-cli` | The main CLI interface (`init`, `ingest`, `recall`, etc.). |

## Patterns & Performance

- **Event Sourcing**: Every command result is an append-only event.
- **CQRS**: Command handlers append events; Query handlers read optimized projection tables.
- **Single-Writer**: The daemon uses a queue to ensure only one process writes to SQLCipher at a time.
- **Path Normalization**: Use `ai-brains-path` to handle Windows-specific path edge cases before storage.

## Security Mandates

- **Secret Redactor**: `ai-brains-security` must scan content for secrets before embedding or cloud model calls.
- **Vault Encryption**: All canonical data must be stored in a SQLCipher-encrypted vault.
- **Privacy Inheritance**: Derived data must inherit the source's privacy flag.
