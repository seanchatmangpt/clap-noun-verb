//! Abstract Syntax Tree for CLI specifications
//!
//! This module defines the intermediate representation for CLI commands
//! parsed from Turtle specifications. The AST is designed with type-first
//! thinking to encode invariants at compile time.
//!
//! ## Design Principles
//!
//! - **Type Safety**: Invalid states are unrepresentable
//! - **Zero-Cost**: Types compile to same representation as raw data
//! - **Explicit**: All invariants encoded in type system
//!
//! ## Examples
//!
//! ```rust
//! use clap_noun_verb::ggen_integration::ast::{Command, Argument, ArgumentKind, TypeAnnotation};
//!
//! let command = Command {
//!     name: "user".to_string(),
//!     description: "User management".to_string(),
//!     subcommands: vec![
//!         Command {
//!             name: "create".to_string(),
//!             description: "Create a user".to_string(),
//!             arguments: vec![
//!                 Argument {
//!                     name: "username".to_string(),
//!                     kind: ArgumentKind::Named {
//!                         long: "username".to_string(),
//!                         short: Some('u'),
//!                     },
//!                     type_annotation: TypeAnnotation::String,
//!                     required: true,
//!                     default: None,
//!                     help: "Username for the new user".to_string(),
//!                 }
//!             ],
//!             flags: vec![],
//!             subcommands: vec![],
//!         }
//!     ],
//!     arguments: vec![],
//!     flags: vec![],
//! };
//! ```

use serde::{Deserialize, Serialize};

/// Represents a CLI command (noun or verb)
///
/// Commands can be nested to create hierarchical command structures.
/// Each command has a name, description, arguments, flags, and subcommands.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Command {
    /// Command name (e.g., "user", "create")
    pub name: String,

    /// Human-readable description
    pub description: String,

    /// Positional and named arguments
    pub arguments: Vec<Argument>,

    /// Boolean flags
    pub flags: Vec<Flag>,

    /// Subcommands (for noun-verb patterns)
    pub subcommands: Vec<Command>,
}

/// Represents a command argument (positional or named)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Argument {
    /// Argument name (as it appears in code)
    pub name: String,

    /// Argument kind (positional or named)
    pub kind: ArgumentKind,

    /// Type annotation for generated code
    pub type_annotation: TypeAnnotation,

    /// Whether argument is required
    pub required: bool,

    /// Default value if not provided
    pub default: Option<String>,

    /// Help text
    pub help: String,
}

/// Argument kind: positional or named
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArgumentKind {
    /// Positional argument (e.g., `<filename>`)
    Positional {
        /// Position index (0-based)
        index: usize,
    },

    /// Named argument (e.g., `--name` or `-n`)
    Named {
        /// Long form (e.g., "name" for --name)
        long: String,

        /// Short form (e.g., 'n' for -n)
        short: Option<char>,
    },
}

/// Boolean flag (no value, presence indicates true)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Flag {
    /// Flag name (as it appears in code)
    pub name: String,

    /// Long form (e.g., "verbose" for --verbose)
    pub long: String,

    /// Short form (e.g., 'v' for -v)
    pub short: Option<char>,

    /// Help text
    pub help: String,

    /// Default value (typically false)
    pub default: bool,
}

/// Type annotation for generated Rust code
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TypeAnnotation {
    /// String type
    String,

    /// Integer type (i64)
    Integer,

    /// Floating point type (f64)
    Float,

    /// Boolean type
    Boolean,

    /// Path type (std::path::PathBuf)
    Path,

    /// Custom type (user-defined)
    Custom(String),
}

impl TypeAnnotation {
    /// Convert type annotation to Rust type string
    pub fn to_rust_type(&self) -> &str {
        match self {
            TypeAnnotation::String => "String",
            TypeAnnotation::Integer => "i64",
            TypeAnnotation::Float => "f64",
            TypeAnnotation::Boolean => "bool",
            TypeAnnotation::Path => "std::path::PathBuf",
            TypeAnnotation::Custom(t) => t.as_str(),
        }
    }
}

impl Command {
    /// Create a new command
    pub fn new(name: String, description: String) -> Self {
        Self {
            name,
            description,
            arguments: Vec::new(),
            flags: Vec::new(),
            subcommands: Vec::new(),
        }
    }

    /// Check if command has subcommands
    pub fn has_subcommands(&self) -> bool {
        !self.subcommands.is_empty()
    }

    /// Check if command has arguments
    pub fn has_arguments(&self) -> bool {
        !self.arguments.is_empty()
    }

    /// Check if command has flags
    pub fn has_flags(&self) -> bool {
        !self.flags.is_empty()
    }
}

impl Argument {
    /// Create a new positional argument
    pub fn positional(
        name: String, index: usize, type_annotation: TypeAnnotation, required: bool, help: String,
    ) -> Self {
        Self {
            name,
            kind: ArgumentKind::Positional { index },
            type_annotation,
            required,
            default: None,
            help,
        }
    }

    /// Create a new named argument
    pub fn named(
        name: String, long: String, short: Option<char>, type_annotation: TypeAnnotation,
        required: bool, help: String,
    ) -> Self {
        Self {
            name,
            kind: ArgumentKind::Named { long, short },
            type_annotation,
            required,
            default: None,
            help,
        }
    }

    /// Check if argument is positional
    pub fn is_positional(&self) -> bool {
        matches!(self.kind, ArgumentKind::Positional { .. })
    }

    /// Check if argument is named
    pub fn is_named(&self) -> bool {
        matches!(self.kind, ArgumentKind::Named { .. })
    }
}

impl Flag {
    /// Create a new flag
    pub fn new(name: String, long: String, short: Option<char>, help: String) -> Self {
        Self {
            name,
            long,
            short,
            help,
            default: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_new() {
        // Arrange & Act
        let cmd = Command::new("test".to_string(), "Test command".to_string());

        // Assert
        assert_eq!(cmd.name, "test");
        assert_eq!(cmd.description, "Test command");
        assert!(cmd.arguments.is_empty());
        assert!(cmd.flags.is_empty());
        assert!(cmd.subcommands.is_empty());
    }

    #[test]
    fn test_command_has_subcommands() {
        // Arrange
        let mut cmd = Command::new("test".to_string(), "Test".to_string());

        // Act & Assert - initially no subcommands
        assert!(!cmd.has_subcommands());

        // Arrange - add subcommand
        cmd.subcommands
            .push(Command::new("sub".to_string(), "Subcommand".to_string()));

        // Act & Assert - now has subcommands
        assert!(cmd.has_subcommands());
    }

    #[test]
    fn test_positional_argument() {
        // Arrange & Act
        let arg = Argument::positional(
            "file".to_string(),
            0,
            TypeAnnotation::Path,
            true,
            "Input file".to_string(),
        );

        // Assert
        assert_eq!(arg.name, "file");
        assert!(arg.is_positional());
        assert!(!arg.is_named());
        assert!(arg.required);
        assert_eq!(arg.type_annotation, TypeAnnotation::Path);
    }

    #[test]
    fn test_named_argument() {
        // Arrange & Act
        let arg = Argument::named(
            "output".to_string(),
            "output".to_string(),
            Some('o'),
            TypeAnnotation::String,
            false,
            "Output file".to_string(),
        );

        // Assert
        assert_eq!(arg.name, "output");
        assert!(!arg.is_positional());
        assert!(arg.is_named());
        assert!(!arg.required);
    }

    #[test]
    fn test_type_annotation_to_rust_type() {
        // Arrange & Act & Assert
        assert_eq!(TypeAnnotation::String.to_rust_type(), "String");
        assert_eq!(TypeAnnotation::Integer.to_rust_type(), "i64");
        assert_eq!(TypeAnnotation::Float.to_rust_type(), "f64");
        assert_eq!(TypeAnnotation::Boolean.to_rust_type(), "bool");
        assert_eq!(TypeAnnotation::Path.to_rust_type(), "std::path::PathBuf");
        assert_eq!(
            TypeAnnotation::Custom("MyType".to_string()).to_rust_type(),
            "MyType"
        );
    }

    #[test]
    fn test_flag_new() {
        // Arrange & Act
        let flag = Flag::new(
            "verbose".to_string(),
            "verbose".to_string(),
            Some('v'),
            "Enable verbose output".to_string(),
        );

        // Assert
        assert_eq!(flag.name, "verbose");
        assert_eq!(flag.long, "verbose");
        assert_eq!(flag.short, Some('v'));
        assert!(!flag.default);
    }
}
