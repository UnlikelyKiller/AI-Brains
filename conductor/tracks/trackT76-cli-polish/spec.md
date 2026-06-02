# Track T76: CLI Polish (project list + backup restore)

**Status:** ✅ **Complete**
**Started:** 2026-06-02
**Owner:** Claude
**Priority:** P2 — small ergonomic fixes surfaced during the 2026-06-02 audit.

---

## Problem Statement

Two small but persistent operator-facing frictions were flagged in the audit:

1. **`ai-brains project list` table readability.** The `name` column is
   truncated to 20 characters and the column header is just `"name"`. With
   project names like `Project 33ec90e0-be7` (auto-generated from
   project_id prefixes), 20 chars is often too narrow to show the
   disambiguating part of the UUID, and the header gives no hint that
   the column actually contains either an alias or a UUID fragment.
   Operators looking at a list of three projects see a wall of
   similar-looking entries with no way to tell them apart at a glance.

2. **`ai-brains backup restore` is friction-laden.** The current
   implementation (`crates/ai-brains-cli/src/commands/backup.rs`) does
   verify integrity and then prints `"Type 'yes' to continue: "` before
   overwriting. The interactive prompt works for humans but blocks
   non-interactive use (CI, scheduled restore, scripted migration).
   There's no `--dry-run` for the verify-and-report case, either.

## Acceptance Criteria

### `project list`
**AC1:** The `name` column is widened to 30 characters.
**AC2:** The column header reads `name (alias|UUID)`.
**AC3:** Existing projects continue to display without regression.

### `backup restore`
**AC4:** `backup restore <path> --dry-run` runs the integrity check,
prints a one-line summary, and exits 0 without touching the destination
vault. No prompt, no overwrite.

**AC5:** `backup restore <path> --force` skips the interactive confirm
prompt and proceeds with the overwrite.

**AC6:** `backup restore <path>` (no flags) retains the existing
interactive prompt behavior.

**AC7:** The flags compose: `--force --dry-run` is treated as
`--dry-run` (the more conservative of the two, since dry-run never
overwrites regardless of force).

## Design Notes

- **`project list`**: two-line change in
  `crates/ai-brains-cli/src/commands/project.rs` — the format string and
  the slice. The header column now contains the parenthetical hint so
  a reader who skims the table knows what's in there.
- **`backup restore` flags**: the `BackupCommands::Restore` clap
  variant grows two `bool` fields (`--force` short `-f`,
  `--dry-run` long-only). `run_restore` checks `dry_run` first
  (returns after the integrity check + summary), then `force` (skips
  the stdin prompt), then performs the SQLite backup. This order means
  `--force --dry-run` always runs dry-run, never overwrite.
- Tests: two new integration tests in `crates/ai-brains-cli/tests/smoke.rs`
  (TDD). `test_backup_restore_dry_run` creates a source + dest vault,
  generates a backup, runs restore with `--dry-run`, asserts the dest
  vault's bytes are unchanged. `test_backup_restore_force_skips_prompt`
  creates the same setup, runs restore with `--force` and **no stdin**,
  asserts success — a prompt would hang the test forever.

## Files

- `crates/ai-brains-cli/src/commands/project.rs` (column + header tweak)
- `crates/ai-brains-cli/src/commands/backup.rs` (add `force` and
  `dry_run` parameters + dry-run short-circuit)
- `crates/ai-brains-cli/src/main.rs` (add clap fields, pass them through)
- `crates/ai-brains-cli/tests/smoke.rs` (add 2 tests)

## Verification

- `cargo fmt --check` clean
- `cargo clippy --workspace --all-targets -- -D warnings` clean
- `cargo nextest run -p ai-brains-cli` — all 5 smoke tests pass
  (3 pre-existing + 2 from T73 + 2 from T76 = 7 in smoke, 30 total
  with `--features graph`)
- Manual `ai-brains project list` shows the widened column with the new
  header
- Manual `ai-brains backup restore <path> --dry-run` returns
  `"dry-run: backup ... verified ok; would overwrite vault at ..."` and
  leaves the dest vault untouched (verified via `fc` byte-count)
