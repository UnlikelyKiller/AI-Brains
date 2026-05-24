# Specification: Track T47 - Safety Sync Hardening & Skill Alignment

## Objective
Fix the parsing crash in `ai-brains safety sync` and update the `ai-brains` skill to reflect recent system improvements.

## 1. Safety Sync Hardening
- **Problem**: `ChangeGuard` recently updated its hotspot schema, returning floating-point values for `frequency` and `complexity`. `ai-brains` currently expects `u32`, causing a JSON deserialization error.
- **Fix**: Update the `ChangeGuardHotspot` struct in `crates/ai-brains-cli/src/commands/safety.rs` to use `f64` for all numeric fields.

## 2. Skill Alignment
- **Requirement**: The `ai-brains` skill (`.agents/skills/ai-brains/SKILL.md`) needs to be updated with the new `agy-hook` command and the expanded discovery paths for `antigravity-import`.
- **Goal**: Ensure that any AI agent using the skill is aware of the hardened infrastructure and the new integration points.

## Verification Plan
- **Safety Sync**: Run `ai-brains safety sync --dry-run` and verify it parses without error.
- **Skill**: Verify the `SKILL.md` content reflects the new commands and discovery capabilities.
