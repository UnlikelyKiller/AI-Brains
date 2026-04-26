# Architectural & Implementation Deviations

This document records the intentional deviations from the original `Implementation-Plan.md` that occurred during development, along with the rationale and technical context for each decision.

## 1. Storage & Encryption (Phase 5)
*   **Original Plan:** Use `sqlcipher-bundled` for transparent, full-database AES-256 encryption.
*   **Deviation:** Degraded to standard `bundled` SQLite for development.
*   **Rationale:** The primary Windows development environment lacked the required OpenSSL development headers, C++ build tools, and Perl needed to compile `sqlcipher-bundled` from source. The architecture was designed to be SQLCipher-ready (using the same connection contracts), so swapping the `libsqlite3-sys` feature flag back to `sqlcipher` in a configured CI/CD environment will seamlessly restore encryption without code changes.

## 2. Graph Database Compilation (Phase 8 & 12)
*   **Original Plan:** The `ai-brains-graph` crate, wrapping LadybugDB (which uses Kuzu, a C++ graph database), is a mandatory dependency for all retrieval and intelligence operations.
*   **Deviation:** Isolated the graph database behind a Cargo feature flag (`graph`) in `ai-brains-cli` and `ai-brains-retrieval`. E2E smoke tests and the CLI now compile and run cleanly without the graph database using a dynamically typed mock (`MockGraphSearch`).
*   **Rationale:** While CMake and MSVC are installed on the Windows host, the Kuzu C++ dependency triggers a known MSVC limitation during debug builds (`fatal error LNK1248: image size exceeds maximum allowable size (FFFFFFFF)`). This 4GB size limit for static libraries in MSVC Debug mode prevents compilation. Decoupling the graph allows the core system (capture, store, summarization, retrieval) to remain highly portable and testable on Windows without complex MSVC workarounds.

## 3. Date & Time Management
*   **Original Plan:** Not strictly specified, but generally leaned towards the `time` crate for lightweight timestamps.
*   **Deviation:** Standardized on `chrono` across all crates.
*   **Rationale:** `chrono` provided better out-of-the-box support for RFC3339 string generation and formatting, which was crucial for generating filename-safe timestamps (e.g., replacing `:` with `-` in backup folder names) and JSON serialization boundaries.

## 4. Retention & Privacy (Phase 11)
*   **Original Plan:** Event log is append-only; projections are deterministic rebuilds. No explicit "forget" mechanism beyond not loading certain nodes into the graph.
*   **Deviation:** Added a `last_accessed_at` column to `turn_projection` and introduced a dedicated `RetentionService` (90-day expiration) and a CLI `forget` command.
*   **Rationale:** Privacy regulations and practical disk space concerns necessitate soft-deletes and data expiration. By updating the `turn_projection` with a `forgotten` status, we prevent sensitive or old data from being retrieved via FTS or Graph, while keeping the underlying Event Log intact for cryptographic auditability.

## 5. Memory Intelligence & RAPTOR (Phase 10)
*   **Original Plan:** Rely heavily on multi-hop graph traversal (Kuzu) for memory synthesis and long-term intelligence.
*   **Deviation:** Implemented RAPTOR-style hierarchical clustering and CRAG factual verification directly over the FTS/Lexical search read-models.
*   **Rationale:** To protect the background Nightly worker from the instability of the C++ graph build on Windows, the synthesis engine was decoupled from the graph. It currently relies on the standard `QueryStore` interface, ensuring high-level knowledge extraction works purely on the SQLite event projections.
