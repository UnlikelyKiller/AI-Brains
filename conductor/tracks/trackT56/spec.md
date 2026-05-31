# Track T56: CLI Output Polish (Smart Preflight)

## Objective
Improve the user experience of the `preflight` command by reducing terminal noise and providing human-centric summaries.

## Problem Statement
`ai-brains preflight` currently defaults to a large JSON dump. While perfect for AI agents, it is overwhelming for humans trying to verify their repository bearings.

## Requirements
- **Smart Defaulting**: Automatically switch to `pretty` (human-readable) output if `stdout` is a terminal (TTY) and no format was explicitly requested.
- **Summary Mode**: Add a `--summary` (or `-s`) flag that prints high-level statistics instead of the full text.
    - Counts of pinned memories, hotspots, active sessions, and word count.
- **Consistency**: Ensure JSON output remains identical for AI agents (detected via pipe or `--format json`).

## Technical Design
- Update `crates/ai-brains-cli/src/commands/preflight.rs`:
    - Add `is_terminal` check using `atty` or `is-terminal` crate.
    - Implement summary calculation (counts of markers in the context text).
- Update `crates/ai-brains-cli/src/main.rs`:
    - Add `summary: bool` to `Preflight` command.

## Verification Plan
- **Human Run**: Run `ai-brains preflight` in a terminal. Verify it prints text, not JSON.
- **Agent Run**: Run `ai-brains preflight | cat`. Verify it prints JSON.
- **Summary Run**: Run `ai-brains preflight --summary`. Verify it prints a concise statistical breakdown.
