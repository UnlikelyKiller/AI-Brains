# Plan: Track T53 - Daemon Lifecycle & Global Install UX

- [ ] **Phase 1: Daemon API & Shutdown Handler**
    - [ ] Add `Shutdown` variant to `DaemonRequest` in `ai-brains-daemon-api`.
    - [ ] Implement shutdown logic in `ai-brainsd/src/main.rs` to break the loop and exit.

- [ ] **Phase 2: CLI Command Implementation**
    - [ ] Add `daemon` subcommand to CLI with `stop` nested command.
    - [ ] Implement IPC call to send the `Shutdown` request.
    - [ ] Add `--force` flag logic (using `taskkill` as fallback if signal fails).

- [ ] **Phase 3: Verification**
    - [ ] Verify `ai-brains daemon stop` correctly terminates the process.
    - [ ] Verify `cargo install` works immediately after stop.
