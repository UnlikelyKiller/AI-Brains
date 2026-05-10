## Plan: T09 Security Scanning

### Phase 1: Crate Setup
- [x] Task 1.1: Replace the crate stub with scanner modules and public API.
- [x] Task 1.2: Add dependencies for core privacy, regex matching, and typed errors.

### Phase 2: Detection and Policy
- [x] Task 2.1: Implement secret patterns and typed findings.
- [x] Task 2.2: Implement privacy escalation.
- [x] Task 2.3: Implement redaction and embedding eligibility policy.

### Phase 3: Verification
- [x] Task 3.1: Add the required tests.
- [x] Task 3.2: Run `cargo test -p ai-brains-security`.
- [x] Task 3.3: Run `cargo clippy -p ai-brains-security --all-targets -- -D warnings`.
