# AI-Brains Hook System Documentation

This document details the configuration, architecture, and troubleshooting steps for the project-aware memory hooks used with the Gemini, Claude, and Codex harnesses.

## Overview

The hook system automates two critical lifecycles:
1.  **Preflight Injection (`SessionStart` / `BeforeAgent`):** Fetches relevant project context from AI-Brains and injects it into the AI's prompt before the session or turn.
2.  **Memory Ingestion (`AfterAgent` / `Stop` / `SessionEnd`):** Captures the AI's final response and records it in the AI-Brains vault for future recall.

## Configuration

Hooks are configured in harness-specific settings files.

### 1. Gemini CLI
**Config Path:** `C:\Users\RyanB\.gemini\settings.json`
**Script Path:** `C:\Users\RyanB\.ai-brains\scripts\target-gemini-hook.ps1` (Source: `scripts\target-gemini-hook.ps1`)

### 2. Claude Code
**Config Path:** Native Claude Code hook registration.
**Script Path:** `C:\Users\RyanB\.ai-brains\scripts\target-claude-hook.ps1` (Source: `scripts\target-claude-hook.ps1`)

### 3. Codex CLI
**Config Path:** `C:\Users\RyanB\.codex\hooks.json`
**Script Path:** `C:\Users\RyanB\.ai-brains\scripts\target-codex-hook.ps1` (Source: `scripts\target-codex-hook.ps1`)

## Hook Script Architecture

Every hook script follows the **Protocol-Safe JSON** pattern:
- **Stdout**: Must be exactly one valid JSON object (e.g., `{"success": true}`).
- **Stderr**: Diagnostics and logging must go to `stderr` with a harness-specific prefix (e.g., `[ai-brains-gemini]`).
- **Silence**: All internal tool output (e.g., `ai-brains ingest`) must be suppressed or redirected to `$null`.

### Core Functions
1.  **Stdin Handling**: Uses `[Console]::In.ReadToEnd()` to capture the full hook envelope from the harness.
2.  **Environment Loading**: Resolves `AI_BRAINS_PROJECT_ID` and `AI_BRAINS_SESSION_ID` via project-local and global `.env` files.
3.  **Harness Adaptation**: Maps harness-specific event names to neutral AI-Brains actions (`preflight`, `ingest`).

## Troubleshooting

### Identifying Loops
If the harness appears to hang or repeat "Executing Hook", it may be due to a hook triggering another command that in turn triggers a hook.
*   **Fix**: Ensure hooks only run for intended commands. Use `Stop` or `AfterAgent` for final capture only.
*   **Check Logs**: Look for prefix-tagged entries in the terminal (check stderr).

### Missing Context
If context isn't being injected:
1.  Verify the hook is registered for `SessionStart` or `BeforeAgent`.
2.  Run `ai-brains preflight` manually to ensure the vault is accessible.
3.  Check `stderr` logs to see if the hook is resolving the correct project directory and loading the `.env`.

### Failed Ingestion
If memories aren't appearing in `ai-brains recall`:
1.  Verify `AI_BRAINS_PROJECT_ID` is set in the local `.env`.
2.  Check for `[ai-brains-*] Missing project_id or session_id` warnings on stderr.
3.  Verify the harness is actually emitting the final response in the hook payload (e.g., `final_response` or `last_assistant_message`).
