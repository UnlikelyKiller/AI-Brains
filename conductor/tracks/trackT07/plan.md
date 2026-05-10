## Plan: T07 Path Normalization

### Phase 1: Crate Setup
- [x] Task 1.1: Replace the default crate stub with the path-normalization module structure.
- [x] Task 1.2: Add crate dependencies needed for typed errors and any path helpers that remain license-safe.

### Phase 2: Canonical Windows Identity
- [x] Task 2.1: Implement drive-case normalization.
- [x] Task 2.2: Implement slash normalization for equivalent Windows paths.
- [x] Task 2.3: Implement extended-length path normalization.

### Phase 3: Cross-Environment Mapping
- [x] Task 3.1: Implement WSL `/mnt/<drive>` to Windows path mapping.
- [x] Task 3.2: Preserve and normalize UNC paths safely.

### Phase 4: Best-Effort Resolution and Display
- [x] Task 4.1: Implement best-effort symlink and alias handling.
- [x] Task 4.2: Implement display-path helpers distinct from canonical storage output.
- [x] Task 4.3: Ensure malformed input returns typed errors, not panics.

### Phase 5: Verification
- [x] Task 5.1: Add the required path-normalization integration tests.
- [x] Task 5.2: Run `cargo test -p ai-brains-path`.
- [x] Task 5.3: Run `cargo clippy -p ai-brains-path --all-targets -- -D warnings`.
