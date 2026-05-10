# AI-Brains Project Rules

## Engineering Mandates
- **Capture Independence**: The capture path (CLI -> Daemon -> Event Log) MUST remain functional without dependencies on models, embeddings, or graph databases.
- **Canonical Source of Truth**: Every state change MUST be recorded as an immutable event in the SQLCipher-backed append-only event log.
- **CQRS Integrity**: Commands append events; queries read projections. DO NOT mix read/write logic in the same service or transaction.
- **Capture Privacy**: DO NOT store hidden chain-of-thought, model reasoning, or raw tool logs. Capture ONLY the final assistant response and user prompt.
- **Privacy Inheritance**: Derived memories (summaries, clusters) MUST inherit the strictest privacy flag from their source events.
- **Rust Safety**: PROHIBITED use of `unwrap()`, `expect()`, or `panic()` in production code. Explicit error handling (`thiserror`, `anyhow`) and `zeroize` for sensitive key material are mandatory.
- **Provenance**: ALL architectural decisions and track implementations MUST be recorded in the `changeguard ledger`.
- **No Repository Pollution**: AI-Brains MUST NOT write project-local files by default. Use global user storage (`$env:USERPROFILE\.ai-brains`) unless the user explicitly invokes a repo-write command.

## Technical Invariants
- **Path Normalization**: Normalization for Windows drive-case, UNC prefixes, and WSL mappings is mandatory for all stored paths.
- **Relational Graph**: Implementation MUST use the native SQLite backend (Recursive CTEs) to avoid C++ build friction.
- **Event Sourcing**: Updating or deleting raw events is PROHIBITED. Use compensating events for corrections.
- **Commercial Safety**: Only permissive licenses (MIT, Apache, BSD) are allowed. AGPL/GPL dependencies are PROHIBITED.

## Hardware & Environment
- **Context Constraints**: Enforce a 38,912 token limit for summarization. Use sequential chunking with context carryover for larger sessions.
- **VRAM Management**: High-performance multi-stage RAG (BGE-M3 + Qwen 3.5) MUST use dynamic model switching via `.env` to prevent VRAM overflow.
- **Shell Consistency**: Use PowerShell for all shell commands. PROHIBITED use of `&&`. Use `;` as the statement separator.

## Workflow & Verification
- **Test-Driven Development**: Behavioral correctness MUST be proven via failing tests before implementation (Two-commit minimum: Red -> Green).
- **CI Gate**: Before every commit, the workspace MUST pass:
  `cargo fmt --check ; cargo clippy --workspace --all-targets -- -D warnings ; cargo nextest run --workspace ; cargo deny check ; cargo audit`
- **Track Discipline**: Implementation MUST follow the `conductor/conductor.md` track-by-track.
- **Change Management**: Run `changeguard scan --impact` before edits and `changeguard verify` before commits.
