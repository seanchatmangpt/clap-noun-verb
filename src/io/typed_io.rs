//! Type-Level Validation for I/O Operations
//!
//! Advanced compile-time validation using:
//! - Phantom Types for state machines
//! - Generic Associated Types (GATs) for format-aware parsing
//! - Const generics for size constraints
//!
//! # State Machine Pattern
//!
//! Ensures operations happen in correct order:
//! - Unvalidated → Validated → Processed → Complete
//!
//! # Example
//!
//! ```rust,ignore
//! use clap_noun_verb::io::typed_io::{ValidatedPath, Unvalidated, Validated};
//!
//! // This won't compile without proper validation
//! let path: ValidatedPath<Unvalidated> = ValidatedPath::new("input.txt");
//! let validated: ValidatedPath<Validated> = path.validate()?;
//! // Now safe to use...
//! ```

use std::marker::PhantomData;
use std::path::{Path, PathBuf};

/// Marker type for unvalidated state
#[derive(Debug, Clone, Copy)]
pub struct Unvalidated;

/// Marker type for validated state
#[derive(Debug, Clone, Copy)]
pub struct Validated;

/// Marker type for processed state
#[derive(Debug, Clone, Copy)]
pub struct Processed;

/// Type-safe path that enforces validation at compile time
#[derive(Debug, Clone)]
pub struct ValidatedPath<State> {
    path: PathBuf,
    _state: PhantomData<State>,
}

impl ValidatedPath<Unvalidated> {
    /// Create new unvalidated path
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            _state: PhantomData,
        }
    }

    /// Validate the path exists and is readable
    pub fn validate(self) -> std::io::Result<ValidatedPath<Validated>> {
        if !self.path.exists() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("path does not exist: {}", self.path.display()),
            ));
        }

        if !self.path.is_file() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("path is not a file: {}", self.path.display()),
            ));
        }

        Ok(ValidatedPath {
            path: self.path,
            _state: PhantomData,
        })
    }

    /// Get reference to unvalidated path
    pub fn as_path(&self) -> &Path {
        &self.path
    }
}

impl ValidatedPath<Validated> {
    /// Mark as processed after reading
    pub fn mark_processed(self) -> ValidatedPath<Processed> {
        ValidatedPath {
            path: self.path,
            _state: PhantomData,
        }
    }

    /// Read the file content (only available on validated paths)
    pub fn read_to_string(&self) -> std::io::Result<String> {
        std::fs::read_to_string(&self.path)
    }

    /// Get reference to validated path
    pub fn as_path(&self) -> &Path {
        &self.path
    }
}

impl ValidatedPath<Processed> {
    /// Get reference to processed path
    pub fn as_path(&self) -> &Path {
        &self.path
    }
}

/// Trait for format-aware parsing with GATs
pub trait FormatParser {
    /// Associated lifetime for borrowed data
    type Input<'a>;

    /// Parse from input
    fn parse<'a>(&self, input: Self::Input<'a>) -> std::io::Result<Vec<u8>>;

    /// Format name
    fn format_name(&self) -> &'static str;
}

/// JSON format parser
#[derive(Debug, Clone)]
pub struct JsonFormat;

impl FormatParser for JsonFormat {
    type Input<'a> = &'a str;

    fn parse<'a>(&self, input: Self::Input<'a>) -> std::io::Result<Vec<u8>> {
        // Validate JSON
        serde_json::from_str::<serde_json::Value>(input)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
        Ok(input.as_bytes().to_vec())
    }

    fn format_name(&self) -> &'static str {
        "json"
    }
}

/// YAML format parser
#[derive(Debug, Clone)]
pub struct YamlFormat;

impl FormatParser for YamlFormat {
    type Input<'a> = &'a str;

    fn parse<'a>(&self, input: Self::Input<'a>) -> std::io::Result<Vec<u8>> {
        // Validate YAML
        serde_yaml::from_str::<serde_yaml::Value>(input)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
        Ok(input.as_bytes().to_vec())
    }

    fn format_name(&self) -> &'static str {
        "yaml"
    }
}

/// Plain text format (no validation)
#[derive(Debug, Clone)]
pub struct PlainFormat;

impl FormatParser for PlainFormat {
    type Input<'a> = &'a str;

    fn parse<'a>(&self, input: Self::Input<'a>) -> std::io::Result<Vec<u8>> {
        Ok(input.as_bytes().to_vec())
    }

    fn format_name(&self) -> &'static str {
        "plain"
    }
}

/// Const-generic validated buffer with size constraints
#[derive(Debug, Clone)]
pub struct ValidatedBuffer<const MIN: usize, const MAX: usize> {
    data: Vec<u8>,
}

impl<const MIN: usize, const MAX: usize> ValidatedBuffer<MIN, MAX> {
    /// Create new buffer with size validation
    pub fn new(data: Vec<u8>) -> Result<Self, String> {
        if data.len() < MIN {
            return Err(format!(
                "buffer too small: {} < {} bytes",
                data.len(),
                MIN
            ));
        }

        if data.len() > MAX {
            return Err(format!(
                "buffer too large: {} > {} bytes",
                data.len(),
                MAX
            ));
        }

        Ok(Self { data })
    }

    /// Get buffer length
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Get as slice
    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }

    /// Consume and get inner data
    pub fn into_vec(self) -> Vec<u8> {
        self.data
    }
}

/// Const-generic string with pattern constraints
pub struct ValidatedString<const MIN_LEN: usize, const MAX_LEN: usize> {
    value: String,
}

impl<const MIN_LEN: usize, const MAX_LEN: usize> ValidatedString<MIN_LEN, MAX_LEN> {
    /// Create new validated string
    pub fn new(value: String) -> Result<Self, String> {
        if value.len() < MIN_LEN {
            return Err(format!(
                "string too short: {} < {} characters",
                value.len(),
                MIN_LEN
            ));
        }

        if value.len() > MAX_LEN {
            return Err(format!(
                "string too long: {} > {} characters",
                value.len(),
                MAX_LEN
            ));
        }

        Ok(Self { value })
    }

    /// Get string reference
    pub fn as_str(&self) -> &str {
        &self.value
    }

    /// Consume and get inner string
    pub fn into_string(self) -> String {
        self.value
    }
}

impl<const MIN_LEN: usize, const MAX_LEN: usize> AsRef<str>
    for ValidatedString<MIN_LEN, MAX_LEN>
{
    fn as_ref(&self) -> &str {
        &self.value
    }
}

impl<const MIN_LEN: usize, const MAX_LEN: usize> std::fmt::Display
    for ValidatedString<MIN_LEN, MAX_LEN>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

/// Capability type for compile-time effect tracking
#[derive(Debug, Clone, Copy)]
pub struct Effect<const SIDE_EFFECTS: bool>;

impl<const SIDE_EFFECTS: bool> Effect<SIDE_EFFECTS> {
    /// Check if operation has side effects at compile time
    pub fn has_side_effects(&self) -> bool {
        SIDE_EFFECTS
    }
}

/// Pure operation (no side effects)
pub type PureOp = Effect<false>;

/// Operation with side effects
pub type ImpureOp = Effect<true>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validated_path_new() {
        let path = ValidatedPath::new("/tmp/test.txt");
        assert_eq!(path.as_path().to_string_lossy(), "/tmp/test.txt");
    }

    #[test]
    fn test_validated_path_validate_not_found() {
        let path = ValidatedPath::new("/nonexistent/path/file.txt");
        let result = path.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_json_format_parser() {
        let parser = JsonFormat;
        let valid_json = r#"{"key": "value"}"#;
        let result = parser.parse(valid_json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_json_format_parser_invalid() {
        let parser = JsonFormat;
        let invalid_json = r#"{"key": invalid}"#;
        let result = parser.parse(invalid_json);
        assert!(result.is_err());
    }

    #[test]
    fn test_plain_format_parser() {
        let parser = PlainFormat;
        let text = "Hello, World!";
        let result = parser.parse(text);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), text.as_bytes());
    }

    #[test]
    fn test_validated_buffer_valid() {
        let data = vec![1, 2, 3, 4, 5];
        let buffer: ValidatedBuffer<2, 10> = ValidatedBuffer::new(data).unwrap();
        assert_eq!(buffer.len(), 5);
    }

    #[test]
    fn test_validated_buffer_too_small() {
        let data = vec![1];
        let result: Result<ValidatedBuffer<2, 10>, _> = ValidatedBuffer::new(data);
        assert!(result.is_err());
    }

    #[test]
    fn test_validated_buffer_too_large() {
        let data = vec![0u8; 20];
        let result: Result<ValidatedBuffer<2, 10>, _> = ValidatedBuffer::new(data);
        assert!(result.is_err());
    }

    #[test]
    fn test_validated_string_valid() {
        let s = "hello".to_string();
        let validated: ValidatedString<2, 10> = ValidatedString::new(s).unwrap();
        assert_eq!(validated.as_str(), "hello");
    }

    #[test]
    fn test_validated_string_too_short() {
        let s = "h".to_string();
        let result: Result<ValidatedString<2, 10>, _> = ValidatedString::new(s);
        assert!(result.is_err());
    }

    #[test]
    fn test_validated_string_too_long() {
        let s = "this is a very long string".to_string();
        let result: Result<ValidatedString<2, 10>, _> = ValidatedString::new(s);
        assert!(result.is_err());
    }

    #[test]
    fn test_effect_pure() {
        let effect: PureOp = Effect;
        assert!(!effect.has_side_effects());
    }

    #[test]
    fn test_effect_impure() {
        let effect: ImpureOp = Effect;
        assert!(effect.has_side_effects());
    }

    #[test]
    fn test_format_parser_name() {
        assert_eq!(JsonFormat.format_name(), "json");
        assert_eq!(YamlFormat.format_name(), "yaml");
        assert_eq!(PlainFormat.format_name(), "plain");
    }

    #[test]
    fn test_validated_buffer_as_slice() {
        let data = vec![1, 2, 3];
        let buffer: ValidatedBuffer<1, 10> = ValidatedBuffer::new(data.clone()).unwrap();
        assert_eq!(buffer.as_slice(), &data[..]);
    }
}
