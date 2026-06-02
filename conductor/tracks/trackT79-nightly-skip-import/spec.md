# Track T79: `nightly --skip-import` opt-out flag

**Status:** ✅ **Complete**
**Started:** 2026-06-02
**Owner:** Claude
**Priority:** P0 — `nightly` was unsafe on isolated vaults.

---

## Problem Statement

`ai-brains nightly` (no flags) at
`crates/ai-brains-cli/src/commands/nightly.rs:151` unconditionally calls
`antigravity_import::run(ctx, 30)`. That function scans the user's real
Antigravity history on disk and writes events into the *currently open*
vault. The audit demonstrated the consequence: a fresh, isolated test
vault at `%TEMP%\ai-brains-eval.…` ended up with 100 sessions, 7,431
AssistantFinalRecorded events, 930 UserPromptRecorded events, and 8,363
memories — all from the user's real Antigravity data, none from the
test setup.

This is unsafe for:

- **Isolated test vaults** (the bug from the audit).
- **Per-project vaults** (the test vault becomes contaminated with data
  from unrelated projects).
- **CI vaults** (a CI runner that runs `nightly` would ingest a developer's
  real Antigravity history into the build artifact).

The function still has utility for the common case (a single vault per
user), so the fix is an *opt-out* flag rather than a removal.

## Acceptance Criteria

**AC1:** `ai-brains nightly --skip-import` skips the `antigravity_import`
call. The rest of the pipeline (MADR ingestion, symbol bridge, summarization)
still runs.

**AC2:** When `--skip-import` is set, a clear message is printed to stderr:
"Skipping Antigravity import (--skip-import). Use this on isolated, CI,
or per-project vaults to prevent cross-vault contamination from the
user's real Antigravity history."

**AC3:** The flag appears in `ai-brains nightly --help` so users can
discover it.

**AC4:** Default behavior is unchanged: `ai-brains nightly` (no flag) still
imports from Antigravity.

## Design Notes

- The flag is added to the `Nightly` clap variant in `main.rs:82-96`,
  threaded through to `nightly::run` as a `bool`, and gates the
  `antigravity_import::run` call.
- The smoke test only verifies that the flag is plumbed through (i.e.,
  it appears in `--help` and is accepted by clap). A full nightly run
  would require a live model server and Antigravity setup; the existing
  test infrastructure does not have those.

## Files

- `crates/ai-brains-cli/src/main.rs` — added `--skip-import` to the
  `Nightly` variant and threaded it through the dispatch arm.
- `crates/ai-brains-cli/src/commands/nightly.rs` — added `skip_import`
  parameter to `run` and gated the import call.
- `crates/ai-brains-cli/tests/smoke.rs` — `test_nightly_skip_import_flag_accepted`.

## Tests (TDD)

Red phase: `test_nightly_skip_import_flag_accepted` asserts
`nightly --help` contains `--skip-import`. Fails because the flag is not
yet wired up.

Green phase: flag added. Test passes.

## Verification

- `cargo nextest run -p ai-brains-cli test_nightly_skip_import_flag_accepted`
  — passes.
- `ai-brains nightly --help` — list includes `--skip-import`.

## Out of Scope

- A deeper fix (T84 — vault origin marker) that prevents cross-vault
  contamination by design. T79 is the surgical fix; T84 is the systemic
  one.
- Defaulting `--skip-import` to true. The default is left as `false` to
  preserve backward compatibility; the audit's `nightly --status` and
  `nightly --schedule` flows should still work without surprise.
