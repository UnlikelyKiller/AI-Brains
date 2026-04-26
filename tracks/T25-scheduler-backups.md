# Track T25: Scheduler and Backups

## Context
Phase 11 focuses on operationalizing the system. This involves scheduling the nightly intelligence sweep and providing a robust backup mechanism.

## Goals
- Provide a command to schedule the nightly intelligence job on Windows.
- Implement a safe backup mechanism for the encrypted vault.
- Ensure backups are recoverable using the same recovery kit/key.

## Implementation Plan

### Phase 1: Scheduler
- [x] Create `ai-brains-scheduler` crate.
- [x] Implement Windows-specific task scheduling logic using `schtasks`.
- [x] Add `ai-brains nightly --schedule` command to the CLI.

### Phase 2: Backups
- [x] Implement `BackupService` in `ai-brains-brain`.
- [x] Add `ai-brains backup` command to the CLI.
- [x] Ensure backups are stored in a `backups/` subdirectory of the vault home.

### Phase 3: Verification
- [x] Test: `render_schtasks_create_command` produces valid Windows syntax.
- [x] Test: `ai-brains backup` creates a valid, timestamped database file.
- [x] Test: E2E recovery from a backup file works.

## Progress
- [x] Phase 1
- [x] Phase 2
- [x] Phase 3
