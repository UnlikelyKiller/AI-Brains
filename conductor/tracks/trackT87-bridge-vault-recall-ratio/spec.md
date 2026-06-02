# Track T87: Bridge:Vault Result Ratio in Recall

**Status:** ⏳ **Pending**
**Owner:** —
**Priority:** P0 — vault-native memories are completely invisible in normal usage.

---

## Problem Statement

`recall` blends ChangeGuard bridge hits first, before vault FTS/semantic results. The bridge returns up to 10 results with BM25 scores of 12–18; vault FTS results score lower. With the default `--limit 5`, truncation happens before a single vault-native memory (from `pin`, `ingest`, or nightly synthesis) can appear. The user's entire conversation history — 470+ memories — is invisible at query time.

## Root Cause

`crates/ai-brains-retrieval/src/recall.rs` — `query_changeguard_bridge()` is called with no result cap. Bridge hits are inserted at the front of the merged list. The combined list is then truncated to `limit`. With bridge scores dominating, vault hits are always pushed out.

## Acceptance Criteria

**AC1:** Bridge results are capped at `ceil(limit / 2)` — e.g. at `--limit 5`, bridge contributes at most 3 results; the remainder are vault FTS + semantic results.

**AC2:** Vault-native memories appear in `recall` output even when the ChangeGuard bridge is reachable and returns results.

**AC3:** A `--no-bridge` flag (or equivalent escape hatch) is available to suppress bridge results entirely and return only vault-native results.

**AC4:** Existing smoke tests continue to pass. A new test asserts that when both bridge and vault return results, at least one vault result appears.

## Design Notes

- In `recall.rs`, after collecting bridge results, slice to `ceil(limit / 2)` before merging.
- Alternatively, pass `--limit N` to `changeguard search` so the bridge never returns more than `N` items, then split budget with vault results.
- The bridge:vault ratio can be a hardcoded constant first (50/50); a config knob can follow as a separate track.
- Do NOT change BM25 scoring — the issue is ordering/truncation, not scores.

## Verification

- Run `recall "GPU driver fix"` on a vault with pinned memories; verify at least one `source: vault` result appears.
- Run `recall "GPU driver fix" --no-bridge`; verify all results are `source: vault`.
- Run smoke tests: `cargo nextest run --workspace`.
