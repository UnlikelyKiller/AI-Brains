# Specification: Track T37 - Transaction Linking & Project Discovery

## Objective
Accept `--tx-id` in `ai-brains context` and `pin`. Auto-discover `project_id` from the `.changeguard` directory if present.

## Architecture & Scope
1. **CLI Argument Parsing (`ai-brains-cli`)**: Add `--tx-id <ID>` optional argument to `context` and `pin` commands.
2. **Context Enrichment (`ai-brains-cli` & `ai-brains-capture`)**: Thread `tx_id` through `AppContext` and pass it to event ingestion payloads.
3. **Project Discovery (`ai-brains-path` or `ai-brains-git`)**: Implement logic to walk up the directory tree to find a `.changeguard` folder. If found, derive or extract the `project_id`.
4. **Event Sourcing**: Ensure `tx_id` and `project_id` are included in the corresponding immutable events in `ai-brains-events` and `ai-brains-contracts`.

## Technical Constraints & Mandates
- **Capture Independence**: Must work without graph database or model dependencies.
- **CQRS Integrity**: Modifying CLI to append context commands must maintain read/write separation.
- **Path Normalization**: Directory walk for `.changeguard` must use proper Windows normalization (UNC, drive-case).
