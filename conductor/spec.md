# Technical Specification: T01 - Core Domain

## Objectives
Implement the pure domain model for AI-Brains within the `ai-brains-core` crate.

## Architecture
- Pure Rust workspace crate.
- Follows Event Sourcing principles for type definitions.
- Strict security invariants: No hidden thinking capture roles, no tool call capture roles.

## Files Owned
`crates/ai-brains-core/*`

## Dependencies
- Allowed: `serde`, `uuid`, `time`, `thiserror`.
- Prohibited: Database, CLI, daemon, HTTP, filesystem, Git, model providers.

## Required Modules
`lib.rs`, `ids.rs`, `clock.rs`, `privacy.rs`, `project.rs`, `user.rs`, `device.rs`, `harness.rs`, `session.rs`, `turn.rs`, `memory.rs`, `conflict.rs`, `recipe.rs`, `status.rs`, `validation.rs`, `errors.rs`

## Required Tests
- `id_serde_roundtrip`
- `privacy_strictest_wins`
- `session_status_transitions`
- `no_thinking_role_exists`
- `no_tool_call_role_exists`
- `domain_validation_rejects_empty_content`
