# Track T57: Nightly --status Subcommand

## Objective
Add a read-only `--status` flag to the `nightly` CLI subcommand that reports the current state of the nightly intelligence sweep without triggering any work.

## Problem Statement
Users and automation have no visibility into:
- When the last nightly sweep ran
- How many sessions remain unsummarized
- How many sessions were summarized in the last run
- Whether the last run encountered errors

## Requirements
- **Read-only**: `--status` must not write to the event store or trigger model calls.
- **Last Run Timestamp**: Read from `sync_state` key `last_nightly_run` (written by `run_nightly` on completion).
- **Unsummarized Count**: `get_unsummarized_sessions().len()`.
- **Last Run Summary Count**: Stored in `sync_state` key `last_nightly_count`.
- **Last Run Errors**: Stored in `sync_state` key `last_nightly_errors` (newline-separated error strings, or `"none"`).
- **Output Format**: Human-readable plain text for now (JSON-friendly structure via `println!`).

## Technical Design
1. `crates/ai-brains-store/src/lib.rs`
   - Add `fn get_last_nightly_run(&self) -> Result<Option<String>>;` to `QueryStore` trait.
   - Implement on `VaultConnection` by reading `sync_state` key `last_nightly_run`.
2. `crates/ai-brains-brain/src/lib.rs`
   - At the end of `run_nightly`, write:
     - `last_nightly_run` → RFC3339 timestamp
     - `last_nightly_count` → count of successfully summarized sessions
     - `last_nightly_errors` → newline-separated error messages, or `"none"` if zero errors.
3. `crates/ai-brains-cli/src/commands/nightly.rs`
   - Add `status` parameter to `run(...)`.
   - When `status` is true, query `get_unsummarized_sessions()`, `get_last_nightly_run()`, and `get_sync_state(...)` for count/errors, then print a formatted report and return `Ok(())`.
4. `crates/ai-brains-cli/src/main.rs`
   - Add `--status` bool flag to the `Nightly` command variant.
   - Pass it through to `commands::nightly::run`.

## Verification Plan
- Run `cargo fmt --check`.
- Run `cargo clippy --workspace --all-targets -- -D warnings`.
- Run `cargo nextest run --workspace`.
- Manual check: `ai-brains nightly --status` prints expected fields without running a sweep.
