# Track T28 - Cross-Agent Memory Synthesis & Hook Implementation

## Owner
Orchestrator

## Status
In Progress

## Description
Implements Phase 15: Cross-Agent Memory Synthesis and standardized hook wrappers for AI harnesses (Gemini, Claude, Codex).

## Goals
- [x] Implement **Global Configuration Fallback** (`~/.ai-brains/.env`) for cross-repo defaults.
- [x] Implement **Project Isolation** via `ai-brains context` command for deterministic project identity.
- [x] Implement **Retrieval Briefing Index** in `preflight` for token-efficient memory pointers.
- [x] Define **AggregatedLearningsService** in `ai-brains-brain` for cross-agent synthesis.
- [x] Implement **Windows Native Search** in `mcptools` skill (PowerShell/Invoke-RestMethod).
- [x] Draft standardized hook scripts/docs for AI harness integration.
- [ ] Replace `AggregatedLearningsService::run_cross_agent_synthesis()` stub with real synthesis behavior.
- [ ] Verify Gemini hook context injection end to end.
- [ ] Implement **Standardized Claude Wrapper** for memory-directory synchronization.
- [ ] Verify Codex hook/preflight/ingest lifecycle end to end.

## Tasks
1. **Hierarchical Config [DONE]**: Add global fallback logic to CLI.
2. **Context Subcommand [DONE]**: Add `context` command to auto-generate project/session IDs.
3. **Briefing Index [DONE]**: Update `preflight` to return a table-of-contents first.
4. **Phase 15 Scaffold [PARTIAL]**: Add `AggregatedLearningsService` to the nightly sweep; real synthesis still returns `Ok(0)`.
5. **mcptools Hardening [DONE]**: Migrate search skills to Windows-native PowerShell.
6. **Harness Hooks [PARTIAL]**: Hook scripts/docs are present, but harness-specific verification remains open.

## Verification
- [x] `ai-brains preflight` returns indexed briefing.
- [x] `ai-brains context` correctly isolates projects.
- [x] Native search works without WSL.
- [ ] Hook script successfully injects context into Gemini CLI session.
- [ ] Claude wrapper verified for memory-directory synchronization.
- [ ] Codex hook verified for preflight -> command -> ingest lifecycle.
- [ ] Cross-agent synthesis emits durable learnings instead of stub count `0`.
- [x] Degraded Windows workspace gate is green as of 2026-04-30.

## Current Known Verification Gaps
- `cargo fmt --check` passes.
- `cargo clippy --workspace --all-targets --exclude ai-brains-graph -- -D warnings` passes.
- `cargo test --workspace --exclude ai-brains-graph` passes.
- Full `cargo check --workspace --all-targets` still fails on LadybugDB/MSVC debug `LNK1248`.
- `cargo-nextest` is not installed in this shell; local verification used `cargo test`.
- ChangeGuard ledger still has stale pending transactions for graph, models, and brain.
