# Track T59: Fix Pre-existing Test Failures

## Context

Three pre-existing test failures were present before T57/T58. These tests were broken due to:
1. Test-code drift after CLI began requiring `--vault-path`
2. Windows-only API usage on cross-platform code
3. Outdated assertions after relative path support was added

## Root Causes

### smoke.rs::test_cli_context_idempotency
The `context` command requires a vault path via `AppContext::from_cli()`, but the test omitted `--vault-path`. The CLI changed at some point to require it, but the test was never updated.

### symlink_resolution_best_effort.rs
Directly called `std::os::windows::fs::symlink_dir()` without `#[cfg(windows)]`. Fails to compile on Linux/WSL.

### malformed_paths_return_error_not_panic::relative_paths_now_supported
Test asserted `canonical().ends_with("relative\\path")` — hardcoded Windows backslash. After T58 made relative paths actually work (resolving against `current_dir`), the canonical path is absolute and platform-dependent.

## Changes

- `crates/ai-brains-cli/tests/smoke.rs` — add `--vault-path` + `init` to all `context` invocations
- `crates/ai-brains-path/tests/symlink_resolution_best_effort.rs` — platform-specific symlink creation via `#[cfg]`
- `crates/ai-brains-path/tests/malformed_paths_return_error_not_panic.rs` — cross-platform assertion using `.contains()` instead of `.ends_with()` with hardcoded backslash

## Verification

Full suite: `cargo test -p ai-brains-path -p ai-brains-store -p ai-brains-brain -p ai-brains-cli` — all pass, zero failures.
