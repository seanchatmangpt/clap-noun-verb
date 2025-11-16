//! CNV Manpage Integration
//!
//! Thin wrapper around manpage generation using the grammar model.
//! Implementation uses clap_mangen but exposes a CNV-native interface.
//!
//! # Design
//!
//! - Single entry point for generating all manpages
//! - Derives from grammar model for consistency
//! - Pluggable backend (currently clap_mangen)
//!
//! # Example
//!
//! ```rust,ignore
//! use clap_noun_verb::kernel::ManpageGenerator;
//! use std::path::Path;
//!
//! let generator = ManpageGenerator::new("myapp", "1.0.0");
//! generator.generate_all(Path::new("./man"))?;
//! ```

use crate::kernel::grammar::{Grammar, GrammarModel};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

/// Manpage section
///
/// Standard Unix manual sections:
/// - 1: User commands
/// - 5: File formats
/// - 7: Miscellaneous
/// - 8: System administration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManSection {
    /// Section 1: User commands (default for CNV)
    UserCommands = 1,
    /// Section 5: File formats
    FileFormats = 5,
    /// Section 7: Miscellaneous
    Miscellaneous = 7,
    /// Section 8: System administration
    SystemAdmin = 8,
}

impl ManSection {
    /// Get the section number
    pub fn number(&self) -> u8 {
        *self as u8
    }
}

impl Default for ManSection {
    fn default() -> Self {
        Self::UserCommands
    }
}

/// Manpage generation configuration
#[derive(Debug, Clone)]
pub struct ManpageConfig {
    /// Manual section
    pub section: ManSection,
    /// Include version in manpage
    pub include_version: bool,
    /// Include examples section
    pub include_examples: bool,
    /// Include see also section
    pub include_see_also: bool,
    /// Author information
    pub author: Option<String>,
}

impl Default for ManpageConfig {
    fn default() -> Self {
        Self {
            section: ManSection::UserCommands,
            include_version: true,
            include_examples: false,
            include_see_also: true,
            author: None,
        }
    }
}

/// Manpage generator
///
/// Generates manpages for all commands in a CNV application
/// using the grammar model as the source of truth.
pub struct ManpageGenerator {
    app_name: String,
    version: Option<String>,
    config: ManpageConfig,
}

impl ManpageGenerator {
    /// Create a new manpage generator
    pub fn new(app_name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            app_name: app_name.into(),
            version: Some(version.into()),
            config: ManpageConfig::default(),
        }
    }

    /// Create without version
    pub fn without_version(app_name: impl Into<String>) -> Self {
        Self {
            app_name: app_name.into(),
            version: None,
            config: ManpageConfig::default(),
        }
    }

    /// Set configuration
    pub fn with_config(mut self, config: ManpageConfig) -> Self {
        self.config = config;
        self
    }

    /// Set author
    pub fn with_author(mut self, author: impl Into<String>) -> Self {
        self.config.author = Some(author.into());
        self
    }

    /// Generate all manpages to a directory
    ///
    /// Creates manpages for:
    /// - Main command (app.1)
    /// - Each noun (app-noun.1)
    /// - Each verb (app-noun-verb.1)
    pub fn generate_all(&self, output_dir: &Path) -> io::Result<Vec<PathBuf>> {
        // Create output directory if it doesn't exist
        fs::create_dir_all(output_dir)?;

        let mut generated = Vec::new();

        // Extract grammar
        let grammar = Grammar::extract_with_name(&self.app_name)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

        // Generate main manpage
        let main_path = self.generate_main(&grammar, output_dir)?;
        generated.push(main_path);

        // Generate noun and verb manpages
        for noun in grammar.nouns() {
            // Noun manpage
            let noun_path = self.generate_noun(&grammar, noun.name.as_str(), output_dir)?;
            generated.push(noun_path);

            // Verb manpages
            for verb in &noun.verbs {
                let verb_path = self.generate_verb(
                    &grammar,
                    noun.name.as_str(),
                    verb.name.as_str(),
                    output_dir,
                )?;
                generated.push(verb_path);
            }
        }

        Ok(generated)
    }

    /// Generate main command manpage
    fn generate_main(&self, grammar: &GrammarModel, output_dir: &Path) -> io::Result<PathBuf> {
        let filename = format!("{}.{}", self.app_name, self.config.section.number());
        let path = output_dir.join(&filename);

        let mut file = fs::File::create(&path)?;

        // Write manpage header
        writeln!(file, ".TH {} {} \"{}\" \"{}\" \"User Commands\"",
            self.app_name.to_uppercase(),
            self.config.section.number(),
            self.version.as_deref().unwrap_or(""),
            self.app_name
        )?;

        // NAME section
        writeln!(file, ".SH NAME")?;
        writeln!(file, "{} - noun-verb CLI application", self.app_name)?;

        // SYNOPSIS section
        writeln!(file, ".SH SYNOPSIS")?;
        writeln!(file, ".B {}", self.app_name)?;
        writeln!(file, "[GLOBAL OPTIONS] <NOUN> <VERB> [OPTIONS] [ARGS]")?;

        // DESCRIPTION section
        writeln!(file, ".SH DESCRIPTION")?;
        writeln!(file, "A noun-verb command-line interface built with clap-noun-verb.")?;

        // NOUNS section
        if !grammar.nouns.is_empty() {
            writeln!(file, ".SH NOUNS")?;
            for noun in &grammar.nouns {
                writeln!(file, ".TP")?;
                writeln!(file, ".B {}", noun.name)?;
                if let Some(help) = &noun.help {
                    writeln!(file, "{}", help)?;
                }
            }
        }

        // GLOBAL OPTIONS section
        if !grammar.global_arguments.is_empty() {
            writeln!(file, ".SH GLOBAL OPTIONS")?;
            for arg in &grammar.global_arguments {
                self.write_argument(&mut file, arg)?;
            }
        }

        // SEE ALSO section
        if self.config.include_see_also && !grammar.nouns.is_empty() {
            writeln!(file, ".SH SEE ALSO")?;
            for noun in &grammar.nouns {
                write!(file, ".BR {}-{} ({})", self.app_name, noun.name, self.config.section.number())?;
                if noun != grammar.nouns.last().unwrap() {
                    write!(file, ",")?;
                }
                writeln!(file)?;
            }
        }

        // AUTHOR section
        if let Some(author) = &self.config.author {
            writeln!(file, ".SH AUTHOR")?;
            writeln!(file, "{}", author)?;
        }

        Ok(path)
    }

    /// Generate noun manpage
    fn generate_noun(
        &self,
        grammar: &GrammarModel,
        noun: &str,
        output_dir: &Path,
    ) -> io::Result<PathBuf> {
        let filename = format!("{}-{}.{}", self.app_name, noun, self.config.section.number());
        let path = output_dir.join(&filename);

        let noun_data = grammar.find_noun(noun)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Noun not found"))?;

        let mut file = fs::File::create(&path)?;

        // Header
        writeln!(file, ".TH {}-{} {} \"{}\" \"{}\"",
            self.app_name.to_uppercase(),
            noun.to_uppercase(),
            self.config.section.number(),
            self.version.as_deref().unwrap_or(""),
            self.app_name
        )?;

        // NAME
        writeln!(file, ".SH NAME")?;
        writeln!(file, "{} {} - {}", self.app_name, noun,
            noun_data.help.as_deref().unwrap_or("noun commands"))?;

        // SYNOPSIS
        writeln!(file, ".SH SYNOPSIS")?;
        writeln!(file, ".B {} {}", self.app_name, noun)?;
        writeln!(file, "<VERB> [OPTIONS] [ARGS]")?;

        // VERBS
        if !noun_data.verbs.is_empty() {
            writeln!(file, ".SH VERBS")?;
            for verb in &noun_data.verbs {
                writeln!(file, ".TP")?;
                writeln!(file, ".B {}", verb.name)?;
                if let Some(help) = &verb.help {
                    writeln!(file, "{}", help)?;
                }
            }
        }

        // SEE ALSO
        if self.config.include_see_also {
            writeln!(file, ".SH SEE ALSO")?;
            for verb in &noun_data.verbs {
                write!(file, ".BR {}-{}-{} ({})",
                    self.app_name, noun, verb.name, self.config.section.number())?;
                if verb != noun_data.verbs.last().unwrap() {
                    write!(file, ",")?;
                }
                writeln!(file)?;
            }
        }

        Ok(path)
    }

    /// Generate verb manpage
    fn generate_verb(
        &self,
        grammar: &GrammarModel,
        noun: &str,
        verb: &str,
        output_dir: &Path,
    ) -> io::Result<PathBuf> {
        let filename = format!("{}-{}-{}.{}", self.app_name, noun, verb, self.config.section.number());
        let path = output_dir.join(&filename);

        let verb_data = grammar.find_verb(noun, verb)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Verb not found"))?;

        let mut file = fs::File::create(&path)?;

        // Header
        writeln!(file, ".TH {}-{}-{} {} \"{}\" \"{}\"",
            self.app_name.to_uppercase(),
            noun.to_uppercase(),
            verb.to_uppercase(),
            self.config.section.number(),
            self.version.as_deref().unwrap_or(""),
            self.app_name
        )?;

        // NAME
        writeln!(file, ".SH NAME")?;
        writeln!(file, "{} {} {} - {}", self.app_name, noun, verb,
            verb_data.help.as_deref().unwrap_or("verb command"))?;

        // SYNOPSIS
        writeln!(file, ".SH SYNOPSIS")?;
        writeln!(file, ".B {} {} {}", self.app_name, noun, verb)?;
        writeln!(file, "[OPTIONS] [ARGS]")?;

        // DESCRIPTION
        if let Some(long_help) = &verb_data.long_help {
            writeln!(file, ".SH DESCRIPTION")?;
            writeln!(file, "{}", long_help)?;
        }

        // OPTIONS
        if !verb_data.arguments.is_empty() {
            writeln!(file, ".SH OPTIONS")?;
            for arg in &verb_data.arguments {
                self.write_argument(&mut file, arg)?;
            }
        }

        Ok(path)
    }

    /// Write argument to manpage
    fn write_argument(
        &self,
        file: &mut fs::File,
        arg: &crate::kernel::grammar::GrammarArgument,
    ) -> io::Result<()> {
        writeln!(file, ".TP")?;

        // Flag line
        if let Some(short) = arg.short {
            write!(file, ".B \\-{}", short)?;
            if arg.long.is_some() {
                write!(file, ", ")?;
            }
        }

        if let Some(long) = &arg.long {
            write!(file, ".B \\-\\-{}", long)?;
        }

        // Value name
        if let Some(value_name) = &arg.value_name {
            write!(file, " <{}>", value_name)?;
        }

        writeln!(file)?;

        // Help text
        if let Some(help) = &arg.help {
            writeln!(file, "{}", help)?;
        }

        // Default value
        if let Some(default) = &arg.default {
            writeln!(file, "(default: {})", default)?;
        }

        Ok(())
    }

    /// Generate a single manpage to stdout
    pub fn generate_to_stdout(&self, command: &str) -> io::Result<()> {
        let temp_dir = std::env::temp_dir().join("cnv-manpages");
        fs::create_dir_all(&temp_dir)?;

        self.generate_all(&temp_dir)?;

        // Find and display the requested manpage
        let filename = format!("{}.{}", command, self.config.section.number());
        let path = temp_dir.join(&filename);

        if path.exists() {
            let content = fs::read_to_string(&path)?;
            print!("{}", content);
        } else {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Manpage for '{}' not found", command),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_man_section() {
        assert_eq!(ManSection::UserCommands.number(), 1);
        assert_eq!(ManSection::FileFormats.number(), 5);
    }

    #[test]
    fn test_manpage_generator_creation() {
        let gen = ManpageGenerator::new("test-app", "1.0.0");
        assert_eq!(gen.app_name, "test-app");
        assert_eq!(gen.version, Some("1.0.0".to_string()));
    }

    #[test]
    fn test_manpage_config() {
        let config = ManpageConfig::default();
        assert_eq!(config.section, ManSection::UserCommands);
        assert!(config.include_version);
    }
}
