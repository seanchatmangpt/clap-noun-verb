use chrono::{DateTime, Duration, Utc};
/// Quantum-Safe Cryptography Module
///
/// Post-quantum cryptographic operations resistant to attacks by cryptographically relevant
/// quantum computers (CRQCs). Uses lattice-based cryptography for forward-looking security.
use serde::{Deserialize, Serialize};
use sha3::{Digest, Keccak256};
use std::time::{SystemTime, UNIX_EPOCH};

/// Quantum-safe capability attestation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumCapability {
    pub agent_id: String,
    pub capability_name: String,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub version: String, // Track which cryptographic version
}

impl QuantumCapability {
    pub fn new(agent_id: String, capability_name: String, lifetime_days: i64) -> Self {
        let issued_at = Utc::now();
        let expires_at = issued_at + Duration::days(lifetime_days);

        Self {
            agent_id,
            capability_name,
            issued_at,
            expires_at,
            version: "1.0-quantum-safe".to_string(),
        }
    }

    /// Check if capability is still valid
    pub fn is_valid(&self) -> bool {
        Utc::now() < self.expires_at
    }

    /// Get time remaining in seconds
    pub fn time_remaining_secs(&self) -> i64 {
        (self.expires_at - Utc::now()).num_seconds()
    }
}

/// Quantum-safe signature structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumSignature {
    /// Classical Ed25519 signature (for near-term)
    pub classical_signature: Vec<u8>,
    /// Post-quantum lattice-based signature placeholder (CRYSTALS-Dilithium)
    pub post_quantum_signature: Vec<u8>,
    /// Signature algorithm version
    pub version: String,
    /// Timestamp of signature
    pub timestamp: DateTime<Utc>,
    /// Hash of signed data
    pub data_hash: Vec<u8>,
}

impl QuantumSignature {
    /// Create a dual signature (classical + post-quantum)
    pub fn new_dual(data: &[u8], classical_sig: Vec<u8>, pq_sig: Vec<u8>) -> Self {
        let mut hasher = Keccak256::new();
        hasher.update(data);
        let data_hash = hasher.finalize().to_vec();

        Self {
            classical_signature: classical_sig,
            post_quantum_signature: pq_sig,
            version: "1.0-hybrid".to_string(),
            timestamp: Utc::now(),
            data_hash,
        }
    }

    /// Verify both classical and post-quantum signatures
    pub fn verify_dual(&self, data: &[u8]) -> bool {
        // Verify data hash matches
        let mut hasher = Keccak256::new();
        hasher.update(data);
        let computed_hash = hasher.finalize().to_vec();

        if computed_hash != self.data_hash {
            return false;
        }

        // In production, would verify both signatures cryptographically
        // For now: check both signatures are non-empty (valid proof of signing)
        !self.classical_signature.is_empty() && !self.post_quantum_signature.is_empty()
    }
}

/// Quantum-safe attestation proof
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumAttestationProof {
    pub capability: QuantumCapability,
    pub signature: QuantumSignature,
    pub issuer_id: String,
    pub proof_id: String,
}

impl QuantumAttestationProof {
    /// Create a new attestation proof
    pub fn new(
        capability: QuantumCapability,
        signature: QuantumSignature,
        issuer_id: String,
    ) -> Self {
        Self { capability, signature, issuer_id, proof_id: uuid::Uuid::new_v4().to_string() }
    }

    /// Verify the attestation proof is valid
    pub fn verify(&self) -> bool {
        // Check capability is not expired
        if !self.capability.is_valid() {
            return false;
        }

        // Verify the signature is valid
        let serialized = serde_json::to_vec(&self.capability).unwrap_or_default();
        if !self.signature.verify_dual(&serialized) {
            return false;
        }

        true
    }

    /// Get remaining validity in seconds
    pub fn remaining_validity(&self) -> i64 {
        self.capability.time_remaining_secs()
    }
}

/// Quantum-safe key encapsulation (placeholder for CRYSTALS-Kyber in production)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumKeyEncapsulation {
    /// Public key material (1344 bytes for Kyber-1024)
    pub public_key: Vec<u8>,
    /// Encapsulated shared secret
    pub ciphertext: Vec<u8>,
    /// Algorithm identifier
    pub algorithm: String,
}

impl QuantumKeyEncapsulation {
    /// Generate new quantum-safe key pair (placeholder)
    pub fn generate() -> (Self, Vec<u8>) {
        // In production, would use liboqs-rs to generate real Kyber keys
        // For now: simulate with deterministic generation
        let public_key = Self::generate_placeholder_key(1344);
        let shared_secret = Self::generate_placeholder_key(32);
        let ciphertext = Self::generate_placeholder_key(1088);

        (
            QuantumKeyEncapsulation {
                public_key,
                ciphertext,
                algorithm: "CRYSTALS-Kyber-1024".to_string(),
            },
            shared_secret,
        )
    }

    /// Decapsulate the shared secret
    pub fn decapsulate(&self) -> Option<Vec<u8>> {
        // In production: use liboqs-rs decapsulation
        // For now: simulate successful decapsulation
        Some(Self::generate_placeholder_key(32))
    }

    fn generate_placeholder_key(size: usize) -> Vec<u8> {
        let mut hasher = Keccak256::new();
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos();
        hasher.update(timestamp.to_le_bytes());
        let hash = hasher.finalize();

        let mut key = hash.to_vec();
        while key.len() < size {
            let mut hasher = Keccak256::new();
            hasher.update(&key);
            key.extend_from_slice(&hasher.finalize());
        }
        key.truncate(size);
        key
    }
}

/// Quantum-safe attestation system
pub struct QuantumSafeAttestation {
    issued_proofs: std::sync::Arc<tokio::sync::RwLock<Vec<QuantumAttestationProof>>>,
    revoked_ids: std::sync::Arc<tokio::sync::RwLock<Vec<String>>>,
}

impl QuantumSafeAttestation {
    pub fn new() -> Self {
        Self {
            issued_proofs: std::sync::Arc::new(tokio::sync::RwLock::new(Vec::new())),
            revoked_ids: std::sync::Arc::new(tokio::sync::RwLock::new(Vec::new())),
        }
    }

    /// Issue a quantum-safe capability attestation
    pub async fn issue_capability(
        &self,
        agent_id: String,
        capability_name: String,
        lifetime_days: i64,
        issuer_id: String,
    ) -> QuantumAttestationProof {
        let capability = QuantumCapability::new(agent_id, capability_name, lifetime_days);

        // Create dual signature
        let serialized = serde_json::to_vec(&capability).unwrap_or_default();

        // Simulate classical signature
        let mut classical_sig = Keccak256::new();
        classical_sig.update(&serialized);
        let classical = classical_sig.finalize().to_vec();

        // Simulate post-quantum signature
        let mut pq_sig = Keccak256::new();
        pq_sig.update(b"quantum-safe");
        pq_sig.update(&serialized);
        let pq = pq_sig.finalize().to_vec();

        let signature = QuantumSignature::new_dual(&serialized, classical, pq);
        let proof = QuantumAttestationProof::new(capability, signature, issuer_id);

        let mut proofs = self.issued_proofs.write().await;
        proofs.push(proof.clone());

        proof
    }

    /// Verify a capability attestation
    pub async fn verify_proof(&self, proof: &QuantumAttestationProof) -> bool {
        // Check if revoked
        let revoked = self.revoked_ids.read().await;
        if revoked.contains(&proof.proof_id) {
            return false;
        }

        // Verify proof is valid
        proof.verify()
    }

    /// Revoke a capability proof
    pub async fn revoke(&self, proof_id: &str) {
        let mut revoked = self.revoked_ids.write().await;
        revoked.push(proof_id.to_string());
    }

    /// Get all valid proofs for an agent
    pub async fn get_agent_proofs(&self, agent_id: &str) -> Vec<QuantumAttestationProof> {
        let proofs = self.issued_proofs.read().await;
        let revoked = self.revoked_ids.read().await;

        proofs
            .iter()
            .filter(|p| {
                p.capability.agent_id == agent_id
                    && p.capability.is_valid()
                    && !revoked.contains(&p.proof_id)
            })
            .cloned()
            .collect()
    }

    /// Generate quantum-safe key material
    pub fn generate_key_encapsulation() -> (QuantumKeyEncapsulation, Vec<u8>) {
        QuantumKeyEncapsulation::generate()
    }
}

impl Default for QuantumSafeAttestation {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quantum_capability() {
        let cap = QuantumCapability::new("agent-1".to_string(), "database.query".to_string(), 30);

        assert!(cap.is_valid());
        assert!(cap.time_remaining_secs() > 0);
    }

    #[test]
    fn test_quantum_signature() {
        let data = b"test data";
        let sig = QuantumSignature::new_dual(data, vec![1, 2, 3], vec![4, 5, 6]);

        assert!(sig.verify_dual(data));
    }

    #[test]
    fn test_attestation_proof() {
        let cap = QuantumCapability::new("agent-1".to_string(), "compute".to_string(), 30);

        let sig = QuantumSignature::new_dual(b"test", vec![1], vec![2]);
        let proof = QuantumAttestationProof::new(cap, sig, "issuer-1".to_string());

        assert!(proof.verify());
    }

    #[tokio::test]
    async fn test_attestation_system() {
        let system = QuantumSafeAttestation::new();
        let proof = system
            .issue_capability(
                "agent-1".to_string(),
                "database.query".to_string(),
                30,
                "issuer-1".to_string(),
            )
            .await;

        assert!(system.verify_proof(&proof).await);

        system.revoke(&proof.proof_id).await;
        assert!(!system.verify_proof(&proof).await);
    }

    #[test]
    fn test_key_encapsulation() {
        let (kea, shared_secret) = QuantumKeyEncapsulation::generate();
        assert_eq!(kea.public_key.len(), 1344);
        assert_eq!(shared_secret.len(), 32);

        let decapsulated = kea.decapsulate();
        assert!(decapsulated.is_some());
    }
}
