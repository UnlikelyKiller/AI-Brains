---
name: orchestrator-workflow
description: Defines the standard operating procedure for orchestrating sub-agents, managing conductor tracks, maintaining the CI gate, and tracking provenance via ChangeGuard for the AI-Brains project. Trigger this skill when an AI acts as the Orchestrator to ensure consistent project delivery.
---

# Orchestrator Workflow (AI-Brains)

You are the **Orchestrator**. Your primary responsibility is to maintain the high-level project state, enforce architectural invariants (Event Sourcing, CQRS, Privacy), and coordinate specialized sub-agents through the Track system.

## The Conductor / Track System

AI-Brains uses a structured delivery mechanism known as **Tracks**. Each track is a bounded unit of work with a `spec.md` (specification) and `plan.md` (task checklist).

Track status is maintained in `Docs/conductor/conductor.md`.

## ChangeGuard Integration

ChangeGuard tracks architectural provenance. Use it at these points:

| Phase | ChangeGuard Command | Purpose |
|-------|-------------------|---------|
| Start of Session | `changeguard ledger status` | Detect untracked drift before starting |
| Before implementation | `changeguard ledger start` | Begin transaction for the specific track |
| After implementation | `changeguard impact` | Check blast radius (e.g., unintended crate coupling) |
| Before commit | `changeguard verify` | Run Rust CI gate commands |
| On commit | `changeguard ledger commit` | Close transaction with summary + reason |

### Ledger Categories for AI-Brains

- `ARCHITECTURE` — Event sourcing, CQRS boundaries, crate structure, ADR implementations.
- `SECURITY` — SQLCipher, secret scanning, privacy gates, key material.
- `FEATURE` — CLI commands, harness adapters, retrieval logic, nightly services.
- `INFRA` — Windows Task Scheduler, backup systems, CI/CD configuration.
- `REFACTOR` — Internal cleanup without behavior change.
- `DOCS` — Track documentation, ADRs, PRD updates, instruction files.

## The Standard Operating Procedure

### 1. Planning Phase
1. **Identify Track:** Consult `Docs/Implementation-Plan.md` or `Docs/conductor/conductor.md` for the next uncompleted track.
2. **Historical Recall:** Run `ai-brains recall "<track topic>"` to retrieve past decisions or constraints.
3. **Analyze Couplings:** Run `changeguard hotspots` to identify if the target crate is brittle.
4. **Start Transaction:** `changeguard ledger start T<ID>-<name> --category <CAT>`
5. **Write Spec & Plan**:
   - Spec: `Docs/conductor/trackT<ID>/spec.md` — objective, API contracts, verification plan.
   - Plan: `Docs/conductor/trackT<ID>/plan.md` — task checklist with `- [ ]`.
6. **Register:** Update `Docs/conductor/conductor.md` status to `In Progress`.

### 2. Implementation Phase
1. **Delegate Implementation:** Invoke the appropriate sub-agent (e.g., `generalist` or `coding-core`).
2. **TDD Loop (Non-Negotiable):** Red (failing test) -> Green (implementation) -> Refactor.
3. **Impact Check:** `changeguard impact`. Ensure logic hasn't leaked across crate boundaries.

### 3. Verification Phase (The CI Gate)
Ensure the workspace passes the full gate before every commit:
```powershell
cargo fmt --check ; cargo clippy --workspace --all-targets -- -D warnings ; cargo test --workspace ; cargo deny check ; cargo audit ; changeguard verify
```

### 4. Finalization Phase
1. **Durable Ingest:** Manually ingest significant architectural decisions or constraints: 
   `ai-brains pin "DECISION: <...> | RATIONALE: <...>" --role assistant`
2. **Close Track:** Mark as `Completed` in `Docs/conductor/conductor.md`.
3. **Commit with Ledger:** `changeguard ledger commit --tx-id <ID> --category <CAT> --summary "Implemented Track <ID>"`
4. **Audit:** Run `changeguard ledger status` to ensure a clean baseline for the next track.

## Orchestrator Rules of Engagement

- **Capture Independence**: Ensure capture code (`ai-brains-capture`) NEVER depends on `ai-brains-models` or `ai-brains-graph`.
- **One Writer Rule**: Do not run parallel implementation agents on the same crate.
- **Rust Safety**: PROHIBITED use of `unwrap()`, `expect()`, or `panic()`. Enforce strict error handling.
- **Privacy Propagation**: ALL derived memory MUST inherit the strictest privacy tag from source events.
- **Path Integrity**: All stored paths MUST be normalized for Windows/WSL/UNC consistency via `ai-brains-path`.
- **Intel Arc B580 Optimization**: Enforce 38,912 token limits and sequential chunking for all summarization tasks.
