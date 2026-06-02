# Track T74: Graph Health Smoke Test

**Status:** ✅ **Complete**
**Started:** 2026-06-02
**Owner:** Claude
**Priority:** P1 — closes a coverage gap that would let the graph projector
silently fail without any test catching it.

---

## Problem Statement

The live graph projector (T69) is invoked on every event append. If it
silently regresses (e.g. a projector handler forgets to call
`flush()` after `apply()`), there is no test that catches it: the
`ai-brains ingest` and `ai-brains recall` commands still succeed, and only
`ai-brains graph update` would show the problem. But `graph update` itself
has no smoke test — so a regression could ship unnoticed.

The vault health today (2026-06-02) is 861 nodes / 72 edges, with the live
projector active. There is no documented assertion of "what good looks
like" beyond "it returns JSON."

## Acceptance Criteria

**AC1:** A new test `test_graph_health_smoke` exists in
`crates/ai-brains-cli/tests/smoke.rs`. It:
1. Creates a fresh tempdir vault.
2. Runs `init`, then `ingest` (one turn), then `pin` (one memory), then
   `recall` (which exercises T67's `MemoryPinned` emission).
3. Runs `graph update` and parses the JSON output.
4. Asserts `nodes >= 1`, `edges >= 1`, `status == "live"`, and exit 0.

**AC2:** The test is gated on the `graph` feature so it only runs when the
graph binary is compiled in.

**AC3:** Running `cargo nextest run -p ai-brains-cli --features graph
test_graph_health_smoke` exits 0.

## Design Notes

- The test follows the existing `tempfile::tempdir` + `assert_cmd::Command`
  pattern from the rest of `smoke.rs`. No new test infrastructure.
- The `--features graph` gate mirrors the `#[cfg(feature = "graph")]`
  attribute on the `Graph` command variant in `main.rs`. Without the feature,
  the binary has no `graph` subcommand and the test would fail with an
  "unknown subcommand" error — hence the gate.
- The test does not assert *exact* node/edge counts because the live
  projector includes a baseline of 1 project + 1 session + 1 turn + N
  pin/recall edges that varies. Asserting `>= 1` is the right level of
  strictness: it catches the regression we care about (projector silent
  failure) without being brittle to ordinary state changes.

## Files

- `crates/ai-brains-cli/tests/smoke.rs` (append `test_graph_health_smoke`)

## Verification

- `cargo nextest run -p ai-brains-cli --features graph test_graph_health_smoke`
  exits 0.
- Running without `--features graph` skips the test (cargo nextest's standard
  feature-gated skip behavior).
