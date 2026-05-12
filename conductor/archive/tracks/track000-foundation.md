# Track T00 — Foundation and Workspace

## Owner
Orchestrator

## Status
Completed

## Objective
Establish the Rust workspace foundation, scaffold the crate structure, and configure the project-wide linting and safety tools.

## Scope
- Root `Cargo.toml` (Workspace definition)
- Crate directory scaffolding (`crates/`)
- Configuration files (`clippy.toml`, `deny.toml`, `nextest.toml`, `rust-toolchain.toml`, `rustfmt.toml`)
- Helper scripts (`scripts/dev-check.ps1`)

## Out of Scope
- Implementation of domain logic (T01+)
- Database initialization (T05)

## Files Owned
- `Cargo.toml`
- `rust-toolchain.toml`
- `clippy.toml`
- `deny.toml`
- `nextest.toml`
- `rustfmt.toml`
- `scripts/dev-check.ps1`

## Required Tests First
- [x] `cargo fmt --check` must pass.
- [x] `cargo clippy --workspace` must pass (even with empty crates).
- [x] `cargo deny check` must pass.

## Implementation Steps
1. [x] Initialize Root `Cargo.toml` as a workspace.
2. [x] Scaffold empty crates with minimal `Cargo.toml` for each:
   - `ai-brains-core`, `ai-brains-events`, `ai-brains-contracts`, `ai-brains-crypto`, `ai-brains-store`, `ai-brains-path`, `ai-brains-git`, `ai-brains-security`, `ai-brains-capture`, `ai-brains-adapters`, `ai-brains-daemon-api`, `ai-brainsd`, `ai-brains-cli`.
3. [x] Configure `rust-toolchain.toml` for version 1.95.0.
4. [x] Configure `clippy.toml` and `deny.toml` with project rules.
5. [x] Create `scripts/dev-check.ps1` that runs the full CI gate.
6. [x] Verify everything via `changeguard verify`.

## Acceptance Criteria
- [x] `cargo build` succeeds at the workspace level.
- [x] All scaffolding matches the PRD/Implementation Plan crate list.
- [x] CI gate script exists and passes.
