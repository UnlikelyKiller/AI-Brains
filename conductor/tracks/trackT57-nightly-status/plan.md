# Plan: Track T57 - Nightly --status Subcommand

- [x] **Phase 1: Conductor Setup**
  - [x] Read `conductor.md` to confirm next track is T57.
  - [x] Create `conductor/tracks/trackT57-nightly-status/spec.md`.
  - [x] Create `conductor/tracks/trackT57-nightly-status/plan.md`.

- [x] **Phase 2: Store Trait Update**
  - [x] Add `get_last_nightly_run()` to `QueryStore` trait in `ai-brains-store/src/lib.rs`.
  - [x] Implement `get_last_nightly_run()` on `VaultConnection` in `ai-brains-store/src/query_store.rs`.

- [x] **Phase 3: Brain Service Update**
  - [x] At end of `NightlyService::run_nightly`, write `last_nightly_run`, `last_nightly_count` via `event_store.set_sync_state(...)`.

- [x] **Phase 4: CLI Command Update**
  - [x] Add `status: bool` to `Nightly` variant in `ai-brains-cli/src/main.rs`.
  - [x] Pass `status` into `commands::nightly::run(...)`.
  - [x] Implement `--status` read-only report in `commands/nightly.rs`.

- [x] **Phase 5: CI Gate**
  - [x] `cargo clippy -p ai-brains-cli -p ai-brains-store -p ai-brains-brain -- -D warnings` — PASSED
  - [x] `cargo test -p ai-brains-store -p ai-brains-brain -p ai-brains-cli` — PASSED (22 tests green; 1 pre-existing bridge failure unrelated)
  - [ ] `cargo fmt --check` — SKIPPED (pre-existing CRLF in repo)

- [x] **Phase 6: ChangeGuard Ledger**
  - [x] Branch created, changes committed, pushed to origin.

- [x] **Phase 7: Codex Review**
  - [x] Review completed. Findings addressed (see commit message).

- [x] **Phase 8: Completion**
  - [x] Branch pushed: track-t57-nightly-status
  - [x] Update `conductor/conductor.md` registry (status = Completed) — in this commit.
