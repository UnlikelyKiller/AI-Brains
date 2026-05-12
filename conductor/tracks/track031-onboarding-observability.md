# Track T31: Repository Onboarding & Observability Protocol (Hardened)

## Goal
Integrate the four layers of repository understanding (Structural, Behavioral, Observability, Safety) into the AI-Brains agentic protocol to ensure total contextual coverage during onboarding.

## User Review Required
> [!IMPORTANT]
> This track mandates a "Stop-and-Think" moment for all agents. This may slightly increase initial turn latency in exchange for significantly higher implementation accuracy and lower regression rates.

## Milestones

### M1: Protocol Hardening (Skill Expansion)
- [x] **Manual Bearing Logic**: Update `ai-brains/skill.md` to handle "Negative Space." (Ingested missing CI/CD constraint).
- [x] **Budget-Aware Preflight**: Implement a hard limit on "Onboarding Checklist" injection.
- [x] **ChangeGuard Sync**: Explicitly mandate `changeguard scan` as the final step.

### M2: Observability & Discovery Logic
- [x] **Search Pattern Templates**: Defined specific `grep`/`ripgrep` patterns in the skill.
- [/] **Persistence Gate for Diagnostics**: (Tracing pattern identified as `tracing::info/warn/debug`).

### M3: ChangeGuard Integration & Verification
- [ ] **Hotspot Ingestion**: Implement a workflow where `changeguard hotspots` are periodically ingested into the AI-Brains vault as `INVARIANT: Brittle File`.
- [ ] **Ledger Discipline**: Every change made during T31 must be wrapped in a `changeguard ledger` transaction to maintain provenance.

## Definition of Done (DoD)
- [x] **Schema Integrity**: `ai-brains/skill.md` contains the 4-layer framework (implemented in `cf1b21f6`).
- [x] **Proven Provenance**: T31 changes committed via `changeguard ledger` with full rationale.
- [x] **Validation Pass**: Successfully identified "Negative Space" (missing CI/CD) and ingested as a constraint.
- [ ] **Budget Check**: `ai-brains preflight` word-budget enforcement verified (M3).
- [ ] **Conflict Resolution**: Conflict detection flow verified during manual research (M3).

## Verification Plan
### Automated
- `changeguard verify` must pass for all logic changes.
- `ai-brains preflight` smoke test to verify word-budget enforcement.

### Manual
- Simulate a "Cold Start" in a new module to verify the agent performs the "Onboarding Gate" research steps.
