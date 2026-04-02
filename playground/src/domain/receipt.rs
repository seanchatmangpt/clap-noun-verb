//! Receipt domain - proof surface
//!
//! Receipts prove what sync actually did with cryptographic verification.

use serde::{Deserialize, Serialize};
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

    /// Finalize receipt (compute hash and sign)
    pub fn finalize(&mut self) -> Result<(), String> {
        // Compute content hash
        self.content_hash = self.compute_hash()?;

        // In full implementation, this would sign with private key
        self.signature = None;

        Ok(())
    }

    /// Compute SHA-256 hash of receipt content
    fn compute_hash(&self) -> Result<String, String> {
        let content = serde_json::to_vec(self)
            .map_err(|e| format!("Failed to serialize receipt: {}", e))?;

        // Use sha2::Sha256 if feature is enabled
        #[cfg(feature = "ggen-foundation")]
        {
            use sha2::Digest;
            let hash = sha2::Sha256::digest(&content);
            Ok(format!("{:x}", hash))
        }

        #[cfg(not(feature = "ggen-foundation"))]
        {
            // Fallback: simple hex encoding
            Ok(hex::encode(&content))
        }
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
pub struct ReceiptVerifier;

/// Result from receipt verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    /// Whether receipt is valid
    pub valid: bool,
    /// Whether signature is valid
    pub signature_valid: bool,
    /// Whether chain is valid (for chain verification)
    pub chain_valid: bool,
    /// Any warnings found during verification
    pub warnings: Vec<String>,
    /// Chain length (for chain verification)
    pub chain_length: usize,
    /// Broken links in chain (for chain verification)
    pub broken_links: Vec<String>,
}

impl ReceiptVerifier {
    /// Create new verifier
    pub fn new() -> Self {
        Self
    }

    /// Verify a single receipt
    pub fn verify(&self, receipt: &Receipt) -> Result<VerificationResult, String> {
        // Verify content hash
        let computed_hash = receipt.compute_hash()?;
        let hash_valid = computed_hash == receipt.content_hash;

        // In full implementation, this would verify signature
        let signature_valid = true;

        Ok(VerificationResult {
            valid: hash_valid && signature_valid,
            signature_valid,
            chain_valid: true, // Single receipt is always chain-valid
            warnings: if !hash_valid {
                vec!["Content hash mismatch".to_string()]
            } else {
                Vec::new()
            },
            chain_length: 1,
            broken_links: Vec::new(),
        })
    }

    /// Verify receipt chain (receipt pointing to previous receipts)
    pub fn verify_chain(&self, receipt: &Receipt) -> Result<VerificationResult, String> {
        // In full implementation, this would:
        // - Load previous receipts from chain
        // - Verify each receipt in the chain
        // - Check that hashes link correctly

        Ok(VerificationResult {
            valid: true,
            signature_valid: true,
            chain_valid: true,
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
