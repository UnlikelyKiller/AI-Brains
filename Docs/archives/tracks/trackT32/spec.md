# Specification: T32 - Preflight ANSI Cleanup, Deduplication, and Hotspot Condensation

## 1. Overview
The preflight context injection system had four compounding bugs: ANSI escape codes from ChangeGuard output polluting the vault and injected context, safety section entries duplicated in the memory index, hotspot table dumps consuming excessive word budget, and no ANSI stripping utility. This track fixes all four with a defense-in-depth approach — stripping at pin time (source), stripping at display time (preflight), deduplicating between sections, and condensing hotspot data before storage.

## 2. Dependencies
- `strip-ansi-escapes` crate (v0.2, MIT/Apache-2.0) — minimal, well-maintained, zero-unsafe
- `ai-brains-core` — existing workspace dependency
- `ai-brains-store` — existing workspace dependency for `VaultConnection` and SQL queries

## 3. Required Modules
- `crates/ai-brains-retrieval/src/ansi.rs` — NEW: `strip_ansi()` wrapper function
- `crates/ai-brains-retrieval/src/preflight.rs` — MODIFIED: ANSI stripping and deduplication
- `crates/ai-brains-cli/src/hotspot.rs` — NEW: `sanitize_and_condense()` and `condense_hotspots()` functions
- `crates/ai-brains-cli/src/main.rs` — MODIFIED: integrated hotspot module into `run_safety_sync()`
- `crates/ai-brains-retrieval/Cargo.toml` — MODIFIED: added `strip-ansi-escapes` dependency
- `crates/ai-brains-cli/Cargo.toml` — MODIFIED: added `strip-ansi-escapes` dependency
- `Cargo.toml` — MODIFIED: added `strip-ansi-escapes` to workspace dependencies

## 4. Rules
- ANSI codes MUST be stripped at pin time (source) AND at display time (defense-in-depth).
- Safety section entries (CONSTRAINT, INVARIANT, HOTSPOT) MUST NOT appear in the general memory index.
- Hotspot memories MUST be condensed to at most 5 file entries plus a truncation notice.
- The `strip_ansi` function MUST handle empty strings, clean text, and multiline input.
- Existing preflight tests MUST continue to pass without modification to their assertions.
- The word budget MUST still be respected after all changes.
- No ANSI escape sequences SHALL appear in any preflight output under any circumstances.