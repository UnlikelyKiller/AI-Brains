use std::fmt;
use zeroize::{Zeroize, ZeroizeOnDrop};

pub const KEY_LEN: usize = 32; // 256 bits

#[derive(Zeroize, ZeroizeOnDrop)]
pub struct DataKey {
    material: [u8; KEY_LEN],
}

impl DataKey {
    /// Generate a new random DataKey
    pub fn generate() -> Self {
        let mut material = [0u8; KEY_LEN];
        rand::fill(&mut material);
        Self { material }
    }

    /// Create a DataKey from raw bytes (consumes bytes)
    pub fn from_bytes(bytes: [u8; KEY_LEN]) -> Self {
        Self { material: bytes }
    }

    /// Access the raw key material
    pub fn expose_secret(&self) -> &[u8; KEY_LEN] {
        &self.material
    }
}

impl fmt::Debug for DataKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DataKey([REDACTED])")
    }
}

impl Clone for DataKey {
    fn clone(&self) -> Self {
        Self {
            material: self.material,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn data_key_generated_randomly() {
        let key1 = DataKey::generate();
        let key2 = DataKey::generate();
        assert_ne!(key1.expose_secret(), key2.expose_secret());
    }

    #[test]
    fn key_material_debug_redacted() {
        let key = DataKey::generate();
        let debug_str = format!("{:?}", key);
        assert!(debug_str.contains("[REDACTED]"));
        assert!(!debug_str.contains(&hex::encode(key.expose_secret())));
    }
}
