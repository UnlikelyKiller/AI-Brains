# UX: Friendly default project name

**Status:** ✅ **Complete**
**Started:** 2026-06-02
**Owner:** Claude
**Priority:** P2 — friction; doesn't break anything but bloats `project list`.

---

## Problem Statement

When a project is auto-created (e.g. by `ai-brains context`), the
default `name` is set to `Project <full-uuid>`, where the full UUID
is 32 hex characters. `project list` renders the name in a 30-column
field, so most rows get truncated to `Project 6a4de384-56a7-8d73-000`,
which is noisy and not very scannable:

```
project_id                           name (alias|UUID)              alias                     memories
6a4de384-56a7-8d73-0000-000000000000 Project 6a4de384-56a7-8d73-000                           0
```

The full UUID is already shown in the dedicated `project_id` column,
so duplicating it in the name field is redundant.

## Acceptance Criteria

**AC1:** A project auto-created with no alias has its `name` set to
`(no alias) — <short-uuid>`, where `<short-uuid>` is the first 8 hex
characters of the UUID.

**AC2:** `project list` displays the friendly form. The full project_id
remains available in the dedicated `project_id` column.

**AC3:** Projects that already have an alias continue to use the alias
as the display name. Only the default (no-alias) case is changed.

## Design Notes

- The change is one line in
  `crates/ai-brains-cli/src/context.rs:113`, where the `ProjectRegistered`
  payload is built.
- The 8-char prefix is taken via string slicing on the UUID's standard
  hyphenated Display form. For a `Uuid::nil` (e.g.
  `00000000-0000-0000-0000-000000000000`), the prefix is `00000000`.
- Existing projects in vaults created before this change still show
  `Project <full-uuid>` — that data is on disk, and migrating it is
  not required (it still works, just looks ugly). The friendly form
  applies to newly registered projects only.
- The `name` column is a free-form label, so this change is a pure
  presentation improvement. No code reads the old prefix.

## Files

- `crates/ai-brains-cli/src/context.rs` — changed the default name
  format in the `ProjectRegistered` payload.
- `crates/ai-brains-cli/tests/smoke.rs` — `test_project_list_friendly_default_name`.

## Tests (TDD)

Red phase: `test_project_list_friendly_default_name` seeds a project via
`context`, runs `project list`, and asserts the `(no alias)` form is
present. Fails because the old form was `Project <full-uuid>`.

Green phase: changed the format string. Test passes.

## Verification

- `cargo nextest run -p ai-brains-cli test_project_list_friendly_default_name` — passes.
- `cargo nextest run -p ai-brains-cli` — 36/36 pass.
- Manual: `ai-brains project list` after `ai-brains context` shows
  `(no alias) — <8-char-prefix>` instead of `Project <full-uuid>`.

## Out of Scope

- Migrating existing projects on disk to the new name format.
- Changing the table column widths (still 30 chars, but the friendly
  form fits comfortably with room to spare).
- Adding a `--rename` subcommand to let users give projects human
  aliases (separate, much larger feature).
