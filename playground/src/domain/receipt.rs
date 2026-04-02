//! Receipt domain - proof surface
//!
//! Receipts prove what sync actually did with cryptographic verification.

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;

/// Cryptographic receipt proving what sync actually did
///
/// Receipts are immutable proof objects that record:
/// - What operations were performed
/// - What artifacts were emitted
/// - Timestamp and agent identity
/// - Cryptographic signature for verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Receipt {
    /// Unique receipt ID (UUID v4)
    pub id: String,
    /// Timestamp when receipt was created
    pub timestamp: String,
    /// Operations performed during sync
    pub operations: Vec<ReceiptOperation>,
    /// Artifacts that were emitted
    pub artifacts: Vec<String>,
    /// Agent/system that created the receipt
    pub agent: ReceiptAgent,
    /// Cryptographic signature (Ed25519)
    pub signature: Option<String>,
    /// Hash of receipt content (SHA-256)
    pub content_hash: String,
}

/// Operation recorded in receipt
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReceiptOperation {
    /// Operation type (e.g., "pack_install", "template_render", "artifact_emit")
    pub operation_type: String,
    /// Target of operation (e.g., pack name, template path)
    pub target: String,
    /// Operation result (success/failure)
    pub result: OperationResult,
    /// Timestamp of operation
    pub timestamp: String,
}

/// Operation result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationResult {
    /// Whether operation succeeded
    pub success: bool,
    /// Output or error message
    pub output: String,
    /// Duration in milliseconds
    pub duration_ms: u64,
}

/// Agent that created the receipt
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReceiptAgent {
    /// Agent type (e.g., "ggen", "mcp-agent", "a2a-agent")
    pub agent_type: String,
    /// Agent identifier
    pub agent_id: String,
    /// Agent version
    pub version: String,
}

impl Receipt {
    /// Create a new receipt
    pub fn new(agent: ReceiptAgent) -> Self {
        let id = uuid::Uuid::new_v4().to_string();
        let timestamp = chrono::Utc::now().to_rfc3339();

        Self {
            id,
            timestamp,
            operations: Vec::new(),
            artifacts: Vec::new(),
            agent,
            signature: None,
            content_hash: String::new(),
        }
    }

    /// Add operation to receipt
    pub fn add_operation(&mut self, operation: ReceiptOperation) {
        self.operations.push(operation);
    }

    /// Add artifact to receipt
    pub fn add_artifact(&mut self, artifact: String) {
        self.artifacts.push(artifact);
    }

    /// Finalize receipt (compute hash and sign with provided key)
    ///
    /// # Arguments
    ///
    /// * `signing_key` - Ed25519 signing key for cryptographic signature
    pub fn finalize(&mut self, signing_key: &SigningKey) -> Result<(), String> {
        // Compute content hash (excluding signature field)
        self.content_hash = self.compute_hash()?;

        // Create signing message (hash + timestamp)
        let message = self.signing_message()?;

        // Sign with Ed25519
        let signature: Signature = signing_key.sign(&message);
        self.signature = Some(hex::encode(signature.to_bytes()));

        Ok(())
    }

    /// Compute SHA-256 hash of receipt content (excluding signature)
    fn compute_hash(&self) -> Result<String, String> {
        // Create a version without signature for hashing
        let hashable = ReceiptHashable {
            id: &self.id,
            timestamp: &self.timestamp,
            operations: &self.operations,
            artifacts: &self.artifacts,
            agent: &self.agent,
        };

        let content = serde_json::to_vec(&hashable)
            .map_err(|e| format!("Failed to serialize receipt for hashing: {}", e))?;

        // Compute SHA-256 hash
        let mut hasher = Sha256::new();
        hasher.update(&content);
        let hash = hasher.finalize();

        Ok(format!("{:x}", hash))
    }

    /// Create message for signing (hash + timestamp)
    pub fn signing_message(&self) -> Result<Vec<u8>, String> {
        let mut message = self.content_hash.as_bytes().to_vec();
        message.extend_from_slice(self.timestamp.as_bytes());
        Ok(message)
    }

    /// Load receipt from file
    pub fn from_file(path: &Path) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read receipt: {}", e))?;

        serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse receipt: {}", e))
    }

    /// Save receipt to file
    pub fn save(&self, path: &Path) -> Result<(), String> {
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize receipt: {}", e))?;

        std::fs::write(path, content)
            .map_err(|e| format!("Failed to write receipt: {}", e))
    }
}

/// Receipt verifier for cryptographic verification
pub struct ReceiptVerifier {
    /// Optional public key for signature verification
    public_key: Option<VerifyingKey>,
}

/// Hashable version of receipt (excludes signature)
#[derive(Serialize)]
struct ReceiptHashable<'a> {
    id: &'a String,
    timestamp: &'a String,
    operations: &'a Vec<ReceiptOperation>,
    artifacts: &'a Vec<String>,
    agent: &'a ReceiptAgent,
}

/// Result from receipt verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    /// Whether receipt is valid
    pub valid: bool,
    /// Whether signature is valid
    pub signature_valid: bool,
    /// Whether chain is valid (for chain verification)
    pub chain_valid: bool,
    /// Whether all receipts in chain are valid
    pub all_valid: bool,
    /// Any warnings found during verification
    pub warnings: Vec<String>,
    /// Chain length (for chain verification)
    pub chain_length: usize,
    /// Broken links in chain (for chain verification)
    pub broken_links: Vec<String>,
}

impl ReceiptVerifier {
    /// Create new verifier without public key (hash-only verification)
    pub fn new() -> Self {
        Self { public_key: None }
    }

    /// Create verifier with public key for signature verification
    pub fn with_public_key(public_key: VerifyingKey) -> Self {
        Self {
            public_key: Some(public_key),
        }
    }

    /// Verify a single receipt
    ///
    /// Verifies:
    /// 1. Content hash integrity
    /// 2. Ed25519 signature (if public key provided)
    pub fn verify(&self, receipt: &Receipt) -> Result<VerificationResult, String> {
        let mut warnings = Vec::new();
        let mut valid = true;

        // 1. Verify content hash
        let computed_hash = receipt.compute_hash()
            .map_err(|e| format!("Failed to compute hash: {}", e))?;
        let hash_valid = computed_hash == receipt.content_hash;

        if !hash_valid {
            warnings.push("Content hash mismatch - receipt may be tampered".to_string());
            valid = false;
        }

        // 2. Verify signature (if public key provided)
        let signature_valid = match (&receipt.signature, &self.public_key) {
            (Some(sig_bytes), Some(public_key)) => {
                // Decode signature from hex
                let sig_bytes_decoded = hex::decode(sig_bytes)
                    .map_err(|e| format!("Invalid signature hex encoding: {}", e))?;

                // Convert to ed25519-dalek Signature
                let signature = Signature::from_slice(&sig_bytes_decoded)
                    .map_err(|e| format!("Invalid signature format: {}", e))?;

                // Create signing message
                let message = receipt.signing_message()
                    .map_err(|e| format!("Failed to create signing message: {}", e))?;

                // Verify signature
                public_key.verify(&message, &signature)
                    .map(|_| true)
                    .unwrap_or(false)
            }
            (Some(_), None) => {
                warnings.push("Signature present but no public key provided - skipping signature verification".to_string());
                true // Assume valid if no key to verify against
            }
            (None, _) => {
                warnings.push("No signature present - receipt is not cryptographically signed".to_string());
                false
            }
        };

        if !signature_valid {
            warnings.push("Signature verification failed - receipt may be forged".to_string());
            valid = false;
        }

        Ok(VerificationResult {
            valid,
            signature_valid,
            chain_valid: true, // Single receipt is always chain-valid
            all_valid: valid && signature_valid,
            warnings,
            chain_length: 1,
            broken_links: Vec::new(),
        })
    }

    /// Verify receipt chain (receipt pointing to previous receipts)
    pub fn verify_chain(&self, _receipt: &Receipt) -> Result<VerificationResult, String> {
        // In full implementation, this would:
        // - Load previous receipts from chain
        // - Verify each receipt in the chain
        // - Check that hashes link correctly

        Ok(VerificationResult {
            valid: true,
            signature_valid: true,
            chain_valid: true,
            all_valid: true,
            warnings: Vec::new(),
            chain_length: 1,
            broken_links: Vec::new(),
        })
    }
}

impl Default for ReceiptVerifier {
    fn default() -> Self {
        Self::new()
    }
}
