# Track T48: Automated Project Mapping

## 1. Objective
Eliminate manual project ID switching during Antigravity (`agy`) ingest operations by automatically mapping the Antigravity `projectHash` to the internal AI-Brains `project_id`.

## 2. Rationale
Currently, users or hooks must manually ensure the correct `project_id` is supplied when importing conversations from Antigravity. As the Antigravity log schema contains a `projectHash`, AI-Brains can intercept this hash, map it to an internal project ID, and automatically assign the correct context for all ingested turns. This creates a smoother user experience and guarantees correct project attribution without manual environment variable management.

## 3. Architecture & Requirements

### 3.1 Data Model: Alias Auto-Discovery
*   **Projection**: Introduce a `project_alias_projection` table in the local vault to store the mappings between aliases (hashes) and project IDs.
*   **Event**: Create a new immutable event, `ProjectAliasAdded { project_id: Uuid, alias: String }`.
*   **Constraint**: Aliases must be unique globally across the vault.

### 3.2 Auto-Link Behavior
*   When `antigravity-import` or the `agy-hook` reads a conversation turn, it will extract the `projectHash`.
*   The system queries `project_alias_projection` for the `projectHash`.
    *   **If found**: The resolved `project_id` is used for the ingestion request.
    *   **If missing**: If the current environment has an `AI_BRAINS_PROJECT_ID` explicitly set (either via ENV or workspace `.ai-brains-project`), the system creates a `ProjectAliasAdded` event associating the `projectHash` with this explicit `project_id`.
    *   **If missing & no explicit project ID**: The ingestion must either gracefully fallback to a default/global project or prompt the user/fail if strict project isolation is required.

### 3.3 Lookup Logic (`AppContext`)
*   Extend `AppContext` (or a dedicated project resolution service in `ai-brains-core`) with a method `resolve_project_id_from_alias(alias: &str) -> Result<Option<Uuid>>`.
*   This lookup must be fast and executed efficiently during the ingestion pipeline.

### 3.4 CLI & Hook Integration
*   Update the `antigravity-import` CLI command and the `agy-hook` capture pathways to invoke this resolution logic.
*   Ensure that any generated `ProjectAliasAdded` events are written to the event log prior to writing the captured turns.

## 4. Engineering Mandates Adherence
*   **Canonical Source of Truth**: The alias mapping is handled via an append-only `ProjectAliasAdded` event. 
*   **CQRS Integrity**: The write side dispatches the event; the read side queries `project_alias_projection`.
*   **Rust Safety**: All lookup and insertion logic must handle errors using `anyhow`/`thiserror` without `unwrap()` or `expect()`.
