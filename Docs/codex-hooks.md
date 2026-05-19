# AI-Brains Hook System for Codex CLI

This document covers the Codex CLI adapter for AI-Brains. The adapter is Windows-first, user-level, and JSON-only on stdout so it does not break Codex's hook protocol.

## Current Capability

Codex is treated as a **Level 4 adapter: hook prompt + final capture**.

It can reliably capture:

- Session startup context injection.
- User prompts via `UserPromptSubmit`.
- Final assistant responses via `Stop`.

It cannot currently capture:

- A true whole-session close event equivalent to Claude Code `SessionEnd`.
- A pre-compaction event equivalent to Claude Code `PreCompact`.

Codex does compact long conversations internally, but no native `PreCompact`, `Compact`, or `PostCompact` hook is exposed. AI-Brains therefore relies on prompt/final turn capture instead of a compaction boundary.

## Installed Files

Codex hooks are enabled globally in:

```text
C:\Users\RyanB\.codex\config.toml
```

Required feature flag:

```toml
[features]
codex_hooks = true
```

The global hook configuration lives at:

```text
C:\Users\RyanB\.codex\hooks.json
```

All configured events call the same adapter:

```text
C:\Users\RyanB\.ai-brains\scripts\target-codex-hook.ps1
```

The repo source copy is:

```text
AI-Brains\scripts\target-codex-hook.ps1
```

After editing the repo copy, reinstall it to the user-level script path before expecting Codex to run the change.

## Event Mapping

| Codex event | Hook name | AI-Brains role | Purpose |
|---|---|---|---|
| `SessionStart` | `ai-brains-preflight` | Query only | Loads memory context and injects it before the first model turn. |
| `UserPromptSubmit` | `ai-brains-user-prompt` | `user` | Captures only the newly submitted user prompt and does not inject memory context. |
| `Stop` | `ai-brains-ingest` | `assistant` | Captures the final assistant response for the completed turn. |

Tool hooks are intentionally not used for canonical capture:

| Codex event | Recommendation |
|---|---|
| `PreToolUse` | Optional future guardrail only. Do not store tool input. |
| `PostToolUse` | Avoid for memory capture. Tool output is not canonical memory. |
| `PermissionRequest` | Optional future privacy/security policy hook. Not needed for capture. |

This follows the implementation plan rule that AI-Brains must not store hidden thinking, raw tool calls, tool results, actions, traces, or scratchpads.

## hooks.json

Current installed configuration:

```json
{
  "hooks": {
    "SessionStart": [
      {
        "matcher": "startup|resume|clear",
        "hooks": [
          {
            "type": "command",
            "name": "ai-brains-preflight",
            "command": "powershell -NoProfile -Command \"& 'C:\\Users\\RyanB\\.ai-brains\\scripts\\target-codex-hook.ps1'\"",
            "timeout": 30,
            "statusMessage": "Loading AI-Brains context"
          }
        ]
      }
    ],
    "UserPromptSubmit": [
      {
        "hooks": [
          {
            "type": "command",
            "name": "ai-brains-user-prompt",
            "command": "powershell -NoProfile -Command \"& 'C:\\Users\\RyanB\\.ai-brains\\scripts\\target-codex-hook.ps1'\"",
            "timeout": 30,
            "statusMessage": "Capturing AI-Brains prompt"
          }
        ]
      }
    ],
    "Stop": [
      {
        "hooks": [
          {
            "type": "command",
            "name": "ai-brains-ingest",
            "command": "powershell -NoProfile -Command \"& 'C:\\Users\\RyanB\\.ai-brains\\scripts\\target-codex-hook.ps1'\"",
            "timeout": 30,
            "statusMessage": "Capturing AI-Brains memory"
          }
        ]
      }
    ]
  }
}
```

## Adapter Behavior

The adapter reads Codex's JSON hook envelope from stdin and branches on `hook_event_name`.

It loads environment values in this order:

1. Project-local `.env` from the hook `cwd`.
2. Global `~\.ai-brains\.env`.

The project-local file usually provides `AI_BRAINS_PROJECT_ID`, `AI_BRAINS_SESSION_ID`, and `AI_BRAINS_HARNESS_ID`. The global file usually provides vault/model configuration.

### SessionStart

1. Runs `ai-brains preflight --max-words 1500`.
2. Appends a session skill-routing gate reminding Codex to check mandatory or applicable skills before choosing tools, research backends, or implementation workflows.
3. If preflight succeeds, returns:

```json
{
  "continue": true,
  "hookSpecificOutput": {
    "hookEventName": "SessionStart",
    "additionalContext": "<preflight text plus skill-routing gate>"
  }
}
```

4. If preflight fails, logs to stderr and returns `{ "continue": true }`.

### UserPromptSubmit

1. Reads the prompt from `prompt`, `user_prompt`, or `message`.
2. Records it as a `user` turn.
3. Prefers the project helper:

```text
.agents\skills\ai-brains\scripts\ingest.ps1
```

4. Falls back to direct `ai-brains ingest`.
5. Returns `{ "continue": true }` without `additionalContext`.

This keeps per-prompt capture incremental and prevents the full memory briefing from being re-injected into every model turn.

### Stop

1. Reads `last_assistant_message` from the Codex hook payload.
2. Falls back to scanning `transcript_path` JSONL when direct content is unavailable.
3. Records the content as an `assistant` turn.
4. Prefers the project helper, then falls back to direct `ai-brains ingest`.
5. Always returns `{ "continue": true }`.

`Stop` is an assistant-turn boundary, not a guaranteed whole-session close event.

## Safety Rules

- The adapter must write only valid JSON to stdout.
- Diagnostics must go to stderr with the `[ai-brains-codex]` prefix.
- Hook failures must not block Codex; return `{ "continue": true }`.
- Do not capture tool calls, tool results, hidden thinking, reasoning traces, or scratchpads.
- Do not use Codex tool hooks as canonical memory capture.
- Keep capture independent of graph, embeddings, nightly jobs, and local model availability.

## Smoke Test

Run synthetic hooks and verify stdout parses as JSON:

```powershell
$payload = @{
  hook_event_name = 'UserPromptSubmit'
  session_id = 'smoke-session'
  turn_id = 'smoke-user-turn'
  cwd = 'C:\dev'
  prompt = 'Synthetic Codex UserPromptSubmit hook smoke test.'
  model = 'gpt-5.5'
  permission_mode = 'default'
} | ConvertTo-Json -Compress

$payload | powershell -NoProfile -Command "& 'C:\Users\RyanB\.ai-brains\scripts\target-codex-hook.ps1'"
```

Expected result: stdout is a single JSON object with `continue: true`. Stderr may contain `[ai-brains-codex]` diagnostics.

## Troubleshooting

If context is not injected:

1. Confirm `features.codex_hooks = true` in `C:\Users\RyanB\.codex\config.toml`.
2. Confirm `C:\Users\RyanB\.codex\hooks.json` contains `SessionStart`.
3. Run `ai-brains preflight --max-words 1500` manually.
4. Check that `.env` or `~\.ai-brains\.env` provides the vault path and project/session IDs.

If memories are not captured:

1. Confirm `Stop` and `UserPromptSubmit` are present in `hooks.json`.
2. Confirm the installed script matches `AI-Brains\scripts\target-codex-hook.ps1`.
3. Run `ai-brains recall "<recent prompt or answer>" --limit 5`.
4. Check stderr logs for `Missing project_id or session_id for direct ingest`.

If Codex compacts context:

- There is no Codex hook to intercept compaction.
- AI-Brains relies on already-captured `UserPromptSubmit` and `Stop` turns.
- Start a new Codex thread for very long tasks when accuracy starts to degrade.
