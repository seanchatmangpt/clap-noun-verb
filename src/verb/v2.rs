//! Verb command trait - validation and delegation only
//!
//! Verb commands can ONLY validate arguments and delegate to
//! business logic. No business logic is allowed in the CLI layer.

use crate::error::Result;
use crate::cli::validator::ArgValidator;
use crate::logic::{HandlerInput, HandlerOutput, HandlerContext};
use clap::{ArgMatches, Command};

/// Verb command trait - validation and delegation only
///
/// This trait enforces that CLI code can ONLY validate arguments
/// and delegate to business logic. Business logic must be provided
/// separately through the `BusinessLogicHandler` type.
///
/// # Pattern Enforcement
///
/// Verbs must:
/// 1. Validate arguments using `validate()`
/// 2. Delegate to business logic using `delegate()`
///
/// This ensures CLI code contains NO business logic.
pub trait VerbCommand: Send + Sync {
    /// The name of the verb command
    fn name(&self) -> &'static str;

    /// Description of what this verb command does
    fn about(&self) -> &'static str;

    /// Validate arguments for this verb
    ///
    /// This method validates all arguments and options for the verb.
    /// It does NOT execute any business logic - only validation.
    ///
    /// # Arguments
    ///
    /// * `matches` - The clap matches for this verb
    /// * `parent_matches` - Parent matches for global args (optional)
    ///
    /// # Errors
    ///
    /// Returns an error if validation fails.
    fn validate(
        &self,
        matches: &ArgMatches,
        parent_matches: Option<&ArgMatches>,
    ) -> Result<HandlerInput>;

    /// Delegate to business logic handler
    ///
    /// This method delegates execution to a business logic handler.
    /// The handler is provided separately, ensuring separation of concerns.
    ///
    /// # Arguments
    ///
    /// * `input` - Validated input from `validate()`
    /// * `handler` - Business logic handler function
    ///
    /// # Errors
    ///
    /// Returns an error if delegation fails.
    fn delegate<F>(&self, input: HandlerInput, handler: F) -> Result<HandlerOutput>
    where
        F: FnOnce(HandlerInput) -> Result<HandlerOutput>;

    /// Build the clap command for this verb
    ///
    /// Default implementation creates a basic command with name and description.
    /// Override to customize command building.
    fn build_command(&self) -> Command {
        Command::new(self.name()).about(self.about())
    }

    /// Get additional arguments for this verb (override to add custom args)
    ///
    /// Returns an empty vector by default. Override to provide custom arguments.
    fn additional_args(&self) -> Vec<clap::Arg> {
        Vec::new()
    }
}

/// Helper function for default validation
///
/// This provides a default validation implementation using `ArgValidator`.
/// Verbs can use this in their `validate()` implementation.
pub fn default_validate(
    matches: &ArgMatches,
    parent_matches: Option<&ArgMatches>,
    noun: Option<&str>,
    verb: &str,
) -> Result<HandlerInput> {
    let validator = ArgValidator::new();

    // Extract validated arguments
    let args = validator.extract_args(matches);

    // Extract validated options
    let opts = validator.extract_opts(matches);

    // Build context
    let mut context = HandlerContext::new(verb);
    if let Some(noun_name) = noun {
        context = context.with_noun(noun_name);
    }

    // Add global args to context if needed
    if let Some(parent) = parent_matches {
        let global_opts = validator.extract_opts(parent);
        for (key, value) in global_opts {
            context = context.with_data(key, value);
        }
    }

    Ok(HandlerInput {
        args,
        opts,
        context,
    })
}

