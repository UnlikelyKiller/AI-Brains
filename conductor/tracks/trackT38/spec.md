# Specification: Track T38 - Structured Bridge (NDJSON Fallback)

## Objective
Define `BridgeRecord` schema and implement `sync pull --from-file` using NDJSON to act as a fallback bridge with ChangeGuard.

## Architecture & Scope
1. **Shared Schema (`ai-brains-contracts`)**: Define `BridgeRecord` DTO containing `bridge_version`, `direction`, `timestamp`, `parent_hash`, `project_id`, `session_id`, `tx_id`, `record_kind`, `payload`, and `privacy`.
2. **CLI Subcommand (`ai-brains-cli`)**: Implement a new `sync` subcommand with `pull --from-file <PATH>` functionality.
3. **Ingestion Logic (`ai-brains-capture`)**: Implement parsing of NDJSON files containing `BridgeRecord` objects. Dispatch these as events.

## Technical Constraints & Mandates
- **Immutable Provenance**: Records pulled from file must be persisted as immutable events.
- **Privacy Inheritance**: `BridgeRecord` privacy field must be enforced and correctly translated to AI-Brains event privacy levels.
- **Fail-open**: If the file contains malformed JSON lines, gracefully skip or log errors while continuing to process valid lines.
