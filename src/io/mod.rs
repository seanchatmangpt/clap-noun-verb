//! I/O Integration for clap-noun-verb
//!
//! Provides ergonomic, production-grade I/O handling with Typer-style simplicity.
//! Built on the proven `clio` crate with full integration into the #[verb] macro system.
//!
//! # Overview
//!
//! The I/O module brings ecosystem-grade I/O capabilities while maintaining
//! clap-noun-verb's philosophy of zero boilerplate. The #[verb] macro automatically
//! detects I/O types and wires them with appropriate clap configuration.
//!
//! # Key Types
//!
//! - **Input/Output**: Re-exported from clio with value_parser support
//! - **InputExt/OutputExt**: Convenience trait extensions
//! - **IoError**: Rich, contextual error type
//! - **IoType**: Type registry for macro-level I/O detection
//!
//! # Examples
//!
//! ## Basic Usage
//!
//! ```rust,ignore
//! use clap_noun_verb::io::Input;
//! use clap_noun_verb_macros::verb;
//!
//! #[verb]
//! fn process(
//!     #[arg(short, long)]
//!     input: Input,
//! ) -> Result<String> {
//!     // Automatically handles stdin vs file!
//!     let content = input.read_to_string()?;
//!     Ok(content)
//! }
//! ```
//!
//! ## Advanced Usage with Output
//!
//! ```rust,ignore
//! use clap_noun_verb::io::{Input, Output};
//! use clap_noun_verb_macros::verb;
//!
//! #[verb]
//! fn transform(
//!     #[arg(short, long)] input: Input,
//!     #[arg(short, long)] output: Option<Output>,
//! ) -> Result<String> {
//!     let content = input.read_to_string()?;
//!     let result = transform_data(&content)?;
//!
//!     if let Some(out) = output {
//!         out.write_all(result.as_bytes())?;
//!     } else {
//!         println!("{}", result);
//!     }
//!
//!     Ok(result)
//! }
//! ```
//!
//! # Integration with Kernel
//!
//! The I/O module complements kernel/io.rs:
//! - **kernel/io.rs**: Lower-level FileIO, InputSource, OutputSink
//! - **io/mod.rs**: Higher-level clio integration, macro support
//!
//! Users can choose:
//! - clio types + #[verb] auto-detection (recommended for new code)
//! - kernel FileIO (for advanced control)
//! - Mix both (via extension traits)

pub mod error;
pub mod types;

// Re-export core clio types with clap-parse support
pub use clio::{Input, InputPath, Output, OutputPath};

// Re-export error types
pub use error::{IoError, Result};

// Re-export type detection for macro use
pub use types::{IoType, IoTypeRegistry, TypeInspector};

/// Version of I/O module
pub const IO_MODULE_VERSION: &str = "4.0.0";

/// Check if clio is available
pub const CLIO_AVAILABLE: bool = true;

/// Convenience type alias for optional Output
pub type OutputOpt = Option<Output>;

/// Trait for convenient I/O operations
pub trait InputExt: std::io::Read {
    /// Read entire input to string
    fn read_to_string(&mut self) -> std::io::Result<String>;

    /// Read entire input to bytes
    fn read_to_bytes(&mut self) -> std::io::Result<Vec<u8>>;

    /// Get path if available
    fn path(&self) -> Option<&std::path::Path>;

    /// Check if reading from stdin
    fn is_stdin(&self) -> bool;
}

/// Trait for convenient Output operations
pub trait OutputExt: std::io::Write {
    /// Write string to output
    fn write_str(&mut self, s: &str) -> std::io::Result<()>;

    /// Get path if available
    fn path(&self) -> Option<&std::path::Path>;

    /// Check if writing to stdout
    fn is_stdout(&self) -> bool;
}

/// Advanced I/O pipeline for complex scenarios
pub struct IoPipeline {
    inputs: Vec<Input>,
    output: Option<Output>,
    buffer_size: usize,
}

impl IoPipeline {
    /// Create new I/O pipeline
    pub fn new() -> Self {
        Self {
            inputs: Vec::new(),
            output: None,
            buffer_size: 8192,
        }
    }

    /// Add input to pipeline
    pub fn with_input(mut self, input: Input) -> Self {
        self.inputs.push(input);
        self
    }

    /// Set output
    pub fn with_output(mut self, output: Output) -> Self {
        self.output = Some(output);
        self
    }

    /// Set buffer size
    pub fn with_buffer_size(mut self, size: usize) -> Self {
        self.buffer_size = size;
        self
    }

    /// Process pipeline - reads all inputs and writes to output
    pub fn process<F>(&mut self, processor: F) -> std::io::Result<u64>
    where
        F: Fn(&[u8]) -> std::io::Result<Vec<u8>>,
    {
        use std::io::{Read, Write};

        let mut total_written = 0u64;

        for input in &mut self.inputs {
            let mut buffer = Vec::new();
            input.read_to_end(&mut buffer)?;
            let processed = processor(&buffer)?;

            if let Some(ref mut output) = &mut self.output {
                output.write_all(&processed)?;
                total_written += processed.len() as u64;
            }
        }

        Ok(total_written)
    }

    /// Get inputs
    pub fn inputs(&self) -> &[Input] {
        &self.inputs
    }

    /// Get output
    pub fn output(&self) -> Option<&Output> {
        self.output.as_ref()
    }

    /// Get buffer size
    pub fn buffer_size(&self) -> usize {
        self.buffer_size
    }
}

impl Default for IoPipeline {
    fn default() -> Self {
        Self::new()
    }
}

/// Create an I/O pipeline builder for fluent API
pub fn pipeline() -> IoPipelineBuilder {
    IoPipelineBuilder::new()
}

/// Builder for IoPipeline
pub struct IoPipelineBuilder {
    inputs: Vec<Input>,
    output: Option<Output>,
    buffer_size: usize,
}

impl IoPipelineBuilder {
    /// Create new builder
    pub fn new() -> Self {
        Self {
            inputs: Vec::new(),
            output: None,
            buffer_size: 8192,
        }
    }

    /// Add input
    pub fn input(mut self, input: Input) -> Self {
        self.inputs.push(input);
        self
    }

    /// Set output
    pub fn output(mut self, output: Output) -> Self {
        self.output = Some(output);
        self
    }

    /// Set buffer size
    pub fn buffer_size(mut self, size: usize) -> Self {
        self.buffer_size = size;
        self
    }

    /// Build the pipeline
    pub fn build(self) -> IoPipeline {
        IoPipeline {
            inputs: self.inputs,
            output: self.output,
            buffer_size: self.buffer_size,
        }
    }
}

impl Default for IoPipelineBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_io_module_version() {
        assert_eq!(IO_MODULE_VERSION, "4.0.0");
        assert!(CLIO_AVAILABLE);
    }

    #[test]
    fn test_pipeline_builder() {
        let pipeline = pipeline().buffer_size(4096).build();
        assert_eq!(pipeline.buffer_size(), 4096);
        assert!(pipeline.inputs().is_empty());
        assert!(pipeline.output().is_none());
    }

    #[test]
    fn test_io_pipeline_default() {
        let pipe = IoPipeline::default();
        assert_eq!(pipe.buffer_size(), 8192);
    }
}
