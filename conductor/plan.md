## Plan: T01 Core Domain
### Phase 1: Setup and Infrastructure
- [ ] Task 1.1: Configure `Cargo.toml` with dependencies (`serde`, `uuid`, `time`, `thiserror`).
- [ ] Task 1.2: Set up `src/lib.rs` and core module files.

### Phase 2: Domain Types Implementation
- [ ] Task 2.1: Implement `ids.rs` with Serde support.
- [ ] Task 2.2: Implement `clock.rs` for unified time.
- [ ] Task 2.3: Implement `privacy.rs` modeling escalation.
- [ ] Task 2.4: Implement `status.rs` and transitions.

### Phase 3: Core Entities Implementation
- [ ] Task 3.1: Implement `project.rs`, `user.rs`, `device.rs`, `harness.rs`.
- [ ] Task 3.2: Implement `session.rs`, `turn.rs`, `memory.rs`.
- [ ] Task 3.3: Implement `conflict.rs`, `recipe.rs`.

### Phase 4: Validation and Errors
- [ ] Task 4.1: Implement `validation.rs` rejecting empty content and enforcing security rules.
- [ ] Task 4.2: Implement `errors.rs` using `thiserror`.

### Phase 5: Testing
- [ ] Task 5.1: Write tests for `id_serde_roundtrip`, `privacy_strictest_wins`, `session_status_transitions`.
- [ ] Task 5.2: Write tests for `no_thinking_role_exists`, `no_tool_call_role_exists`.
- [ ] Task 5.3: Write test for `domain_validation_rejects_empty_content`.
