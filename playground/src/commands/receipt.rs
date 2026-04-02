//! Receipt commands - proof surface
//!
//! Receipts prove what sync actually did.

use clap_noun_verb_macros::verb;
use clap_noun_verb::Result;

use crate::domain::receipt::{Receipt, ReceiptVerifier};
use crate::outputs::{ReceiptVerifyOutput, ReceiptInfoOutput, ReceiptChainVerifyOutput};

/// Verify a receipt
///
/// Validates cryptographic signature and content hash.
///
/// # Arguments
/// * `file` - Receipt file path [value_hint: FilePath]
#[verb("verify")]
fn verify_receipt(
    file: String,
) -> Result<ReceiptVerifyOutput> {
    let receipt = Receipt::from_file(std::path::Path::new(&file))?;
    let verifier = ReceiptVerifier::new();
    let result = verifier.verify(&receipt)?;

    Ok(ReceiptVerifyOutput {
        receipt_id: receipt.id,
        is_valid: result.valid,
        signature_valid: result.signature_valid,
        chain_valid: result.chain_valid,
        warnings: result.warnings,
    })
}

/// Get receipt info
///
/// Displays detailed information about a receipt.
///
/// # Arguments
/// * `file` - Receipt file path [value_hint: FilePath]
#[verb("info")]
fn receipt_info(
    file: String,
) -> Result<ReceiptInfoOutput> {
    let receipt = Receipt::from_file(std::path::Path::new(&file))?;

    Ok(ReceiptInfoOutput {
        id: receipt.id,
        timestamp: receipt.timestamp,
        operations: receipt.operations.len(),
        artifacts: receipt.artifacts,
        agent_type: receipt.agent.agent_type,
        agent_id: receipt.agent.agent_id,
        agent_version: receipt.agent.version,
    })
}

/// Verify receipt chain
///
/// Verifies a chain of receipts (receipt pointing to previous receipts).
///
/// # Arguments
/// * `file` - Receipt file path [value_hint: FilePath]
#[verb("chain-verify")]
fn chain_verify(
    file: String,
) -> Result<ReceiptChainVerifyOutput> {
    let receipt = Receipt::from_file(std::path::Path::new(&file))?;
    let verifier = ReceiptVerifier::new();
    let result = verifier.verify_chain(&receipt)?;

    Ok(ReceiptChainVerifyOutput {
        chain_length: result.chain_length,
        all_valid: result.all_valid,
        broken_links: result.broken_links,
    })
}
