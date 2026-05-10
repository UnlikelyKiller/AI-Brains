# Antigravity Integration with AI-Brains

Antigravity does not support session hooks, so AI-Brains cannot automatically capture conversation turns during an active session. However, Antigravity stores full conversation logs at `~/.gemini/antigravity/brain/<id>/.system_generated/logs/overview.txt`. AI-Brains can import these logs via `ai-brains antigravity-import` or automatically during `ai-brains nightly`. During an active session, you must still pin important information manually.

## What Does NOT Happen Automatically

- Per-turn ingestion of assistant responses
- Session start preflight context injection
- Session end finalization
- Pre-compaction context rescue

These all require hooks (Stop, SessionStart, SessionEnd, PreCompact) which Antigravity does not support.

## Manual Capture Workflow

Since nothing is captured automatically, you are responsible for persisting anything worth remembering. Use `ai-brains pin` for this.

### After Making a Decision

```
ai-brains pin "DECISION: chose SQLite over LadybugDB for the graph backend because LadybugDB requires MSVC"
```

### After Discovering a Constraint

```
ai-brains pin "CONSTRAINT: cargo deny check must pass before every commit — AGPL dependencies are rejected"
```

### After Recording an Invariant

```
ai-brains pin "INVARIANT: never update or delete raw events — use compensating events for corrections"
```

### After a User Correction

```
ai-brains pin "CONSTRAINT: user prefers single bundled PRs over many small ones" --role user
```

## Orientation at Session Start

Run these commands at the beginning of every Antigravity session to load context:

```powershell
# 1. Sync safety signals from ChangeGuard
ai-brains safety sync

# 2. Load recent project state and constraints
ai-brains preflight --max-words 1000
```

This is what the Claude Code and Gemini hooks do automatically. In Antigravity you must do it yourself.

## Searching Memory

When you need to recall past decisions or constraints:

```powershell
ai-brains recall "database choice"
```

This searches the relational graph for pinned memories matching the query.

## Pin Format Reference

| Prefix | Purpose | Example |
|---|---|---|
| `DECISION:` | A choice made and why | `DECISION: migrated to SQLite CTEs for graph traversal` |
| `CONSTRAINT:` | A rule or limitation | `CONSTRAINT: Windows-first paths must handle UNC prefixes` |
| `INVARIANT:` | An unbreakable rule | `INVARIANT: never unwrap() in production code` |

## Frequency Guidance

Pin early and pin often. Without hooks, there is no safety net. If you close the session without pinning, that knowledge is gone. Prefer many small pins over one large one — each pin is indexed separately for recall.

## Commands Reference

| Action | Command | Notes |
|---|---|---|
| Orient yourself | `ai-brains preflight --max-words 1000` | Run at session start |
| Sync safety | `ai-brains safety sync` | Run at session start |
| Search memory | `ai-brains recall "topic"` | Searches pinned memories |
| Record a decision | `ai-brains pin "DECISION: ..."` | Manual capture only |
| Record a constraint | `ai-brains pin "CONSTRAINT: ..."` | Manual capture only |
| Record user correction | `ai-brains pin "..." --role user` | Marks it as user-sourced |
| Import Antigravity sessions | `ai-brains antigravity-import --days 7` | Also runs automatically with nightly |
| Nightly audit | `ai-brains nightly` | Heavy batch — includes Antigravity import |
| Initialize project | `ai-brains context` | Creates .env with project IDs |

## Automatic Import via Nightly

When you run `ai-brains nightly`, Antigravity sessions from `~/.gemini/antigravity/brain/` are automatically discovered and imported. This happens **before** summarization, so imported sessions get summarized in the same nightly run.

You can also run `ai-brains antigravity-import` manually to import recent sessions without waiting for nightly:

```powershell
# Import sessions from the last 7 days
ai-brains antigravity-import --days 7

# Import sessions from the last 30 days (default)
ai-brains antigravity-import
```

Import is idempotent — sessions that already exist in the vault are skipped. Mandate #4 is enforced: hidden thinking and tool-only calls are filtered out; only user prompts and final assistant responses are captured.

## What NOT to Do

- **Do not run `ai-brains ingest`** — it requires structured JSON on stdin and is designed for hook pipelines, not interactive use.
- **Do not run `ai-brains nightly` as a substitute for pinning** — it summarizes existing sessions but cannot capture a session that has no recorded turns.
- **Do not assume memory exists for recent sessions** — `antigravity-import` and `nightly` import past sessions, but only after they end. During an active session, pin manually.