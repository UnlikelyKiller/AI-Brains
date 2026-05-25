# Plan: Track T52 - Nightly Resilience & Async Alignment

- [ ] **Phase 1: Async Refactoring**
    - [ ] Change `commands::nightly::run` signature to `pub async fn run`.
    - [ ] Update `crates/ai-brains-cli/src/main.rs` to `.await` the nightly run.
    - [ ] Replace `tokio::runtime::Runtime::new()` with direct calls to async functions.

- [ ] **Phase 2: Lifecycle & Auto-Start**
    - [ ] Call `DaemonClient::ensure_running` at the start of `nightly::run`.
    - [ ] Verify that summarization captures are correctly spooled if the daemon was just started.

- [ ] **Phase 3: Verification**
    - [ ] Run `ai-brains nightly` manually and verify no panic occurs.
    - [ ] Check `recall` to ensure new summaries are present.
