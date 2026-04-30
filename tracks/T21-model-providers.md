# Track T21 - Model Providers

## Owner
architecture-planner

## Status
Implemented / Verification and Provenance Reconciliation Pending

## Objective
Implement a provider-agnostic model abstraction with initial support for local Ollama and a mock for testing.

## Scope
- Scaffold `ai-brains-models` crate.
- Define `ModelProvider` trait (completion and embedding).
- Implement `OllamaProvider` using `reqwest`.
- Implement `MockProvider` for testing.
- Implement `ProviderRegistry` to handle fallback and privacy-aware routing.

## Out of Scope
- Cloud providers (OpenAI, Anthropic) - focus on local first.
- Complex orchestration (AutoGPT style).

## Files Owned
- `crates/ai-brains-models/**`

## Files Allowed To Touch
- `Cargo.toml`
- `Docs/conductor/conductor.md`
- `Docs/status.md`

## Files Forbidden To Touch
- `crates/ai-brains-core/**`
- `crates/ai-brains-events/**`

## Public Contracts Consumed
- `ai_brains_core::privacy::Privacy`

## Public Contracts Produced
- `ai_brains_models::ModelProvider`
- `ai_brains_models::CompletionRequest`
- `ai_brains_models::EmbeddingRequest`

## Required Tests First
- `tests/ollama_provider_returns_mocked_completion.rs`
- `tests/registry_respects_privacy_gate.rs`

## Implementation Steps
1. [x] Scaffold `ai-brains-models` crate and add to workspace.
2. [x] Define core traits and DTOs in `lib.rs`.
3. [x] Implement `ollama.rs` (local only).
4. [x] Implement `registry.rs` for provider selection.
5. [x] Implement `mock.rs` for tests.
6. [x] Verification and CI gate reconciliation for degraded Windows workspace.

## Failure Modes To Handle
- Provider offline (actionable error).
- Timeout.
- Rate limit (local rate limit logic).

## Security Requirements
- No transmission of `local_only` memory to cloud providers (enforced by registry).
- API keys stored in vault or environment (never in code).

## Acceptance Criteria
- Registry returns an error if a cloud provider is requested for `local_only` data.
- Ollama provider successfully communicates with a local endpoint (mocked in tests).
- CI pass with clippy and nextest.

## Handoff Notes
- `ModelProvider`, `OllamaProvider`, `MockProvider`, `ProviderRegistry`, and local llama.cpp routing artifacts are present.
- Focused tests exist for mocked Ollama completion and privacy-aware registry selection.
- Degraded Windows verification passes with `cargo clippy --workspace --all-targets --exclude ai-brains-graph -- -D warnings` and `cargo test --workspace --exclude ai-brains-graph`.
- Full all-target verification remains blocked by the graph crate's documented LadybugDB/MSVC debug linker issue.
- ChangeGuard still reports a stale pending transaction for `crates/ai-brains-models`; provenance must be reconciled before this track is closed.
