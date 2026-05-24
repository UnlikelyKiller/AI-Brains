# Plan: Track T47 - Safety Sync Hardening & Skill Alignment

### Phase 1: Safety Sync Fix
- [ ] Task 1.1: Modify `ChangeGuardHotspot` in `crates/ai-brains-cli/src/commands/safety.rs` to use `f64` for `frequency` and `complexity`.
- [ ] Task 1.2: Update the `render_hotspots` and text parsing logic to handle float values for these fields.
- [ ] Task 1.3: Verify with `ai-brains safety sync --dry-run`.

### Phase 2: Skill Update
- [ ] Task 2.1: Update `.agents/skills/ai-brains/SKILL.md` with:
    - `agy-hook` command instructions.
    - Mention of expanded `antigravity-import` discovery (CLI, IDE, and Project-specific paths).
    - Note about the ultra-fast handshake and fast-fail behavior.

### Phase 3: Verification
- [ ] Task 3.1: Pass full CI gate.
- [ ] Task 3.2: Record decisions in the vault.
