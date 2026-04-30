# AI-Brains

AI-Brains is an event-sourced, privacy-first memory system for AI agents, optimized for Windows 11 and PowerShell.

## Core Mandate
Capture must be fast, durable, encrypted, and independent of every advanced memory feature. The system ensures that your project history is never lost, even if intelligence services are offline.

## Key Features
- **Canonical Event Log**: SQLCipher-backed append-only history.
- **CQRS Architecture**: Commands append events; queries read read-optimized projections.
- **Privacy First**: Four levels of privacy protection (`CloudOk` to `Sealed`).
- **Nightly Intelligence**: Background workers for summarization, conflict detection, and cross-agent synthesis (Phase 15).
- **Windows Native**: First-class support for PowerShell, DPAPI, and Task Scheduler.

## Quick Start

### 1. Initialize a Vault
```powershell
ai-brains init
```

### 2. Set Up a Project
Run this in any new repository to wire up project-specific isolation:
```powershell
ai-brains context
```

### 3. Record a Turn
```powershell
powershell .agents/skills/ai-brains/scripts/ingest.ps1 -Content "Finalizing Phase 15..."
```

### 4. Start a Session (Preflight)
```powershell
ai-brains preflight
```
*Returns an indexed briefing followed by recent technical context.*

## 📂 Configuration Hierarchy
AI-Brains uses a hierarchical loading strategy for cross-repository flexibility:
1.  **Local `.env`**: Scopes IDs to the current repo (Created via `context`).
2.  **Global `~/.ai-brains/.env`**: Stores the shared `VAULT_PATH` and Model URLs.
3.  **Env Vars**: Override any of the above.

## Documentation
- [Architecture](./Docs/ARCHITECTURE.md)
- [Operations Guide](./Docs/OPERATIONS.md)
- [Project Status](./Docs/status.md)
- [Implementation Plan](./Docs/Implementation-Plan.md)
- [Architectural Deviations](./Docs/Deviations.md)

## Development
This project uses a track-based implementation method managed via ChangeGuard.
```powershell
./scripts/dev-check.ps1
```
