# Track T33: Antigravity Conversation Import

## Overview
Parse Antigravity JSONL conversation logs from `~/.gemini/antigravity/brain/`, extract clean user/assistant turns, and import them into the AI-Brains vault. Enforces mandate #4 (no hidden thinking or tool-only calls). Integrated into both the nightly sweep and a standalone `antigravity-import` CLI subcommand.

## Architecture & Design

1. **Adapter Layer** (`crates/ai-brains-adapters/src/antigravity.rs`):
   - `AntigravityStep`: Deserialized from overview.txt JSONL lines
   - `AntigravityTurn`: Cleaned turn ready for ingestion (role + content)
   - `discover_sessions()`: Scans `~/.gemini/antigravity/brain/*/` for `.system_generated/logs/overview.txt`
   - `filter_recent_sessions()`: Filters by modification time within N days
   - `parse_overview_file()`: JSONL line-by-line parsing with graceful skip of malformed lines
   - `extract_turns()`: State machine filtering — USER_EXPLICIT/USER_INPUT → user, MODEL/PLANNER_RESPONSE with content → assistant, tool-only responses and TOOL_OUTPUT → skipped
   - `strip_user_xml_tags()`: Removes `<ADDITIONAL_METADATA>`, `<USER_SETTINGS_CHANGE>`, extracts inner text from `<USER_REQUEST>`
   - `import_antigravity_sessions()`: Full orchestration — discover, filter, quiescence check, start session, ingest turns with deterministic UUIDv5 IDs, stop session

2. **CLI Integration** (`crates/ai-brains-cli/src/main.rs`):
   - `antigravity-import` subcommand with `--days` flag (default 30)
   - Delegates to `import_antigravity_sessions()` via `AppContext`

3. **Nightly Integration**:
   - Antigravity import runs before summarization in `run_nightly()`
   - Ensures fresh Antigravity sessions are available for memory synthesis

## Idempotency & Safety

- **UUIDv5 Turn IDs**: Deterministic IDs derived from `(session_uuid, turn-index)`, ensuring re-imports don't duplicate turns
- **Session status check**: Queries `session_projection` via `QueryStore::get_session_status()` — skips sessions already marked "completed"
- **Quiescence check**: Skips overview.txt files modified within last 5 minutes (avoids importing an active/in-flight session)
- **Canonical harness ID**: All Antigravity imports use `HarnessId("00000000-0000-0000-0000-000000000001")`

## Constraints & Rules
- **Mandate #4 enforced**: Tool-only MODEL responses (content=None, tool_calls present) are dropped. TOOL_OUTPUT steps are dropped. Only final user prompts and assistant text responses are captured.
- **Privacy**: All imported turns default to `LocalOnly`
- **Adapter capability**: Registered as `Partial` — batch import only, no real-time hooks

## Definition of Done
- [x] `AntigravityStep` and `AntigravityTurn` structs defined
- [x] `discover_sessions()`, `filter_recent_sessions()`, `parse_overview_file()`, `extract_turns()`, `strip_user_xml_tags()` implemented
- [x] `antigravity-import` CLI subcommand with `--days` flag
- [x] Integrated into `run_nightly()` before summarization
- [x] Idempotent: skips already-imported sessions
- [x] 5-minute quiescence check for active sessions
- [x] Deterministic UUIDv5 turn IDs
- [x] 9 adapter unit tests + 1 integration test passing
- [x] `Docs/antigravity-rule.md` and `Docs/antigravity-memory-review.md` authored

## Verification Plan
### Automated
- `cargo test -p ai-brains-adapters` — 9 unit tests for extraction, parsing, XML stripping, path parsing
- `cargo test -p ai-brains-adapters --test antigravity_manual_import` — integration test
- `cargo clippy --workspace --all-targets -- -D warnings`

### Manual
- Run `ai-brains antigravity-import --days 30` and verify sessions appear in vault
- Run `ai-brains nightly` and verify Antigravity import step completes before summarization
- Verify re-running import does not duplicate turns (UUIDv5 idempotency)
