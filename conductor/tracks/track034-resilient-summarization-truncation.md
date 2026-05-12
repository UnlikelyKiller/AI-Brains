# Spec: T34 Resilient Summarization Truncation

## 1. Goal
Ensure the `NightlyService` can summarize sessions of any length by intelligently truncating conversation history that exceeds the model's context window.

## 2. Technical Requirements

### 2.1 Context Budget
- **Limit**: Dynamically determined from `AI_BRAINS_CTX_SIZE` env var, falling back to **38,912** (Stable Intel Arc B580 VRAM threshold).
- **Overhead Buffer**: Reserve **1,412 tokens** for system instructions, session rules, and response generation.
- **Effective Budget**: **37,500 tokens**.

### 2.2 Token Estimation
- **Primary**: Use `llama.cpp` `/tokenize` endpoint via `ai-brains-models`.
- **Fallback**: 1 token â‰ˆ 3.5 characters (conservative).
- **Hardening**: If estimation is within 10% of the budget, apply an additional 5% "safety margin" truncation.

### 2.3 Summarization Strategy: Sequential Chunking
- **Primary Logic**:
  1. If tokens <= effective_budget: Process as a single session.
  2. If tokens > effective_budget:
     - Split turns into `K` sequential chunks (respecting turn boundaries).
     - **Process Sequentially**:
       - Summarize Chunk 1. Output: `Summary 1`.
       - Summarize Chunk 2, providing `Summary 1` as "Previous Context" in the prompt.
       - Continue until Chunk K is processed.
       - Synthesize all partial results into the final `Session Summary`.
- **Overhead Handling**: Ensure `Previous Context` from the prior chunk is factored into the budget for the current chunk.
- **Context Preservation**: The sequential nature ensures the LLM maintains a "chain of thought" across extremely large logs.

### 2.4 Hardening & Resilience
- **Turn Integrity**: Never split a single turn unless it ALONE exceeds the context window.
- **Sequential Stability**: Since `parallel = 1` is an environment constraint, sequential processing is natively compatible with the hardware.
- **Metadata Tagging**: Explicitly label chunks as "Part X of Y" to aid LLM orientation.

## 3. Implementation Details
- `crates/ai-brains-brain/src/lib.rs`: Modify `summarize_session`.
- Add a dedicated `truncation.rs` or private helper functions.

## 4. Definition of Done (DoD)
- [ ] No "400 Bad Request" context-exceeded errors during `nightly` runs.
- [ ] Large sessions (e.g., 50k+ tokens) are summarized successfully.
- [ ] Truncation respects turn boundaries.
- [ ] Truncation marker is clearly visible in the prompt if truncation occurred.
- [ ] Unit tests cover various truncation scenarios.
- [ ] CI Gate passes (fmt, clippy, tests).

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

