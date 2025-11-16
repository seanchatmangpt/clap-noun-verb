//! Execution receipts for audit and analysis

use super::guards::GuardResult;
use super::planes::{Plane, PlaneInteraction};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;

/// Configuration for receipt generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiptConfig {
    /// Whether to include the full output in the receipt
    pub include_output: bool,
    /// Whether to hash the result
    pub hash_result: bool,
    /// Whether to include argument values (may contain sensitive data)
    pub include_args: bool,
}

impl Default for ReceiptConfig {
    fn default() -> Self {
        Self { include_output: true, hash_result: true, include_args: true }
    }
}

impl ReceiptConfig {
    /// Create a new receipt configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Disable output inclusion (receipt metadata only)
    pub fn without_output(mut self) -> Self {
        self.include_output = false;
        self
    }

    /// Enable result hashing
    pub fn with_hash(mut self) -> Self {
        self.hash_result = true;
        self
    }

    /// Disable argument inclusion (for sensitive commands)
    pub fn without_args(mut self) -> Self {
        self.include_args = false;
        self
    }
}

/// Execution receipt for a command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionReceipt {
    /// Full command that was executed
    pub command: String,
    /// Arguments passed (optional, may be omitted for sensitive commands)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<HashMap<String, serde_json::Value>>,
    /// Timestamp of execution (ISO 8601)
    pub timestamp: String,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// Guard evaluation result
    pub guard: GuardResult,
    /// Plane interactions
    pub planes: HashMap<Plane, Vec<String>>,
    /// Hash of the result (SHA-256)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result_hash: Option<String>,
    /// Correlation ID for tracing
    pub correlation_id: String,
    /// Success status
    pub success: bool,
    /// Error information (if failed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<super::errors::StructuredError>,
}

impl ExecutionReceipt {
    /// Create a new execution receipt
    pub fn new(command: impl Into<String>) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| {
                let secs = d.as_secs();
                chrono::DateTime::<chrono::Utc>::from_timestamp(secs as i64, 0)
                    .map(|dt| dt.to_rfc3339())
                    .unwrap_or_else(|| "unknown".to_string())
            })
            .unwrap_or_else(|_| "unknown".to_string());

        Self {
            command: command.into(),
            args: None,
            timestamp,
            duration_ms: 0,
            guard: GuardResult::default(),
            planes: HashMap::new(),
            result_hash: None,
            correlation_id: uuid::Uuid::new_v4().to_string(),
            success: true,
            error: None,
        }
    }

    /// Set the arguments
    pub fn with_args(mut self, args: HashMap<String, serde_json::Value>) -> Self {
        self.args = Some(args);
        self
    }

    /// Set the duration
    pub fn with_duration_ms(mut self, duration_ms: u64) -> Self {
        self.duration_ms = duration_ms;
        self
    }

    /// Set the guard result
    pub fn with_guard(mut self, guard: GuardResult) -> Self {
        self.guard = guard;
        self
    }

    /// Set plane interactions
    pub fn with_planes(mut self, planes: &PlaneInteraction) -> Self {
        for (plane, interactions) in &planes.interactions {
            self.planes.insert(
                *plane,
                interactions.iter().map(|i| format!("{:?}", i).to_lowercase()).collect(),
            );
        }
        self
    }

    /// Set result hash
    pub fn with_result_hash(mut self, hash: impl Into<String>) -> Self {
        self.result_hash = Some(hash.into());
        self
    }

    /// Set correlation ID
    pub fn with_correlation_id(mut self, id: impl Into<String>) -> Self {
        self.correlation_id = id.into();
        self
    }

    /// Mark as failed with error
    pub fn with_error(mut self, error: super::errors::StructuredError) -> Self {
        self.success = false;
        self.error = Some(error);
        self
    }

    /// Compute SHA-256 hash of a value
    pub fn compute_hash(value: &impl Serialize) -> Option<String> {
        use sha2::{Digest, Sha256};

        serde_json::to_vec(value).ok().map(|bytes| {
            let hash = Sha256::digest(&bytes);
            format!("sha256:{}", hex::encode(hash))
        })
    }

    /// Convert to JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

/// Wrapper for receipt with optional output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiptWithOutput<T> {
    /// The execution receipt
    pub receipt: ExecutionReceipt,
    /// The command output (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<T>,
}

impl<T: Serialize> ReceiptWithOutput<T> {
    /// Create a new receipt with output
    pub fn new(receipt: ExecutionReceipt, output: Option<T>) -> Self {
        Self { receipt, output }
    }

    /// Convert to JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}
