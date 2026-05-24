# Specification: Track T46 - Multi-Path Antigravity Discovery

## Objective
Expand the `antigravity-import` logic to discover and ingest conversation history from all known Antigravity storage locations and formats, ensuring comprehensive memory capture across CLI and IDE versions.

## Storage Locations & Formats

### 1. Brain Directories (Format: `overview.txt` or `transcript.jsonl`)
- **Paths**:
  - `%USERPROFILE%\.gemini\antigravity\brain\`
  - `%USERPROFILE%\.gemini\antigravity-cli\brain\`
  - `%USERPROFILE%\.gemini\antigravity-ide\brain\`
  - `\\wsl$\Ubuntu\home\<user>\.gemini\antigravity\brain\`
- **Structure**: `<session-id>/.system_generated/logs/<file>`
- **Content**: JSONL steps with `source`, `step_type`, `content`.

### 2. Project Temp Directories (Format: `session-*.jsonl`)
- **Path**: `%USERPROFILE%\.gemini\tmp\<project_hash>\chats\`
- **Structure**: Direct JSONL files.
- **Content**:
  - Line 0: Header with `sessionId`, `projectHash`.
  - Lines 1+: Turn objects with `type` (role), `content`, `thoughts`.

## Implementation Strategy

### 1. Unified Discovery
- Create a unified discovery service that crawls all 4 path patterns.
- Return a list of `AntigravitySessionSource` objects containing path, session ID, and format type.

### 2. Multi-Format Parsers
- **Legacy Parser**: Handles `overview.txt` and `transcript.jsonl`.
- **Project-Chat Parser**: Handles `session-*.jsonl`.
  - Maps turn `type` (e.g., "gemini", "user") to `role` ("assistant", "user").
  - Extracts `content` while adhering to privacy mandates (skip empty tool calls).

### 3. Project Mapping
- If a session is found in `tmp/<project_hash>`, associate it with that hash in the vault if possible.

## Verification Plan
- Unit tests for the new `Project-Chat Parser` using the discovered JSONL format.
- Integration test simulating multiple storage paths.
- Smoke test: `ai-brains antigravity-import` correctly identifies turns from a `tmp` chat.
