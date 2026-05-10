---
name: tooling
description: Use this skill when using Sourcebot, GitHub CLI, or ChangeGuard for research, CI verification, or codebase exploration. Trigger when searching the codebase, checking CI, or running changeguard commands.
---

# Tooling & Research - AI-Brains

Load this skill when using research or operational tools on this codebase.

## Sourcebot (Deep Research)

Sourcebot is the primary tool for codebase navigation.

### Patterns

1. **Symbol Search**: Find trait implementations (e.g., `impl Event`).
2. **Context Mapping**: Before modifying a crate, use `ask_codebase` to identify cross-crate dependencies.

## GitHub CLI (`gh`)

The `gh` CLI bridges local development and the remote repository.

### Patterns

1. **CI Status**: `gh run list` to check the status of the GitHub Actions pipeline.
2. **PR Review**: `gh pr diff` to review changes before final verification.

## ChangeGuard (Governance)

ChangeGuard provides local risk assessment and architectural provenance.

### Patterns

1. **Pre-Flight**: `changeguard scan --impact` before starting an edit.
2. **Provenance**: `changeguard ledger start` for tracked architectural decisions and track implementations.
3. **Verification**: `changeguard verify` to run the Rust CI gate (`cargo nextest`, etc.).

## Key Reference Documents

- `Docs/Implementation-Plan.md` — Technical implementation roadmap (Tracks).
- `.agents/rules/core-mandates.md` — Security, TDD, and Engineering mandates.
