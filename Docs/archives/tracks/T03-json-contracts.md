# Track T03 — JSON Contracts

## Owner
architecture-planner

## Status
Completed

## Objective
Define the JSON DTOs for CLI, daemon, hooks, and tests in the `ai-brains-contracts` crate.

## Scope
- Define JSON Data Transfer Objects (DTOs) for API request/responses.
- Define request/response structures for: ingest, hooks, preflight, recall, sessions, projects, memory, backup, doctor, and version.
- Ensure hook response shapes contain no stdout noise fields.
- Guarantee backward compatibility of contracts.
- Isolate the contract layer strictly from storage and daemon implementations.

## Out of Scope
- Implementation of the CLI tool or Daemon API handlers.
- Database schemas or ORM definitions.
- Transport-layer implementations (HTTP servers, sockets).

## Files Owned
`crates/ai-brains-contracts/*`

## Files Allowed To Touch
`crates/ai-brains-contracts/src/lib.rs`
`crates/ai-brains-contracts/src/response.rs`
`crates/ai-brains-contracts/src/errors.rs`
`crates/ai-brains-contracts/src/ingest.rs`
`crates/ai-brains-contracts/src/hook.rs`
`crates/ai-brains-contracts/src/preflight.rs`
`crates/ai-brains-contracts/src/recall.rs`
`crates/ai-brains-contracts/src/sessions.rs`
`crates/ai-brains-contracts/src/projects.rs`
`crates/ai-brains-contracts/src/memory.rs`
`crates/ai-brains-contracts/src/backup.rs`
`crates/ai-brains-contracts/src/doctor.rs`
`crates/ai-brains-contracts/src/version.rs`
`crates/ai-brains-contracts/tests/*.rs`
`crates/ai-brains-contracts/Cargo.toml`
`Docs/conductor/trackT03/spec.md`
`Docs/conductor/trackT03/plan.md`
`Docs/conductor/conductor.md`

## Files Forbidden To Touch
Any file outside `crates/ai-brains-contracts/` and the conductor planning docs.
Must NOT depend on or touch `store`, `adapters`, `daemon` crates.

## Public Contracts Consumed
- `ai-brains-core` (IDs, Privacy, Status, Time structs)

## Public Contracts Produced
- Rust struct definitions with `serde` traits representing API contracts over JSON.

## Required Tests First
- `tests/api_response_shape.rs`
- `tests/ingest_request_shape.rs`
- `tests/preflight_response_shape.rs`
- `tests/hook_response_has_no_stdout_noise_fields.rs`
- `tests/contracts_are_backward_compatible.rs`

## Implementation Steps
1. Add dependencies to `ai-brains-contracts/Cargo.toml` (`ai-brains-core`, `serde`, `serde_json`).
2. Create standard generic API response envelopes (`response.rs`, `errors.rs`).
3. Define request and response schemas for specific domains (`ingest.rs`, `hook.rs`, `preflight.rs`, etc.).
4. Add backwards compatibility checks and annotations.
5. Create strict unit tests verifying serialization output to ensure exact JSON representations match specs.

## Failure Modes To Handle
- JSON shape changes breaking external clients.
- Trailing commas, newlines, and unescaped characters in hook stdout causing JSON parsing to fail (needs to ensure structs themselves don't propagate noise).
- Missing optional fields causing deserialization errors.

## Security Requirements
- Privacy markers should be propagated correctly in DTOs.
- Hook outputs must clearly demarcate valid data from stdout garbage.

## Acceptance Criteria
- All required tests pass (specifically `hook_response_has_no_stdout_noise_fields`).
- Backward compatibility checks pass.
- No dependencies on `store` or `daemon`.

## Commands To Run
`cargo test -p ai-brains-contracts`
`cargo clippy -p ai-brains-contracts -- -D warnings`

## Handoff Notes
Next track will likely focus on implementing the Daemon or CLI tools using these contracts.
