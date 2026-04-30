# AI-Brains Architecture

## 1. Overview
AI-Brains implements a Command Query Responsibility Segregation (CQRS) pattern with an Event Sourced core. This ensures that every state change is recorded as an immutable event, providing a high-fidelity audit trail and the ability to rebuild any state from scratch.

## 2. Core Components

### 2.1 Event Store (`ai-brains-store`)
The source of truth is an append-only table in a SQLCipher-encrypted SQLite database.
- **Immutability**: Database triggers prevent updates or deletes to the `events` table.
- **Schema Versions**: Supports upcasting for backward-compatible event evolution.

### 2.2 Projections
Read-optimized views are derived from the event log:
- **Project/Session/Turn Projections**: Current state of active and historic sessions.
- **Memory Projection**: Pinned or synthesized knowledge.
- **FTS5 Projection**: Full-text search index for sub-second lexical recall.
- **Graph Projection**: Dual-backend property graph for multi-hop retrieval.
  - **Default Backend**: Deterministic in-memory projection used for routine Windows verification and schema/projector tests.
  - **Ladybug Backend**: Opt-in native C++ backend (LadybugDB) for high-performance multi-hop traversal (requires `ladybug` feature).

### 2.3 Capture Core (`ai-brains-capture`)
A pure logic layer that handles incoming requests (from CLI or hooks) and converts them into normalized events.
- **Privacy Gate**: Scans content for secrets and escalates privacy levels automatically.
- **Path Normalization**: Handles Windows/WSL/UNC path differences.

### 2.4 Retrieval (`ai-brains-retrieval`)
Aggregates data from multiple projections to serve high-level queries like `preflight` and `recall`.
- **Lexical Search**: Primary search mechanism via SQLite FTS5.
- **Vector/Graph Search**: (Optional) augmentation for semantic or relational discovery.

### 2.5 Models & Intelligence
- **Models**: Integration with local `llama.cpp` router via `LlamaCppProvider`.
- **Hardware Optimization**: Optimized for Intel Arc B580 (12GB VRAM) with dynamic model switching between BGE-M3 (embeddings) and Qwen 3.5 (completion).
- **Indexing Strategy**: Dual-path indexing.
  - **Synchronous**: Every conversation turn is immediately indexed into `memory_projection` and FTS for lexical recall.
  - **Asynchronous**: Nightly jobs perform summarization, conflict detection, and hierarchical memory synthesis.

### 2.6 Brain (`ai-brains-brain`)
The "Nightly" intelligence worker that operates on background tasks:
- **Summarization**: Compresses session turns into session summaries.
- **RAPTOR**: Hierarchical clustering of memories (Level 0->1 and Level 1->2) for long-term knowledge retention.
- **CRAG**: Corrective Retrieval Augmented Generation to verify factual accuracy.

## 3. Data Flow

### Command Path (Write)
1. **Request**: CLI/Hook sends JSON.
2. **Validation**: Validate schema and ids.
3. **Enrichment**: Attach Git metadata and normalize paths.
4. **Privacy Check**: Run security scanner.
5. **Persistence**: Append `Envelope` to Event Log.
6. **Projection**: Synchronously (or asynchronously) update read tables.

### Query Path (Read)
1. **Query**: User requests `recall` or `preflight`.
2. **Retrieval**: Read from FTS5, SQLite tables, or Graph.
3. **Filtering**: Apply privacy filters (e.g., exclude `Sealed` or `NeverInject`).
4. **Synthesis**: Format results as Markdown or JSON.

## 4. Configuration and Scale

### 4.1 Hierarchical Configuration
The system uses a 3-layer resolution strategy:
1. **Environment Variables**: Primary overrides.
2. **Local `.env`**: Repo-specific IDs (Project, Session).
3. **Global `.env`**: User-wide defaults (`~/.ai-brains/.env`) for storage paths and model endpoints.

### 4.2 Cross-Agent Synthesis (Phase 15)
AI-Brains implements an **Aggregated Learnings Service** that identifies technical patterns across isolated agents. By analyzing `SessionSummaryCreated` events from multiple projects, the system promotes project-specific insights into a global knowledge base.

### 4.3 Retrieval Briefing Index
To optimize for long-context LLMs, the retrieval layer generates a **Briefing Index** (Topical Pointer) before the main memory payload. This allows the AI to "choose" its focus based on an indexed summary of available history.

## 5. Portability and Degradation
AI-Brains is designed to run in environments with varying levels of service:
- **Minimum**: SQLite + Lexical Search (Works everywhere).
- **Hardened**: SQLCipher (Full encryption).
- **Intelligence**: Ollama/Cloud Models + RAPTOR (Full synthesis).
- **Relational**: LadybugDB (Graph traversal).

If a dependency is missing (e.g., MSVC limits on Windows blocking LadybugDB), the system degrades gracefully to the Lexical + SQLite mode without data loss.
