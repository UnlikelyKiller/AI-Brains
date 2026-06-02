# Track T83: JSON Schemas for `agy-hook` and `sync pull`

**Status:** ✅ **Complete**
**Started:** 2026-06-02
**Owner:** Claude
**Priority:** P2 — documentation gap that produced trial-and-error debugging.

---

## Problem Statement

Two CLI subcommands expect JSON input whose shape was not documented in
`--help`:

1. `ai-brains agy-hook --payload` — the audit caught users hitting
   `"missing field 'transcriptPath'"` without a hint that the schema
   had three required fields (`transcriptPath`, `sessionId`,
   `projectHash`).

2. `ai-brains sync pull --from-file` — the NDJSON lines must be
   `BridgeRecord`s with a specific shape. The audit caught users
   hitting `"missing field 'bridge_version'"` because the field is
   technically optional in the Rust struct but required at the wire
   format level.

Both flags were used by hook scripts and external integrations, so the
errors were hard to debug.

## Acceptance Criteria

**AC1:** `ai-brains agy-hook --schema` prints a JSON Schema document to
stdout, exits 0, and the document is valid JSON that requires
`transcriptPath`, `sessionId`, and `projectHash`.

**AC2:** `ai-brains sync pull --schema` prints a JSON Schema document
to stdout, exits 0, and the document requires `bridge_version` (and
the other BridgeRecord fields).

**AC3:** The schemas are also committed to the repo at
`Docs/schemas/agy-hook-payload.json` and
`Docs/schemas/sync-pull-record.json` for tools that consume them
directly.

**AC4:** Existing behavior is unchanged: `agy-hook --payload <json>`
and `sync pull --from-file <path>` still work.

## Design Notes

- The schemas are bundled at compile time via `include_str!` in
  `crates/ai-brains-cli/src/main.rs`, so `--schema` works regardless
  of cwd or installation method. The source-of-truth files at
  `Docs/schemas/*.json` are mirrored into the binary; changes to the
  files require a rebuild to propagate.
- A single `print_schema` helper is reused by both subcommands. It
  validates the embedded JSON and pretty-prints it.
- The schema files are JSON Schema 2020-12 (the latest draft supported
  by the major validators). `$id` is set to a stable URL so consumers
  can reference them programmatically.

## Files

- `Docs/schemas/agy-hook-payload.json` (new)
- `Docs/schemas/sync-pull-record.json` (new)
- `crates/ai-brains-cli/src/main.rs` — `SCHEMA_AGY_HOOK` and
  `SCHEMA_SYNC_PULL` constants, `print_schema` helper, `--schema`
  flag on `AgyHook` and `SyncCommands::Pull`, updated dispatch arms.
- `crates/ai-brains-cli/tests/smoke.rs` — `test_agy_hook_schema_flag`
  and `test_sync_pull_schema_flag`.

## Tests (TDD)

Red phase: two new tests assert that `--schema` exits 0, the output is
valid JSON, and the schema requires the expected fields. Both fail
because the flag is not yet accepted by clap.

Green phase: flag added, schemas bundled. Tests pass.

## Verification

- `cargo nextest run -p ai-bbrains-cli test_agy_hook_schema_flag
  test_sync_pull_schema_flag` — passes.
- `cargo nextest run -p ai-brains-cli` — 35/35 pass.
- Manual: `ai-brains agy-hook --schema` and `ai-brains sync pull
  --schema` print valid JSON Schema documents.

## Out of Scope

- Schemas for `ingest` (which has more fields, including `thinking`),
  `pin` (which is much simpler), and other commands. T83 only covers
  the two commands identified in the audit.
- Auto-syncing the source-of-truth JSON files with the embedded
  constants. The current setup uses `include_str!` for binary
  self-containment; a build script could automate the mirror, but it's
  not required.
