# Plan: T34 Resilient Summarization Truncation

### Phase 1: Model Provider Upgrades
- [ ] Task 1.1: Add `tokenize` method to `ModelProvider` trait in `ai-brains-models`.
- [ ] Task 1.2: Implement `LlamaCppProvider::tokenize` using the `/tokenize` endpoint.
- [ ] Task 1.3: Implement `estimate_tokens` with 3.5 char/token fallback.

### Phase 2: Sequential Chunking Logic
- [ ] Task 2.1: Implement `SessionSplitter` to divide turns into budgeted chunks.
- [ ] Task 2.2: Implement sequential loop for chunk processing in `NightlyService`.
- [ ] Task 2.3: Implement "Context Carryover" to pass previous summary to next chunk.
- [ ] Task 2.4: Ensure turn boundaries and overlap are handled correctly.
- [ ] Task 2.5: Write unit tests for sequential splitting and context transfer.

### Phase 3: Service Integration
- [ ] Task 3.1: Integrate truncation into `NightlyService::summarize_session`.
- [ ] Task 3.2: Add logging/tracing when truncation occurs.
- [ ] Task 3.3: Verify with mock provider.

### Phase 4: Verification & Hardening
- [ ] Task 4.1: `cargo fmt --check` passes.
- [ ] Task 4.2: `cargo clippy --workspace --all-targets -- -D warnings` clean.
- [ ] Task 4.3: `cargo test --workspace` all pass.
- [ ] Task 4.4: Manual verification: Run `ai-brains nightly` and verify the large session (`0e85a3e0...`) now summarizes without error.

### Phase 5: Documentation
- [ ] Task 5.1: Update `Docs/status.md` with T34 entry.
- [ ] Task 5.2: Update `Docs/conductor/conductor.md`.
- [ ] Task 5.3: Update `Docs/Deviations.md` with T34 architectural choices.
