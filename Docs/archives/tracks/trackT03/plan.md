# Plan: JSON Contracts

### Phase 1: Setup and Foundation DTOs
- [ ] Task 1.1: Initialize `ai-brains-contracts` module structure (lib.rs, response.rs, errors.rs).
- [ ] Task 1.2: Add `serde`, `serde_json`, and `ai-brains-core` to Cargo.toml.
- [ ] Task 1.3: Implement generic API envelope (`ApiResult<T>`) and generic `ApiError`.
- [ ] Task 1.4: Write test `api_response_shape.rs`.

### Phase 2: Core Domain DTOs
- [ ] Task 2.1: Implement session and memory DTOs (`sessions.rs`, `memory.rs`).
- [ ] Task 2.2: Implement ingest and recall queries (`ingest.rs`, `recall.rs`).
- [ ] Task 2.3: Write test `ingest_request_shape.rs`.

### Phase 3: Infrastructure and System DTOs
- [ ] Task 3.1: Implement preflight, doctor, and version contracts (`preflight.rs`, `doctor.rs`, `version.rs`).
- [ ] Task 3.2: Implement project and backup definitions (`projects.rs`, `backup.rs`).
- [ ] Task 3.3: Write test `preflight_response_shape.rs`.

### Phase 4: Hook Contracts and Compatibility
- [ ] Task 4.1: Implement hook request/response objects (`hook.rs`). Ensure the structure strictly parses inner JSON payloads, guarding against string stdout noise.
- [ ] Task 4.2: Write test `hook_response_has_no_stdout_noise_fields.rs`.
- [ ] Task 4.3: Write test `contracts_are_backward_compatible.rs` to validate serialization snapshots across all domains.
- [ ] Task 4.4: Ensure crate complies with `cargo clippy` and formats properly.
