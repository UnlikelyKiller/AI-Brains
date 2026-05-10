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
- **Fallback**: 1 token ≈ 3.5 characters (conservative).
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
