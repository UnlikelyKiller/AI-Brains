# Track T75: OPERATIONS.md Modernization

**Status:** ✅ **Complete**
**Started:** 2026-06-02
**Owner:** Claude
**Priority:** P1 — the operations guide was significantly out of date; it reflected a pre-T44 surface (no daemon lifecycle, no scheduled tasks, no forget, no safety sync, no sync query, no bridge).

---

## Problem Statement

`Docs/OPERATIONS.md` (last meaningful update in the Phase 15 era) described a CLI surface that no longer matched reality. Concrete gaps:

- No mention of `daemon start/status/stop/schedule/unschedule`.
- No mention of `nightly --schedule` / `nightly --unschedule` / `nightly --status`.
- No mention of `forget --memory-id / --match / --list-forgotten / --restore` (a whole memory-hygiene story).
- No mention of `safety sync` (and the T70 ChangeGuard bridge it relies on).
- No mention of `sync query`, the unified search across AI-Brains + ChangeGuard.
- The "Vault Locked" / "Projection Drift" troubleshooting entries were stale.
- The environment-variable table was missing `CHANGEGUARD_TX_ID`, `AI_BRAINS_PROJECT_ID`, `AI_BRAINS_SESSION_ID`, and `AI_BRAINS_SCOPE`.

For an operator trying to use the system today, the document was actively misleading.

## Acceptance Criteria

**AC1:** The rewritten `Docs/OPERATIONS.md` includes sections covering every
top-level subcommand that is real and working as of T72: `init`, `ingest`,
`antigravity-import`, `agy-hook`, `recall`, `sync query`, `preflight`,
`context`, `project list/resolve/detect`, `daemon`, `nightly`, `forget`,
`backup`, `restore`, `safety sync`, `graph update`, `graph rebuild`.

**AC2:** A new "Troubleshooting" entry warns about the `cargo audit`
0.22.x no-summary-on-clean-run behavior (cross-references
`Docs/ci-tooling.md#behavior-notes`).

**AC3:** A new "Troubleshooting" entry covers the T73 "init refuses on
populated vault" behavior with the `--force` remedy.

**AC4:** The environment-variable table includes all env vars the
system actually reads: `AI_BRAINS_VAULT_PATH`, `AI_BRAINS_KEY`,
`AI_BRAINS_PROJECT_ID`, `AI_BRAINS_SESSION_ID`, `CHANGEGUARD_TX_ID`,
`AI_BRAINS_MODEL_URL`, `AI_BRAINS_EMBEDDING_URL`,
`AI_BRAINS_EMBEDDING_MODEL`, `AI_BRAINS_COMPLETION_MODEL`,
`AI_BRAINS_SCOPE`.

**AC5:** Every command shown in the document's worked examples matches
the current `ai-brains <subcommand> --help` output (verified by spot-check
during the audit).

## Design Notes

- The document is rewritten as a single coherent guide. The structure
  follows the operator's mental model: install → ingest → retrieve →
  schedule → hygiene → safety → troubleshoot. Each command has a
  realistic example using the `--vault-path` flag and current env vars.
- A "Command Summary" table at the end gives a one-glance reference.
- The rewrite does not add any new operational guidance that wasn't
  already implied by the current code; it's a faithful documentation
  pass, not a feature change.

## Files

- `Docs/OPERATIONS.md` (rewrite)

## Verification

- Manual review: every command in the document exists and runs as
  described (verified during the 2026-06-02 audit; the
  verification ran live in the same session).
- `git diff --stat` shows only the single file changed.
