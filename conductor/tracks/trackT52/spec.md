# Track T52: Nightly Resilience & Async Alignment

## Objective
Fix the `nightly` command's runtime panic and ensure all background intelligence tasks are correctly aligned with the CLI's async lifecycle.

## Problem Statement
The `ai-brains nightly` command currently panics with "Cannot start a runtime from within a runtime" because it attempts to create a new `tokio::runtime::Runtime` while already executing inside a `#[tokio::main]` block. Additionally, the nightly sweep doesn't ensure the daemon is running, which can lead to missed captures of summarized turns.

## Requirements
- **Async Alignment**: Refactor `crates/ai-brains-cli/src/commands/nightly.rs` to use the existing tokio runtime handle (via `tokio::spawn` or by making the inner functions async).
- **Auto-Start**: Ensure `DaemonClient::ensure_running` is called before the intelligence sweep begins.
- **Progress Visibility**: Maintain the existing progress markers while operating within the unified runtime.

## Technical Design
- Update `commands::nightly::run` to be an `async fn`.
- Remove `Runtime::new()?` and `tokio_runtime.block_on(...)`.
- Update `main.rs` to await the `nightly::run` call.

## Verification Plan
- **Reproduction**: Run `ai-brains nightly` and verify it no longer panics.
- **Integration**: Verify that the sweep successfully summarizes a session and that summarized turns appear in the recall index.
