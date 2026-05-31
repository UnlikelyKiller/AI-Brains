# Track T58 Plan: Fix Unix Absolute Path Normalization

## Phase 1: Patch canonical.rs
Add `else if abs_str.starts_with('/')` branch in `normalize_project_path()` to handle Unix absolute paths.

## Phase 2: Compile Check
`cargo check -p ai-brains-path`

## Phase 3: Run Tests
- `cargo test -p ai-brains-path`
- `cargo test -p ai-brains-cli --test mapping_delta_smoke`
- `cargo test -p ai-brains-store -p ai-brains-brain -p ai-brains-cli`

## Phase 4: Clippy
`cargo clippy -p ai-brains-path -- -D warnings`

## Phase 5: Register in conductor.md
Add T58 entry to track registry.

## Phase 6: Branch + Commit + Push
Branch: `track-t58-unix-path-normalization`
