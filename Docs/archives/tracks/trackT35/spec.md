# Track T35: CLI Refactor (Decomposing main.rs God File)

## Overview
Currently, `crates/ai-brains-cli/src/main.rs` is a ~900-line God File that manually instantiates providers and services, defines all Subcommands inline, and houses complex procedural logic for tasks like Antigravity import. The objective of this track is to decompose `main.rs`, introduce an `AppContext` for shared state, and extract domain logic (like Antigravity imports) into appropriate adapter or brain modules.

## Architecture & Design
1. **CLI Commands Sub-modules:**
   - Create `crates/ai-brains-cli/src/commands/mod.rs`
   - Split commands into `commands/init.rs`, `commands/ingest.rs`, `commands/recall.rs`, `commands/preflight.rs`, `commands/nightly.rs`, `commands/backup.rs`, `commands/forget.rs`, `commands/stop_session.rs`, `commands/context.rs`, `commands/pin.rs`, `commands/safety.rs`, `commands/antigravity_import.rs`.
   - Update `main.rs` to only handle `clap` parsing and routing to sub-modules.

2. **AppContext / CliState:**
   - Introduce `crates/ai-brains-cli/src/context.rs` (or similar) defining `AppContext`.
   - `AppContext` handles `VaultConnection` pool, configuration parsing (e.g., `AI_BRAINS_VAULT_PATH`, `AI_BRAINS_KEY`), and service dependencies (like `CaptureService`, `EventStore`, `ModelProvider`).
   - Command handlers will accept `&AppContext` (or `&mut AppContext`) to perform their tasks.

3. **Antigravity Import Logic:**
   - Move the procedural Antigravity session discovery and import logic out of `main.rs`.
   - Re-home this logic into `crates/ai-brains-adapters/src/antigravity.rs` (or similar).
   - Ensure the CLI command merely delegates to this new adapter/brain function.

## Constraints & Rules
- **No Behavioral Changes:** This is purely a refactoring track. Do not change existing command line arguments, JSON formats, or external behavior.
- **Test Integrity:** All existing tests (e.g., `smoke.rs`, `ingest_reads_json_stdin.rs`) must continue to pass.
- **Safety First:** Avoid `unwrap()`, `expect()`, or `panic()`. Use the existing `anyhow` or `thiserror` patterns.
- **Workspace Verification:** Run `cargo clippy` and `cargo nextest run` to ensure structural integrity post-refactor.