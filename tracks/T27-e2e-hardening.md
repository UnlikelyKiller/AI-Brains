# Track T27: E2E Hardening

## Context
Phase 12 is the final push for production readiness. We need to verify that all components work together seamlessly under load and in failure scenarios.

## Goals
- Implement E2E smoke tests for the entire capture-to-recall loop.
- Verify daemon concurrency with multiple simultaneous sessions.
- Implement and verify failure drills (corrupt projections, missing models, etc.).
- Ensure all CI gates pass with a clean clippy and fmt.

## Implementation Plan

### Phase 1: E2E Test Suite
- [x] Create `tests/e2e` directory.
- [x] Implement `cli_capture_smoke.rs`.
- [x] Implement `daemon_concurrency.rs`.
- [x] Implement `recovery_restore.rs`.

### Phase 2: Failure Drills
- [x] Create `Docs/conductor/failure-drills.md`.
- [x] Implement `scripts/run-failure-drills.ps1`.
- [x] Verify "Kill daemon mid-ingest" drill.
- [x] Verify "Corrupt projection rebuild" drill.

### Phase 3: Final Polishing
- [x] Run `cargo clippy --workspace --all-targets -- -D warnings`.
- [x] Run `cargo fmt --check`.
- [x] Run `cargo nextest run --workspace`.

## Progress
- [x] Phase 1
- [x] Phase 2
- [x] Phase 3
