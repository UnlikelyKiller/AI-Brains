# Track T81: `--quiet` silences ChangeGuard-bridge warnings

**Status:** ✅ **Complete**
**Started:** 2026-06-02
**Owner:** Claude
**Priority:** P1 — every recall call from a non-git dir spammed stderr.

---

## Problem Statement

`ai-brains recall` (and `preflight`, and `sync query`) invokes the
ChangeGuard bridge as a first phase. When the bridge fails — typically
because the cwd is not a git repository — the retrieval crate prints:

```
ChangeGuard bridge query failed, falling back to local FTS5 only: changeguard search failed: Could not find a git repository in 'C:\Users\…' or in any of its parents
```

The audit (June 2026) demonstrated this fires on **every** `recall` call
from a non-git directory (e.g., a tempdir, a fresh checkout, a CI runner
that doesn't init git). It clutters stderr and breaks the
`recall | jq` idiom in non-git environments.

The `sync query --quiet` flag existed but only suppressed
daemon-down errors. The bridge-failed warning was a separate code path.

## Acceptance Criteria

**AC1:** `ai-brains recall --quiet` from a non-git directory does not
print the bridge-failed warning on stderr. The local FTS5 fallback still
runs; only the chatter is silenced.

**AC2:** Default behavior is unchanged: `recall` (no flag) still prints
the warning.

**AC3:** A new global `--quiet` flag is plumbed through the entire
recall path: clap → `RecallRunOptions` → `RecallOptions` →
`ai_brains_retrieval::recall`.

## Design Notes

- Added `quiet: bool` to `ai_brains_retrieval::RecallOptions`. Other
  call sites in the codebase that construct `RecallOptions` use
  `..Default::default()` to keep them forward-compatible.
- The `eprintln!` in `crates/ai-brains-retrieval/src/recall.rs:126-129`
  is gated on `!options.quiet`.
- The flag is plumbed as a regular `bool` rather than a tracing-subscriber
  filter: the warning is *one* specific eprintln, not a broad log level.
  A future track can extend the same mechanism to other warnings.

## Files

- `crates/ai-brains-retrieval/src/recall.rs` — added `quiet` to
  `RecallOptions`, gated the eprintln.
- `crates/ai-brains-cli/src/main.rs` — added `--quiet` to the `Recall`
  variant.
- `crates/ai-brains-cli/src/commands/recall.rs` — added `quiet` to
  `RecallRunOptions`, passed through to `RecallOptions`.
- `crates/ai-brains-cli/src/commands/sync.rs` — added `quiet` to the
  `RecallRunOptions` construction in `run_query`.
- `crates/ai-brains-cli/src/commands/symbol_bridge.rs` — added
  `..Default::default()` (no behavior change, but required to keep
  compiling after the struct change).
- `crates/ai-brainsd/src/lib.rs` — same `..Default::default()` change.
- `crates/ai-brains-cli/tests/smoke.rs` —
  `test_recall_quiet_silences_bridge_warning`.

## Tests (TDD)

Red phase: `test_recall_quiet_silences_bridge_warning` runs `recall
--quiet` from a non-git tempdir and asserts no "bridge query failed"
in stderr. Fails because the flag is not yet accepted by clap.

Green phase: flag added and gated. Test passes.

## Verification

- `cargo nextest run -p ai-brains-cli test_recall_quiet_silences_bridge_warning`
  — passes.
- Manual: `ai-brains --quiet recall "test"` from a non-git tempdir
  produces clean stderr.

## Out of Scope

- A `--quiet` flag on `preflight` and other commands that hit the bridge.
  T81 covers recall; follow-ups can extend the pattern.
- A tracing-subscriber `--log-level=warn` filter as a more general
  mechanism. T81 is a focused fix.
- Changing the warning text or routing it to a different stream.
