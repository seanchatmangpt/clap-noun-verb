//! Integration tests for receipt verification
//!
//! Tests Ed25519 signature verification and SHA-256 hashing.

use playground_cli::domain::receipt::{Receipt, ReceiptAgent, ReceiptOperation, OperationResult, ReceiptVerifier};
use ed25519_dalek::SigningKey;

#[test]
fn test_receipt_creation_and_finalization() {
    // Create agent
    let agent = ReceiptAgent {
        agent_type: "test".to_string(),
        agent_id: "test-001".to_string(),
        version: "1.0.0".to_string(),
    };

    // Create receipt
    let mut receipt = Receipt::new(agent);

    // Add operation
    receipt.add_operation(ReceiptOperation {
        operation_type: "test_operation".to_string(),
        target: "test_target".to_string(),
        result: OperationResult {
            success: true,
            output: "Test output".to_string(),
            duration_ms: 100,
        },
        timestamp: chrono::Utc::now().to_rfc3339(),
    });

    // Finalize with signing key
    let signing_key = SigningKey::generate(&mut rand::rngs::OsRng);
    receipt.finalize(&signing_key).expect("Finalization failed");

    // Verify hash was computed
    assert!(!receipt.content_hash.is_empty());

    // Verify signature was created
    assert!(receipt.signature.is_some());
}

#[test]
fn test_receipt_hash_verification() {
    let agent = ReceiptAgent {
        agent_type: "test".to_string(),
        agent_id: "test-002".to_string(),
        version: "1.0.0".to_string(),
    };

    let mut receipt = Receipt::new(agent.clone());
    let signing_key = SigningKey::generate(&mut rand::rngs::OsRng);

    receipt.finalize(&signing_key).expect("Finalization failed");

    // Verify hash
    let verifier = ReceiptVerifier::new();
    let result = verifier.verify(&receipt).expect("Verification failed");

    // Should be valid (hash matches)
    assert!(result.valid);
    assert!(result.signature_valid); // No public key, so assumes valid
}

#[test]
fn test_receipt_signature_verification() {
    let agent = ReceiptAgent {
        agent_type: "test".to_string(),
        agent_id: "test-003".to_string(),
        version: "1.0.0".to_string(),
    };

    let mut receipt = Receipt::new(agent);
    let signing_key = SigningKey::generate(&mut rand::rngs::OsRng);
    let public_key = signing_key.verifying_key();

    receipt.finalize(&signing_key).expect("Finalization failed");

    // Verify with public key
    let verifier = ReceiptVerifier::with_public_key(*public_key);
    let result = verifier.verify(&receipt).expect("Verification failed");

    // Should be valid (signature matches)
    assert!(result.valid);
    assert!(result.signature_valid);
}

#[test]
fn test_receipt_tampering_detection() {
    let agent = ReceiptAgent {
        agent_type: "test".to_string(),
        agent_id: "test-004".to_string(),
        version: "1.0.0".to_string(),
    };

    let mut receipt = Receipt::new(agent);
    let signing_key = SigningKey::generate(&mut rand::rngs::OsRng);
    let public_key = signing_key.verifying_key();

    receipt.finalize(&signing_key).expect("Finalization failed");

    // Tamper with receipt
    receipt.content_hash = "tampered_hash".to_string();

    // Verify with public key
    let verifier = ReceiptVerifier::with_public_key(*public_key);
    let result = verifier.verify(&receipt).expect("Verification failed");

    // Should be invalid (hash mismatch)
    assert!(!result.valid);
    assert!(!result.signature_valid);
    assert!(result.warnings.iter().any(|w| w.contains("hash mismatch")));
}
