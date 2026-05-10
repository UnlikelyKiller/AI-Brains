# AI-Brains Project Instructions

## Core Mandates
1. **Capture Independence**: The capture path (CLI -> Daemon -> Event Log) must work when graph DB, embeddings, and models are offline.
2. **Canonical Event Store**: SQLCipher-backed append-only event log is the only source of truth.
3. **CQRS Strictness**: Commands append events. Queries read projections. No mixing.
4. **No Hidden Thinking**: Capture only final assistant responses and user prompts. No raw tool logs.
5. **Windows-First**: Paths must handle UNC, WSL, and drive-case normalization correctly.
6. **Provenance**: All major changes must be recorded in the `changeguard ledger`.

## Architectural Decisions
- **Relational Graph**: Native SQLite backend using Recursive CTEs (ADR-0009). No C++ dependencies to avoid Windows build friction.
- **Resilient Summarization**: Sequential chunking with context carryover for sessions > 38,912 tokens (Track T34).
- **Hardened Capture**: Encoding resilience (BOM-less UTF-8) and ANSI stripping for all inputs.

## Hardware Optimization
- **Intel Arc B580 (12GB VRAM)**:
  - Context Budget: 38,912 tokens.
  - Multi-stage RAG: BGE-M3 (embeddings) + Qwen 3.5 (completion).
  - Dynamic switching via `.env` to prevent VRAM overflow.
