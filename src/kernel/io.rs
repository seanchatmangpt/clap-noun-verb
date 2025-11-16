//! CNV File IO Integration
//!
//! Provides stream-based input/output abstraction with conventional flags:
//! - `--input` / `-i`: Input file or stdin
//! - `--output` / `-o`: Output file or stdout
//!
//! # Design
//!
//! Verb handlers see streams (readers/writers), not raw paths.
//! CNV handles opening/closing and error reporting through the
//! structured error pipeline.
//!
//! # Example
//!
//! ```rust,ignore
//! use clap_noun_verb::kernel::FileIO;
//!
//! #[verb]
//! fn process_data(
//!     #[arg(short, long)] input: Option<String>,
//!     #[arg(short, long)] output: Option<String>,
//! ) -> Result<()> {
//!     let io = FileIO::from_args(input.as_deref(), output.as_deref())?;
//!
//!     let mut reader = io.reader()?;
//!     let mut writer = io.writer()?;
//!
//!     // Process: read from reader, write to writer
//!     std::io::copy(&mut reader, &mut writer)?;
//!
//!     Ok(())
//! }
//! ```

use crate::kernel::output::{ExitCodeClass, StructuredError, StructuredResult};
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};

/// Input source - file or stdin
#[derive(Debug, Clone)]
pub enum InputSource {
    /// Read from stdin
    Stdin,
    /// Read from file
    File(PathBuf),
}

impl InputSource {
    /// Create from optional path
    ///
    /// None or "-" → stdin
    /// Some(path) → file
    pub fn from_path(path: Option<&str>) -> Self {
        match path {
            None | Some("-") => Self::Stdin,
            Some(p) => Self::File(PathBuf::from(p)),
        }
    }

    /// Check if this is stdin
    pub fn is_stdin(&self) -> bool {
        matches!(self, Self::Stdin)
    }

    /// Get the path (if file)
    pub fn path(&self) -> Option<&Path> {
        match self {
            Self::Stdin => None,
            Self::File(p) => Some(p),
        }
    }

    /// Open the input source as a readable stream
    pub fn open(&self) -> StructuredResult<Box<dyn Read>> {
        match self {
            Self::Stdin => Ok(Box::new(io::stdin())),
            Self::File(path) => {
                File::open(path)
                    .map(|f| Box::new(f) as Box<dyn Read>)
                    .map_err(|e| {
                        StructuredError::new("input_error", format!("Failed to open input: {}", e))
                            .with_context("path", path.to_string_lossy().to_string())
                            .with_exit_code(ExitCodeClass::InputError)
                    })
            }
        }
    }

    /// Open as buffered reader
    pub fn buffered(&self) -> StructuredResult<Box<dyn BufRead>> {
        match self {
            Self::Stdin => Ok(Box::new(io::stdin().lock())),
            Self::File(path) => {
                File::open(path)
                    .map(|f| Box::new(BufReader::new(f)) as Box<dyn BufRead>)
                    .map_err(|e| {
                        StructuredError::new("input_error", format!("Failed to open input: {}", e))
                            .with_context("path", path.to_string_lossy().to_string())
                            .with_exit_code(ExitCodeClass::InputError)
                    })
            }
        }
    }
}

impl Default for InputSource {
    fn default() -> Self {
        Self::Stdin
    }
}

/// Output sink - file or stdout
#[derive(Debug, Clone)]
pub enum OutputSink {
    /// Write to stdout
    Stdout,
    /// Write to file
    File(PathBuf),
}

impl OutputSink {
    /// Create from optional path
    ///
    /// None or "-" → stdout
    /// Some(path) → file
    pub fn from_path(path: Option<&str>) -> Self {
        match path {
            None | Some("-") => Self::Stdout,
            Some(p) => Self::File(PathBuf::from(p)),
        }
    }

    /// Check if this is stdout
    pub fn is_stdout(&self) -> bool {
        matches!(self, Self::Stdout)
    }

    /// Get the path (if file)
    pub fn path(&self) -> Option<&Path> {
        match self {
            Self::Stdout => None,
            Self::File(p) => Some(p),
        }
    }

    /// Open the output sink as a writable stream
    pub fn open(&self) -> StructuredResult<Box<dyn Write>> {
        match self {
            Self::Stdout => Ok(Box::new(io::stdout())),
            Self::File(path) => {
                File::create(path)
                    .map(|f| Box::new(f) as Box<dyn Write>)
                    .map_err(|e| {
                        StructuredError::new("output_error", format!("Failed to create output: {}", e))
                            .with_context("path", path.to_string_lossy().to_string())
                            .with_exit_code(ExitCodeClass::InputError)
                    })
            }
        }
    }

    /// Open as buffered writer
    pub fn buffered(&self) -> StructuredResult<Box<dyn Write>> {
        match self {
            Self::Stdout => Ok(Box::new(io::stdout())),
            Self::File(path) => {
                File::create(path)
                    .map(|f| Box::new(BufWriter::new(f)) as Box<dyn Write>)
                    .map_err(|e| {
                        StructuredError::new("output_error", format!("Failed to create output: {}", e))
                            .with_context("path", path.to_string_lossy().to_string())
                            .with_exit_code(ExitCodeClass::InputError)
                    })
            }
        }
    }
}

impl Default for OutputSink {
    fn default() -> Self {
        Self::Stdout
    }
}

/// File IO configuration for verb handlers
///
/// Encapsulates input/output sources and provides
/// convenient methods for opening streams.
#[derive(Debug, Clone)]
pub struct FileIO {
    /// Input source
    input: InputSource,
    /// Output sink
    output: OutputSink,
    /// Additional input sources (for commands that take multiple inputs)
    additional_inputs: Vec<InputSource>,
}

impl FileIO {
    /// Create from input/output paths
    pub fn new(input: InputSource, output: OutputSink) -> Self {
        Self {
            input,
            output,
            additional_inputs: Vec::new(),
        }
    }

    /// Create from optional command-line arguments
    pub fn from_args(input: Option<&str>, output: Option<&str>) -> Self {
        Self {
            input: InputSource::from_path(input),
            output: OutputSink::from_path(output),
            additional_inputs: Vec::new(),
        }
    }

    /// Create with multiple inputs
    pub fn with_inputs(mut self, inputs: Vec<InputSource>) -> Self {
        self.additional_inputs = inputs;
        self
    }

    /// Add an additional input source
    pub fn add_input(mut self, input: InputSource) -> Self {
        self.additional_inputs.push(input);
        self
    }

    /// Get the primary input source
    pub fn input(&self) -> &InputSource {
        &self.input
    }

    /// Get the output sink
    pub fn output(&self) -> &OutputSink {
        &self.output
    }

    /// Get additional input sources
    pub fn additional_inputs(&self) -> &[InputSource] {
        &self.additional_inputs
    }

    /// Open a reader for the primary input
    pub fn reader(&self) -> StructuredResult<Box<dyn Read>> {
        self.input.open()
    }

    /// Open a buffered reader for the primary input
    pub fn buffered_reader(&self) -> StructuredResult<Box<dyn BufRead>> {
        self.input.buffered()
    }

    /// Open a writer for the output
    pub fn writer(&self) -> StructuredResult<Box<dyn Write>> {
        self.output.open()
    }

    /// Open a buffered writer for the output
    pub fn buffered_writer(&self) -> StructuredResult<Box<dyn Write>> {
        self.output.buffered()
    }

    /// Read all input as a string
    pub fn read_to_string(&self) -> StructuredResult<String> {
        let mut reader = self.reader()?;
        let mut content = String::new();
        reader
            .read_to_string(&mut content)
            .map_err(|e| {
                StructuredError::new("read_error", format!("Failed to read input: {}", e))
                    .with_exit_code(ExitCodeClass::InputError)
            })?;
        Ok(content)
    }

    /// Read all input as bytes
    pub fn read_to_bytes(&self) -> StructuredResult<Vec<u8>> {
        let mut reader = self.reader()?;
        let mut content = Vec::new();
        reader
            .read_to_end(&mut content)
            .map_err(|e| {
                StructuredError::new("read_error", format!("Failed to read input: {}", e))
                    .with_exit_code(ExitCodeClass::InputError)
            })?;
        Ok(content)
    }

    /// Read input line by line
    pub fn read_lines(&self) -> StructuredResult<impl Iterator<Item = io::Result<String>>> {
        let reader = self.buffered_reader()?;
        Ok(reader.lines())
    }

    /// Write content to output
    pub fn write(&self, content: &[u8]) -> StructuredResult<()> {
        let mut writer = self.writer()?;
        writer
            .write_all(content)
            .map_err(|e| {
                StructuredError::new("write_error", format!("Failed to write output: {}", e))
                    .with_exit_code(ExitCodeClass::InputError)
            })?;
        Ok(())
    }

    /// Write a string to output
    pub fn write_str(&self, content: &str) -> StructuredResult<()> {
        self.write(content.as_bytes())
    }

    /// Copy input to output
    pub fn copy(&self) -> StructuredResult<u64> {
        let mut reader = self.reader()?;
        let mut writer = self.writer()?;

        io::copy(&mut reader, &mut writer)
            .map_err(|e| {
                StructuredError::new("io_error", format!("Failed to copy data: {}", e))
                    .with_exit_code(ExitCodeClass::InputError)
            })
    }
}

impl Default for FileIO {
    fn default() -> Self {
        Self {
            input: InputSource::Stdin,
            output: OutputSink::Stdout,
            additional_inputs: Vec::new(),
        }
    }
}

/// Builder for FileIO configuration
#[derive(Debug, Default)]
pub struct FileIOBuilder {
    input: Option<InputSource>,
    output: Option<OutputSink>,
    additional_inputs: Vec<InputSource>,
}

impl FileIOBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set input from path
    pub fn input(mut self, path: Option<&str>) -> Self {
        self.input = Some(InputSource::from_path(path));
        self
    }

    /// Set input source directly
    pub fn input_source(mut self, source: InputSource) -> Self {
        self.input = Some(source);
        self
    }

    /// Set output from path
    pub fn output(mut self, path: Option<&str>) -> Self {
        self.output = Some(OutputSink::from_path(path));
        self
    }

    /// Set output sink directly
    pub fn output_sink(mut self, sink: OutputSink) -> Self {
        self.output = Some(sink);
        self
    }

    /// Add an additional input
    pub fn add_input(mut self, source: InputSource) -> Self {
        self.additional_inputs.push(source);
        self
    }

    /// Build the FileIO
    pub fn build(self) -> FileIO {
        FileIO {
            input: self.input.unwrap_or_default(),
            output: self.output.unwrap_or_default(),
            additional_inputs: self.additional_inputs,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_source_from_path() {
        assert!(InputSource::from_path(None).is_stdin());
        assert!(InputSource::from_path(Some("-")).is_stdin());

        let file_input = InputSource::from_path(Some("test.txt"));
        assert!(!file_input.is_stdin());
        assert_eq!(file_input.path(), Some(Path::new("test.txt")));
    }

    #[test]
    fn test_output_sink_from_path() {
        assert!(OutputSink::from_path(None).is_stdout());
        assert!(OutputSink::from_path(Some("-")).is_stdout());

        let file_output = OutputSink::from_path(Some("output.txt"));
        assert!(!file_output.is_stdout());
        assert_eq!(file_output.path(), Some(Path::new("output.txt")));
    }

    #[test]
    fn test_file_io_from_args() {
        let io = FileIO::from_args(None, None);
        assert!(io.input().is_stdin());
        assert!(io.output().is_stdout());

        let io = FileIO::from_args(Some("input.txt"), Some("output.txt"));
        assert!(!io.input().is_stdin());
        assert!(!io.output().is_stdout());
    }

    #[test]
    fn test_file_io_builder() {
        let io = FileIOBuilder::new()
            .input(Some("input.txt"))
            .output(Some("output.txt"))
            .build();

        assert!(!io.input().is_stdin());
        assert!(!io.output().is_stdout());
    }

    #[test]
    fn test_multiple_inputs() {
        let io = FileIO::from_args(Some("main.txt"), None)
            .add_input(InputSource::from_path(Some("extra1.txt")))
            .add_input(InputSource::from_path(Some("extra2.txt")));

        assert_eq!(io.additional_inputs().len(), 2);
    }
}
