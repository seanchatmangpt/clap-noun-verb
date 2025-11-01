//! Compile-time command registry for attribute macros
//!
//! This module provides a registry that collects functions marked with
//! `#[verb]` and `#[noun]` attributes at compile time using linkme.
//!
//! These attribute macros are provided by the `clap-noun-verb-macros` crate.

use crate::error::Result;
use crate::logic::{HandlerInput, HandlerOutput};
use linkme::distributed_slice;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

/// Apply validation constraints to a clap Arg
///
/// This function applies min/max value and length validators based on the
/// validation metadata stored in ArgMetadata.
fn apply_validators(arg: &mut clap::Arg, arg_meta: &ArgMetadata) {
    // For numeric types with min/max values, apply range validators
    if arg_meta.min_value.is_some() || arg_meta.max_value.is_some() {
        // Try to parse as integers first
        let min_i64 = arg_meta.min_value.as_ref().and_then(|v| v.parse::<i64>().ok());
        let max_i64 = arg_meta.max_value.as_ref().and_then(|v| v.parse::<i64>().ok());
        let min_u64 = arg_meta.min_value.as_ref().and_then(|v| v.parse::<u64>().ok());
        let max_u64 = arg_meta.max_value.as_ref().and_then(|v| v.parse::<u64>().ok());

        // Apply range validators based on what we can parse
        if let (Some(min), Some(max)) = (min_i64, max_i64) {
            *arg = arg.clone().value_parser(clap::value_parser!(i64).range(min..=max));
        } else if let Some(min) = min_i64 {
            *arg = arg.clone().value_parser(clap::value_parser!(i64).range(min..));
        } else if let Some(max) = max_i64 {
            *arg = arg.clone().value_parser(clap::value_parser!(i64).range(..=max));
        } else if let (Some(min), Some(max)) = (min_u64, max_u64) {
            *arg = arg.clone().value_parser(clap::value_parser!(u64).range(min..=max));
        } else if let Some(min) = min_u64 {
            *arg = arg.clone().value_parser(clap::value_parser!(u64).range(min..));
        } else if let Some(max) = max_u64 {
            *arg = arg.clone().value_parser(clap::value_parser!(u64).range(..=max));
        }
    }

    // For string types with min_length, ensure non-empty
    if let Some(min_len) = arg_meta.min_length {
        if min_len > 0 {
            *arg = arg.clone().value_parser(clap::builder::NonEmptyStringValueParser::new());
        }
    }
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
}

/// Metadata for a registered noun
struct NounMetadata {
    #[allow(dead_code)] // Reserved for future use
    name: String,
    about: String,
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
            Mutex::new(CommandRegistry { nouns: HashMap::new(), verbs: HashMap::new() })
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
            Mutex::new(CommandRegistry { nouns: HashMap::new(), verbs: HashMap::new() })
        });
        // Lock poisoning should not happen in practice, but handle it gracefully
        let mut reg = registry.lock().unwrap_or_else(|e| e.into_inner());
        reg.nouns.insert(
            name.to_string(),
            NounMetadata { name: name.to_string(), about: about.to_string() },
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
            Mutex::new(CommandRegistry { nouns: HashMap::new(), verbs: HashMap::new() })
        });
        // Lock poisoning should not happen in practice, but handle it gracefully
        let mut reg = registry.lock().unwrap_or_else(|e| e.into_inner());
        reg.verbs.entry(noun_name.to_string()).or_default().insert(
            verb_name.to_string(),
            VerbMetadata {
                noun_name: noun_name.to_string(),
                verb_name: verb_name.to_string(),
                about: about.to_string(),
                args,
                handler_fn: Box::new(handler),
            },
        );
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
        let mut cmd = clap::Command::new("cli");

        for (noun_name, noun_meta) in &self.nouns {
            let noun_name: &'static str = Box::leak(noun_name.clone().into_boxed_str());
            let about: &'static str = Box::leak(noun_meta.about.clone().into_boxed_str());

            let mut noun_cmd = clap::Command::new(noun_name).about(about);

            // Add verbs as subcommands
            if let Some(verbs) = self.verbs.get(noun_name) {
                for (verb_name, verb_meta) in verbs {
                    let verb_name: &'static str = Box::leak(verb_name.clone().into_boxed_str());
                    let about: &'static str = Box::leak(verb_meta.about.clone().into_boxed_str());

                    let mut verb_cmd = clap::Command::new(verb_name).about(about);

                    // Add arguments from metadata
                    for arg_meta in &verb_meta.args {
                        let arg_name: &'static str =
                            Box::leak(arg_meta.name.clone().into_boxed_str());
                        let value_name: &'static str =
                            Box::leak(arg_meta.name.to_uppercase().into_boxed_str());
                        let mut arg = clap::Arg::new(arg_name).long(arg_name);

                        if arg_meta.is_flag {
                            arg = arg.action(clap::ArgAction::SetTrue);
                        } else {
                            arg = arg.value_name(value_name);
                            if arg_meta.required {
                                arg = arg.required(true);
                            }

                            // Apply auto-inferred and explicit validation constraints
                            // Try to apply validators based on the stored metadata
                            apply_validators(&mut arg, arg_meta);
                        }

                        // Add help text from docstring if available
                        if let Some(help_text) = &arg_meta.help {
                            let help: &'static str = Box::leak(help_text.clone().into_boxed_str());
                            arg = arg.help(help);
                        }

                        verb_cmd = verb_cmd.arg(arg);
                    }

                    noun_cmd = noun_cmd.subcommand(verb_cmd);
                }
            }

            cmd = cmd.subcommand(noun_cmd);
        }

        cmd
    }

    /// Run CLI with auto-discovered commands
    pub fn run(&self, args: Vec<String>) -> Result<()> {
        let cmd = self.build_command();
        let matches = cmd
            .try_get_matches_from(args)
            .map_err(|e| crate::error::NounVerbError::argument_error(e.to_string()))?;

        // Route command
        if let Some((noun_name, noun_matches)) = matches.subcommand() {
            if let Some((verb_name, verb_matches)) = noun_matches.subcommand() {
                // Execute verb - extract arguments from matches
                let mut args_map = std::collections::HashMap::new();

                // Get verb metadata to know which arguments exist
                if let Some(verbs) = self.verbs.get(noun_name) {
                    if let Some(verb_meta) = verbs.get(verb_name) {
                        // Extract each argument by name from clap matches
                        for arg_meta in &verb_meta.args {
                            let arg_name = &arg_meta.name;
                            if let Some(value) = verb_matches.get_one::<String>(arg_name) {
                                args_map.insert(arg_name.clone(), value.clone());
                            } else if arg_meta.is_flag {
                                // For flags, check if they're set
                                if verb_matches.get_flag(arg_name) {
                                    args_map.insert(arg_name.clone(), "true".to_string());
                                }
                            }
                        }
                    }
                }

                let input = crate::logic::HandlerInput {
                    args: args_map,
                    opts: std::collections::HashMap::new(),
                    context: crate::logic::HandlerContext::new(verb_name).with_noun(noun_name),
                };

                let output = self.execute_verb(noun_name, verb_name, input)?;
                let json = output.to_json()?;
                println!("{}", json);
            } else {
                return Err(crate::error::NounVerbError::invalid_structure("No verb specified"));
            }
        } else {
            return Err(crate::error::NounVerbError::invalid_structure("No noun specified"));
        }

        Ok(())
    }
}
