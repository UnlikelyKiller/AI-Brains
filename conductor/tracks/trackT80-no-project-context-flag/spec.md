# Track T80: `--no-project-context` env-var escape hatch

**Status:** ✅ **Complete**
**Started:** 2026-06-02
**Owner:** Claude
**Priority:** P1 — broke programmatic use of pin, agy-hook, and other env-var-driven commands.

---

## Problem Statement

`main()` at `crates/ai-brains-cli/src/main.rs:323-343` (pre-T80) was:

```rust
if !std::path::Path::new(".env").exists() {
    std::env::remove_var("AI_BRAINS_PROJECT_ID");
    std::env::remove_var("AI_BRAINS_SESSION_ID");
} else {
    dotenvy::dotenv_override().ok();
}
```

The intent (per the comment) was to "prevent stale inheritance from other
projects in the same shell session." The side effect is that when no
`.env` exists in cwd, the env vars the caller set in the shell are
silently dropped. This breaks any non-interactive caller:

- A `pin` invocation from a hook script, where the env vars were set by
  the parent process.
- A CI step that exports `AI_BRAINS_PROJECT_ID` then runs `ai-brains pin`.
- A user's interactive shell where they `export` the vars to a temporary
  vault, but the cwd is the home directory (no `.env`).

The audit (June 2026) reproduced the bug: `ai-brains pin "..."` from a
tempdir with the env vars exported returned
`AI_BRAINS_PROJECT_ID not set. Run 'ai-brains context' first.`

## Acceptance Criteria

**AC1:** A new global `--no-project-context` flag is accepted by clap on
every subcommand.

**AC2:** When `--no-project-context` is set, `main()` does not call
`remove_var` on `AI_BRAINS_PROJECT_ID` / `AI_BRAINS_SESSION_ID` and does
not load any `.env` file. The env vars the caller set are preserved
verbatim.

**AC3:** Default behavior is unchanged: without the flag, the env-var-clear
and `.env`-load logic still runs as before.

**AC4:** A smoke test verifies that `pin` from a tempdir (no `.env`) with
`AI_BRAINS_PROJECT_ID` and `AI_BRAINS_SESSION_ID` exported, plus
`--no-project-context`, succeeds.

## Design Notes

- The flag is added as a top-level `#[arg(long, global = true)]` on the
  `Cli` struct. `global = true` makes it available on every subcommand.
- To keep the env-var block simple, we read the flag once at the top of
  `main()` by scanning `std::env::args()` for the string `--no-project-context`.
  This avoids re-ordering the parse/env-load sequence (which would risk
  loading `.env` before we know the flag was set).
- The block is wrapped in `if !no_project_context { ... }`. Both the
  per-project `.env` load and the global `~/.ai-brains/.env` fallback are
  inside the gate.
- A flag, not a config-file change: the change must be explicit and
  discoverable, and the env-var-clear default behavior is preserved for
  existing interactive users.

## Files

- `crates/ai-brains-cli/src/main.rs` — added `no_project_context` field
  to `Cli`, gated the env-var block in `main()`.
- `crates/ai-brains-cli/tests/smoke.rs` —
  `test_no_project_context_preserves_env_vars`.

## Tests (TDD)

Red phase: `test_no_project_context_preserves_env_vars` invokes `pin`
with the env vars exported and `--no-project-context` from a tempdir
(no `.env`). Fails because the flag is not yet accepted by clap.

Green phase: flag added and gated. Test passes.

## Verification

- `cargo nextest run -p ai-brains-cli test_no_project_context_preserves_env_vars`
  — passes.
- Manual test: from a tempdir with `AI_BRAINS_PROJECT_ID=...` exported,
  `ai-brains --no-project-context pin "..."` succeeds.

## Out of Scope

- Defaulting `--no-project-context` to true. Left as `false` to preserve
  the existing behavior for interactive users.
- A `--debug-env` flag that prints the resolved env vars before they are
  applied. Useful but a separate track.
- Per-subcommand escape hatches (e.g., `pin --no-context`). The single
  global flag is simpler and sufficient.
