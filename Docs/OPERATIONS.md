# AI-Brains Operations Guide

This guide covers the day-to-day operations, configuration, and troubleshooting of the AI-Brains system.

> **Current state (June 2026):** Phase 15 (Cross-Agent Memory Synthesis) plus T44–T71 are shipped. The CLI has 17 top-level subcommands, the daemon auto-launches, nightly schedules via Windows Task Scheduler, and the ChangeGuard bridge is live. The Operations surface is significantly larger than the pre-T44 era this document originally described.

## 1. Installation and Setup

### Prerequisites
- Rust (Stable, MSVC toolchain)
- PowerShell 7+ (Recommended for Windows)
- `cargo-nextest`, `cargo-deny`, `cargo-audit` — see [ci-tooling.md](ci-tooling.md) for pins

### Build
```powershell
cargo build --release
```

### Initializing a Vault
Every project or user needs a vault. **T73 made `init` safe to re-run.**
- If the vault file does not exist, it is created and migrations are applied.
- If the vault is empty, the command succeeds idempotently.
- If the vault is populated, the command **refuses** with exit 1 unless `--force` is set.

```powershell
ai-brains --vault-path C:\path\to\vault.db init           # create new vault
ai-brains --vault-path C:\path\to\vault.db init           # re-run on empty vault: no-op
ai-brains --vault-path C:\path\to\vault.db init --force   # explicit overwrite
```

When the refused case triggers, the CLI returns a structured JSON error envelope on stderr (the same shape used by every other failure path) and exits 1.

## 2. Ingesting Data

AI-Brains follows an "Ingest-First" philosophy. All conversation data should be piped into the CLI as JSON.

### Manual Ingestion
```powershell
$json = @{
    session_id = "uuid"
    project_id = "uuid"
    harness_id = "uuid"
    turn_id    = "uuid"
    role       = "user"
    content    = "This is a memory."
    privacy    = "CloudOk"
} | ConvertTo-Json -Compress

echo $json | ai-brains --vault-path ./vault.db ingest
```

### Antigravity Import
Bulk-import Antigravity conversation logs from local tool-specific brain dirs.
```powershell
ai-brains antigravity-import --days 30
```
- `--days <N>`: only import sessions modified in the last N days (default 30).
- Idempotent: skips sessions already in the vault.
- Tool-only and hidden-thinking entries are filtered out (Mandate #4).

### `agy` Hook
Real-time capture from the Antigravity CLI hooks integration:
```powershell
ai-brains agy-hook --payload '{"transcriptPath": "C:\\path\\to\\session.jsonl", ...}'
```
A well-formed payload returns `{"ok":true,"status":"success",...}`. A malformed payload (e.g. missing `transcriptPath`) returns `{"ok":false,"status":"error","message":"..."}` — the harness hook treats this as a non-fatal failure.

## 3. Retrieving Memories

### Lexical Recall
```powershell
ai-brains --vault-path ./vault.db recall "authentication logic" --limit 5
```
Options worth knowing:
- `--format pretty` for human-readable scores
- `--semantic` for vector (embedding) search alongside FTS5
- `--graph-boost <0.0–1.0>` to weight graph-neighbor hits
- `--project-id` / `--session-id` to scope

### Unified Search (AI-Brains + ChangeGuard)
The T70 bridge lets a single command search both your memory vault and the ChangeGuard ledger.
```powershell
ai-brains sync query "rust" --format pretty
```
Output has two sections — `--- AI-Brains Recall ---` (vault FTS hits) and `--- ChangeGuard Ledger Search ---` (ledger entries). Use `--quiet` to suppress the second section if you only want the vault view.

### Generating Preflight Context
```powershell
ai-brains preflight --max-words 1500
```
- `--summary` for a concise statistical summary
- `--pretty` / `--format human` for human-readable text
- `--scope "src/foo.rs,src/bar.rs"` for contextual risk analysis on a specific path set

## 4. Project & Session Management

### Project Setup
```powershell
ai-brains context
```
This command generates a deterministic `PROJECT_ID` based on your directory and a fresh `SESSION_ID`, storing them in a local `.env` file. Subsequent operations (recall, ingest) automatically use these env values.

- `--show` — print current context without modifying `.env`
- `--new-project` — force a fresh project ID
- `--new-session` — rotate the session ID (useful for long sessions)
- `--tx-id <uuid>` — link the context to a ChangeGuard transaction (T37)

### Listing Projects
```powershell
ai-brains project list
```
Output (post-T76): a table with `project_id`, `name (alias|UUID)`, `alias`, and `memories` columns.

### Resolving Aliases
```powershell
ai-brains project resolve ai-brains          # exact alias match, falls back to fuzzy
ai-brains project detect --export            # auto-detect from current git repo
```

## 5. Background Intelligence & Scheduling

### Daemon Lifecycle
The `ai-brainsd` daemon is a single-writer queue that serializes event writes for concurrency safety.
```powershell
ai-brains daemon start             # start in background
ai-brains daemon status            # show PID + listening ports
ai-brains daemon stop              # graceful shutdown (use --force if it hangs)
ai-brains daemon schedule          # register Windows Task Scheduler logon task
ai-brains daemon unschedule        # remove the logon task
```
- The CLI auto-launches the daemon if it is unreachable, so most users never need `daemon start` explicitly.
- If the Task Scheduler call fails with `Access is denied`, run from an elevated PowerShell session.

### Nightly Intelligence Sweep
```powershell
ai-brains --vault-path ./vault.db nightly
```
The nightly job does:
- Antigravity session import (T33)
- Summarization of unsummarized sessions (with T34 chunking for sessions over 38,912 tokens)
- Memory synthesis (RAPTOR-style clustering + CRAG factual verification)
- Symbol-bridge ingestion from ChangeGuard (T70)
- MemoryPinned / MemorySynthesized event emission (T67, T68) for the live graph

### Scheduling Nightly
```powershell
ai-brains nightly --schedule --start-time "03:00"
ai-brains nightly --status             # show last run timestamp + pending work
ai-brains nightly --unschedule
```

## 6. Memory Hygiene

### Soft-Delete
```powershell
ai-brains forget --memory-id <uuid>           # prompt; -f to skip
ai-brains forget --match "outdated fact" -f   # find by content; -f to forget
ai-brains forget --list-forgotten             # show everything soft-deleted
ai-brains forget --restore <uuid>             # undo with a compensating event
```
Forgotten memories remain in the event log for audit but are excluded from FTS, graph, and preflight.

### Backup
```powershell
ai-brains backup                                # create with timestamped default path
ai-brains backup create --output-dir D:\backups # custom directory
```
Backups include an integrity check; corrupt backups are rejected at creation time.

### Restore
```powershell
ai-brains backup restore <path>               # interactive confirm + overwrite
ai-brains backup restore <path> --force       # non-interactive (CI/automation)
ai-brains backup restore <path> --dry-run     # verify integrity, report, no changes
```
`--dry-run` runs the integrity check, prints the planned destination, and exits 0 without touching the vault. Use it in scripts before a real restore.

## 7. Safety & Hotspot Sync

ChangeGuard scans the codebase for hotspots (frequently-edited, complex files). The bridge re-pins these as AI-Brains memories so they appear in preflight and recall.

```powershell
ai-brains safety sync                # sync top 5 hotspots
ai-brains safety sync --limit 20     # sync top 20
ai-brains safety sync --dry-run      # preview what would be synced
```

## 8. Troubleshooting

### `cargo audit` appears to hang
Plain `cargo audit` 0.22.x **exits 0 with no final summary line** on a clean run — the visible output ends with `Scanning Cargo.lock for vulnerabilities …`. This is a CLI behavior change, not a hang. To confirm a clean run:
```powershell
cargo audit --json
# => {"vulnerabilities":{"found":false,"count":0,"list":[]}, ...}
```
See [ci-tooling.md](ci-tooling.md#behavior-notes) for more.

### `init` refuses on a populated vault
Caused by T73's safety gate. The vault at the given path already contains projects. Re-run with `--force` to acknowledge the overwrite:
```powershell
ai-brains init --force
```

### `daemon schedule` reports "Access is denied"
The Task Scheduler registration requires elevation. Open PowerShell **as Administrator** and retry. The CLI prints the exact `schtasks` command it tried to run, which you can also paste manually.

### Recalls return only code files, not session memories
This is correct FTS5 behavior when no session context has been pinned. After a few ingest+recall cycles, the relevant session memory will surface. The `safety sync` command intentionally pins file paths as memories so the same query can return both kinds of result.

### Graph health check
```powershell
ai-brains graph update
```
Reports `{ nodes, edges, status: "live", note }`. If `status` is not `"live"` or counts are unexpectedly zero, run:
```powershell
ai-brains graph rebuild
```

### Vault Locked
If the vault cannot be opened, ensure the `AI_BRAINS_KEY` environment variable is set or the correct `--key` argument is provided.

### Missing Graph Database
If the graph features are missing on Windows, verify that the `graph` feature was enabled during build and that the MSVC 4GB image size limit was not exceeded. If it was, the system will gracefully fall back to Lexical search.

## 9. Environment Variables

| Variable | Description |
|---|---|
| `AI_BRAINS_VAULT_PATH` | Default path to the vault database. |
| `AI_BRAINS_KEY` | Hex-encoded SQLCipher key (or dummy in degraded mode). |
| `AI_BRAINS_PROJECT_ID` | Default `project_id` for capture/recall (set by `ai-brains context`). |
| `AI_BRAINS_SESSION_ID` | Default `session_id` (set by `ai-brains context`). |
| `CHANGEGUARD_TX_ID` | ChangeGuard transaction ID for ledger cross-linking (T37). |
| `AI_BRAINS_MODEL_URL` | Endpoint for the local LLM completion server (default: `http://127.0.0.1:8081`). |
| `AI_BRAINS_EMBEDDING_URL` | Endpoint for the local embedding server (default: `http://127.0.0.1:8083`). |
| `AI_BRAINS_EMBEDDING_MODEL` | Name of the embedding model (default: `nomic-embed-text-v1.5`). |
| `AI_BRAINS_COMPLETION_MODEL` | Name of the completion model (default: `gemma-4-E4B-it-Q6_K.gguf`). |
| `AI_BRAINS_SCOPE` | Comma-separated paths for preflight contextual risk analysis. |

## 10. Command Summary

| Action | Command |
|---|---|
| Initialize Vault | `ai-brains init` (use `--force` to overwrite populated vault) |
| Show Context | `ai-brains context --show` |
| Sync Safety Signals | `ai-brains safety sync` (use `--dry-run` to preview) |
| Unified Search | `ai-brains sync query "<topic>"` (searches vault + ChangeGuard) |
| Get Orientation | `ai-brains preflight` (use `--pretty` for full text, `--summary` for stats) |
| Deep Search | `ai-brains recall` (use `--format pretty` for readable results) |
| Pinned Record | `ai-brains pin` (use `--tag` for categories, `--stdin` piped) |
| Forget Memory | `ai-brains forget` (use `--match` for search, `--restore` undo, `-f` to skip confirm) |
| Antigravity Capture Hook | `ai-brains agy-hook --payload "{...}"` (used by agy CLI hooks) |
| Import Antigravity | `ai-brains antigravity-import --days 30` (incremental scan) |
| Nightly Sweep | `ai-brains nightly` (summarization + graph + bridge) |
| Schedule Nightly | `ai-brains nightly --schedule --start-time "03:00"` |
| Daemon Control | `ai-brains daemon start/status/stop/schedule/unschedule` |
| Backup Vault | `ai-brains backup` |
| Restore Vault | `ai-brains backup restore <path>` (use `--force` non-interactive, `--dry-run` to preview) |
| Manage Projects | `ai-brains project list/resolve/detect` |
| Graph Health | `ai-brains graph update` (use `graph rebuild` if stale) |
