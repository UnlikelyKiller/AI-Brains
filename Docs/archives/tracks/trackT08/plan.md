## Plan: T08 Git Metadata

### Phase 1: Crate Setup
- [x] Task 1.1: Replace the default crate stub with the Git metadata module structure.
- [x] Task 1.2: Add crate dependencies for typed errors and stable hashing.

### Phase 2: Repository Discovery
- [x] Task 2.1: Implement repository root discovery.
- [x] Task 2.2: Implement bounded Git command execution helpers.

### Phase 3: Metadata Readers
- [x] Task 3.1: Implement branch and commit readers.
- [x] Task 3.2: Implement remote URL hashing.
- [x] Task 3.3: Implement dirty status and bounded untracked filename parsing.
- [x] Task 3.4: Implement summary-only diffstat collection.

### Phase 4: Verification
- [x] Task 4.1: Add the required Git metadata integration tests.
- [x] Task 4.2: Run `cargo test -p ai-brains-git`.
- [x] Task 4.3: Run `cargo clippy -p ai-brains-git --all-targets -- -D warnings`.
