# Specification: Track T40 - Unified Retrieval & Feedback Loop

## Objective
Blend preflight/ask results with ChangeGuard data and implement a nightly sweep accuracy check.

## Architecture & Scope
1. **Blended Retrieval (`ai-brains-retrieval`)**: Update retrieval queries to incorporate synced ChangeGuard data (`BridgeRecord`s that were ingested into events and projected).
2. **Feedback Loop / Nightly Sweep (`ai-brains-brain` / `ai-brains-scheduler`)**: Introduce a scheduled job that compares past predictions/context with actual observed `BridgeRecord` data to evaluate accuracy.
3. **Reporting**: Generate accuracy metrics or compensating events if confidence drifts.

## Technical Constraints & Mandates
- **CQRS Integrity**: Retrieval updates must purely read from projections.
- **Event Sourcing**: The nightly sweep must NOT modify past events. Any feedback adjustments must be recorded as new compensating or metric events.
- **Privacy Inheritance**: Retrieval blending MUST respect the privacy flag of the incorporated `BridgeRecord`s.
