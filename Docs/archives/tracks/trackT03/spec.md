# Specification: T03 - JSON Contracts

## 1. Overview
The `ai-brains-contracts` crate defines the strict JSON Data Transfer Objects (DTOs) used for communication between the CLI, the Daemon, external integrations (hooks), and tests. These contracts are the boundary layer ensuring consistent API shapes.

## 2. Core Constraints
- **Framework Agnostic**: Must not depend on any specific HTTP framework (e.g., Axum, Actix) or Daemon logic.
- **Dependency Isolation**: Can depend on `ai-brains-core`, `serde`, and `serde_json`. Must **NOT** depend on `store`, `adapters`, or `daemon`.
- **Backward Compatibility**: JSON serialization signatures must be heavily guarded against breaking changes. Additions are allowed (if optional); removals/renames are forbidden unless explicitly versioned.
- **Noise Isolation**: Specifically for `hook.rs`, the response struct must be designed so that if a subprocess hook injects stdout noise, the structured data payload is distinct and safely parsable.

## 3. Data Structures

### 3.1 Generic Envelopes
- `response.rs`: Standard JSON envelope `ApiResult<T>` with `success`, `data`, `metadata`.
- `errors.rs`: Standard `ApiError` representing HTTP-like errors with codes, messages, and optional details.

### 3.2 Domain Contracts
- `ingest.rs`: Requests for ingesting text/context.
- `hook.rs`: Request/Response formats for calling out to external integrations.
- `preflight.rs`: Responses for system capability checks and status.
- `recall.rs`: Queries and responses for retrieving memories.
- `sessions.rs`: DTOs for session lifecycle management.
- `projects.rs`: Workspace and project configuration definitions.
- `memory.rs`: Payload definitions for atomic memory items.
- `backup.rs`: Contract for initiating or reporting on data backups.
- `doctor.rs`: System health check reports.
- `version.rs`: Version info DTO.

## 4. Test Strategy
All tests will be driven by explicit JSON schema validation.
- `api_response_shape`: Ensures the generic envelope serializes properly.
- `ingest_request_shape`: Validates specific field mappings.
- `hook_response_has_no_stdout_noise_fields`: Ensures hook payload structures ignore/reject standard string outputs and expect strict JSON objects in their inner 'data' envelope.
- `contracts_are_backward_compatible`: Snapshot tests or static assertions comparing serialized forms to a known, frozen standard JSON representation.
