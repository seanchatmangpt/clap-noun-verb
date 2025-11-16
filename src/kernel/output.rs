//! CNV Output Pipeline
//!
//! Provides deterministic, structured output for all verbs with:
//! - Consistent result envelopes
//! - Machine-readable error surface
//! - Stable field ordering
//! - Format-specific rendering rules
//!
//! # Design
//!
//! Every verb returns a structured result that CNV owns:
//! - Serialization
//! - Envelope wrapping (if enabled)
//! - Format-specific rendering
//! - Error normalization
//!
//! # Example
//!
//! ```rust,ignore
//! use clap_noun_verb::kernel::{OutputPipeline, TelemetryProfile};
//!
//! fn my_verb(profile: &TelemetryProfile) -> StructuredResult<MyData> {
//!     let data = do_work()?;
//!     Ok(data.into())
//! }
//!
//! // In the CLI layer:
//! let result = my_verb(&profile);
//! OutputPipeline::render(result, &profile)?;
//! ```

use crate::format::OutputFormat;
use crate::kernel::telemetry::TelemetryProfile;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::process::ExitCode;

/// Exit codes for structured error reporting
///
/// Provides consistent exit codes across all CNV applications:
/// - 0: Success
/// - 1: General error
/// - 2: Usage error (invalid arguments)
/// - 3: Input error (invalid input data)
/// - 4: Not found (resource doesn't exist)
/// - 5: Permission denied
/// - 6: Timeout
/// - 7-63: Reserved for future use
/// - 64+: Application-specific
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitCodeClass {
    /// Success (0)
    Success = 0,
    /// General error (1)
    GeneralError = 1,
    /// Usage error - invalid arguments (2)
    UsageError = 2,
    /// Input error - invalid input data (3)
    InputError = 3,
    /// Resource not found (4)
    NotFound = 4,
    /// Permission denied (5)
    PermissionDenied = 5,
    /// Operation timeout (6)
    Timeout = 6,
}

impl From<ExitCodeClass> for ExitCode {
    fn from(class: ExitCodeClass) -> Self {
        ExitCode::from(class as u8)
    }
}

/// Structured error for machine-readable error reporting
///
/// Provides:
/// - Error code/kind for classification
/// - Human-readable message
/// - Optional context (key-value pairs)
/// - Optional source chain
/// - Consistent exit code mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredError {
    /// Error kind/code for machine consumption
    pub kind: String,
    /// Human-readable error message
    pub message: String,
    /// Additional context (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<HashMap<String, serde_json::Value>>,
    /// Source error chain (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<Box<StructuredError>>,
    /// Exit code for this error
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<u8>,
}

impl StructuredError {
    /// Create a new structured error
    pub fn new(kind: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            kind: kind.into(),
            message: message.into(),
            context: None,
            source: None,
            exit_code: Some(ExitCodeClass::GeneralError as u8),
        }
    }

    /// Add context to the error
    pub fn with_context(mut self, key: impl Into<String>, value: impl Serialize) -> Self {
        let context = self.context.get_or_insert_with(HashMap::new);
        if let Ok(json_value) = serde_json::to_value(value) {
            context.insert(key.into(), json_value);
        }
        self
    }

    /// Set the exit code
    pub fn with_exit_code(mut self, code: ExitCodeClass) -> Self {
        self.exit_code = Some(code as u8);
        self
    }

    /// Add a source error
    pub fn with_source(mut self, source: StructuredError) -> Self {
        self.source = Some(Box::new(source));
        self
    }

    /// Get the exit code
    pub fn exit_code(&self) -> u8 {
        self.exit_code.unwrap_or(ExitCodeClass::GeneralError as u8)
    }
}

impl fmt::Display for StructuredError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.kind, self.message)?;
        if let Some(ctx) = &self.context {
            write!(f, " (context: {:?})", ctx)?;
        }
        if let Some(source) = &self.source {
            write!(f, "\nCaused by: {}", source)?;
        }
        Ok(())
    }
}

impl std::error::Error for StructuredError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None // We store source as StructuredError, not dyn Error
    }
}

/// Convert standard errors to structured errors
impl From<Box<dyn std::error::Error>> for StructuredError {
    fn from(err: Box<dyn std::error::Error>) -> Self {
        Self::new("error", err.to_string())
    }
}

impl From<String> for StructuredError {
    fn from(msg: String) -> Self {
        Self::new("error", msg)
    }
}

impl From<&str> for StructuredError {
    fn from(msg: &str) -> Self {
        Self::new("error", msg)
    }
}

/// Result type for structured operations
pub type StructuredResult<T> = Result<T, StructuredError>;

/// Output envelope for wrapping verb results
///
/// Provides a consistent structure for all command outputs:
///
/// ```json
/// {
///   "status": "success" | "error",
///   "data": { ... },
///   "error": { ... },
///   "metadata": { ... }
/// }
/// ```
///
/// The envelope is optional and can be disabled for simpler output.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "lowercase")]
pub enum OutputEnvelope<T> {
    /// Success response with data
    Success {
        /// The actual payload
        data: T,
        /// Optional metadata (execution time, version, etc.)
        #[serde(skip_serializing_if = "Option::is_none")]
        metadata: Option<HashMap<String, serde_json::Value>>,
    },
    /// Error response
    Error {
        /// Structured error information
        error: StructuredError,
        /// Optional metadata
        #[serde(skip_serializing_if = "Option::is_none")]
        metadata: Option<HashMap<String, serde_json::Value>>,
    },
}

impl<T> OutputEnvelope<T> {
    /// Create a success envelope
    pub fn success(data: T) -> Self {
        Self::Success {
            data,
            metadata: None,
        }
    }

    /// Create an error envelope
    pub fn error(error: StructuredError) -> Self {
        Self::Error {
            error,
            metadata: None,
        }
    }

    /// Add metadata to the envelope
    pub fn with_metadata(
        mut self,
        key: impl Into<String>,
        value: impl Serialize,
    ) -> Self {
        let metadata_map = match &mut self {
            Self::Success { metadata, .. } => metadata,
            Self::Error { metadata, .. } => metadata,
        };

        let map = metadata_map.get_or_insert_with(HashMap::new);
        if let Ok(json_value) = serde_json::to_value(value) {
            map.insert(key.into(), json_value);
        }
        self
    }

    /// Check if this is a success
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success { .. })
    }

    /// Check if this is an error
    pub fn is_error(&self) -> bool {
        matches!(self, Self::Error { .. })
    }
}

impl<T> From<Result<T, StructuredError>> for OutputEnvelope<T> {
    fn from(result: Result<T, StructuredError>) -> Self {
        match result {
            Ok(data) => Self::success(data),
            Err(error) => Self::error(error),
        }
    }
}

/// Output pipeline configuration
#[derive(Debug, Clone)]
pub struct OutputPipelineConfig {
    /// Use envelope wrapping
    pub use_envelope: bool,
    /// Stable field ordering (slower but deterministic)
    pub stable_ordering: bool,
    /// Pretty print JSON/YAML
    pub pretty: bool,
    /// Include metadata (execution time, etc.)
    pub include_metadata: bool,
}

impl Default for OutputPipelineConfig {
    fn default() -> Self {
        Self {
            use_envelope: false, // Start simple, can be enabled later
            stable_ordering: true, // Determinism by default
            pretty: true, // Human-friendly by default
            include_metadata: false, // Opt-in for metadata
        }
    }
}

/// Output pipeline for rendering structured results
///
/// Handles:
/// - Envelope wrapping (optional)
/// - Format-specific rendering
/// - Error normalization
/// - Deterministic output
pub struct OutputPipeline;

impl OutputPipeline {
    /// Render a result to stdout/stderr according to the telemetry profile
    ///
    /// Success goes to stdout, errors to stderr.
    /// Exit code is set based on the error class.
    pub fn render<T: Serialize>(
        result: StructuredResult<T>,
        profile: &TelemetryProfile,
    ) -> Result<(), Box<dyn std::error::Error>> {
        Self::render_with_config(result, profile, &OutputPipelineConfig::default())
    }

    /// Render with custom configuration
    pub fn render_with_config<T: Serialize>(
        result: StructuredResult<T>,
        profile: &TelemetryProfile,
        config: &OutputPipelineConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match result {
            Ok(data) => {
                let output = if config.use_envelope {
                    let envelope = OutputEnvelope::success(data);
                    profile.format_output(&envelope)?
                } else {
                    profile.format_output(&data)?
                };
                println!("{}", output);
                Ok(())
            }
            Err(error) => {
                let output = if config.use_envelope {
                    let envelope: OutputEnvelope<()> = OutputEnvelope::error(error.clone());
                    profile.format_output(&envelope)?
                } else {
                    profile.format_output(&error)?
                };
                eprintln!("{}", output);
                Err(Box::new(error))
            }
        }
    }

    /// Format a result to a string without printing
    pub fn format<T: Serialize>(
        result: StructuredResult<T>,
        format: OutputFormat,
    ) -> Result<String, Box<dyn std::error::Error>> {
        match result {
            Ok(data) => format.format(&data),
            Err(error) => format.format(&error),
        }
    }

    /// Format with envelope
    pub fn format_with_envelope<T: Serialize>(
        result: StructuredResult<T>,
        format: OutputFormat,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let envelope: OutputEnvelope<T> = result.into();
        format.format(&envelope)
    }
}

/// Helper trait for converting types into structured results
pub trait IntoStructuredResult<T> {
    /// Convert into a structured result
    fn into_structured(self) -> StructuredResult<T>;
}

impl<T, E: std::error::Error> IntoStructuredResult<T> for Result<T, E> {
    fn into_structured(self) -> StructuredResult<T> {
        self.map_err(|e| StructuredError::new("error", e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Serialize, Deserialize)]
    struct TestData {
        value: i32,
    }

    #[test]
    fn test_structured_error() {
        let err = StructuredError::new("test_error", "Something went wrong")
            .with_context("key", "value")
            .with_exit_code(ExitCodeClass::UsageError);

        assert_eq!(err.kind, "test_error");
        assert_eq!(err.message, "Something went wrong");
        assert_eq!(err.exit_code(), 2);
        assert!(err.context.is_some());
    }

    #[test]
    fn test_output_envelope_success() {
        let data = TestData { value: 42 };
        let envelope = OutputEnvelope::success(data);

        assert!(envelope.is_success());
        assert!(!envelope.is_error());
    }

    #[test]
    fn test_output_envelope_error() {
        let error = StructuredError::new("test", "failed");
        let envelope: OutputEnvelope<TestData> = OutputEnvelope::error(error);

        assert!(envelope.is_error());
        assert!(!envelope.is_success());
    }

    #[test]
    fn test_output_envelope_from_result() {
        let result: StructuredResult<TestData> = Ok(TestData { value: 42 });
        let envelope: OutputEnvelope<TestData> = result.into();
        assert!(envelope.is_success());

        let result: StructuredResult<TestData> = Err(StructuredError::new("test", "failed"));
        let envelope: OutputEnvelope<TestData> = result.into();
        assert!(envelope.is_error());
    }

    #[test]
    fn test_output_pipeline_format() {
        let data = TestData { value: 42 };
        let result: StructuredResult<TestData> = Ok(data);

        let output = OutputPipeline::format(result, OutputFormat::Json);
        assert!(output.is_ok());
        assert!(output.unwrap().contains("\"value\""));
    }

    #[test]
    fn test_exit_code_conversion() {
        let code: ExitCode = ExitCodeClass::Success.into();
        assert_eq!(code, ExitCode::from(0));

        let code: ExitCode = ExitCodeClass::NotFound.into();
        assert_eq!(code, ExitCode::from(4));
    }
}
