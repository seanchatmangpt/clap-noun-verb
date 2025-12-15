//! Compile-time command registry for attribute macros
//!
//! This module provides a registry that collects functions marked with
//! `#[verb]` and `#[noun]` attributes at compile time using linkme.
//!
//! These attribute macros are provided by the `clap-noun-verb-macros` crate.
//!
//! # Memory Management: Box::leak for Static Strings
//!
//! This module uses `Box::leak()` extensively to convert owned Strings into
//! `&'static str` references required by the clap library for command metadata.
//!
//! ## Why Box::leak is Used
//!
//! The clap command builder API requires all command names, help text, argument
//! names, etc. as `&'static str` (references with static lifetime). Converting
//! dynamic runtime strings (from metadata, configuration, etc.) to static
//! references requires "leaking" the memory so it persists for the program's
//! entire duration:
//!
//! ```rust,ignore
//! // What we need to do (16+ instances in this file):
//! let noun_name: &'static str = Box::leak(noun_name.to_string().into_boxed_str());
//! ```
//!
//! ## Memory Impact Assessment
//!
//! This pattern is **acceptable for CLI applications** because:
//!
//! - **Minimal total allocation**: Typical CLI has <100 commands → <50KB total leaked memory
//! - **One-time cost**: Leaks occur only during initialization (not in hot loops)
//! - **Unavoidable for clap integration**: There's no alternative that maintains
//!   clap's ergonomic API while supporting dynamic command discovery
//! - **Negligible impact**: CLI memory usage dominated by other factors
//!   (parsing, runtime state, etc.), not metadata strings
//!
//! ## Alternative Approaches (Not Used)
//!
//! For reference, other approaches we considered:
//!
//! 1. **once_cell/lazy_static**: Would require refactoring the entire command
//!    builder architecture and runtime registration system. Adds significant
//!    complexity with minimal benefit for CLI use case.
//!
//! 2. **Custom 'static lifetime manager**: Would require unsafe code and careful
//!    lifetime tracking. Not worth the complexity for CLI applications.
//!
//! 3. **Dynamic clap Commands**: Rebuild command structure dynamically (no static
//!    strings). Possible but defeats clap's performance benefits and requires
//!    restructuring around clap's static assumptions.
//!
//! For library use cases or long-running services, alternatives should be
//! investigated. For typical CLI applications, Box::leak is the idiomatic solution.

use crate::cli::value_parser;
use crate::error::Result;
use crate::logic::{HandlerInput, HandlerOutput};
use linkme::distributed_slice;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

/// Apply validation constraints and auto-inferred parsers to a clap Arg
///
/// This function applies min/max value and length validators based on the
/// validation metadata stored in ArgMetadata, as well as auto-inferred
/// value parsers for common types.
///
/// For explicit value_parser expressions, it uses pattern matching on
/// the string representation to apply common patterns.
/// Parse a value hint string to clap::ValueHint
///
/// Supported hints (case-insensitive):
/// - file_path, filepath, file - File path
/// - dir_path, dirpath, dir, directory - Directory path
/// - any_path, anypath, path - Any path
/// - executable, exe - Executable command
/// - command, cmd, cmdname - Command name
/// - username, user - Username
/// - hostname, host - Hostname
/// - url - URL
/// - email - Email address
/// - other - Other
fn parse_value_hint(hint: &str) -> clap::ValueHint {
    match hint.to_lowercase().as_str() {
        "file_path" | "filepath" | "file" => clap::ValueHint::FilePath,
        "dir_path" | "dirpath" | "dir" | "directory" => clap::ValueHint::DirPath,
        "any_path" | "anypath" | "path" => clap::ValueHint::AnyPath,
        "executable" | "exe" => clap::ValueHint::ExecutablePath,
        "command" | "cmd" | "cmdname" | "command_name" => clap::ValueHint::CommandName,
        "commandstring" | "command_string" | "cmdstring" => clap::ValueHint::CommandString,
        "commandwitharguments" | "commandwithargs" => clap::ValueHint::CommandWithArguments,
        "username" | "user" => clap::ValueHint::Username,
        "hostname" | "host" => clap::ValueHint::Hostname,
        "url" => clap::ValueHint::Url,
        "email" | "emailaddress" => clap::ValueHint::EmailAddress,
        _ => clap::ValueHint::Other,
    }
}

///
/// Takes ownership of Arg and returns the modified Arg to avoid unnecessary
/// cloning while applying builder-pattern methods that consume and return self.
fn apply_validators(mut arg: clap::Arg, arg_meta: &ArgMetadata) -> clap::Arg {
    // Apply value parser if specified (auto-inferred or explicit)
    // Note: value_parser is stored as a string representation, so we match on the string
    if let Some(ref vp_str) = arg_meta.value_parser {
        // Try to apply value parser from pattern matching
        if value_parser::apply_value_parser(&mut arg, vp_str) {
            return arg;
        }
    }

    // For numeric types with min/max values, apply range validators
    if arg_meta.min_value.is_some() || arg_meta.max_value.is_some() {
        // Try to parse as integers first
        let min_i64 = arg_meta.min_value.as_ref().and_then(|v| v.parse::<i64>().ok());
        let max_i64 = arg_meta.max_value.as_ref().and_then(|v| v.parse::<i64>().ok());
        let min_u64 = arg_meta.min_value.as_ref().and_then(|v| v.parse::<u64>().ok());
        let max_u64 = arg_meta.max_value.as_ref().and_then(|v| v.parse::<u64>().ok());

        // Apply range validators based on what we can parse using match for clarity
        match (min_i64, max_i64, min_u64, max_u64) {
            (Some(min), Some(max), _, _) => {
                arg = arg.value_parser(clap::value_parser!(i64).range(min..=max));
            }
            (Some(min), None, _, _) => {
                arg = arg.value_parser(clap::value_parser!(i64).range(min..));
            }
            (None, Some(max), _, _) => {
                arg = arg.value_parser(clap::value_parser!(i64).range(..=max));
            }
            (_, _, Some(min), Some(max)) => {
                arg = arg.value_parser(clap::value_parser!(u64).range(min..=max));
            }
            (_, _, Some(min), None) => {
                arg = arg.value_parser(clap::value_parser!(u64).range(min..));
            }
            (_, _, None, Some(max)) => {
                arg = arg.value_parser(clap::value_parser!(u64).range(..=max));
            }
            _ => {}
        }
    }

    // For string types with min_length, ensure non-empty
    if let Some(min_len) = arg_meta.min_length {
        if min_len > 0 {
            arg = arg.value_parser(clap::builder::NonEmptyStringValueParser::new());
        }
    }

    arg
}

/// Distributed slice for noun registrations
#[distributed_slice]
pub static __NOUN_REGISTRY: [fn()] = [..];

/// Distributed slice for verb registrations
#[distributed_slice]
pub static __VERB_REGISTRY: [fn()] = [..];

/// Global registry for registered commands
static REGISTRY: OnceLock<Mutex<CommandRegistry>> = OnceLock::new();

/// Command registry for attribute macro discovered functions
pub struct CommandRegistry {
    /// Registered nouns (name -> noun metadata)
    nouns: HashMap<String, NounMetadata>,
    /// Registered verbs (noun_name -> verb_name -> verb metadata)
    verbs: HashMap<String, HashMap<String, VerbMetadata>>,
    /// Root-level verbs (verb_name -> verb metadata) - verbs without a noun
    root_verbs: HashMap<String, VerbMetadata>,
}

/// Metadata for a registered noun
struct NounMetadata {
    #[allow(dead_code)] // Reserved for future use
    name: String,
    about: String,
    long_about: Option<String>,
}

/// Argument metadata for a verb function parameter
#[derive(Clone)]
pub struct ArgMetadata {
    pub name: String,
    pub required: bool,
    pub is_flag: bool,
    pub help: Option<String>,
    pub min_value: Option<String>,
    pub max_value: Option<String>,
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    /// Short flag character (e.g., 'v' for -v)
    pub short: Option<char>,
    /// Default value as string (will be parsed by clap)
    pub default_value: Option<String>,
    /// Environment variable name
    pub env: Option<String>,
    /// Whether this argument accepts multiple values
    pub multiple: bool,
    /// Custom value name for help text (e.g., "FILE", "PORT")
    pub value_name: Option<String>,
    /// Aliases for the argument (e.g., ["verbose", "v"])
    pub aliases: Vec<String>,
    /// Positional argument index (e.g., 0, 1, 2)
    pub positional: Option<usize>,
    /// Custom action type (e.g., Count, SetFalse)
    pub action: Option<clap::ArgAction>,
    /// Argument group name (for exclusive/multiple groups)
    pub group: Option<String>,
    /// Arguments this requires
    pub requires: Vec<String>,
    /// Arguments this conflicts with
    pub conflicts_with: Vec<String>,
    /// Custom value parser expression (stored as string for macro expansion)
    pub value_parser: Option<String>,
    /// Hide from help text
    pub hide: bool,
    /// Next help heading (for grouping in help)
    pub next_help_heading: Option<String>,
    /// Long help text (separate from help)
    pub long_help: Option<String>,
    /// Next line help formatting
    pub next_line_help: bool,
    /// Display order in help output
    pub display_order: Option<usize>,
    /// Exclusive group flag
    pub exclusive: Option<bool>,
    /// Trailing varargs support
    pub trailing_vararg: bool,
    /// Allow negative numbers
    pub allow_negative_numbers: bool,
    /// Value hint for shell completion (e.g., FilePath, DirPath, Url)
    pub value_hint: Option<String>,
    /// Global flag - propagates to subcommands
    pub global: bool,
}

/// Metadata for a registered verb
struct VerbMetadata {
    #[allow(dead_code)] // Reserved for future use
    noun_name: String,
    #[allow(dead_code)] // Reserved for future use
    verb_name: String,
    about: String,
    args: Vec<ArgMetadata>,
    handler_fn: Box<dyn Fn(HandlerInput) -> Result<HandlerOutput> + Send + Sync>,
}

impl CommandRegistry {
    /// Initialize the registry (called once during first access)
    pub fn init() -> &'static Mutex<CommandRegistry> {
        // Use get_or_init to create and store the registry
        // During initialization, we'll run registration functions
        // which will call register_noun/register_verb_with_args
        // These will call REGISTRY.get() which will return None
        // until initialization completes, so we need a different approach
        let registry = REGISTRY.get_or_init(|| {
            // Create empty registry
            Mutex::new(CommandRegistry {
                nouns: HashMap::new(),
                verbs: HashMap::new(),
                root_verbs: HashMap::new(),
            })
        });

        // After registry is stored, run registration functions
        // Now REGISTRY.get() will return the stored value
        for init_fn in __NOUN_REGISTRY {
            init_fn();
        }
        for init_fn in __VERB_REGISTRY {
            init_fn();
        }

        // Return the stored registry (get_or_init guarantees it's Some)
        registry
    }

    /// Get the global registry instance
    pub fn get() -> &'static Mutex<CommandRegistry> {
        Self::init()
    }

    /// Register a noun (called by macro-generated code)
    pub fn register_noun(name: &'static str, about: &'static str) {
        // Get the registry - this will initialize it if needed
        // During initialization, this will wait until init() completes
        let registry = REGISTRY.get_or_init(|| {
            Mutex::new(CommandRegistry {
                nouns: HashMap::new(),
                verbs: HashMap::new(),
                root_verbs: HashMap::new(),
            })
        });
        // Lock poisoning should not happen in practice, but handle it gracefully
        let mut reg = registry.lock().unwrap_or_else(|e| e.into_inner());
        reg.nouns.insert(
            name.to_string(),
            NounMetadata { name: name.to_string(), about: about.to_string(), long_about: None },
        );
    }

    /// Register a verb (called by macro-generated code)
    pub fn register_verb<F>(
        noun_name: &'static str,
        verb_name: &'static str,
        about: &'static str,
        handler: F,
    ) where
        F: Fn(HandlerInput) -> Result<HandlerOutput> + Send + Sync + 'static,
    {
        Self::register_verb_with_args(noun_name, verb_name, about, Vec::new(), handler)
    }

    /// Register a verb with argument metadata
    ///
    /// If noun_name is empty (""), the verb is registered as a root-level verb
    /// that appears directly under the CLI binary (e.g., `ggen sync` instead of `ggen noun sync`)
    pub fn register_verb_with_args<F>(
        noun_name: &'static str,
        verb_name: &'static str,
        about: &'static str,
        args: Vec<ArgMetadata>,
        handler: F,
    ) where
        F: Fn(HandlerInput) -> Result<HandlerOutput> + Send + Sync + 'static,
    {
        // Get the registry - this will initialize it if needed
        // During initialization, this will wait until init() completes
        let registry = REGISTRY.get_or_init(|| {
            Mutex::new(CommandRegistry {
                nouns: HashMap::new(),
                verbs: HashMap::new(),
                root_verbs: HashMap::new(),
            })
        });
        // Lock poisoning should not happen in practice, but handle it gracefully
        let mut reg = registry.lock().unwrap_or_else(|e| e.into_inner());

        let verb_metadata = VerbMetadata {
            noun_name: noun_name.to_string(),
            verb_name: verb_name.to_string(),
            about: about.to_string(),
            args,
            handler_fn: Box::new(handler),
        };

        // If noun_name is empty, register as root verb
        if noun_name.is_empty() {
            reg.root_verbs.insert(verb_name.to_string(), verb_metadata);
        } else {
            reg.verbs
                .entry(noun_name.to_string())
                .or_default()
                .insert(verb_name.to_string(), verb_metadata);
        }
    }

    /// Get all registered nouns
    pub fn get_nouns(&self) -> Vec<(&str, &str)> {
        self.nouns.iter().map(|(name, meta)| (name.as_str(), meta.about.as_str())).collect()
    }

    /// Get all verbs for a noun
    pub fn get_verbs(&self, noun_name: &str) -> Vec<(&str, &str)> {
        self.verbs
            .get(noun_name)
            .map(|verbs| {
                verbs.iter().map(|(name, meta)| (name.as_str(), meta.about.as_str())).collect()
            })
            .unwrap_or_default()
    }

    /// Get all verbs for a noun with their full metadata including arguments
    pub fn get_verbs_with_metadata(&self, noun_name: &str) -> Vec<(&str, &str, &Vec<ArgMetadata>)> {
        self.verbs
            .get(noun_name)
            .map(|verbs| {
                verbs
                    .iter()
                    .map(|(name, meta)| (name.as_str(), meta.about.as_str(), &meta.args))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all registered noun names
    pub fn get_all_noun_names(&self) -> Vec<&str> {
        self.nouns.keys().map(|n| n.as_str()).collect()
    }

    /// Execute a verb handler
    pub fn execute_verb(
        &self,
        noun_name: &str,
        verb_name: &str,
        input: HandlerInput,
    ) -> Result<HandlerOutput> {
        let verbs = self
            .verbs
            .get(noun_name)
            .ok_or_else(|| crate::error::NounVerbError::command_not_found(noun_name))?;

        let verb = verbs
            .get(verb_name)
            .ok_or_else(|| crate::error::NounVerbError::verb_not_found(noun_name, verb_name))?;

        (verb.handler_fn)(input)
    }

    /// Build clap command structure from registry
    pub fn build_command(&self) -> clap::Command {
        let mut cmd = clap::Command::new("cli")
            .version(env!("CARGO_PKG_VERSION"))
            .arg_required_else_help(true);

        // Add root-level verbs directly as subcommands
        for (verb_name, verb_meta) in &self.root_verbs {
            let verb_cmd = self.build_verb_command(verb_name, verb_meta);
            cmd = cmd.subcommand(verb_cmd);
        }

        // Add nouns with their nested verbs
        for (noun_name, noun_meta) in &self.nouns {
            let noun_cmd = self.build_noun_command(noun_name, noun_meta);
            cmd = cmd.subcommand(noun_cmd);
        }

        cmd
    }

    /// Build a noun command with all its verb subcommands
    ///
    /// Note: Uses Box::leak to convert owned strings to &'static str required by clap.
    /// This is acceptable for CLI apps - see module documentation for details.
    fn build_noun_command(&self, noun_name: &str, noun_meta: &NounMetadata) -> clap::Command {
        // Box::leak: Converts dynamic String to &'static str for clap's Command::new()
        // This is necessary because clap requires static lifetimes for performance
        let noun_name_static: &'static str = Box::leak(noun_name.to_string().into_boxed_str());
        let about: &'static str = Box::leak(noun_meta.about.clone().into_boxed_str());
        let mut noun_cmd = clap::Command::new(noun_name_static).about(about);

        // Apply long_about if available
        if let Some(ref long_about) = noun_meta.long_about {
            let long_about_static: &'static str = Box::leak(long_about.clone().into_boxed_str());
            noun_cmd = noun_cmd.long_about(long_about_static);
        }

        // Add verbs as subcommands
        if let Some(verbs) = self.verbs.get(noun_name) {
            for (verb_name, verb_meta) in verbs {
                let verb_cmd = self.build_verb_command(verb_name, verb_meta);
                noun_cmd = noun_cmd.subcommand(verb_cmd);
            }
        }

        noun_cmd
    }

    /// Build a verb command with all its arguments
    ///
    /// Note: Uses Box::leak to convert owned strings to &'static str required by clap.
    /// This is acceptable for CLI apps - see module documentation for details.
    fn build_verb_command(&self, verb_name: &str, verb_meta: &VerbMetadata) -> clap::Command {
        // Box::leak: Converts dynamic String to &'static str for clap's Command::new()
        let verb_name_static: &'static str = Box::leak(verb_name.to_string().into_boxed_str());
        let about: &'static str = Box::leak(verb_meta.about.clone().into_boxed_str());
        let mut verb_cmd = clap::Command::new(verb_name_static).about(about);

        // Add argument groups and arguments
        verb_cmd = self.add_arg_groups(verb_cmd, verb_meta);
        verb_cmd = self.add_arguments(verb_cmd, verb_meta);

        verb_cmd
    }

    /// Add argument groups to a command
    fn add_arg_groups(
        &self,
        mut verb_cmd: clap::Command,
        verb_meta: &VerbMetadata,
    ) -> clap::Command {
        // Collect argument groups first with exclusivity info
        let mut groups: std::collections::HashMap<String, (bool, Vec<String>)> =
            std::collections::HashMap::new();
        for arg_meta in &verb_meta.args {
            if let Some(ref group_name) = arg_meta.group {
                let exclusive = arg_meta.exclusive.unwrap_or(true); // Default to exclusive
                let entry =
                    groups.entry(group_name.clone()).or_insert_with(|| (exclusive, Vec::new()));
                entry.1.push(arg_meta.name.clone());
                // If any arg in group is exclusive, mark group as exclusive
                if !exclusive {
                    entry.0 = false;
                }
            }
        }

        // Create ArgGroup for each group with proper exclusivity
        for (group_name, (exclusive, arg_names)) in &groups {
            if arg_names.len() > 1 {
                let group_static: &'static str = Box::leak(group_name.clone().into_boxed_str());
                let group = clap::ArgGroup::new(group_static).multiple(!exclusive);
                let mut group = group;
                for arg_name in arg_names {
                    let arg_name_static: &'static str =
                        Box::leak(arg_name.clone().into_boxed_str());
                    group = group.arg(arg_name_static);
                }
                verb_cmd = verb_cmd.group(group);
            }
        }

        verb_cmd
    }

    /// Add arguments to a command
    fn add_arguments(
        &self,
        mut verb_cmd: clap::Command,
        verb_meta: &VerbMetadata,
    ) -> clap::Command {
        for arg_meta in &verb_meta.args {
            let arg = self.build_argument(arg_meta);
            verb_cmd = verb_cmd.arg(arg);
        }
        verb_cmd
    }

    /// Build a single argument
    ///
    /// Note: Uses multiple Box::leak calls to convert argument metadata strings to
    /// &'static str references required by clap's builder API. This is the expected
    /// pattern for dynamic argument configuration in CLI applications.
    fn build_argument(&self, arg_meta: &ArgMetadata) -> clap::Arg {
        // Box::leak: Convert argument name to static string for clap::Arg::new()
        let arg_name: &'static str = Box::leak(arg_meta.name.clone().into_boxed_str());
        // Generate uppercase variant for value_name display (e.g., "FILE", "PORT")
        let default_value_name: &'static str =
            Box::leak(arg_meta.name.to_uppercase().into_boxed_str());

        // Create argument - positional args use index(), others use long()
        let mut arg = if let Some(index) = arg_meta.positional {
            let mut pos_arg = clap::Arg::new(arg_name).index(index);
            if arg_meta.trailing_vararg {
                pos_arg = pos_arg.num_args(1..);
            }
            pos_arg
        } else {
            clap::Arg::new(arg_name).long(arg_name)
        };

        // Apply short flag if specified (only for non-positional args)
        if arg_meta.positional.is_none() {
            if let Some(short_char) = arg_meta.short {
                arg = arg.short(short_char);
            }

            // Apply aliases if specified (only for non-positional args)
            for alias in &arg_meta.aliases {
                let alias_static: &'static str = Box::leak(alias.clone().into_boxed_str());
                arg = arg.alias(alias_static);
            }
        }

        // Apply environment variable if specified
        if let Some(ref env_var) = arg_meta.env {
            let env_static: &'static str = Box::leak(env_var.clone().into_boxed_str());
            arg = arg.env(env_static);
        }

        // Apply default value if specified
        if let Some(ref default_val) = arg_meta.default_value {
            let default_static: &'static str = Box::leak(default_val.clone().into_boxed_str());
            arg = arg.default_value(default_static);
        }

        // Apply custom action if specified, otherwise use defaults
        if let Some(action) = &arg_meta.action {
            arg = arg.action(action.clone());
        } else if arg_meta.is_flag {
            arg = arg.action(clap::ArgAction::SetTrue);
        } else {
            let value_name: &'static str = if let Some(ref vn) = arg_meta.value_name {
                Box::leak(vn.clone().into_boxed_str())
            } else {
                default_value_name
            };
            arg = arg.value_name(value_name);

            if arg_meta.multiple {
                arg = arg.action(clap::ArgAction::Append);
            }

            if arg_meta.required {
                arg = arg.required(true);
            }

            arg = apply_validators(arg, arg_meta);

            if arg_meta.allow_negative_numbers {
                arg = arg.allow_negative_numbers(true);
            }
        }

        // Apply help text
        if let Some(help_text) = &arg_meta.help {
            let help: &'static str = Box::leak(help_text.clone().into_boxed_str());
            arg = arg.help(help);
        }

        // Apply long_help if specified
        if let Some(long_help_text) = &arg_meta.long_help {
            let long_help: &'static str = Box::leak(long_help_text.clone().into_boxed_str());
            arg = arg.long_help(long_help);
        }

        // Apply next_line_help if specified
        if arg_meta.next_line_help {
            arg = arg.next_line_help(true);
        }

        // Apply display_order if specified
        if let Some(order) = arg_meta.display_order {
            arg = arg.display_order(order);
        }

        // Apply requires
        for req in &arg_meta.requires {
            let req_static: &'static str = Box::leak(req.clone().into_boxed_str());
            arg = arg.requires(req_static);
        }

        // Apply conflicts_with
        for conflict in &arg_meta.conflicts_with {
            let conflict_static: &'static str = Box::leak(conflict.clone().into_boxed_str());
            arg = arg.conflicts_with(conflict_static);
        }

        // Apply group membership
        if let Some(ref group_name) = arg_meta.group {
            let group_static: &'static str = Box::leak(group_name.clone().into_boxed_str());
            arg = arg.group(group_static);
        }

        // Apply hide if specified
        if arg_meta.hide {
            arg = arg.hide(true);
        }

        // Apply help_heading if specified
        if let Some(ref heading) = arg_meta.next_help_heading {
            let heading_static: &'static str = Box::leak(heading.clone().into_boxed_str());
            arg = arg.help_heading(heading_static);
        }

        // Apply value_hint for shell completion
        if let Some(ref hint) = arg_meta.value_hint {
            arg = arg.value_hint(parse_value_hint(hint));
        }

        // Apply global flag - propagates to subcommands
        if arg_meta.global {
            arg = arg.global(true);
        }

        // Apply exclusive if specified (cannot be used with any other args)
        if let Some(true) = arg_meta.exclusive {
            arg = arg.exclusive(true);
        }

        arg
    }

    /// Extract a value from ArgMatches as a string
    ///
    /// Uses get_raw() to get the original CLI string value, avoiding type mismatch
    /// panics that occur when value_parser stores values as numeric types.
    fn extract_value_as_string(verb_matches: &clap::ArgMatches, arg_name: &str) -> Option<String> {
        // Use get_raw() to get the original string value from CLI
        // This works regardless of what value_parser was used
        if let Some(raw_values) = verb_matches.get_raw(arg_name) {
            // get_raw returns an iterator of OsStrings, take the first one
            if let Some(os_str) = raw_values.into_iter().next() {
                return os_str.to_str().map(|s| s.to_string());
            }
        }
        None
    }

    /// Extract arguments from clap matches into a HashMap
    fn extract_args(
        &self,
        verb_meta: &VerbMetadata,
        verb_matches: &clap::ArgMatches,
    ) -> std::collections::HashMap<String, String> {
        let mut args_map = std::collections::HashMap::new();

        for arg_meta in &verb_meta.args {
            let arg_name = &arg_meta.name;

            // Handle positional arguments differently
            if let Some(_index) = arg_meta.positional {
                // For positional args, clap extracts by name automatically
                if let Some(value) = Self::extract_value_as_string(verb_matches, arg_name) {
                    args_map.insert(arg_name.clone(), value);
                }
            } else if let Some(action) = &arg_meta.action {
                // Handle custom actions
                match action {
                    clap::ArgAction::Count => {
                        let count = verb_matches.get_count(arg_name);
                        args_map.insert(arg_name.clone(), count.to_string());
                    }
                    clap::ArgAction::SetTrue => {
                        if verb_matches.get_flag(arg_name) {
                            args_map.insert(arg_name.clone(), "true".to_string());
                        }
                    }
                    clap::ArgAction::SetFalse => {
                        // SetFalse is handled differently - need to check if present
                        // Note: clap doesn't have get_flag for SetFalse, so we check presence
                        if verb_matches.contains_id(arg_name) {
                            args_map.insert(arg_name.clone(), "false".to_string());
                        }
                    }
                    clap::ArgAction::Append => {
                        // Append collects multiple values
                        if let Some(values) = verb_matches.get_many::<String>(arg_name) {
                            let values_vec: Vec<String> = values.cloned().collect();
                            args_map.insert(arg_name.clone(), values_vec.join(","));
                        }
                    }
                    _ => {
                        // For Set and other actions, try type-aware extraction
                        if let Some(value) = Self::extract_value_as_string(verb_matches, arg_name) {
                            args_map.insert(arg_name.clone(), value);
                        }
                    }
                }
            } else if arg_meta.is_flag {
                // For flags, check if they're set
                if verb_matches.get_flag(arg_name) {
                    args_map.insert(arg_name.clone(), "true".to_string());
                }
            } else {
                // For regular named arguments - use type-aware extraction
                if let Some(value) = Self::extract_value_as_string(verb_matches, arg_name) {
                    args_map.insert(arg_name.clone(), value);
                }
            }
        }

        args_map
    }

    /// Run CLI with auto-discovered commands
    pub fn run(&self, args: Vec<String>) -> Result<()> {
        let cmd = self.build_command();
        let matches = match cmd.try_get_matches_from(args) {
            Ok(m) => m,
            Err(e) => {
                // Clap returns an error for help/version, but with exit code 0
                // In this case, we should print it and exit successfully
                let exit_code = e.exit_code();
                let help_or_version_msg = e.to_string();

                // Print the message (help or version)
                print!("{}", help_or_version_msg);

                // Exit with the code clap wants (0 for help/version, non-0 for errors)
                if exit_code == 0 {
                    return Ok(());
                } else {
                    return Err(crate::error::NounVerbError::argument_error(help_or_version_msg));
                }
            }
        };

        // Route command
        if let Some((subcommand_name, sub_matches)) = matches.subcommand() {
            // First check if this is a root-level verb
            if let Some(verb_meta) = self.root_verbs.get(subcommand_name) {
                // Execute root verb directly
                let args_map = self.extract_args(verb_meta, sub_matches);

                let input = crate::logic::HandlerInput {
                    args: args_map,
                    opts: std::collections::HashMap::new(),
                    context: crate::logic::HandlerContext::new(subcommand_name),
                };

                let output = self.execute_root_verb(subcommand_name, input)?;
                let json = output.to_json()?;
                println!("{}", json);
            } else if let Some((verb_name, verb_matches)) = sub_matches.subcommand() {
                // This is a noun with a verb subcommand
                let noun_name = subcommand_name;
                // Execute verb - extract arguments from matches
                let args_map = if let Some(verbs) = self.verbs.get(noun_name) {
                    if let Some(verb_meta) = verbs.get(verb_name) {
                        self.extract_args(verb_meta, verb_matches)
                    } else {
                        std::collections::HashMap::new()
                    }
                } else {
                    std::collections::HashMap::new()
                };

                let input = crate::logic::HandlerInput {
                    args: args_map,
                    opts: std::collections::HashMap::new(),
                    context: crate::logic::HandlerContext::new(verb_name).with_noun(noun_name),
                };

                let output = self.execute_verb(noun_name, verb_name, input)?;
                let json = output.to_json()?;
                println!("{}", json);
            } else {
                // No verb specified - show help for the noun
                let noun_name = subcommand_name;
                if let Some(noun_meta) = self.nouns.get(noun_name) {
                    let noun_name_static: &'static str =
                        Box::leak(noun_name.to_string().into_boxed_str());
                    let about_static: &'static str =
                        Box::leak(noun_meta.about.clone().into_boxed_str());

                    let mut noun_cmd = clap::Command::new(noun_name_static).about(about_static);

                    // Apply long_about if available
                    if let Some(ref long_about) = noun_meta.long_about {
                        let long_about_static: &'static str =
                            Box::leak(long_about.clone().into_boxed_str());
                        noun_cmd = noun_cmd.long_about(long_about_static);
                    }

                    // Add verbs as subcommands for help display
                    if let Some(verbs) = self.verbs.get(noun_name) {
                        for (verb_name, verb_meta) in verbs {
                            let verb_name_static: &'static str =
                                Box::leak(verb_name.clone().into_boxed_str());
                            let verb_about_static: &'static str =
                                Box::leak(verb_meta.about.clone().into_boxed_str());
                            noun_cmd = noun_cmd.subcommand(
                                clap::Command::new(verb_name_static).about(verb_about_static),
                            );
                        }
                    }

                    noun_cmd.print_help().map_err(|e| {
                        crate::error::NounVerbError::execution_error(format!(
                            "Failed to print help: {}",
                            e
                        ))
                    })?;
                } else {
                    return Err(crate::error::NounVerbError::invalid_structure(
                        "No verb specified",
                    ));
                }
            }
        } else {
            // No noun specified - show root help
            let mut cmd = self.build_command();
            cmd.print_help().map_err(|e| {
                crate::error::NounVerbError::execution_error(format!("Failed to print help: {}", e))
            })?;
        }

        Ok(())
    }

    /// Execute a root-level verb handler (verbs without a noun)
    pub fn execute_root_verb(
        &self,
        verb_name: &str,
        input: HandlerInput,
    ) -> Result<HandlerOutput> {
        let verb = self
            .root_verbs
            .get(verb_name)
            .ok_or_else(|| crate::error::NounVerbError::command_not_found(verb_name))?;

        (verb.handler_fn)(input)
    }
}
