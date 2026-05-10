## Plan: Crypto Recovery

### Phase 1: Foundation and Key Memory Safety
- [ ] Task 1.1: Initialize `crates/ai-brains-crypto/Cargo.toml` with dependencies (`serde`, `zeroize`, `thiserror`, `rand`, `subtle`). Ensure crate is included in workspace if not already.
- [ ] Task 1.2: Implement `src/errors.rs` to define `CryptoError` (e.g., InvalidPassphrase, DpapiError, KeyGenerationFailed).
- [ ] Task 1.3: Implement `src/zeroize.rs` providing utility structures and asserting traits for memory safety.
- [ ] Task 1.4: Implement `src/data_key.rs` with `DataKey` struct. Implement secure random generation and enforce `ZeroizeOnDrop`.
- [ ] Task 1.5: Write and pass tests: `data_key_generated_randomly`, `key_material_debug_redacted`.

### Phase 2: Windows DPAPI Wrapping
- [ ] Task 2.1: Add Windows dependency to `Cargo.toml` for DPAPI (`windows` crate, `Win32_Security_Cryptography`).
- [ ] Task 2.2: Implement `src/dpapi.rs` providing `wrap_key` and `unwrap_key` functions relying on `CryptProtectData` and `CryptUnprotectData`.
- [ ] Task 2.3: Implement `src/key_wrap.rs` defining the `DpapiWrappedKey` struct.
- [ ] Task 2.4: Write and pass test: `windows_dpapi_roundtrip`.

### Phase 3: Passphrase Key Wrapping
- [ ] Task 3.1: Add cryptographic dependencies to `Cargo.toml` (`argon2`, `aes-gcm` or similar for symmetric wrapping).
- [ ] Task 3.2: Implement `src/passphrase.rs` for password hashing/key derivation.
- [ ] Task 3.3: Implement `src/key_wrap.rs` defining `PassphraseWrappedKey` struct containing salt, nonce, and ciphertext.
- [ ] Task 3.4: Write and pass tests: `passphrase_wrap_roundtrip`, `wrong_passphrase_fails`.

### Phase 4: Recovery Kit Integration
- [ ] Task 4.1: Implement `src/recovery_kit.rs` defining `RecoveryKit` which aggregates DPAPI and Passphrase wrapped artifacts.
- [ ] Task 4.2: Provide methods on `RecoveryKit` to attempt restoration via DPAPI or Passphrase.
- [ ] Task 4.3: Write and pass tests: `recovery_kit_restores_key`, `recovery_kit_missing_reports_actionable_error`, `does_not_write_plaintext_key_to_disk`.

### Phase 5: SQLCipher Key Provisioning
- [ ] Task 5.1: Implement `src/sqlcipher.rs` providing `SqlCipherKey` that formats the raw key for SQLite/SQLCipher consumption.
- [ ] Task 5.2: Ensure `SqlCipherKey` enforces zeroize on drop.
- [ ] Task 5.3: Write and pass test: `sqlcipher_key_zeroized`.
- [ ] Task 5.4: Finalize `src/lib.rs` and `src/test_support.rs` exposing the public API. Run `cargo clippy -p ai-brains-crypto` and resolve all warnings.