//! Blake3 hash wrapper for lockchain operations

use serde::{Deserialize, Serialize};

/// Blake3 hash wrapper (32 bytes)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Blake3Hash(pub [u8; 32]);

impl Blake3Hash {
    /// Create from bytes
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Create from data by hashing
    pub fn hash(data: &[u8]) -> Self {
        let hash = blake3::hash(data);
        Self(*hash.as_bytes())
    }

    /// Convert to hex string
    pub fn to_hex(&self) -> String {
        blake3::Hash::from(self.0).to_hex().to_string()
    }

    /// Parse from hex string
    pub fn from_hex(hex: &str) -> Result<Self, anyhow::Error> {
        let hash = blake3::Hash::from_hex(hex)?;
        Ok(Self(*hash.as_bytes()))
    }

    /// Get inner bytes
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl From<[u8; 32]> for Blake3Hash {
    fn from(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

impl AsRef<[u8]> for Blake3Hash {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blake3_hash_creation() {
        let bytes = [42u8; 32];
        let hash = Blake3Hash::from_bytes(bytes);
        assert_eq!(hash.0, bytes);
    }

    #[test]
    fn test_blake3_hash_from_data() {
        let data = b"test data";
        let hash = Blake3Hash::hash(data);
        assert_ne!(hash.0, [0u8; 32]);
    }

    #[test]
    fn test_blake3_hash_deterministic() {
        let data = b"test data";
        let hash1 = Blake3Hash::hash(data);
        let hash2 = Blake3Hash::hash(data);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_blake3_hash_to_hex() {
        let hash = Blake3Hash::hash(b"test");
        let hex = hash.to_hex();
        assert_eq!(hex.len(), 64); // 32 bytes = 64 hex chars
    }

    #[test]
    fn test_blake3_hash_from_hex() {
        let hash = Blake3Hash::hash(b"test");
        let hex = hash.to_hex();
        let parsed = Blake3Hash::from_hex(&hex).unwrap();
        assert_eq!(hash, parsed);
    }
}
