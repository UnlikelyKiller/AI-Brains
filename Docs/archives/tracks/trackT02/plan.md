# Implementation Plan: Track T02 - Event Contracts

### Phase 1: Crate Setup and Core Types
- [ ] Task 1.1: Initialize `ai-brains-events` crate and add `Cargo.toml` dependencies (`ai-brains-core`, `serde`, `serde_json`, `sha2`, `time`, `uuid`, `thiserror`).
- [ ] Task 1.2: Create module structure (`lib.rs`, `actor.rs`, `aggregate.rs`, `errors.rs`).
- [ ] Task 1.3: Define `Actor` and `Aggregate` enums/structs in `actor.rs` and `aggregate.rs`.
- [ ] Task 1.4: Define the `EventError` enum in `errors.rs`.

### Phase 2: Event Definitions and Security Constraints
- [ ] Task 2.1: Define `EventKind` enum in `event_kind.rs` representing all domain events. Ensure no hidden thinking/tool call variants exist.
- [ ] Task 2.2: Implement structs for all event payloads in `payload.rs`.
- [ ] Task 2.3: Implement `Envelope` struct in `envelope.rs` enforcing `Privacy` requirement on every event.
- [ ] Task 2.4: Implement `constructors.rs` to provide safe builder methods for Envelopes that mandate Privacy context.

### Phase 3: Hashing, Versioning, and Upcasting
- [ ] Task 3.1: Implement stable payload hashing in `hash.rs` using `sha2` (canonical JSON sorting).
- [ ] Task 3.2: Implement versioning enums/constants in `version.rs`.
- [ ] Task 3.3: Provide an upcasting trait/strategy in `upcast.rs` for future schema upgrades.

### Phase 4: Test Driven Verification
- [ ] Task 4.1: Write `tests/no_hidden_thinking_event_kind.rs` and `tests/no_tool_call_event_kind.rs` to explicitly statically assert/reflect on variants.
- [ ] Task 4.2: Write `tests/no_mutating_event_payloads.rs` to ensure payloads only contain value types (or use clippy configs to assert).
- [ ] Task 4.3: Write `tests/privacy_included_on_every_event.rs`.
- [ ] Task 4.4: Write `tests/envelope_hash_stable.rs` to guarantee hashing doesn't change due to struct field reordering or random JSON map orders.
- [ ] Task 4.5: Write `tests/event_roundtrip.rs` for serialize/deserialize flow.
- [ ] Task 4.6: Write `tests/event_upcast_unknown_future_event.rs` to test fallback degradation handling.

### Phase 5: Conductor Review and Cleanup
- [ ] Task 5.1: Run `cargo clippy` and `cargo test` on `ai-brains-events`.
- [ ] Task 5.2: Ensure all Phase Gates align with Conductor checklist.
