# Codex Hooks Research

This document summarizes the research into the Codex hook system and how to implement it for AI-Brains.

## Summary of Findings

Codex utilizes a robust hook system that discovery hooks from `hooks.json` or inline `[hooks]` tables in `config.toml`. It is particularly effective for turn-level capture and context injection.

### Global Configuration Locations

| Location | Windows Path | Scope |
| :--- | :--- | :--- |
| `~/.codex/config.toml` | `%USERPROFILE%\.codex\config.toml` | Feature flag & layer trust |
| `~/.codex/hooks.json` | `%USERPROFILE%\.codex\hooks.json` | Hook definitions (Global) |

### Critical Lifecycle Events for AI-Brains

AI-Brains uses three turn-scoped events for reliable memory synchronization:

| Event | When It Fires | AI-Brains Action |
| :--- | :--- | :--- |
| **`session_start`** | Startup/Resume | **Preflight**: Injects initial memory context. |
| **`user_prompt_submit`** | Before every prompt | **Ingest**: Captures the user's intent immediately. |
| **`stop`** | End of turn | **Ingest**: Captures the final assistant response. |

### Technical Nuances

*   **Concurrency**: Multiple matching hooks run concurrently.
*   **Silence**: Similar to other CLI hooks, Codex expects valid JSON on `stdout`.
*   **Feature Flag**: Hooks must be explicitly enabled in `config.toml` via `hooks = true`.
*   **Matchers**: Matchers use Regular Expressions for tool events and Exact Strings for lifecycle events.

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
    "session_start": [
      {
        "matcher": "*",
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
    "user_prompt_submit": [
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
    "stop": [
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

---
*Research conducted on: 2026-05-11*
