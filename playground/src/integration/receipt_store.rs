//! Receipt storage integration layer
//!
//! Handles storing and retrieving cryptographic receipts.

use crate::domain::receipt::Receipt;
use std::path::{Path, PathBuf};

/// Receipt storage manager
pub struct ReceiptStore {
    receipts_dir: PathBuf,
}

impl ReceiptStore {
    /// Default receipts directory name
    pub const DIRNAME: &'static str = "receipts";

    /// Create new receipt store for workspace
    pub fn new(workspace_root: PathBuf) -> Result<Self, String> {
        let receipts_dir = workspace_root.join(Self::DIRNAME);

        // Create receipts directory if it doesn't exist
        if !receipts_dir.exists() {
            std::fs::create_dir_all(&receipts_dir)
                .map_err(|e| format!("Failed to create receipts directory: {}", e))?;
        }

        Ok(Self { receipts_dir })
    }

    /// Get path to receipt file
    pub fn receipt_path(&self, receipt_id: &str) -> PathBuf {
        self.receipts_dir.join(format!("{}.json", receipt_id))
    }

    /// Store receipt
    pub fn store(&self, receipt: &Receipt) -> Result<(), String> {
        let path = self.receipt_path(&receipt.id);
        receipt.save(&path)
    }

    /// Load receipt by ID
    pub fn load(&self, receipt_id: &str) -> Result<Receipt, String> {
        let path = self.receipt_path(receipt_id);
        Receipt::from_file(&path)
    }

    /// List all receipt IDs
    pub fn list_receipts(&self) -> Result<Vec<String>, String> {
        let mut receipts = Vec::new();

        for entry in std::fs::read_dir(&self.receipts_dir)
            .map_err(|e| format!("Failed to read receipts directory: {}", e))?
        {
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                    receipts.push(name.to_string());
                }
            }
        }

        receipts.sort();
        receipts.reverse(); // Most recent first
        Ok(receipts)
    }

    /// Get receipts directory path
    pub fn receipts_dir(&self) -> &Path {
        &self.receipts_dir
    }
}
