# Track T36: CLI Polish & Cross-Command Workflow

## Overview

Manual testing of all nine CLI commands revealed friction points, missing safety guards, and workflow gaps. Commands work as atomic tools but are "islands" — `recall` → copy UUID → `forget` is manual and fragile, `pin` doesn't echo back its ID, `context` silently overwrites sessions, and `safety sync` parses ANSI terminal tables when ChangeGuard supports `--json`. This track addresses all findings with surgical, non-breaking improvements.

## Architecture & Design

### 1. Breaking Change Policy

The following contracts are consumed by four live hook scripts in `~/.ai-brains/scripts/`:
- **`IngestRequest` JSON** (stdin to `ai-brains ingest`): `session_id`, `project_id`, `harness_id`, `turn_id`, `role`, `content`, `privacy` — all four hooks construct this payload via `ConvertTo-Json`.
- **`PreflightContextResponse` JSON** (stdout from `ai-brains preflight`): hooks parse `.text` and `.word_count`. Claude/Codex/Gemini hooks read `$preflightJson.text`.
- **`RecallResponse` JSON** (stdout from `ai-brains recall`): `results[].memory_id`, `.content`, `.source`.
- **`.env` keys**: `AI_BRAINS_PROJECT_ID`, `AI_BRAINS_SESSION_ID`, `AI_BRAINS_HARNESS_ID`.

**Rule**: All improvements must be additive — new optional fields (with `skip_serializing_if`), new CLI flags (with defaults matching current behavior), new subcommands. No field renames, no required-field additions, no `.env` key changes.

### 2. ChangeGuard Structured Output

`safety sync` currently shells out to `changeguard hotspots --limit N`, captures ANSI-formatted terminal table text, then runs a fragile line-based heuristic parser (`hotspot.rs`) to strip escape codes, dividers, and log preamble. This breaks if ChangeGuard changes its terminal output.

ChangeGuard supports `changeguard hotspots --json` which returns structured data. Switching to this:
- Eliminates the entire `hotspot.rs` sanitization pipeline
- Preserves per-file metadata (rank, score, frequency, complexity, file path) as structured fields
- Fixes the truncation mismatch: `condense_hotspots()` hard-truncates to 5 lines regardless of `--limit`
- Allows meaningful summary output: "3 hotspots synced: cli_capture_smoke.rs (0.13), ..."

The structured data will be rendered into a cleaner condensed text format for pinning, maintaining backward compatibility with the existing `HOTSPOT:` memory format consumed by `preflight`.

### 3. Error Envelope Consistency

Currently `ingest` returns flat text errors while `recall`/`preflight` return JSON. All errors should use a consistent JSON envelope with an error code and human-readable message, while preserving backward compat by keeping the same exit codes and stderr behavior.

### 4. Context Session Preservation

`context` generates a fresh `SessionId` every run and silently overwrites any existing session in `.env`. For harness hooks that set `$env:AI_BRAINS_SESSION_ID` at session start and expect it to persist, re-running `context` mid-session clobbers the session. Fix: if `AI_BRAINS_SESSION_ID` already exists in `.env`, warn and ask for confirmation (or require `--new-session` to force).

## Constraints & Rules

- No breaking changes to any contract consumed by hook scripts
- All new CLI flags must default to current behavior
- New JSON fields must use `#[serde(skip_serializing_if = "Option::is_none")]`
- `.env` key names must not change
- CI gate must pass: `cargo fmt --check ; cargo clippy --workspace --all-targets -- -D warnings ; cargo test --workspace`
- ChangeGuard `scan --impact` before edits, `verify` before commit

## Files Owned

| File | Purpose |
|------|---------|
| `crates/ai-brains-cli/src/commands/context.rs` | Context init improvements |
| `crates/ai-brains-cli/src/commands/safety.rs` | `--json` switch, `--dry-run`, better output |
| `crates/ai-brains-cli/src/hotspot.rs` | **DELETE** — replaced by structured JSON parsing |
| `crates/ai-brains-cli/src/commands/preflight.rs` | `--pretty` flag |
| `crates/ai-brains-retrieval/src/preflight.rs` | Top-3 recent memory, hotspot format cleanup |
| `crates/ai-brains-cli/src/commands/recall.rs` | Empty-result suggestions, `--format` flag |
| `crates/ai-brains-retrieval/src/lexical.rs` | Match score in output |
| `crates/ai-brains-cli/src/commands/pin.rs` | Echo memory ID, `--tag`, stdin support |
| `crates/ai-brains-cli/src/commands/ingest.rs` | Field-specific errors |
| `crates/ai-brains-capture/src/errors.rs` | Structured validation errors |
| `crates/ai-brains-cli/src/commands/nightly.rs` | Progress output, project_id validation |
| `crates/ai-brains-cli/src/commands/forget.rs` | `--match`, `--list`, confirmation prompt |
| `crates/ai-brains-cli/src/commands/backup.rs` | `--output-dir`, `restore` subcommand |
| `crates/ai-brains-brain/src/backup.rs` | Restore support, integrity verification |
| `crates/ai-brains-cli/src/main.rs` | New CLI args registration |
| `crates/ai-brains-contracts/src/recall.rs` | Optional `score` field |
| `crates/ai-brains-contracts/src/ingest.rs` | (no changes — read-only reference) |
| `crates/ai-brains-contracts/src/preflight.rs` | (no changes — read-only reference) |

## Files Forbidden To Touch

- `~/.ai-brains/scripts/target-*.ps1` — hook scripts must not require updates
- `.env` key names — `AI_BRAINS_PROJECT_ID`, `AI_BRAINS_SESSION_ID`, `AI_BRAINS_HARNESS_ID`
- `crates/ai-brains-events/src/payload.rs` — event schema is stable
- `crates/ai-brains-store/migrations/` — no new migrations needed

## Plan: Phase-by-Phase Implementation

### Phase 1: `safety sync` — ChangeGuard `--json` Switch

Highest-impact change. Eliminates a fragile parsing pipeline and fixes the truncation mismatch.

**Task 1.1**: Add `changeguard hotspots --json` invocation
- In `safety.rs`, try `changeguard hotspots --json --limit N` first
- Parse JSON response into a structured `ChangeGuardHotspot { rank, score, frequency, complexity, file_path }` vec
- Fall back to text-mode parsing if `--json` fails (older ChangeGuard versions)
- Add subprocess timeout (30s default)

**Task 1.2**: Replace text parsing with structured rendering
- Delete `hotspot.rs` (ANSI stripping, condense_hotspots, line filters)
- Render hotspots into clean text format for pinning: `"HOTSPOT: N files identified by ChangeGuard\n 1. crates/... (score: 0.13, freq: 2)\n 2. ..."`
- Fix truncation: use actual `--limit` value instead of hard-coded 5

**Task 1.3**: Add `--dry-run` flag to `safety sync`
- Print what would be pinned without actually pinning
- Show count: "Would sync 3 hotspots, 0 constraints"

**Task 1.4**: Better progress output
- "ChangeGuard scan complete: 12 hotspots found"
- "Syncing top 5 hotspots to vault..."
- "Pinned memory abc123 (5 hotspots)"

### Phase 2: `context` — Session Preservation

**Task 2.1**: Add `--show` flag
- `ai-brains context --show` prints current project ID, session ID, harness ID from `.env` without modifying anything
- Reads `.env`, extracts the three keys, prints them

**Task 2.2**: Add `--new-session` flag and overwrite warning
- Default behavior: if `AI_BRAINS_SESSION_ID` exists in `.env`, print warning and ask for confirmation via stdin (or require `--new-session` to force)
- `context --new-session` explicitly creates a new session, replacing the old one
- Without `--new-session` and with existing session: `"Session 4ffee084 already exists. Use --new-session to replace, or --show to view."`

### Phase 3: `preflight` — Readability & Content

**Task 3.1**: Add `--pretty` flag
- `--pretty` outputs the preflight text directly to stdout (no JSON wrapper), making it readable for human CLI use
- Default (no flag) preserves JSON `PreflightContextResponse` for hook compatibility

**Task 3.2**: Top-3 most recent memories instead of 1
- Change "Most Recent Memory" section to "Most Recent Memories" showing top 3
- Each entry gets a compact header with timestamp

**Task 3.3**: Cleaner hotspot format in safety section
- Instead of raw markdown tables, render hotspots as compact list:
  ```
  --- Hotspots (ChangeGuard) ---
  crates/ai-brains-cli/tests/cli_capture_smoke.rs (0.13, 2 changes)
  crates/ai-brains-cli/tests/ingest_reads_json_stdin.rs (0.13, 2 changes)
  ```
- This reduces word count and improves scanability for the LLM

### Phase 4: `recall` — Smarter Search

**Task 4.1**: Add optional `score` field to `RecallResult`
- In `contracts/recall.rs`: add `#[serde(skip_serializing_if = "Option::is_none")] pub score: Option<f64>`
- Populate from FTS5 BM25 rank in `lexical.rs`

**Task 4.2**: Add `--format` flag (default: `json`)
- `--format pretty` prints results as readable text with memory IDs and content snippets

**Task 4.3**: Suggestive empty results
- When `results: []`, add stderr message: `"No results for 'cross-project memory bleed'. Try shorter terms or check spelling."`
- Does NOT change JSON response format — suggestions go to stderr

### Phase 5: `pin` — Workflow Improvements

**Task 5.1**: Echo memory ID on success
- Current output: `"Memory successfully pinned to vault."`
- New output: `"Memory abc123-def-456 successfully pinned to vault."`
- Still parseable, adds actionable information

**Task 5.2**: Accept stdin for content
- `ai-brains pin --stdin` reads content from stdin instead of positional arg
- Enables piping long content: `cat long-decision.txt | ai-brains pin --stdin`

**Task 5.3**: Add `--tag` flag (repeatable)
- `ai-brains pin "DECISION: ..." --tag architecture --tag database`
- Tags stored as metadata in the memory content (formatted as `"TAGS: architecture, database"` prefix)
- No schema changes — embedded in content string for backward compat

### Phase 6: `ingest` — Better Error Messages

**Task 6.1**: Field-specific validation errors
- Add a `ValidationError` type in capture errors with field name and message
- Parse errors map JSON path to field name: `"invalid 'turn_id': expected UUID format"`
- Report all validation errors at once (collect, don't fail-fast)

**Task 6.2**: Privacy value aliases
- Accept lowercase/snake_case in JSON: `"local_only"`, `"cloud_ok"`, `"never_inject"`, `"sealed"` as aliases for the PascalCase variants
- Backward compatible — PascalCase still works

### Phase 7: `nightly` — Graceful Defaults

**Task 7.1**: Validate project_id
- If `AI_BRAINS_PROJECT_ID` is unset or parses to all-zeros, print warning: "AI_BRAINS_PROJECT_ID not set. Run 'ai-brains context' first. Using default project." to stderr
- Continue execution (non-fatal) but make the user aware

**Task 7.2**: Progress indicators
- Print service name before execution: "Summarizing sessions...", "Running memory synthesis...", "Cleaning up retention..."
- Print count after each: "3 sessions summarized", "2 memories synthesized", "5 expired memories cleaned up"

**Task 7.3**: `--schedule` actually registers the task
- Current: prints a command string for the user to run manually
- New: `--schedule` creates the Windows scheduled task directly via `Register-ScheduledTask` or `schtasks.exe`
- `--unschedule` removes the task

### Phase 8: `forget` — Content-Based Workflow

**Task 8.1**: Add `--match` flag for content-based lookup
- `ai-brains forget --match "ingest command test"` searches for matching memories, lists them with IDs, asks for confirmation
- If exactly one match, forgets it directly (with confirmation)
- If multiple matches, lists them and exits (user re-runs with specific ID)
- Uses same `lexical_search` as `recall`

**Task 8.2**: Add confirmation prompt for direct UUID
- `ai-brains forget <uuid>` shows the memory's first line and asks "Forget this memory? [y/N]"
- `--force` / `-f` flag skips confirmation for scripting

**Task 8.3**: Add `--list-forgotten` flag
- Lists all memories with status='forgotten', showing memory_id and first content line
- Add `--restore <uuid>` to un-forget (sets status back to 'pinned' via compensating event)

### Phase 9: `backup` — Restore Support

**Task 9.1**: Add `backup restore <path>` subcommand
- Validates the backup file exists and is a valid SQLite database
- Warns if current vault has data: "Current vault will be replaced. Continue? [y/N]"
- Backs up the current vault before restoring (safety net)
- Copies backup over the vault file

**Task 9.2**: Add `--output-dir` flag
- `ai-brains backup --output-dir D:\backups` specifies custom directory
- Default unchanged: `<vault_parent>/backups/`

**Task 9.3**: Integrity verification
- After backup, run `PRAGMA integrity_check` on the backup file
- Report result: "Backup created and verified: <path>" or "Backup created but verification failed: <error>"
- Use SQLite backup API (`sqlite3_backup_init`) instead of raw file copy for consistency

### Phase 10: CI Gate & Conductor Registration

**Task 10.1**: Register in conductor.md
- Add T36 row to track table with status, owner, link

**Task 10.2**: Full CI gate
- `cargo fmt --check ; cargo clippy --workspace --all-targets -- -D warnings ; cargo test --workspace`

**Task 10.3**: ChangeGuard impact scan before implementation begins

## Verification

### Automated
```powershell
# Unit tests for new functionality
cargo test -p ai-brains-cli -- safety
cargo test -p ai-brains-cli -- context
cargo test -p ai-brains-cli -- forget
cargo test -p ai-brains-retrieval -- lexical
cargo test -p ai-brains-retrieval -- preflight

# Full workspace
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

### Manual
1. `ai-brains context --show` — displays current IDs without mutating
2. `ai-brains context` with existing session — warns, doesn't overwrite without `--new-session`
3. `ai-brains safety sync --dry-run` — shows what would sync
4. `ai-brains safety sync --json` — uses structured ChangeGuard output
5. `ai-brains preflight --pretty` — readable text output
6. `ai-brains pin "test"` — echoes memory ID on success
7. `ai-brains forget --match "test"` — finds and forgets by content
8. `ai-brains backup restore <path>` — restores from backup
9. All 4 hook scripts still function unchanged after improvements
