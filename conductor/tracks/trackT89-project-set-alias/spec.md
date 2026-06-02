# Track T89: `project set-alias` Command

**Status:** ⏳ **Pending**
**Owner:** —
**Priority:** P1 — `project detect` and `project resolve` are dead commands without this.

---

## Problem Statement

All vault projects are created with auto-generated names (`(no alias) — <uuid-prefix>`). There is no CLI command to assign a human-readable alias to a project. As a consequence, `project detect` (which matches by git-slug against project names/aliases) and `project resolve <alias>` always fail — they are decorative.

## Acceptance Criteria

**AC1:** `ai-brains project set-alias <project_id> <alias>` sets the alias for the given project and exits 0.

**AC2:** After `set-alias`, `project list` shows the alias in the Name column for that project.

**AC3:** After `set-alias`, `project resolve <alias>` returns the project's UUID.

**AC4:** After `set-alias` where the git repo slug matches the alias, `project detect` returns the correct project ID.

**AC5:** Setting an alias that is already in use by another project exits 1 with a clear error: `"Alias '<name>' is already assigned to project <other-id>."`.

**AC6:** A `project rename` alias for `set-alias` is acceptable but not required.

## Design Notes

- This requires a new event kind (e.g. `ProjectAliasSet { project_id, alias }`) appended to the event log, and an update to the `projects` projection to apply it.
- The command wiring lives in `crates/ai-brains-cli/src/main.rs` and a new function in `commands/project.rs` (or the existing project command module).
- `project detect` reads the alias from the projection during slug comparison — no change to detect logic is required once the projection is correct.
- Follow the event-sourcing mandate: no direct DB updates; emit a compensating event if alias must be removed.

## Verification

```
ai-brains project list                              # note a project_id
ai-brains project set-alias <project_id> my-vault   # exits 0
ai-brains project list                              # Name column shows "my-vault"
ai-brains project resolve my-vault                  # prints <project_id>
# from within a git repo named "my-vault":
ai-brains project detect                            # prints <project_id>
```
