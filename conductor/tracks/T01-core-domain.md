# Track T01 — Core Domain

## Owner
architecture-planner

## Status
Completed

## Objective
Implement the pure domain model for AI-Brains within the `ai-brains-core` crate.

## Scope
- Domain type definitions (IDs, Time, Privacy, Status).
- Core entities (Project, User, Device, Harness, Session, Turn, Memory, Conflict, Recipe).
- Domain validation logic.

## Out of Scope
- Database persistence or SQL definitions.
- CLI, HTTP, daemon, filesystem, Git, or model provider logic.
- Event envelope or complete event definitions (handled in T02).

## Files Owned
`crates/ai-brains-core/*`

## Files Allowed To Touch
`crates/ai-brains-core/src/lib.rs`
`crates/ai-brains-core/src/ids.rs`
`crates/ai-brains-core/src/clock.rs`
`crates/ai-brains-core/src/privacy.rs`
`crates/ai-brains-core/src/project.rs`
`crates/ai-brains-core/src/user.rs`
`crates/ai-brains-core/src/device.rs`
`crates/ai-brains-core/src/harness.rs`
`crates/ai-brains-core/src/session.rs`
`crates/ai-brains-core/src/turn.rs`
`crates/ai-brains-core/src/memory.rs`
`crates/ai-brains-core/src/conflict.rs`
`crates/ai-brains-core/src/recipe.rs`
`crates/ai-brains-core/src/status.rs`
`crates/ai-brains-core/src/validation.rs`
`crates/ai-brains-core/src/errors.rs`
`crates/ai-brains-core/tests/*.rs`
`crates/ai-brains-core/Cargo.toml`

## Files Forbidden To Touch
Any file outside `crates/ai-brains-core/`.

## Public Contracts Consumed
None (pure domain).

## Public Contracts Produced
Pure Rust structs and enums modeling the AI-Brains domain, usable by all other crates.

## Required Tests First
- `tests/id_serde_roundtrip.rs`
- `tests/privacy_strictest_wins.rs`
- `tests/session_status_transitions.rs`
- `tests/no_thinking_role_exists.rs`
- `tests/no_tool_call_role_exists.rs`
- `tests/domain_validation_rejects_empty_content.rs`

## Implementation Steps
1. Add required dependencies (`serde`, `uuid`, `time`, `thiserror`) to `crates/ai-brains-core/Cargo.toml`.
2. Define strong identifier types (e.g., `ProjectId`, `SessionId`) in `ids.rs` with Serde support.
3. Implement `clock.rs` for unified time handling using `time` crate.
4. Implement `privacy.rs` modeling escalation and strictness.
5. Implement status enums and state transitions in `status.rs`.
6. Implement core entity structs (`session.rs`, `turn.rs`, `memory.rs`, etc.) ensuring no hidden thinking or tool call capture roles exist.
7. Add domain validation rules in `validation.rs`.
8. Define domain errors in `errors.rs` using `thiserror`.
9. Ensure all modules are exported properly in `lib.rs`.
10. Write the required tests and make them pass.

## Failure Modes To Handle
- Invalid identifiers during deserialization.
- Empty content validation failures.
- Invalid session state transitions.

## Security Requirements
- Privacy struct must default to the strictest level when combined.
- Types modeling Turns and Sessions must NOT allow storage of hidden thinking, chain-of-thought, or raw tool logs.

## Acceptance Criteria
- All domain tests pass.
- `cargo clippy` and `cargo fmt` pass without warnings.
- The `ai-brains-core` crate contains only pure logic (no DB, IO, etc.).

## Commands To Run
`cargo test -p ai-brains-core`
`cargo clippy -p ai-brains-core -- -D warnings`

## Handoff Notes
Ensure next track (T02) utilizes these domain models for event definitions.
