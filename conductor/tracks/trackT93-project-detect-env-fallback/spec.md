# Track T93: `project detect` Fallback to `.env` `AI_BRAINS_PROJECT_ID`

**Status:** ⏳ **Pending**
**Owner:** —
**Priority:** P2 — `project detect` always fails even when the project is already configured in `.env`.

---

## Problem Statement

`ai-brains project detect` performs git-slug matching against project names/aliases in the vault. If no aliases have been set (which is the case for all projects until T89 is done), it returns nothing. However, when a project is actively in use, `AI_BRAINS_PROJECT_ID` is already set in the local `.env` file — the project is effectively "detected" by the environment. The command should use this as a fallback before giving up.

## Acceptance Criteria

**AC1:** If git-slug matching finds a match, return that result as today (no change to primary path).

**AC2:** If git-slug matching fails but `AI_BRAINS_PROJECT_ID` is set in the local `.env` (cwd lookup), and that project ID exists in the vault, `project detect` prints the project ID and exits 0 — with an indicator that the match came from `.env` (e.g. `(from .env)`).

**AC3:** If neither slug match nor `.env` ID is found, `project detect` exits 1 with a clear message: `"No project detected. Set an alias with 'project set-alias' or initialize a project with 'init'."` — not a silent empty response.

**AC4:** `--json` output includes a `source` field: `"git_slug"`, `"env_file"`, or `null` (not found).

## Design Notes

- The `.env` file path is already resolved by `AppContext` — the project ID is already loaded into `ctx.project_id` by the time any command runs. Reading it again from `.env` is redundant; use `ctx.project_id` directly as the fallback.
- Before using the ctx fallback, confirm the project ID exists in the vault projection (a single `SELECT` by ID).
- This is a UX safety net; T89 (`project set-alias`) is the permanent fix. This track should be implemented independently.

## Verification

```
# in a directory with a .env containing AI_BRAINS_PROJECT_ID=<uuid>
ai-brains project detect
# Expected: "<uuid>  (from .env)"  exit 0

# in a directory with no .env and no alias match
ai-brains project detect
# Expected: error message, exit 1 (not empty output + exit 0)
```
