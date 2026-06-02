# Track T88: Fix `pin` to Print Projection `memory_id`

**Status:** ⏳ **Pending**
**Owner:** —
**Priority:** P1 — every `pin` → `forget --memory-id` workflow is broken.

---

## Problem Statement

`ai-brains pin "..."` prints `"Memory <event_id> successfully pinned to vault."` but the projection stores `turn_id` as `memory_id`. The `event_id` and `turn_id` are different UUIDs. As a result, any user who copies the UUID printed by `pin` and passes it to `forget --memory-id <uuid>` gets `"Memory not found."` — the only working workaround is `forget --match <content>`.

## Root Cause

`crates/ai-brains-cli/src/commands/pin.rs` lines 99–103 (approx) emit `e.event_id.to_string()` in the success message. The memory projection stores `turn_id` as `memory_id`. These are different UUIDs generated at different points in the event lifecycle.

## Acceptance Criteria

**AC1:** After `pin <content>`, the UUID printed in the success message matches the `memory_id` returned by `forget --list-forgotten` and accepted by `forget --memory-id`.

**AC2:** The fix uses the same `turn_id` field that the projection stores — do not invent a new ID.

**AC3:** A regression test asserts: `pin` → capture printed UUID → `forget --memory-id <uuid>` → exits 0.

## Design Notes

- Find where `pin.rs` calls `store.append(event)` and captures the returned envelope. The envelope exposes both `event_id` (internal append UUID) and `turn_id` (the conversation-turn UUID used as `memory_id` in the projection).
- Change the success `println!` to reference `turn_id` instead of `event_id`.
- One-line change; no architecture required.

## Verification

```
ai-brains pin "DECISION: use turn_id not event_id"
# copy the UUID printed
ai-brains forget --memory-id <copied-uuid>
# must exit 0 with "Memory <uuid> forgotten."
```
