//! Type Detection Helpers for Macro Expansion
//!
//! Provides utilities for detecting I/O types in function signatures.
//! Used by the #[verb] and #[noun] macros to auto-enhance parameters
//! with I/O handling capabilities.
//!
//! # Design
//!
//! Type detection works at the syn level to identify:
//! - `clio::Input` parameters
//! - `clio::Output` parameters
//! - `Option<clio::Output>` parameters
//!
//! This allows the macro to inject appropriate ValueParser and clap configuration.

use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, RwLock};

/// Information about a detected I/O type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IoType {
    /// Required Input file/stdin
    Input,
    /// Required Output file/stdout
    Output,
    /// Optional Output file/stdout
    OutputOptional,
    /// Custom I/O type with properties
    Custom { name: String, properties: HashMap<String, String> },
}

impl fmt::Display for IoType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Input => write!(f, "Input"),
            Self::Output => write!(f, "Output"),
            Self::OutputOptional => write!(f, "OutputOptional"),
            Self::Custom { name, .. } => write!(f, "{}", name),
        }
    }
}

impl IoType {
    /// Check if this is an input type
    pub fn is_input(&self) -> bool {
        matches!(self, Self::Input)
    }

    /// Check if this is an output type
    pub fn is_output(&self) -> bool {
        matches!(self, Self::Output | Self::OutputOptional)
    }

    /// Check if this is optional
    pub fn is_optional(&self) -> bool {
        matches!(self, Self::OutputOptional)
    }

    /// Get the clap ValueParser expression
    pub fn value_parser_expr(&self) -> String {
        match self {
            Self::Input => "clio::Input::value_parser()".to_string(),
            Self::Output | Self::OutputOptional => "clio::Output::value_parser()".to_string(),
            Self::Custom { name, .. } => format!("{}::value_parser()", name),
        }
    }

    /// Get help text for this type
    pub fn help_text(&self) -> String {
        match self {
            Self::Input => "Input file or path (use '-' for stdin)".to_string(),
            Self::Output => "Output file or path (use '-' for stdout)".to_string(),
            Self::OutputOptional => "Optional output file or path (use '-' for stdout)".to_string(),
            Self::Custom { name, .. } => format!("{} (custom I/O type)", name),
        }
    }
}

/// Type registry for I/O types
pub struct IoTypeRegistry {
    types: Arc<RwLock<HashMap<String, IoType>>>,
}

impl IoTypeRegistry {
    /// Create a new registry with built-in types
    pub fn new() -> Self {
        let mut types = HashMap::new();
        types.insert("Input".to_string(), IoType::Input);
        types.insert("Output".to_string(), IoType::Output);

        Self { types: Arc::new(RwLock::new(types)) }
    }

    /// Register a custom I/O type
    pub fn register(&self, name: String, io_type: IoType) -> Result<(), String> {
        let mut types =
            self.types.write().map_err(|_| "failed to acquire write lock".to_string())?;
        types.insert(name, io_type);
        Ok(())
    }

    /// Detect I/O type from name
    pub fn detect(&self, type_name: &str) -> Option<IoType> {
        let types = self.types.read().ok()?;
        types.get(type_name).cloned()
    }

    /// Check if a name is a registered I/O type
    pub fn is_io_type(&self, type_name: &str) -> bool {
        self.types.read().map(|types| types.contains_key(type_name)).unwrap_or(false)
    }

    /// List all registered I/O types
    pub fn list_types(&self) -> Vec<(String, IoType)> {
        self.types
            .read()
            .map(|types| types.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .unwrap_or_default()
    }
}

impl Default for IoTypeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Global I/O type registry (lazily initialized)
lazy_static::lazy_static! {
    static ref GLOBAL_REGISTRY: IoTypeRegistry = IoTypeRegistry::new();
}

/// Get global I/O type registry
pub fn global_registry() -> &'static IoTypeRegistry {
    &GLOBAL_REGISTRY
}

/// Type inspection utility (for proc macro use)
#[derive(Debug)]
pub struct TypeInspector {
    registry: Arc<RwLock<HashMap<String, IoType>>>,
}

impl TypeInspector {
    /// Create new inspector with custom registry
    pub fn new() -> Self {
        Self { registry: Arc::new(RwLock::new(HashMap::new())) }
    }

    /// Inspect a type name
    pub fn inspect(&self, type_name: &str) -> Option<IoType> {
        self.registry
            .read()
            .ok()
            .and_then(|r| r.get(type_name).cloned())
            .or_else(|| global_registry().detect(type_name))
    }

    /// Register custom type
    pub fn register(&self, name: String, io_type: IoType) {
        if let Ok(mut registry) = self.registry.write() {
            registry.insert(name, io_type);
        }
    }
}

impl Default for TypeInspector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_io_type_properties() {
        assert!(IoType::Input.is_input());
        assert!(IoType::Output.is_output());
        assert!(IoType::OutputOptional.is_optional());
    }

    #[test]
    fn test_io_type_value_parser() {
        assert_eq!(IoType::Input.value_parser_expr(), "clio::Input::value_parser()");
        assert_eq!(IoType::Output.value_parser_expr(), "clio::Output::value_parser()");
    }

    #[test]
    fn test_registry_operations() {
        let registry = IoTypeRegistry::new();
        assert!(registry.is_io_type("Input"));
        assert!(registry.is_io_type("Output"));
        assert!(!registry.is_io_type("NonExistent"));
    }

    #[test]
    fn test_type_inspector() {
        let inspector = TypeInspector::new();
        inspector.register("CustomIO".to_string(), IoType::Input);
        assert_eq!(inspector.inspect("CustomIO"), Some(IoType::Input));
    }

    #[test]
    fn test_help_text_generation() {
        let input_help = IoType::Input.help_text();
        assert!(input_help.contains("Input"));
        assert!(input_help.contains("stdin"));
    }
}
