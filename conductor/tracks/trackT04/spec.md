# Specification: Crypto Recovery (Track T04)

## Architecture Overview
The `ai-brains-crypto` crate acts as the central authority for sensitive key material. Its main responsibility is to securely generate, store (in memory), wrap, and unwrap the symmetric key used for the SQLCipher database. It prevents the system from writing plaintext keys to persistent storage by offering secure wrapping mechanics, specifically DPAPI on Windows for transparent unlocking, and a passphrase-based recovery mechanism.

## Crate Dependencies
- `ai-brains-core`: Shared types and errors.
- `serde`, `serde_json`: For serializing the `RecoveryKit`.
- `zeroize`: To ensure key materials are purged from RAM immediately when dropped.
- `rand` / `ring` / `aes-gcm` / `argon2`: For secure random generation and passphrase wrapping.
- `windows` / `winapi`: For CryptProtectData / CryptUnprotectData (DPAPI) integration.

## Key Abstractions

### 1. DataKey
The fundamental symmetric key (e.g., 256-bit).
- Implements `ZeroizeOnDrop` via the `zeroize` crate.
- Provides a method to generate a new key securely.
- Debug and Display implementations must be redacted.

### 2. Passphrase Wrapping
A module dedicated to deriving a key from a user password and wrapping the `DataKey`.
- Employs a robust KDF (Key Derivation Function) like Argon2.
- Handles AEAD encryption (e.g., AES-GCM) of the `DataKey`.

### 3. DPAPI Wrapping
Windows-specific integration to allow the current user to transparently unwrap the key on startup.
- Uses `CryptProtectData` and `CryptUnprotectData`.
- Must handle scenarios gracefully where DPAPI is unavailable (e.g., running tests on Linux/macOS, though environment requires Windows focus, conditional compilation `#[cfg(windows)]` may be used to isolate DPAPI logic).

### 4. RecoveryKit
A DTO (often saved as JSON, but saving is handled by other crates) that contains the encrypted artifacts:
- The `DataKey` wrapped by DPAPI.
- The `DataKey` wrapped by a User Passphrase (along with KDF parameters/salts and Nonce).
- Exposes methods to attempt unlocking the `DataKey` either automatically via DPAPI, or manually via Passphrase.

### 5. SqlCipherKey
A view over the `DataKey` formatted exactly as SQLCipher requires (e.g., PRAGMA key formatted strings).
- Must also be wrapped in `Zeroize`.
- Ensures the raw database key isn't leaked into logs or panics.

## Safety Constraints
- `does_not_write_plaintext_key_to_disk`: The crate must not include any logic to write `DataKey` to disk directly. Persistence is strictly the domain of `RecoveryKit` (which only contains wrapped bytes).
- Debug implementations for all key structs must yield `[REDACTED]` or similar.