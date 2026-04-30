# AI-Brains Hook System Documentation

This document details the configuration, architecture, and troubleshooting steps for the project-aware memory hooks used with the Gemini CLI.

## Overview

The hook system automates two critical lifecycles:
1.  **Preflight Injection (`BeforeAgent` / `SessionStart`):** Fetches relevant project context from AI-Brains and injects it into the AI's prompt before every turn.
2.  **Memory Ingestion (`AfterAgent`):** Captures the AI's final response and records it in the AI-Brains vault for future recall.

## Configuration

The hooks are configured globally in `C:\Users\RyanB\.gemini\settings.json`.

### Event Mapping
*   **`SessionStart`**: Triggers when a new session begins.
*   **`BeforeAgent`**: Triggers before every user prompt is processed by the agent.
*   **`AfterAgent`**: Triggers after the agent provides its final response.

### Settings Example
```json
{
  "hooks": {
    "SessionStart": [{ "matcher": "*", "hooks": [{ "name": "ai-brains-preflight", "type": "command", "command": "powershell \"C:\\Users\\RyanB\\.ai-brains\\scripts\\target-gemini-hook.ps1\"" }] }],
    "BeforeAgent": [{ "matcher": "*", "hooks": [{ "name": "ai-brains-preflight", "type": "command", "command": "powershell \"C:\\Users\\RyanB\\.ai-brains\\scripts\\target-gemini-hook.ps1\"" }] }],
    "AfterAgent": [{ "matcher": "*", "hooks": [{ "name": "ai-brains-ingest", "type": "command", "command": "powershell \"C:\\Users\\RyanB\\.ai-brains\\scripts\\target-gemini-hook.ps1\"" }] }]
  }
}
```

## Hook Script Architecture

The core logic resides in `C:\Users\RyanB\.ai-brains\scripts\target-gemini-hook.ps1`.

### Key Improvements (Fixed 2026-04-28)
1.  **Robust Stdin Handling**: Uses `[Console]::In.ReadToEnd()` to reliably capture JSON input from the Gemini CLI, bypassing common PowerShell pipeline issues.
2.  **Absolute Path Resolution**: Utilizes `$env:GEMINI_PROJECT_DIR` to find local `.env` files and `ingest.ps1` scripts, ensuring the hook works regardless of the current working directory.
3.  **The "Golden Rule" (Silence)**: All standard output from internal commands (like `ai-brains ingest`) is redirected to `$null`. This ensures the hook only returns the required JSON response to the CLI.
4.  **Stderr Logging**: All internal steps are logged to `stderr` with the `[ai-brains-hook]` prefix for easier debugging without breaking JSON compliance.
5.  **Environment Loading**: Manually loads project-specific `.env` files if they haven't been inherited, ensuring `AI_BRAINS_PROJECT_ID` and `AI_BRAINS_SESSION_ID` are available.

## Troubleshooting

### Identifying Loops
If the CLI appears to hang or repeat "Executing Hook", it may be due to a hook triggering another CLI command that in turn triggers a hook.
*   **Fix**: Ensure hooks only run for intended commands and that the hook script itself is optimized.
*   **Check Logs**: Look for `[ai-brains-hook]` entries in the terminal output.

### Missing Context
If context isn't being injected:
1.  Verify `BeforeAgent` is present in `settings.json`.
2.  Run `ai-brains preflight` manually to ensure the vault is accessible.
3.  Check `stderr` logs to see if the hook is finding the correct project directory.

### Failed Ingestion
If memories aren't appearing in `ai-brains recall`:
1.  Verify `$env:GEMINI_PROJECT_DIR` is correctly set by the CLI.
2.  Check if `AI_BRAINS_PROJECT_ID` is set in the local `.env`.
3.  Verify that `ingest.ps1` exists at `.agents/skills/ai-brains/scripts/ingest.ps1`.
