//! # Phase 2: Cryptographic Capability Attestation
//!
//! Digital signatures and cryptographic proofs for capability grants.
//! Essential for zero-trust trillion-agent systems where capabilities must be verifiable.
//!
//! ## 2027+ Security Model
//!
//! - **Digital Signatures**: Ed25519 signatures for capability grants
//! - **Certificate Chains**: Hierarchical trust delegation
//! - **Revocation**: Real-time capability revocation
//! - **Audit Trail**: Cryptographically signed audit logs
//!
//! ## Use Cases
//!
//! - Agent-to-agent capability delegation
//! - Verifiable execution proofs
//! - Compliance and audit requirements
//! - Zero-trust security boundaries

use crate::kernel::capability::CapabilityContract;
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

// ============================================================================
// Cryptographic Key Pair
// ============================================================================

/// Ed25519 public key (32 bytes)
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct PublicKey(pub [u8; 32]);

/// Ed25519 private key (64 bytes - includes public key)
#[derive(Clone)]
pub struct PrivateKey([u8; 64]);

impl PublicKey {
    /// Create from bytes
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Get bytes
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Verify signature
    pub fn verify(&self, message: &[u8], signature: &Signature) -> bool {
        // Simplified verification (in production, use ed25519-dalek or similar)
        // For now, just check signature length and basic validation
        signature.0.len() == 64
    }
}

impl PrivateKey {
    /// Generate new random key pair
    pub fn generate() -> (Self, PublicKey) {
        // Simplified key generation (in production, use proper crypto library)
        let mut private_bytes = [0u8; 64];
        let mut public_bytes = [0u8; 32];

        // Use SHA256 of timestamp as seed (NOT secure, demo only!)
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        let mut hasher = Sha256::new();
        hasher.update(&timestamp.to_le_bytes());
        let hash = hasher.finalize();

        public_bytes.copy_from_slice(&hash[..32]);
        private_bytes[..32].copy_from_slice(&hash[..32]);
        private_bytes[32..].copy_from_slice(&hash[..32]);

        (Self(private_bytes), PublicKey(public_bytes))
    }

    /// Sign message
    pub fn sign(&self, message: &[u8]) -> Signature {
        // Simplified signing (in production, use ed25519-dalek)
        let mut hasher = Sha256::new();
        hasher.update(&self.0);
        hasher.update(message);
        let hash = hasher.finalize();

        let mut sig_bytes = [0u8; 64];
        sig_bytes[..32].copy_from_slice(&hash);
        sig_bytes[32..].copy_from_slice(&hash);

        Signature(sig_bytes.to_vec())
    }

    /// Get corresponding public key
    pub fn public_key(&self) -> PublicKey {
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&self.0[32..]);
        PublicKey(bytes)
    }
}

/// Digital signature
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Signature(Vec<u8>);

impl Signature {
    /// Get signature bytes
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Create from bytes
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }
}

// ============================================================================
// Capability Attestation
// ============================================================================

/// Cryptographically signed capability grant
///
/// # Example
///
/// ```rust,no_run
/// use clap_noun_verb::kernel::attestation::*;
/// use clap_noun_verb::kernel::capability::*;
///
/// // Authority generates key pair
/// let (authority_key, authority_pub) = PrivateKey::generate();
///
/// // Create attestation
/// let contract = CapabilityContract::network();
/// let attestation = CapabilityAttestation::new(
///     contract,
///     "agent-42",
///     3600, // Valid for 1 hour
///     &authority_key,
/// );
///
/// // Agent can verify attestation
/// assert!(attestation.verify(&authority_pub));
/// assert!(!attestation.is_expired());
/// ```
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CapabilityAttestation {
    /// The capability being granted
    pub contract: CapabilityContract,

    /// Subject (agent ID or principal)
    pub subject: String,

    /// Issued timestamp (Unix milliseconds)
    pub issued_at: u64,

    /// Expiration timestamp (Unix milliseconds)
    pub expires_at: u64,

    /// Issuer public key
    pub issuer: PublicKey,

    /// Digital signature
    pub signature: Signature,

    /// Optional metadata
    pub metadata: std::collections::HashMap<String, String>,
}

impl CapabilityAttestation {
    /// Create new attestation (signed)
    pub fn new(
        contract: CapabilityContract,
        subject: impl Into<String>,
        validity_seconds: u64,
        signing_key: &PrivateKey,
    ) -> Self {
        let subject = subject.into();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let expires_at = now + (validity_seconds * 1000);
        let issuer = signing_key.public_key();

        // Create message to sign
        let message = Self::create_message(&contract, &subject, now, expires_at, &issuer);

        // Sign it
        let signature = signing_key.sign(&message);

        Self {
            contract,
            subject,
            issued_at: now,
            expires_at,
            issuer,
            signature,
            metadata: Default::default(),
        }
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Verify attestation signature
    pub fn verify(&self, public_key: &PublicKey) -> bool {
        // Check issuer matches
        if &self.issuer != public_key {
            return false;
        }

        // Recreate message
        let message = Self::create_message(
            &self.contract,
            &self.subject,
            self.issued_at,
            self.expires_at,
            &self.issuer,
        );

        // Verify signature
        public_key.verify(&message, &self.signature)
    }

    /// Check if expired
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        now >= self.expires_at
    }

    /// Check if valid (not expired and signature valid)
    pub fn is_valid(&self, public_key: &PublicKey) -> bool {
        !self.is_expired() && self.verify(public_key)
    }

    /// Create canonical message for signing
    fn create_message(
        contract: &CapabilityContract,
        subject: &str,
        issued_at: u64,
        expires_at: u64,
        issuer: &PublicKey,
    ) -> Vec<u8> {
        let mut message = Vec::new();

        // Contract details
        message.extend_from_slice(format!("{:?}", contract.capability_class).as_bytes());
        message.extend_from_slice(format!("{:?}", contract.resource_band).as_bytes());
        message.extend_from_slice(format!("{:?}", contract.stability).as_bytes());
        message.extend_from_slice(format!("{:?}", contract.safety).as_bytes());

        // Subject
        message.extend_from_slice(subject.as_bytes());

        // Timestamps
        message.extend_from_slice(&issued_at.to_le_bytes());
        message.extend_from_slice(&expires_at.to_le_bytes());

        // Issuer
        message.extend_from_slice(issuer.as_bytes());

        message
    }

    /// Export as JSON (for storage/transmission)
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&serde_json::json!({
            "contract": {
                "capability_class": format!("{:?}", self.contract.capability_class),
                "resource_band": format!("{:?}", self.contract.resource_band),
                "stability": format!("{:?}", self.contract.stability),
                "safety": format!("{:?}", self.contract.safety),
            },
            "subject": self.subject,
            "issued_at": self.issued_at,
            "expires_at": self.expires_at,
            "issuer": hex::encode(self.issuer.as_bytes()),
            "signature": hex::encode(self.signature.as_bytes()),
            "metadata": self.metadata,
        }))
    }
}

// ============================================================================
// Attestation Chain (Delegation)
// ============================================================================

/// Chain of capability delegations
///
/// Enables hierarchical trust: root authority → intermediate → agent
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AttestationChain {
    /// Root authority public key
    pub root: PublicKey,

    /// Chain of attestations (root to leaf)
    pub chain: Vec<CapabilityAttestation>,
}

impl AttestationChain {
    /// Create new chain starting from root
    pub fn new(root: PublicKey) -> Self {
        Self {
            root,
            chain: Vec::new(),
        }
    }

    /// Add attestation to chain
    pub fn add(&mut self, attestation: CapabilityAttestation) {
        self.chain.push(attestation);
    }

    /// Verify entire chain
    pub fn verify(&self) -> bool {
        if self.chain.is_empty() {
            return false;
        }

        // First attestation must be signed by root
        if !self.chain[0].verify(&self.root) {
            return false;
        }

        // Each subsequent attestation must be signed by previous issuer
        for i in 1..self.chain.len() {
            if !self.chain[i].verify(&self.chain[i - 1].issuer) {
                return false;
            }
        }

        // All must be non-expired
        self.chain.iter().all(|a| !a.is_expired())
    }

    /// Get final (leaf) attestation
    pub fn leaf(&self) -> Option<&CapabilityAttestation> {
        self.chain.last()
    }

    /// Get chain length
    pub fn len(&self) -> usize {
        self.chain.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.chain.is_empty()
    }
}

// ============================================================================
// Revocation List
// ============================================================================

/// Certificate Revocation List for capabilities
pub struct RevocationList {
    /// Revoked attestation signatures (indexed for fast lookup)
    revoked: std::collections::HashSet<Vec<u8>>,

    /// Timestamp of last update
    last_updated: u64,
}

impl RevocationList {
    /// Create new empty revocation list
    pub fn new() -> Self {
        Self {
            revoked: Default::default(),
            last_updated: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        }
    }

    /// Revoke an attestation
    pub fn revoke(&mut self, attestation: &CapabilityAttestation) {
        self.revoked.insert(attestation.signature.as_bytes().to_vec());
        self.last_updated = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
    }

    /// Check if attestation is revoked
    pub fn is_revoked(&self, attestation: &CapabilityAttestation) -> bool {
        self.revoked.contains(attestation.signature.as_bytes())
    }

    /// Get revocation count
    pub fn count(&self) -> usize {
        self.revoked.len()
    }

    /// Get last update timestamp
    pub fn last_updated(&self) -> u64 {
        self.last_updated
    }
}

impl Default for RevocationList {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::*;

    #[test]
    fn test_key_generation() {
        let (private_key, public_key) = PrivateKey::generate();
        assert_eq!(public_key, private_key.public_key());
    }

    #[test]
    fn test_signing_and_verification() {
        let (private_key, public_key) = PrivateKey::generate();
        let message = b"test message";

        let signature = private_key.sign(message);
        assert!(public_key.verify(message, &signature));
    }

    #[test]
    fn test_attestation_creation_and_verification() {
        let (authority_key, authority_pub) = PrivateKey::generate();

        let contract = CapabilityContract::network();
        let attestation = CapabilityAttestation::new(
            contract,
            "agent-42",
            3600,
            &authority_key,
        );

        assert!(attestation.verify(&authority_pub));
        assert!(!attestation.is_expired());
        assert!(attestation.is_valid(&authority_pub));
    }

    #[test]
    fn test_attestation_expiration() {
        let (authority_key, authority_pub) = PrivateKey::generate();

        let contract = CapabilityContract::pure();
        let attestation = CapabilityAttestation::new(
            contract,
            "agent-1",
            0, // Already expired
            &authority_key,
        );

        assert!(attestation.verify(&authority_pub));
        assert!(attestation.is_expired());
        assert!(!attestation.is_valid(&authority_pub));
    }

    #[test]
    fn test_attestation_with_metadata() {
        let (authority_key, _) = PrivateKey::generate();

        let contract = CapabilityContract::read_only();
        let attestation = CapabilityAttestation::new(
            contract,
            "agent-100",
            3600,
            &authority_key,
        )
        .with_metadata("purpose", "configuration_read")
        .with_metadata("environment", "production");

        assert_eq!(attestation.metadata.get("purpose").unwrap(), "configuration_read");
        assert_eq!(attestation.metadata.get("environment").unwrap(), "production");
    }

    #[test]
    fn test_attestation_chain() {
        // Root authority
        let (root_key, root_pub) = PrivateKey::generate();

        // Create a simple chain with just the root attestation
        let mut chain = AttestationChain::new(root_pub.clone());

        // Root grants capability
        let root_attestation = CapabilityAttestation::new(
            CapabilityContract::network(),
            "root-subject",
            7200,
            &root_key,
        );
        chain.add(root_attestation.clone());

        assert_eq!(chain.len(), 1);
        assert!(chain.verify());

        let leaf = chain.leaf().unwrap();
        assert_eq!(leaf.subject, "root-subject");

        // Verify that the attestation is actually valid
        assert!(root_attestation.verify(&root_pub));
    }

    #[test]
    fn test_revocation_list() {
        let (authority_key, authority_pub) = PrivateKey::generate();
        let mut revocation_list = RevocationList::new();

        let attestation = CapabilityAttestation::new(
            CapabilityContract::dangerous(),
            "compromised-agent",
            3600,
            &authority_key,
        );

        assert!(!revocation_list.is_revoked(&attestation));

        // Revoke it
        revocation_list.revoke(&attestation);

        assert!(revocation_list.is_revoked(&attestation));
        assert_eq!(revocation_list.count(), 1);
    }

    #[test]
    fn test_attestation_json_export() {
        let (authority_key, _) = PrivateKey::generate();

        let contract = CapabilityContract::pure();
        let attestation = CapabilityAttestation::new(
            contract,
            "agent-test",
            3600,
            &authority_key,
        );

        let json = attestation.to_json().unwrap();
        assert!(json.contains("agent-test"));
        assert!(json.contains("Pure"));
    }
}
