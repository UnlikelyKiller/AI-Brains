# Specification: Track T02 - Event Contracts

## 1. Overview
The `ai-brains-events` crate serves as the foundation for the CQRS/Event Sourcing architecture. It defines the strictly immutable events that are appended to the canonical store. The crate does not interact with a database, it merely defines the shapes and rules.

## 2. Event Envelope Structure
Every event must be wrapped in an `Envelope`.
Fields required:
- `event_id`: UUID (v4)
- `schema_version`: u32 (starts at 1)
- `aggregate_type`: String/Enum
- `aggregate_id`: UUID or string ID
- `event_type`: Enum/String
- `occurred_at`: Timestamp (from `ai-brains-core::clock`)
- `actor`: JSON/Struct mapping who caused it
- `causation_id`: Option<UUID>
- `correlation_id`: Option<UUID>
- `privacy`: Privacy Level (from `ai-brains-core::privacy`)
- `payload`: Serde JSON value or Struct
- `payload_hash`: SHA-256 digest of payload for tamper evidence

## 3. Mandatory Security & Privacy Rules
- **Privacy Level**: The `privacy` field MUST be set on every single event envelope.
- **No Mutability**: Event payloads cannot contain mutable references or methods that mutate state. They are strictly data transfer objects.
- **No Hidden Thinking / Tool Calls**: 
  - There must NOT be an event type for `ThinkingRecorded`, `ToolCallRecorded`, or anything similar.
  - Payloads must actively avoid capturing fields named `thinking`, `chain_of_thought`, etc. (Enforced via structural omission and tests).

## 4. Supported Event Types
1. System/Admin Events:
   - `SystemInitialized`
   - `RecoveryKitCreated`
2. Project Events:
   - `ProjectRegistered`
   - `ProjectAliasAdded`
3. Session Events:
   - `SessionStarted`
   - `UserPromptRecorded`
   - `AssistantFinalRecorded`
   - `SessionCompleted`
   - `SessionFailed`
4. Memory Events:
   - `MemoryPinned`
   - `MemoryForgotten`
   - `PrivacyEscalated`
5. Background Events:
   - `NightlyJobStarted`
   - `ConflictDetected`
   - `RecipePromoted`

## 5. Event Hashing
Hashing must use `sha2` crate. The payload is serialized to a canonical JSON string (sorted keys), and then hashed.

## 6. Upcasting & Versioning
When events evolve, older payloads must be seamlessly deserialized. `upcast.rs` provides traits or functions mapped to the `schema_version` of an event to transform v1 -> v2 without changing the database raw string.
