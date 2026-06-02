# Track T92: Debug and Fix `sync pull --hotspots` / `sync pull --ledger`

**Status:** ⏳ **Pending**
**Owner:** —
**Priority:** P2 — sync pipeline silently does nothing; ChangeGuard data never enters vault.

---

## Problem Statement

`ai-brains sync pull --hotspots` and `ai-brains sync pull --ledger` both report `"Successfully synced 0 records."` despite ChangeGuard having active hotspots (confirmed by `safety sync --dry-run` finding 5 real hotspots). The sync pipeline is broken at the parse or filtering stage.

## Root Cause (Hypothesis)

The sync pull code invokes `changeguard scan --impact --json` (or equivalent) and parses the NDJSON output, filtering by a `record_kind` field. The actual NDJSON emitted by the current ChangeGuard version likely uses different field names or values than what the parser expects. This is a format mismatch — the parser has drifted from the producer.

## Acceptance Criteria

**AC1:** Running `changeguard scan --impact --json` (or whatever command `sync pull --hotspots` invokes) produces NDJSON that is fully parsed by the sync pull code — `synced N records` where N > 0 when hotspots exist.

**AC2:** Running `sync pull --ledger` similarly syncs ledger entries into vault memories when the ledger has entries.

**AC3:** A `--dry-run` flag (or the existing one if present) shows how many records would be synced without writing to the vault.

**AC4:** If the ChangeGuard command format changes in a future version, the parser emits a descriptive error rather than silently returning 0.

## Design Notes

- **Investigation first**: Run `changeguard scan --impact --json` and capture raw output. Compare every field name and value against what `crates/ai-brains-cli/src/commands/sync.rs` (or the retrieval crate) expects. Document the mismatch.
- Common mismatches: `record_kind` vs `kind`, `hotspot` vs `Hotspot`, camelCase vs snake_case.
- After identifying the mismatch, update the parser (not the ChangeGuard output) since ChangeGuard is the source of truth.
- If the fix requires a ChangeGuard schema change, open a companion track in ChangeGuard.

## Verification

```
changeguard scan --impact --json | head -5            # inspect raw NDJSON
ai-brains sync pull --hotspots                        # must report N > 0 synced
ai-brains recall "hotspot"                            # must return at least one vault record
cargo nextest run --workspace
```
