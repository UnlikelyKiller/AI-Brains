# AI-Brains Hook System for Claude Code

This document details the configuration, architecture, and troubleshooting steps for the project-aware memory hooks used with Claude Code.

## Overview

The Claude Code hook system automates four critical lifecycles:

1.  **Preflight Injection (`SessionStart`):** Fetches relevant project context from AI-Brains and injects it into Claude's context before the first prompt.
2.  **Memory Ingestion (`Stop`):** Captures Claude's most recent response from the session transcript and records it in the AI-Brains vault.
3.  **Final Ingestion (`SessionEnd`):** Performs a last-chance ingest when the session closes, catching any responses that weren't captured by the `Stop` hook.
4.  **Pre-Compaction Capture (`PreCompact`):** Ingests the last several assistant turns before Claude Code compacts (trims) the conversation context, preserving memories that would otherwise be lost.

## Why a Separate Script

Claude Code and Gemini CLI have different hook input/output formats. A shared script would require branching logic for every parse and emit step. Instead, each CLI gets its own thin adapter script that translates between the CLI's JSON schema and the `ai-brains` CLI:

| Concern | Gemini CLI | Claude Code |
|---|---|---|
| Input format | `hook_event_name` + `payload` envelope | `hook_event_name` + `session_id` + `transcript_path` + `cwd` envelope |
| Context injection | `hookSpecificOutput.additionalContext` | `hookSpecificOutput.additionalContext` (same shape, different event names) |
| Env persistence | Not available | `$env:CLAUDE_ENV_FILE` for writing exports that survive into Bash |
| Project directory | `$env:GEMINI_PROJECT_DIR` | `cwd` field in stdin JSON, or `$env:CLAUDE_PROJECT_DIR` |
| Ingest source | `payload.final_response` or `payload.prompt_response` | Transcript JSONL file at `transcript_path` |

Because these differences are structural, each CLI has its own entry-point script under `~/.ai-brains/scripts/`.

## Event Mapping

Gemini and Claude Code use different event names for the same lifecycle points. The mapping:

| Lifecycle | Gemini CLI Event | Claude Code Event | Claude Code Matcher |
|---|---|---|---|
| Session boot | `SessionStart` | `SessionStart` | `""` (all sources) |
| Per-turn context | `BeforeAgent` | *(not wired by default)* | — |
| Response captured | `AfterAgent` | `Stop` | `""` (all sources) |
| Session close | *(not wired)* | `SessionEnd` | `""` (all sources) |
| Pre-compaction | *(not applicable)* | `PreCompact` | `""` (all sources) |

### Why `SessionEnd`?

The `Stop` hook fires after each assistant turn, but there are scenarios where it may miss a final response — for example, if the session terminates before the `Stop` hook completes, or if the hook encounters a transient error. `SessionEnd` acts as a safety net, running one last ingest when the session closes. It scans a wider 100-line window (vs. `Stop`'s 50) to catch anything that slipped through.

### Why `PreCompact`?

When a Claude Code session runs long, the CLI compacts (trims) older conversation context to stay within the model's context window. Any assistant responses that get trimmed are permanently lost to the session — and if they were never ingested, they're lost to AI-Brains as well. `PreCompact` fires immediately before compaction occurs, giving the hook a chance to ingest the last 3 assistant turns (scanning up to 200 lines) before they're discarded. The hook always returns `{ "continue": true }` so compaction proceeds without delay.

### Why not `UserPromptSubmit`?

Claude Code's `UserPromptSubmit` event fires before every user prompt, making it the closest equivalent to Gemini's `BeforeAgent`. However, running `ai-brains preflight` on every prompt adds latency to each turn. The preflight context injected at `SessionStart` is sufficient for most sessions. If per-turn refresh is needed later, add a `UserPromptSubmit` hook entry to `settings.json` pointing at the same script — the `switch` block will route it correctly.

## Configuration

The hooks are configured globally in `C:\Users\RyanB\.claude\settings.json`.

```json
{
  "autoUpdatesChannel": "latest",
  "theme": "dark",
  "skipDangerousModePermissionPrompt": true,
  "hooks": {
    "SessionStart": [
      {
        "matcher": "",
        "hooks": [
          {
            "type": "command",
            "name": "ai-brains-preflight",
            "command": "powershell -NoProfile -Command \"& 'C:\\Users\\RyanB\\.ai-brains\\scripts\\target-claude-hook.ps1'\""
          }
        ]
      }
    ],
    "Stop": [
      {
        "matcher": "",
        "hooks": [
          {
            "type": "command",
            "name": "ai-brains-ingest",
            "command": "powershell -NoProfile -Command \"& 'C:\\Users\\RyanB\\.ai-brains\\scripts\\target-claude-hook.ps1'\""
          }
        ]
      }
    ],
    "SessionEnd": [
      {
        "matcher": "",
        "hooks": [
          {
            "type": "command",
            "name": "ai-brains-session-end",
            "command": "powershell -NoProfile -Command \"& 'C:\\Users\\RyanB\\.ai-brains\\scripts\\target-claude-hook.ps1'\""
          }
        ]
      }
    ],
    "PreCompact": [
      {
        "matcher": "",
        "hooks": [
          {
            "type": "command",
            "name": "ai-brains-pre-compact",
            "command": "powershell -NoProfile -Command \"& 'C:\\Users\\RyanB\\.ai-brains\\scripts\\target-claude-hook.ps1'\""
          }
        ]
      }
    ]
  }
}
```

All events invoke the same script. The script reads `hook_event_name` from stdin and branches accordingly.

### Why `powershell -NoProfile`?

`-NoProfile` skips loading `$PROFILE` scripts, which saves ~200–500ms per invocation. Since the hook must return within the CLI's timeout window, this margin matters. The script does not depend on any profile-defined aliases or modules.

## Hook Script Architecture

The core logic resides in `C:\Users\RyanB\.ai-brains\scripts\target-claude-hook.ps1`.

### Stdin Handling

Claude Code sends a JSON envelope on stdin. The script reads it with `[Console]::In.ReadToEnd()`, the same robust method used in the Gemini hook:

```powershell
$stdin = [Console]::In.ReadToEnd()
if (-not $stdin) { $stdin = $input | Out-String }
$inputJson = $stdin | ConvertFrom-Json
```

If stdin is empty or unparseable, the script emits `{ "continue": true }` and exits 0 — never blocking the session.

### Environment Loading

The script loads `.env` files from two locations, in order:

1.  **Project-local**: `$inputJson.cwd/.env` (or `$env:CLAUDE_PROJECT_DIR/.env` as fallback)
2.  **Global**: `$HOME/.ai-brains/.env`

This ensures `AI_BRAINS_PROJECT_ID`, `AI_BRAINS_SESSION_ID`, `AI_BRAINS_VAULT_PATH`, and model URLs are available regardless of whether the CLI inherited them.

### SessionStart — Preflight

1.  Runs `ai-brains preflight --max-words 1500` to generate context text.
2.  Writes environment variables (`AI_BRAINS_PROJECT_ID`, `AI_BRAINS_SESSION_ID`, `AI_BRAINS_HARNESS_ID`) to `$env:CLAUDE_ENV_FILE` so they survive into subsequent Bash tool calls in the same session. This is a Claude Code-specific mechanism — Gemini does not have an equivalent.
3.  Emits JSON with `additionalContext` containing the preflight text. Claude Code injects this into the conversation as a system reminder at the start of the session.

```json
{
  "continue": true,
  "hookSpecificOutput": {
    "hookEventName": "SessionStart",
    "additionalContext": "<preflight text from ai-brains>"
  }
}
```

If `ai-brains preflight` fails (non-zero exit), the script logs to stderr and emits `{ "continue": true }` without context — the session proceeds uninterrupted, just without injected memories.

### Stop — Ingest

1.  Reads `transcript_path` from the hook input. Claude Code writes the full session transcript to this JSONL file.
2.  Scans the last 50 lines of the transcript for the most recent assistant message (identified by `role: "assistant"` or `type: "assistant"`).
3.  Extracts text content, handling both plain strings and array-of-content-blocks formats.
4.  Attempts the **local ingest script** at `.agents/skills/ai-brains/scripts/ingest.ps1` first, falling back to `ai-brains ingest` CLI directly.
5.  Emits `{ "continue": true }` regardless of ingestion success — ingestion failures never block the session.

The fallback path constructs a JSON payload and pipes it through `ai-brains ingest` using a temp file to avoid PowerShell encoding issues:

```powershell
$ingestPayload = @{
    session_id = $env:AI_BRAINS_SESSION_ID
    project_id = $env:AI_BRAINS_PROJECT_ID
    harness_id = "claude-code"
    turn_id = [guid]::NewGuid().ToString()
    role = "assistant"
    content = $lastContent
    privacy = "LocalOnly"
} | ConvertTo-Json -Compress
```

### SessionEnd — Final Ingestion

Fires when a Claude Code session terminates (user exits, session cleared, etc.). This is a safety net for responses that may not have been captured by the `Stop` hook.

1.  Reads `transcript_path` from the hook input (same as `Stop`).
2.  Scans the last 100 lines of the transcript for the most recent assistant message — a wider window than `Stop`'s 50 lines, since this is the last chance to capture anything missed.
3.  Follows the same ingest pipeline as `Stop`: local `ingest.ps1` first, then `ai-brains ingest` CLI fallback.
4.  Emits `{ "continue": true }` regardless of outcome. `SessionEnd` has no decision control — it cannot block the session close.

The wider scan window exists because `SessionEnd` may fire after several rapid turns where `Stop` only caught the most recent response. The extra lines increase the chance of capturing a message that was skipped or partially ingested.

### PreCompact — Context Preservation Before Compaction

Fires immediately before Claude Code compacts (trims) the conversation context to stay within the model's token limit. Without this hook, any assistant responses that get trimmed are permanently lost from the session, and if they were never ingested by `Stop`, they're lost from AI-Brains as well.

1.  Reads `transcript_path` from the hook input.
2.  Scans the last 200 lines of the transcript, collecting up to 3 recent assistant messages (not just 1, since compaction risks losing multiple turns at once).
3.  Joins the collected messages with `---` separators and ingests them as a single combined payload.
4.  Follows the same ingest pipeline: local `ingest.ps1` first, then `ai-brains ingest` CLI fallback.
5.  Emits `{ "continue": true }` — the hook **never blocks compaction**. Delaying compaction would degrade the user experience and is not appropriate for a side-effect-only hook like memory ingestion.

The 3-turn capture window is a deliberate choice: compaction typically removes the oldest material first, but the exact number of turns at risk depends on their length. Capturing 3 turns balances coverage against ingest latency. If this proves insufficient for long sessions, the window can be increased in the script.

### The Golden Rule (Silence)

All internal command output (`ai-brains ingest`, `ingest.ps1`) is redirected to `$null`. The only stdout the script produces is the required JSON response object. All diagnostic information goes to stderr with the `[ai-brains-claude]` prefix, which Claude Code displays in the terminal but does not parse as hook output.

## File Map

| File | Role |
|---|---|
| `~/.claude/settings.json` | Hook trigger configuration (Claude Code reads this on startup) |
| `~/.ai-brains/scripts/target-claude-hook.ps1` | Adapter script for Claude Code events |
| `~/.ai-brains/scripts/target-gemini-hook.ps1` | Adapter script for Gemini CLI events (unchanged) |
| `~/.ai-brains/.env` | Global environment (vault path, model URLs) |
| `.agents/skills/ai-brains/scripts/ingest.ps1` | Project-local ingestion worker |

## Troubleshooting

### Missing Context

If preflight context is not appearing in Claude Code sessions:

1.  Verify the `SessionStart` hook exists in `~/.claude/settings.json`.
2.  Run `ai-brains preflight --max-words 1500` manually to confirm the vault is accessible.
3.  Check stderr output for `[ai-brains-claude]` entries.
4.  Confirm `~/.ai-brains/.env` contains `AI_BRAINS_VAULT_PATH` pointing to a valid vault.

### Failed Ingestion

If memories are not appearing in `ai-brains recall`:

1.  Verify the `Stop`, `SessionEnd`, and `PreCompact` hooks exist in `~/.claude/settings.json`.
2.  Check stderr for `[ai-brains-claude]` entries — look for "No transcript available" or "Ingest failed" messages.
3.  Confirm `AI_BRAINS_PROJECT_ID` is set in the project-local `.env` file.
4.  If the project has `.agents/skills/ai-brains/scripts/ingest.ps1`, ensure it is present and executable.
5.  Test manually: pipe a JSON payload to `ai-brains ingest` and verify the vault receives it.

### PreCompact Not Firing

Compaction only occurs during long sessions. If `PreCompact` never fires:

1.  The session may simply not be long enough to trigger compaction — this is normal for short sessions.
2.  Compaction can also be triggered manually via `/compact` in Claude Code.
3.  Check stderr for `[ai-brains-claude]` entries with the `PreCompact:` prefix to confirm the hook is running.

### SessionEnd Not Firing

`SessionEnd` fires when the Claude Code process exits normally. It may not fire if:

1.  The process is killed (e.g., `kill -9`, system crash, power loss).
2.  The terminal is force-closed without a graceful shutdown.
3.  In these cases, the `Stop` hook should still have captured the last completed turn.

### Hook Not Firing

If neither hook appears to run:

1.  Confirm `settings.json` is at `~/.claude/settings.json` (not a project-level `.claude/settings.json` which may not include the hooks).
2.  Check that the PowerShell script path in the `command` field is correct and the file exists.
3.  Run the script manually to check for syntax errors: `powershell -NoProfile -Command "& '~\.ai-brains\scripts\target-claude-hook.ps1'"` with appropriate stdin.

### CLAUDE_ENV_FILE Not Persisting

If environment variables like `AI_BRAINS_SESSION_ID` are not available in subsequent tool calls:

1.  The `CLAUDE_ENV_FILE` mechanism is only available during `SessionStart`, `Setup`, `CwdChanged`, and `FileChanged` events. Verify the hook is firing during `SessionStart`.
2.  Check that the script writes `export` statements (bash syntax) to the file, not PowerShell `Set-Item` calls.

## Differences from Gemini Hooks

| Aspect | Gemini CLI | Claude Code |
|---|---|---|
| Config location | `~/.gemini/settings.json` | `~/.claude/settings.json` |
| Per-turn context | `BeforeAgent` on every turn | `SessionStart` only (per-turn not wired by default) |
| Session close | *(not wired)* | `SessionEnd` — final ingest safety net |
| Pre-compaction | *(not applicable)* | `PreCompact` — preserves context before trimming |
| Project dir source | `$env:GEMINI_PROJECT_DIR` | `cwd` in stdin JSON |
| Response capture | `payload.final_response` in hook stdin | Read from `transcript_path` JSONL file |
| Env persistence | Not available | `$env:CLAUDE_ENV_FILE` for cross-tool-call env |
| Hook matcher syntax | `"*"` matches all | `""` or omitted matches all |