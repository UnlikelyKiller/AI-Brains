use crate::errors::{CryptoError, Result};
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use argon2::Argon2;
use rand::rngs::SysRng;

pub const SALT_LEN: usize = 16;
pub const NONCE_LEN: usize = 12;

pub fn derive_key(passphrase: &[u8], salt: &[u8], output: &mut [u8]) -> Result<()> {
    let argon2 = Argon2::default();
    argon2
        .hash_password_into(passphrase, salt, output)
        .map_err(|e| CryptoError::EncryptionError(format!("KDF failed: {}", e)))
}

pub fn wrap_key(
    key_material: &[u8],
    passphrase: &[u8],
) -> Result<(Vec<u8>, [u8; SALT_LEN], [u8; NONCE_LEN])> {
    use rand::TryRng;
    let mut salt = [0u8; SALT_LEN];
    SysRng
        .try_fill_bytes(&mut salt)
        .map_err(|e| CryptoError::EncryptionError(format!("Entropy failed: {}", e)))?;

    let mut derived_key = [0u8; 32];
    derive_key(passphrase, &salt, &mut derived_key)?;

    let mut nonce_bytes = [0u8; NONCE_LEN];
    SysRng
        .try_fill_bytes(&mut nonce_bytes)
        .map_err(|e| CryptoError::EncryptionError(format!("Entropy failed: {}", e)))?;
    let nonce = Nonce::from_slice(&nonce_bytes);

    let cipher = Aes256Gcm::new_from_slice(&derived_key)
        .map_err(|e| CryptoError::EncryptionError(e.to_string()))?;

    let ciphertext = cipher
        .encrypt(nonce, key_material)
        .map_err(|e| CryptoError::EncryptionError(e.to_string()))?;

    Ok((ciphertext, salt, nonce_bytes))
}

pub fn unwrap_key(
    wrapped_material: &[u8],
    passphrase: &[u8],
    salt: &[u8; SALT_LEN],
    nonce_bytes: &[u8; NONCE_LEN],
) -> Result<Vec<u8>> {
    let mut derived_key = [0u8; 32];
    derive_key(passphrase, salt, &mut derived_key)?;

    let nonce = Nonce::from_slice(nonce_bytes);
    let cipher = Aes256Gcm::new_from_slice(&derived_key)
        .map_err(|e| CryptoError::DecryptionError(e.to_string()))?;

    let plaintext = cipher
        .decrypt(nonce, wrapped_material)
        .map_err(|_| CryptoError::InvalidPassphrase)?;

    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::disallowed_methods)]
    use super::*;

    #[test]
    fn passphrase_wrap_roundtrip() {
        let key = b"secret key material";
        let passphrase = b"correct horse battery staple";

        let (wrapped, salt, nonce) = wrap_key(key, passphrase).expect("Wrap failed");
        let unwrapped = unwrap_key(&wrapped, passphrase, &salt, &nonce).expect("Unwrap failed");

        assert_eq!(key.to_vec(), unwrapped);
    }

    #[test]
    fn wrong_passphrase_fails() {
        let key = b"secret key material";
        let passphrase = b"correct horse battery staple";
        let wrong_passphrase = b"wrong password";

        let (wrapped, salt, nonce) = wrap_key(key, passphrase).expect("Wrap failed");
        let result = unwrap_key(&wrapped, wrong_passphrase, &salt, &nonce);

        assert!(matches!(result, Err(CryptoError::InvalidPassphrase)));
    }
}
