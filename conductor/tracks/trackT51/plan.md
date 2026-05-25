# Plan: Track T51 - Daemon Auto-Launch & Bridge Silence

- [x] **Phase 1: Research & Scoping**
    - [x] Identify all commands in `ai-brains-cli` that currently require the daemon.
    - [x] Verify `ai-brainsd` binary availability on the system path during development.

- [x] **Phase 2: Daemon Client Implementation**
    - [x] Implement `spawn_daemon()` in `crates/ai-brains-cli/src/daemon_client.rs`.
        - [x] Handle Windows-specific background process spawning.
    - [x] Implement `ensure_running()` with retry logic.

- [x] **Phase 3: Integration & UX**
    - [x] Update `crates/ai-brains-cli/src/commands/sync.rs` (`run_query`) to use `ensure_running()`.
    - [x] Add `quiet` parameter to `run_query` and corresponding flag to `SyncCommands::Query`.
    - [x] Update `recall` command to optionally auto-start or improve error messaging. (Note: Recall is local-first, sync query updated).

- [x] **Phase 4: Verification**
    - [x] Verify auto-start works from a fresh terminal without `ai-brainsd` running.
    - [x] Verify `--quiet` suppresses error output when the daemon binary is missing or fails to start.
    - [x] Run full CI gate: `cargo fmt ; cargo clippy ; cargo test`.

- [ ] **Phase 5: Documentation & Handover**
    - [ ] Update `AGENTS.md` or `SKILL.md` if auto-start behavior changes recommended usage.
    - [ ] Close track in `conductor/conductor.md`.
