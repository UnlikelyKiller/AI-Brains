## Plan: T32 Preflight ANSI Cleanup, Deduplication, and Hotspot Condensation

### Phase 1: ANSI Stripping Utility
- [x] Task 1.1: Add `strip-ansi-escapes` to workspace and crate Cargo.toml files
- [x] Task 1.2: Create `crates/ai-brains-retrieval/src/ansi.rs` with `strip_ansi()` wrapper
- [x] Task 1.3: Register `ansi` module in `crates/ai-brains-retrieval/src/lib.rs`
- [x] Task 1.4: Write failing test `preflight_strips_ansi.rs` (Red)
- [x] Task 1.5: Verify test fails (ANSI codes pass through unchanged)

### Phase 2: Preflight ANSI Stripping (Defense-in-Depth)
- [x] Task 2.1: Apply `strip_ansi()` in `build_preflight()` for both safety and index content
- [x] Task 2.2: Verify ANSI stripping test passes (Green)

### Phase 3: Deduplication Between Safety and Index
- [x] Task 3.1: Write failing test `preflight_deduplicates.rs` (Red)
- [x] Task 3.2: Modify `build_preflight()` to collect safety `memory_id`s and skip them in index
- [x] Task 3.3: Verify deduplication test passes (Green)

### Phase 4: Hotspot Condensation and Pin-Time Stripping
- [x] Task 4.1: Create `crates/ai-brains-cli/src/hotspot.rs` with `sanitize_and_condense()` and tests
- [x] Task 4.2: Register `hotspot` module in `main.rs`
- [x] Task 4.3: Modify `run_safety_sync()` to call `sanitize_and_condense()` before pinning
- [x] Task 4.4: Run CLI unit tests (6 new tests pass)

### Phase 5: Verification
- [x] Task 5.1: `cargo fmt --check` passes
- [x] Task 5.2: `cargo clippy --workspace --all-targets -- -D warnings` clean
- [x] Task 5.3: `cargo test --workspace` all pass
- [x] Task 5.4: `ai-brains preflight --max-words 1500` produces clean output (no ANSI, no duplicates)

### Phase 6: Conductor Documentation
- [x] Task 6.1: Create track directory and spec.md
- [x] Task 6.2: Create plan.md
- [ ] Task 6.3: Update conductor.md with T32 entry
- [ ] Task 6.4: Update status.md

### Phase 7: Hook Deployment
- [ ] Task 7.1: Copy repo `target-claude-hook.ps1` to install location
- [ ] Task 7.2: Verify preflight output is clean with deployed hook