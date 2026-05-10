# Track T04 — Crypto Recovery

## Owner
architecture-planner

## Status
Completed

## Objective
Implement data encryption key generation, DPAPI wrapping (for Windows), passphrase recovery, and SQLCipher key material logic in the `ai-brains-crypto` crate.

## Scope
- Secure data encryption key generation.
- Windows DPAPI integration for seamless local decryption.
- Passphrase-based key wrapping for recovery mechanisms.
- Recovery Kit generation and restoration logic.
- SQLCipher key material structures and formatting.
- Memory protection (clearing keys from memory using zeroize).
- Cryptographic error definitions.

## Out of Scope
- Persisting keys directly to the file system or database (plaintext keys must not be written to disk).
- Executing database queries or interacting directly with SQLCipher bindings.
- Daemon or CLI integration code.

## Files Owned
`crates/ai-brains-crypto/*`

## Files Allowed To Touch
`crates/ai-brains-crypto/src/lib.rs`
`crates/ai-brains-crypto/src/data_key.rs`
`crates/ai-brains-crypto/src/key_wrap.rs`
`crates/ai-brains-crypto/src/dpapi.rs`
`crates/ai-brains-crypto/src/passphrase.rs`
`crates/ai-brains-crypto/src/recovery_kit.rs`
`crates/ai-brains-crypto/src/sqlcipher.rs`
`crates/ai-brains-crypto/src/zeroize.rs`
`crates/ai-brains-crypto/src/test_support.rs`
`crates/ai-brains-crypto/src/errors.rs`
`crates/ai-brains-crypto/tests/*.rs`
`crates/ai-brains-crypto/Cargo.toml`
`Docs/conductor/trackT04/spec.md`
`Docs/conductor/trackT04/plan.md`
`Docs/conductor/conductor.md`

## Files Forbidden To Touch
Any file outside `crates/ai-brains-crypto/` and the conductor planning docs.
Must NOT depend on or touch `store`, `cli`, or `daemon` crates.

## Public Contracts Consumed
- `ai-brains-core`
- `serde`, `zeroize`, `rand`/`ring`, `winapi`/`windows` (for DPAPI).

## Public Contracts Produced
- Secure structs for generating, wrapping, unwrapping, and storing keys in memory (`DataKey`, `RecoveryKit`, `SqlCipherKey`).

## Required Tests First
- `data_key_generated_randomly`
- `passphrase_wrap_roundtrip`
- `wrong_passphrase_fails`
- `recovery_kit_restores_key`
- `key_material_debug_redacted`
- `sqlcipher_key_zeroized`
- `windows_dpapi_roundtrip`
- `does_not_write_plaintext_key_to_disk`
- `recovery_kit_missing_reports_actionable_error`

## Acceptance Criteria
- All requested features implemented with robust error handling.
- Zeroize correctly implemented for sensitive structures.
- No plaintext keys are persisted to disk.
- DPAPI wrapping functions correctly (on Windows).
- All required tests pass.

## Commands To Run
`cargo test -p ai-brains-crypto`
`cargo clippy -p ai-brains-crypto -- -D warnings`
