# Track T55: Nightly Performance (Incremental Scan)

## Objective
Reduce the duration of `ai-brains nightly` by skipping the parsing of Antigravity session files that haven't changed since the last import.

## Problem Statement
The `antigravity-import` command (called by `nightly`) currently parses every JSONL file modified within the last 30 days. For active users with 250+ historical sessions, this takes ~2 minutes even when no new data is present.

## Requirements
- **Metadata Tracking**: Store the last known modification time and file size for each imported session in the vault's `sync_state`.
- **Incremental Logic**: Before parsing a file, compare its current metadata against the stored values.
- **Skip Unchanged**: If metadata matches exactly, skip parsing the file.
- **Fast No-Op**: `ai-brains nightly` should complete in < 5 seconds when no new Antigravity data is present.

## Technical Design
- Update `crates/ai-brains-adapters/src/antigravity.rs`:
    - `import_antigravity_sessions` should check `query_store.get_sync_state(key)` where key is `source_meta:<session_id>`.
    - Compare `mtime` and `len`.
    - If mismatch, parse and ingest.
    - After successful ingestion (including `stop_session`), update `sync_state` with new metadata.

## Verification Plan
- **Baseline**: Run `ai-brains nightly` and time it.
- **No-Op Run**: Run it again immediately. Verify it takes < 5 seconds.
- **New Data**: Add a turn to an existing Antigravity session. Run `nightly`. Verify only that session is parsed and the new turn is ingested.
