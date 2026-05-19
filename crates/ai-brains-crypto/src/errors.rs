use thiserror::Error;

#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("Key generation failed")]
    KeyGenerationFailed,

    #[error("Invalid passphrase")]
    InvalidPassphrase,

    #[error("Encryption failed: {0}")]
    EncryptionError(String),

    #[error("Decryption failed: {0}")]
    DecryptionError(String),

    #[error("DPAPI error: {0}")]
    DpapiError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    #[error("Invalid key length")]
    InvalidKeyLength,

    #[error("Recovery kit is missing required components: {0}")]
    RecoveryKitMissing(String),
}

pub type Result<T> = std::result::Result<T, CryptoError>;
