# Codex Hooks Research

This document summarizes the research into the Codex hook system and how to implement it for AI-Brains.

## Summary of Findings

Codex discovers hooks from `hooks.json` files next to active config layers or from inline `[hooks]` tables in `config.toml`. The current hook schema uses PascalCase event keys, and hook scripts receive one JSON object on `stdin`.

### Global Configuration Locations

| Location | Windows Path | Scope |
| :--- | :--- | :--- |
| `~/.codex/config.toml` | `%USERPROFILE%\.codex\config.toml` | Feature flag & layer trust |
| `~/.codex/hooks.json` | `%USERPROFILE%\.codex\hooks.json` | Hook definitions (Global) |

### Critical Lifecycle Events for AI-Brains

AI-Brains uses three events for memory synchronization:

| Event | When It Fires | AI-Brains Action |
| :--- | :--- | :--- |
| **`SessionStart`** | Startup, resume, or clear-created session | **Preflight**: Injects initial memory context through `hookSpecificOutput.additionalContext`. |
| **`UserPromptSubmit`** | Before every prompt | **Ingest**: Captures the user's intent immediately. |
| **`Stop`** | End of turn | **Ingest**: Captures the final assistant response. |

### Technical Nuances

*   **Concurrency**: Multiple matching command hooks for the same event run concurrently.
*   **Stdout protocol**: JSON on `stdout` is parsed as hook output. `SessionStart` and `UserPromptSubmit` can also use plain text as extra developer context, but AI-Brains emits JSON for consistency.
*   **Feature Flag**: Codex CLI v0.130.0 expects `[features].hooks = true`. The public hooks docs still show `codex_hooks = true`, but the local CLI emits a deprecation warning for `codex_hooks` and says to use `hooks`.
*   **Matchers**: `SessionStart` matchers apply to the start source (`startup`, `resume`, `clear`). `UserPromptSubmit` and `Stop` ignore matchers.
*   **Event casing**: Current Codex uses `SessionStart`, `UserPromptSubmit`, and `Stop` as `hooks.json` keys. Older snake_case keys do not match current documentation.
*   **PowerShell quoting**: Literal triple-backtick strings must be single-quoted in PowerShell because the backtick is PowerShell's escape character.

---

## Implementation Roadmap

To implement these hooks for AI-Brains on Windows:

### Stage 1: Script Deployment
Deploy the Codex-specific adapter script.
- **Source**: `AI-Brains\scripts\target-codex-hook.ps1`
- **Destination**: `C:\Users\RyanB\.ai-brains\scripts\target-codex-hook.ps1`

### Stage 2: Configuration Registration

#### 1. Enable Feature Flag
Update `C:\Users\RyanB\.codex\config.toml`:
```toml
[features]
hooks = true
```

#### 2. Register Hooks
Create or update `C:\Users\RyanB\.codex\hooks.json`:

```json
{
  "hooks": {
    "SessionStart": [
      {
        "matcher": "startup|resume|clear",
        "hooks": [
          {
            "name": "ai-brains-preflight",
            "type": "command",
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
            "name": "ai-brains-user-prompt",
            "type": "command",
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
            "name": "ai-brains-ingest",
            "type": "command",
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

### Compatibility Note

`target-codex-hook.ps1` normalizes the old snake_case event names to the current PascalCase names before dispatching. This keeps manual smoke tests or stale installations from failing silently, but the installed Codex `hooks.json` should still use the current PascalCase keys.

### Documentation Drift Note

As of Codex CLI v0.130.0, startup warns that `[features].codex_hooks` is deprecated and should be replaced by `[features].hooks`. The OpenAI hooks/config docs still mention `codex_hooks`; prefer the installed CLI warning for the active runtime until the docs catch up.

---
*Research updated on: 2026-05-12*
