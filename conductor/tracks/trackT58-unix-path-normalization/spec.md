# Track T58: Fix Unix Absolute Path Normalization

## Context

The `mapping_delta_smoke` test fails because `normalize_project_path()` in `crates/ai-brains-path/src/canonical.rs` only recognizes Windows-style absolute paths (UNC `\\` and drive-letter `C:\`). It incorrectly rejects Unix absolute paths starting with `/` as "relative paths".

This prevents `agy-hook` (Antigravity CLI integration) from working correctly on WSL/Linux when transcript paths are absolute, which is the standard behavior of the Antigravity CLI.

## Root Cause

In `canonical.rs` line 33-43:
```rust
} else {
    let mut abs = std::env::current_dir().map_err(|e| PathError::IoError(e.to_string()))?;
    abs.push(trimmed);
    let abs_str = abs.to_string_lossy().to_string();
    if is_unc_path(&abs_str) { ... }
    else if has_drive_prefix(&abs_str) { ... }
    else {
        return Err(PathError::RelativePath(trimmed.to_string()));
    }
}
```

When `trimmed` is a Unix absolute path (e.g., `/tmp/foo.json`), `abs.push(trimmed)` replaces the PathBuf with `/tmp/foo.json`. Then the code checks for UNC and drive prefix — both false on Linux — and falls to the `RelativePath` error.

The error message is misleading: the path IS absolute, but the code can't recognize it.

## Requirements

1. **R1**: `normalize_project_path()` must correctly handle Unix absolute paths (starting with `/`).
2. **R2**: The fix must not break existing Windows/WSL path normalization behavior.
3. **R3**: All existing tests must continue to pass.
4. **R4**: `mapping_delta_smoke` test must pass after the fix.

## Technical Design

### Option A: Add Unix absolute path check
Add an `else if abs_str.starts_with('/')` branch after the `has_drive_prefix` check to accept Unix absolute paths as-is.

**Pros**: Minimal change, targeted fix, preserves all existing behavior.
**Cons**: None identified.

### Option B: Introduce `is_unix_absolute` helper
Extract a helper function for clarity.

**Pros**: More explicit.
**Cons**: Overkill for a single condition.

**Decision**: Option A — inline check is simplest and most maintainable.

## Files to Modify

- `crates/ai-brains-path/src/canonical.rs` — add Unix absolute path handling

## Verification Plan

1. Run `cargo test -p ai-brains-path` — all path unit tests pass
2. Run `cargo test -p ai-brains-cli --test mapping_delta_smoke` — smoke test passes
3. Run `cargo test -p ai-brains-store -p ai-brains-brain -p ai-brains-cli` — full suite passes
4. Run `cargo clippy -p ai-brains-path -- -D warnings` — clean
