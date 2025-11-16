//! CNV Telemetry Profile
//!
//! Provides a standardized, deterministic representation of "how this command is running"
//! to every verb handler. Replaces ad-hoc verbosity and color crates with a first-class
//! CNV concept.
//!
//! # Features
//!
//! - **Verbosity Levels**: 0-3 with deterministic -v/-q precedence
//! - **Color Policy**: auto/always/never with TTY detection
//! - **Output Format**: JSON/YAML/TOML/Table/TSV selection
//! - **Immutable Profile**: Single source of truth for telemetry state
//!
//! # Example
//!
//! ```rust,ignore
//! use clap_noun_verb::kernel::TelemetryProfile;
//!
//! fn my_verb(profile: &TelemetryProfile) -> Result<()> {
//!     if profile.verbosity() >= VerbosityLevel::Verbose {
//!         eprintln!("Verbose mode enabled");
//!     }
//!
//!     let output = do_work()?;
//!     profile.format_output(&output)
//! }
//! ```

use crate::format::OutputFormat;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::io::IsTerminal;

/// Verbosity level for command execution
///
/// Provides a finite, well-defined set of verbosity levels:
/// - **Silent**: No output except errors (quiet mode)
/// - **Normal**: Standard output (default)
/// - **Verbose**: Additional informational output (-v)
/// - **Debug**: Detailed debugging output (-vv)
/// - **Trace**: Maximum verbosity for troubleshooting (-vvv)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VerbosityLevel {
    /// Silent mode - only errors
    Silent = 0,
    /// Normal output (default)
    Normal = 1,
    /// Verbose output (-v)
    Verbose = 2,
    /// Debug output (-vv)
    Debug = 3,
    /// Trace output (-vvv)
    Trace = 4,
}

impl VerbosityLevel {
    /// Create from verbose and quiet counts with deterministic precedence
    ///
    /// # Precedence Rules
    ///
    /// 1. If quiet is set, verbosity is Silent (0)
    /// 2. Otherwise, verbosity = 1 + verbose_count (capped at 4)
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// VerbosityLevel::from_counts(0, false) // Normal
    /// VerbosityLevel::from_counts(1, false) // Verbose
    /// VerbosityLevel::from_counts(2, false) // Debug
    /// VerbosityLevel::from_counts(0, true)  // Silent
    /// VerbosityLevel::from_counts(5, false) // Trace (capped)
    /// ```
    pub fn from_counts(verbose_count: u8, quiet: bool) -> Self {
        if quiet {
            return Self::Silent;
        }

        match verbose_count {
            0 => Self::Normal,
            1 => Self::Verbose,
            2 => Self::Debug,
            _ => Self::Trace, // 3+ all map to Trace
        }
    }

    /// Get the numeric level (0-4)
    pub fn level(&self) -> u8 {
        *self as u8
    }

    /// Check if at least verbose (-v)
    pub fn is_verbose(&self) -> bool {
        *self >= Self::Verbose
    }

    /// Check if at least debug (-vv)
    pub fn is_debug(&self) -> bool {
        *self >= Self::Debug
    }

    /// Check if trace (-vvv)
    pub fn is_trace(&self) -> bool {
        *self >= Self::Trace
    }
}

impl Default for VerbosityLevel {
    fn default() -> Self {
        Self::Normal
    }
}

impl fmt::Display for VerbosityLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Silent => write!(f, "silent"),
            Self::Normal => write!(f, "normal"),
            Self::Verbose => write!(f, "verbose"),
            Self::Debug => write!(f, "debug"),
            Self::Trace => write!(f, "trace"),
        }
    }
}

/// Color output policy
///
/// Determines whether ANSI color codes should be emitted:
/// - **Auto**: Detect based on TTY and environment (NO_COLOR, CLICOLOR, etc.)
/// - **Always**: Force color output regardless of TTY
/// - **Never**: Disable all color output
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ColorPolicy {
    /// Auto-detect based on TTY and environment
    Auto,
    /// Always use color
    Always,
    /// Never use color
    Never,
}

impl ColorPolicy {
    /// Resolve the policy to a boolean decision
    ///
    /// Takes into account:
    /// - NO_COLOR environment variable (disables color)
    /// - CLICOLOR environment variable (0 = disable, 1 = enable)
    /// - CLICOLOR_FORCE environment variable (force enable)
    /// - TTY detection (no color if not a terminal)
    pub fn should_colorize(&self) -> bool {
        match self {
            Self::Always => true,
            Self::Never => false,
            Self::Auto => {
                // Check NO_COLOR first (takes precedence)
                if std::env::var("NO_COLOR").is_ok() {
                    return false;
                }

                // Check CLICOLOR_FORCE
                if let Ok(val) = std::env::var("CLICOLOR_FORCE") {
                    if val != "0" {
                        return true;
                    }
                }

                // Check CLICOLOR
                if let Ok(val) = std::env::var("CLICOLOR") {
                    if val == "0" {
                        return false;
                    }
                }

                // Default: use color if stdout is a TTY
                std::io::stdout().is_terminal()
            }
        }
    }
}

impl Default for ColorPolicy {
    fn default() -> Self {
        Self::Auto
    }
}

impl fmt::Display for ColorPolicy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Auto => write!(f, "auto"),
            Self::Always => write!(f, "always"),
            Self::Never => write!(f, "never"),
        }
    }
}

impl std::str::FromStr for ColorPolicy {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "auto" => Ok(Self::Auto),
            "always" | "yes" | "force" => Ok(Self::Always),
            "never" | "no" => Ok(Self::Never),
            _ => Err(format!(
                "Invalid color policy '{}'. Use: auto, always, never",
                s
            )),
        }
    }
}

/// Telemetry Profile - immutable snapshot of command telemetry state
///
/// This is the single source of truth for:
/// - Verbosity level (silent/normal/verbose/debug/trace)
/// - Color policy (auto/always/never)
/// - Output format (json/yaml/toml/table/tsv)
///
/// Every verb handler receives a `TelemetryProfile` that encodes these
/// cross-cutting concerns in a deterministic, predictable way.
///
/// # Design
///
/// The profile is constructed once at command invocation and passed
/// immutably to all handlers. This ensures:
///
/// - **Determinism**: Same inputs → same profile
/// - **Testability**: Easy to mock and verify
/// - **Thread-safety**: Immutable, can be shared
/// - **Zero overhead**: Small struct, passed by reference
///
/// # Example
///
/// ```rust,ignore
/// use clap_noun_verb::kernel::{TelemetryProfile, VerbosityLevel, ColorPolicy};
/// use clap_noun_verb::OutputFormat;
///
/// let profile = TelemetryProfile::new(
///     VerbosityLevel::Verbose,
///     ColorPolicy::Auto,
///     OutputFormat::Json,
/// );
///
/// if profile.is_verbose() {
///     eprintln!("Starting operation...");
/// }
///
/// let result = do_work();
/// println!("{}", profile.format_output(&result)?);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryProfile {
    /// Verbosity level
    verbosity: VerbosityLevel,
    /// Color output policy
    color: ColorPolicy,
    /// Output format
    format: OutputFormat,
    /// Resolved color decision (cached)
    #[serde(skip)]
    should_colorize: bool,
}

impl TelemetryProfile {
    /// Create a new telemetry profile
    pub fn new(verbosity: VerbosityLevel, color: ColorPolicy, format: OutputFormat) -> Self {
        let should_colorize = color.should_colorize();
        Self {
            verbosity,
            color,
            format,
            should_colorize,
        }
    }

    /// Create from command-line argument counts
    ///
    /// This is the primary constructor for CLI integration.
    ///
    /// # Arguments
    ///
    /// - `verbose_count`: Number of -v flags (0-3+)
    /// - `quiet`: Whether -q/--quiet was set
    /// - `color`: Color policy from --color flag
    /// - `format`: Output format from --format flag
    pub fn from_args(
        verbose_count: u8,
        quiet: bool,
        color: ColorPolicy,
        format: OutputFormat,
    ) -> Self {
        let verbosity = VerbosityLevel::from_counts(verbose_count, quiet);
        Self::new(verbosity, color, format)
    }

    /// Get the verbosity level
    pub fn verbosity(&self) -> VerbosityLevel {
        self.verbosity
    }

    /// Get the numeric verbosity level (0-4)
    pub fn verbosity_level(&self) -> u8 {
        self.verbosity.level()
    }

    /// Check if verbosity is at least verbose (-v)
    pub fn is_verbose(&self) -> bool {
        self.verbosity.is_verbose()
    }

    /// Check if verbosity is at least debug (-vv)
    pub fn is_debug(&self) -> bool {
        self.verbosity.is_debug()
    }

    /// Check if verbosity is trace (-vvv)
    pub fn is_trace(&self) -> bool {
        self.verbosity.is_trace()
    }

    /// Check if in quiet mode
    pub fn is_quiet(&self) -> bool {
        self.verbosity == VerbosityLevel::Silent
    }

    /// Get the color policy
    pub fn color_policy(&self) -> ColorPolicy {
        self.color
    }

    /// Check if output should be colorized
    ///
    /// This is a cached, resolved decision based on the color policy,
    /// environment variables, and TTY detection.
    pub fn should_colorize(&self) -> bool {
        self.should_colorize
    }

    /// Get the output format
    pub fn format(&self) -> OutputFormat {
        self.format
    }

    /// Format a value using the configured output format
    ///
    /// This is a convenience method for verb handlers to format
    /// their output according to the telemetry profile.
    pub fn format_output<S: Serialize>(
        &self,
        value: &S,
    ) -> Result<String, Box<dyn std::error::Error>> {
        self.format.format(value)
    }

    /// Create a builder for constructing profiles
    pub fn builder() -> TelemetryProfileBuilder {
        TelemetryProfileBuilder::default()
    }
}

impl Default for TelemetryProfile {
    fn default() -> Self {
        Self::new(
            VerbosityLevel::Normal,
            ColorPolicy::Auto,
            OutputFormat::Json,
        )
    }
}

/// Builder for constructing telemetry profiles
///
/// Provides a fluent API for building profiles with optional overrides.
#[derive(Debug, Default)]
pub struct TelemetryProfileBuilder {
    verbosity: Option<VerbosityLevel>,
    verbose_count: Option<u8>,
    quiet: bool,
    color: Option<ColorPolicy>,
    format: Option<OutputFormat>,
}

impl TelemetryProfileBuilder {
    /// Set the verbosity level directly
    pub fn verbosity(mut self, level: VerbosityLevel) -> Self {
        self.verbosity = Some(level);
        self
    }

    /// Set the verbose count (-v, -vv, -vvv)
    pub fn verbose_count(mut self, count: u8) -> Self {
        self.verbose_count = Some(count);
        self
    }

    /// Enable quiet mode
    pub fn quiet(mut self) -> Self {
        self.quiet = true;
        self
    }

    /// Set the color policy
    pub fn color(mut self, policy: ColorPolicy) -> Self {
        self.color = Some(policy);
        self
    }

    /// Set the output format
    pub fn format(mut self, format: OutputFormat) -> Self {
        self.format = Some(format);
        self
    }

    /// Build the telemetry profile
    pub fn build(self) -> TelemetryProfile {
        let verbosity = if let Some(v) = self.verbosity {
            v
        } else if let Some(count) = self.verbose_count {
            VerbosityLevel::from_counts(count, self.quiet)
        } else if self.quiet {
            VerbosityLevel::Silent
        } else {
            VerbosityLevel::Normal
        };

        TelemetryProfile::new(
            verbosity,
            self.color.unwrap_or_default(),
            self.format.unwrap_or_default(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verbosity_levels() {
        assert_eq!(VerbosityLevel::Silent.level(), 0);
        assert_eq!(VerbosityLevel::Normal.level(), 1);
        assert_eq!(VerbosityLevel::Verbose.level(), 2);
        assert_eq!(VerbosityLevel::Debug.level(), 3);
        assert_eq!(VerbosityLevel::Trace.level(), 4);
    }

    #[test]
    fn test_verbosity_from_counts() {
        assert_eq!(
            VerbosityLevel::from_counts(0, false),
            VerbosityLevel::Normal
        );
        assert_eq!(
            VerbosityLevel::from_counts(1, false),
            VerbosityLevel::Verbose
        );
        assert_eq!(
            VerbosityLevel::from_counts(2, false),
            VerbosityLevel::Debug
        );
        assert_eq!(
            VerbosityLevel::from_counts(3, false),
            VerbosityLevel::Trace
        );
        assert_eq!(
            VerbosityLevel::from_counts(10, false),
            VerbosityLevel::Trace
        ); // capped
        assert_eq!(
            VerbosityLevel::from_counts(5, true),
            VerbosityLevel::Silent
        ); // quiet wins
    }

    #[test]
    fn test_verbosity_checks() {
        assert!(!VerbosityLevel::Normal.is_verbose());
        assert!(VerbosityLevel::Verbose.is_verbose());
        assert!(VerbosityLevel::Debug.is_debug());
        assert!(VerbosityLevel::Trace.is_trace());
    }

    #[test]
    fn test_color_policy() {
        assert_eq!("auto".parse::<ColorPolicy>().ok(), Some(ColorPolicy::Auto));
        assert_eq!(
            "always".parse::<ColorPolicy>().ok(),
            Some(ColorPolicy::Always)
        );
        assert_eq!(
            "never".parse::<ColorPolicy>().ok(),
            Some(ColorPolicy::Never)
        );
        assert!("invalid".parse::<ColorPolicy>().is_err());
    }

    #[test]
    fn test_telemetry_profile_default() {
        let profile = TelemetryProfile::default();
        assert_eq!(profile.verbosity(), VerbosityLevel::Normal);
        assert_eq!(profile.color_policy(), ColorPolicy::Auto);
        assert_eq!(profile.format(), OutputFormat::Json);
    }

    #[test]
    fn test_telemetry_profile_from_args() {
        let profile = TelemetryProfile::from_args(2, false, ColorPolicy::Always, OutputFormat::Yaml);
        assert_eq!(profile.verbosity(), VerbosityLevel::Debug);
        assert_eq!(profile.color_policy(), ColorPolicy::Always);
        assert_eq!(profile.format(), OutputFormat::Yaml);
        assert!(profile.should_colorize());
    }

    #[test]
    fn test_telemetry_profile_builder() {
        let profile = TelemetryProfile::builder()
            .verbose_count(1)
            .color(ColorPolicy::Never)
            .format(OutputFormat::Table)
            .build();

        assert_eq!(profile.verbosity(), VerbosityLevel::Verbose);
        assert_eq!(profile.color_policy(), ColorPolicy::Never);
        assert_eq!(profile.format(), OutputFormat::Table);
        assert!(!profile.should_colorize());
    }

    #[test]
    fn test_quiet_mode() {
        let profile = TelemetryProfile::from_args(0, true, ColorPolicy::Auto, OutputFormat::Json);
        assert!(profile.is_quiet());
        assert_eq!(profile.verbosity(), VerbosityLevel::Silent);
    }
}
