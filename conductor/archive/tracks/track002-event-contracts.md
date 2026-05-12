# Track T02 — Event Contracts

## Owner
architecture-planner

## Status
Completed

## Objective
Define immutable events and event envelopes for AI-Brains in the `ai-brains-events` crate.

## Scope
- Define the event envelope structure.
- Define Actor and Aggregate identities.
- Define explicit Event Kinds (SystemInitialized, SessionStarted, UserPromptRecorded, AssistantFinalRecorded, SessionCompleted, SessionFailed, MemoryPinned, MemoryForgotten, PrivacyEscalated, NightlyJobStarted, ConflictDetected, RecipePromoted, etc.).
- Define event payloads (immutable structs).
- Implement stable hashing for event envelopes to ensure tamper detection.
- Implement event upcasting patterns for forward/backward compatibility.

## Out of Scope
- Database persistence or SQL implementations.
- REST/JSON API definitions (handled in T03).
- Model-specific structs (Claude, Gemini, etc.).
- Any mutability in events.

## Files Owned
`crates/ai-brains-events/*`

## Files Allowed To Touch
`crates/ai-brains-events/src/lib.rs`
`crates/ai-brains-events/src/envelope.rs`
`crates/ai-brains-events/src/actor.rs`
`crates/ai-brains-events/src/aggregate.rs`
`crates/ai-brains-events/src/event_kind.rs`
`crates/ai-brains-events/src/payload.rs`
`crates/ai-brains-events/src/constructors.rs`
`crates/ai-brains-events/src/hash.rs`
`crates/ai-brains-events/src/version.rs`
`crates/ai-brains-events/src/upcast.rs`
`crates/ai-brains-events/src/errors.rs`
`crates/ai-brains-events/tests/*.rs`
`crates/ai-brains-events/Cargo.toml`
`Docs/conductor/trackT02/spec.md`
`Docs/conductor/trackT02/plan.md`
`Docs/conductor/conductor.md`

## Files Forbidden To Touch
Any file outside `crates/ai-brains-events/` and the conductor planning docs.

## Public Contracts Consumed
- `ai-brains-core` (IDs, Privacy, Status, Time structs)

## Public Contracts Produced
- Event Envelopes and Payloads representing the immutable source of truth for the system.

## Required Tests First
- `tests/envelope_hash_stable.rs`
- `tests/event_roundtrip.rs`
- `tests/event_upcast_unknown_future_event.rs`
- `tests/no_mutating_event_payloads.rs`
- `tests/no_hidden_thinking_event_kind.rs`
- `tests/no_tool_call_event_kind.rs`
- `tests/privacy_included_on_every_event.rs`

## Implementation Steps
1. Add dependencies to `ai-brains-events/Cargo.toml` (`ai-brains-core`, `serde`, `serde_json`, `sha2`, `time`, `uuid`).
2. Create standard `Actor` and `Aggregate` definitions to map events to domains.
3. Define `EventKind` enum listing all domain events.
4. Define strong `Payload` structs for each event.
5. Create `Envelope` struct containing aggregate info, version, event kind, payload, and privacy.
6. Implement stable SHA-256 hashing in `hash.rs` over the serialized payload.
7. Provide upcast strategies to migrate older event schemas in `upcast.rs`.
8. Implement builder/constructor utilities in `constructors.rs` ensuring required privacy parameters.

## Failure Modes To Handle
- Unrecognized event kinds gracefully upcasted or degraded.
- Payload hashing mismatch (tampering detection).
- Deserialization failures of strict properties.

## Security Requirements
- Every event envelope MUST include a `Privacy` level.
- No payload can mutate; all fields are purely data representations.
- Event Kinds for hidden thinking, chain of thought, or raw tool logs MUST NOT exist.
- Event hashing must be stable across serialization mechanisms (e.g. predictable JSON sorting).

## Acceptance Criteria
- All tests pass (including forbidden checks like `no_hidden_thinking_event_kind`).
- Hashing is proven stable.
- Events easily integrate with `ai-brains-core`.

## Commands To Run
`cargo test -p ai-brains-events`
`cargo clippy -p ai-brains-events -- -D warnings`

## Handoff Notes
T03 will use these events to define JSON ingress contracts, ensuring CLI/Daemon matches these immutable definitions.
