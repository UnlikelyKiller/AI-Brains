# Specification: Track T45 - Antigravity CLI (agy) Integration

## Objective
Support the new `agy` CLI for conversation capture by defining a hook configuration and implementing a transcript capture adapter.

## Architecture & Scope
1. **Hook Configuration (`hooks.json`)**: Define the `hooks.json` schema and initial payload structure required for `agy` to trigger `ai-brains`.
2. **Transcript Capture Adapter (`ai-brains-adapters`)**: Implement a reader adapter that parses the `transcriptPath` (JSONL format) provided by `agy` via the hook payload.
3. **Ingestion Integration (`ai-brains-capture` & `ai-brains-cli`)**: Connect the transcript adapter to the standard `CaptureService`, allowing `ai-brains` to ingest `agy` conversations as native events.

## Technical Constraints & Mandates
- **Capture Independence**: The capture path must remain functional without depending on models or external APIs.
- **Canonical Source of Truth**: All ingested `agy` transcript events must be appended as immutable events in the SQLCipher-backed event log.
- **Path Normalization**: The `transcriptPath` provided by `agy` must undergo canonical Windows/WSL/UNC normalization before processing via the `ai-brains-path` crate.
