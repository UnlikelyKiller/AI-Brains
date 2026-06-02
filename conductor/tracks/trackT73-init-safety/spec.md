# Track T73: Idempotent `init`

**Status:** ✅ **Complete**
**Started:** 2026-06-02
**Owner:** Claude
**Priority:** P0 — real safety gap; the existing command silently overwrites a populated vault.

---

## Problem Statement

`ai-brains init` is a silent no-op. The implementation in
`crates/ai-brains-cli/src/commands/init.rs` (prior to T73) was:

```rust
pub fn run(ctx: &AppContext) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "Vault initialized successfully at {}",
        ctx.vault_path.display()
    );
    Ok(())
}
```

It does not check whether the vault already contains data. A downstream caller
(operator, script, or hook) invoking `init` on an existing populated vault
gets a "Vault initialized successfully" success message with no indication
that the vault was already populated. This is a real safety gap because:

1. The user/operator is led to believe the vault was just created, when in
   fact it already had data.
2. A future bug or regression that *does* silently overwrite the vault would
   be invisible: the success path is the same code path as the empty-vault
   path.
3. Conductor automation or CI flows that idempotently run `init` to "ensure
   the vault exists" will mask failures caused by environment changes
   (different `AI_BRAINS_VAULT_PATH`, restored backup, etc.).

## Acceptance Criteria

**AC1:** `ai-brains init` exits 0 with `"Vault initialized successfully at <path>"`
when the vault file does not exist (current behavior, preserved).

**AC2:** `ai-brains init` exits 0 with `"Vault already initialized at <path>"`
when the vault file exists but contains no projects (idempotent success).

**AC3:** `ai-brains init` exits 1 with a clear refusal message when the vault
file exists AND contains at least one project. The message names the vault
path and the project count.

**AC4:** `ai-brains init --force` exits 0 with the success message regardless
of populated state, acknowledging the operator's intent to overwrite.

**AC5:** The refusal in AC3 propagates through the existing structured error
envelope path so callers receive a JSON `ApiResult::error` on stderr.

## Design Notes

- The "is this vault populated?" signal is `ctx.conn.list_projects()` from
  `QueryStore`. This is a cheap `SELECT` against the projections table. We
  deliberately do not use file existence as the signal, because
  `AppContext::from_cli` creates the file as a side effect of opening the
  connection — so by the time `init::run` runs, the file is always present.
- The "force" affordance is the only escape hatch. There is no `--yes` or
  `--assume-yes` — operators get one explicit flag, named after the action
  (force), not the response.
- Refusal message is a single `String` returned as `Err(Box<dyn Error>)`. The
  CLI's `main()` wraps every error in `ApiError::new("COMMAND_FAILED", ...)`
  and serializes to JSON on stderr, so the structured envelope comes for free.

## Files

- `crates/ai-brains-cli/src/commands/init.rs` (rewrite, ~50 lines)
- `crates/ai-brains-cli/src/main.rs` (change `Init` from unit to
  `Init { force: bool }`, update the dispatch arm)
- `crates/ai-brains-cli/tests/smoke.rs` (add `test_init_refuses_populated_vault`
  and `test_init_force_overwrites`)

## Tests (TDD)

Both new tests are written first (red), then the implementation (green):

1. `test_init_refuses_populated_vault` — init + ingest a turn (creating one
   project), then run `init` again. Asserts non-zero exit and a stderr
   message containing "already" or "Refusing".
2. `test_init_force_overwrites` — same setup, but the second `init` is passed
   `--force`. Asserts exit 0 and success stdout.

The pre-existing `test_cli_init_smoke` (which expects success on a fresh
vault) still passes after the change because the new code path is identical
for the empty-vault case.

## Verification

- `cargo fmt --check` clean
- `cargo clippy --workspace --all-targets -- -D warnings` clean
- `cargo nextest run -p ai-brains-cli` — all 5 smoke tests pass (3 pre-existing
  + 2 new)
