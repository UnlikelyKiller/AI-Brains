# Track T30 - Hardening and Protocol Optimization

## Owner
Orchestrator

## Status
Completed

## Objective
Harden the AI-Brains system by optimizing context window utilization, fixing data integrity bugs (encoding), and improving the agentic protocol for historical recall. This track addresses feedback from live usage and research into hybrid RAG systems.

## Scope
- **Optimization**: Implement "Index-First" preflight briefing. Gracefully handle low word-budgets by prioritizing the index over detailed memories.
- **Reliability**: Fix UTF-8 encoding in PowerShell hooks using BOM-less UTF8 to prevent pipeline mangling.
- **Contextual Integrity**: Add `session_id` to the memory schema. Ensure backward compatibility with existing NULL-session memories.
- **Agentic Protocol**: Rewrite skill triggers to be prescriptive. Integrate into `orchestrator-workflow` as a required gate.
- **Ingest Quality**: Filter redundant output (test logs, file lists) to prevent vault bloat.

## Resilience & Edge Cases
- **Empty Index**: If no pinned memories exist, `preflight` must skip the index section entirely without error.
- **Migration**: Schema change must be an additive migration; existing code must handle NULL `session_id`.
- **PowerShell Encoding**: Scripts must explicitly set UTF8 for both `stdin` and `stdout` to handle cross-shell piping (e.g., CMD -> PowerShell -> Rust).
- **Word Budget Overflow**: If the memory index itself exceeds the word budget, it must be truncated with a "..." indicator.

## Implementation Steps
1. [ ] **Transactional Foundation (ChangeGuard)**
    - [ ] Initialize ChangeGuard session for T30.
2. [ ] **Preflight & Recall Hardening**
    - [ ] Update `ai-brains-retrieval` with robust index-first logic.
    - [ ] Implement graceful truncation for word budget overflows.
3. [ ] **Encoding & Data Integrity**
    - [ ] Apply BOM-less UTF8 fixes to all harness hooks.
    - [ ] Verify fix by ingesting and recalling smart quotes and emojis.
4. [ ] **Schema & Contextual Search**
    - [ ] Deploy migration `0014_memory_session_context.sql` (additive).
    - [ ] Update projections and `recall` command with NULL-safe session filtering.
5. [ ] **Protocol & Workflow Integration**
    - [ ] Rewrite `ai-brains` skill to use "Protocol" framing.
    - [ ] Mandatory update to `orchestrator-workflow`.
6. [ ] **Final Reconciliation**
    - [ ] Verify ChangeGuard ledger status and reconcile drift.

## Acceptance Criteria
- `ai-brains preflight` saves >50% space and handles overflow gracefully.
- Unicode/Smart quotes are 100% preserved through the capture -> ingest -> recall cycle.
- `recall --session` returns valid results and handles invalid IDs without crashing.
- Orchestrator workflow contains explicit "Recall" and "Ingest" gates.

## Definition of Done (DoD)
- [ ] Code follows `cargo fmt` and `cargo clippy` (no warnings).
- [ ] 100% of new logic covered by unit or integration tests.
- [ ] PowerShell scripts verified to handle UTF8 across different terminal environments.
- [ ] Documentation (ADRs, status) updated to reflect the new protocol.
- [ ] ChangeGuard ledger contains a clean, audited transaction for T30.
