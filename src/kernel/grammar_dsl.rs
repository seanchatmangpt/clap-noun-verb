//! # Advanced Grammar DSL
//!
//! Ergonomic macro-based DSL for defining CNV 4.0 grammars with capabilities.
//! Makes it trivial to create capability-aware command structures for 2027.
//!
//! ## Example
//!
//! ```rust,ignore
//! use clap_noun_verb::grammar_dsl;
//!
//! grammar_dsl! {
//!     app "myapp" version "1.0.0" {
//!         noun "file" help "File operations" {
//!             verb "read" {
//!                 capability: Pure,
//!                 resource: Fast,
//!                 help: "Read a file",
//!                 args: [
//!                     path: String = "Path to file"
//!                 ]
//!             }
//!
//!             verb "write" {
//!                 capability: ReadWriteFS,
//!                 resource: Medium,
//!                 safety: HumanReviewRequired,
//!                 help: "Write to a file",
//!                 args: [
//!                     path: String = "Path to file",
//!                     content: String = "Content to write"
//!                 ]
//!             }
//!         }
//!
//!         noun "network" help "Network operations" {
//!             verb "fetch" {
//!                 capability: Network,
//!                 resource: Slow,
//!                 help: "Fetch from URL",
//!                 args: [
//!                     url: String = "URL to fetch"
//!                 ]
//!             }
//!         }
//!     }
//! }
//! ```

/// Macro for defining CNV 4.0 grammars with capabilities
///
/// This macro provides an ergonomic DSL that compiles down to
/// GrammarModel creation with full capability support.
#[macro_export]
macro_rules! grammar_dsl {
    // Entry point: app definition
    (
        app $app_name:literal version $version:literal {
            $($noun_def:tt)*
        }
    ) => {{
        let mut grammar = $crate::kernel::grammar::GrammarModel::new($app_name)
            .with_version($version);

        $crate::__grammar_dsl_nouns!(grammar, $($noun_def)*);

        grammar
    }};
}

/// Internal: Process noun definitions
#[doc(hidden)]
#[macro_export]
macro_rules! __grammar_dsl_nouns {
    // Base case: no more nouns
    ($grammar:ident,) => {};

    // Process one noun and recurse
    ($grammar:ident,
        noun $noun_name:literal help $help:literal {
            $($verb_def:tt)*
        }
        $($rest:tt)*
    ) => {
        {
            let mut verbs = Vec::new();
            $crate::__grammar_dsl_verbs!(verbs, $noun_name, $($verb_def)*);

            let noun = $crate::kernel::grammar::GrammarNoun {
                name: $noun_name.to_string(),
                help: Some($help.to_string()),
                long_help: None,
                verbs,
                sub_nouns: Vec::new(),
                metadata: Default::default(),
            };

            $grammar.add_noun(noun);
        }

        $crate::__grammar_dsl_nouns!($grammar, $($rest)*);
    };
}

/// Internal: Process verb definitions
#[doc(hidden)]
#[macro_export]
macro_rules! __grammar_dsl_verbs {
    // Base case: no more verbs
    ($verbs:ident, $noun_name:expr,) => {};

    // Process verb with full capability spec
    ($verbs:ident, $noun_name:expr,
        verb $verb_name:literal {
            capability: $cap:ident,
            resource: $resource:ident,
            $(safety: $safety:ident,)?
            $(stability: $stability:ident,)?
            help: $help:literal
            $(, args: [ $($arg_def:tt)* ])?
        }
        $($rest:tt)*
    ) => {
        {
            use $crate::kernel::capability::*;

            // Build capability contract
            let contract = match stringify!($cap) {
                "Pure" => CapabilityContract::new(
                    CapabilityClass::Pure,
                    $crate::__grammar_dsl_resource!($resource),
                    $crate::__grammar_dsl_stability!($($stability)?),
                    $crate::__grammar_dsl_safety!($($safety)?),
                ),
                "ReadOnlyFS" => CapabilityContract::new(
                    CapabilityClass::ReadOnlyFS,
                    $crate::__grammar_dsl_resource!($resource),
                    $crate::__grammar_dsl_stability!($($stability)?),
                    $crate::__grammar_dsl_safety!($($safety)?),
                ),
                "ReadWriteFS" => CapabilityContract::new(
                    CapabilityClass::ReadWriteFS,
                    $crate::__grammar_dsl_resource!($resource),
                    $crate::__grammar_dsl_stability!($($stability)?),
                    $crate::__grammar_dsl_safety!($($safety)?),
                ),
                "Network" => CapabilityContract::new(
                    CapabilityClass::Network,
                    $crate::__grammar_dsl_resource!($resource),
                    $crate::__grammar_dsl_stability!($($stability)?),
                    $crate::__grammar_dsl_safety!($($safety)?),
                ),
                "Subprocess" => CapabilityContract::new(
                    CapabilityClass::Subprocess,
                    $crate::__grammar_dsl_resource!($resource),
                    $crate::__grammar_dsl_stability!($($stability)?),
                    $crate::__grammar_dsl_safety!($($safety)?),
                ),
                "Environment" => CapabilityContract::new(
                    CapabilityClass::Environment,
                    $crate::__grammar_dsl_resource!($resource),
                    $crate::__grammar_dsl_stability!($($stability)?),
                    $crate::__grammar_dsl_safety!($($safety)?),
                ),
                "Dangerous" => CapabilityContract::new(
                    CapabilityClass::Dangerous,
                    $crate::__grammar_dsl_resource!($resource),
                    $crate::__grammar_dsl_stability!($($stability)?),
                    $crate::__grammar_dsl_safety!($($safety)?),
                ),
                _ => panic!("Unknown capability: {}", stringify!($cap)),
            };

            // Parse arguments if provided
            let mut arguments = Vec::new();
            $crate::__grammar_dsl_parse_args!(arguments, $($($arg_def)*)?);

            let verb = $crate::kernel::grammar::GrammarVerb {
                name: $verb_name.to_string(),
                noun: $noun_name.to_string(),
                help: Some($help.to_string()),
                long_help: None,
                arguments,
                deprecated: false,
                deprecation_message: None,
                capability: Some(contract),
                metadata: Default::default(),
            };

            $verbs.push(verb);
        }

        $crate::__grammar_dsl_verbs!($verbs, $noun_name, $($rest)*);
    };
}

/// Internal: Parse argument definitions from DSL
#[doc(hidden)]
#[macro_export]
macro_rules! __grammar_dsl_parse_args {
    // Base case: no more arguments
    ($args:ident,) => {};

    // Parse one argument: name: Type = "help text"
    ($args:ident,
        $arg_name:ident : $arg_type:ty = $arg_help:literal
        $(, $($rest:tt)*)?
    ) => {
        {
            let arg = $crate::kernel::grammar::GrammarArgument {
                name: stringify!($arg_name).to_string(),
                short: None,
                long: Some(stringify!($arg_name).to_string()),
                arg_type: $crate::kernel::grammar::ArgumentType::Named,
                help: Some($arg_help.to_string()),
                required: true,
                default: None,
                env: None,
                value_name: Some(stringify!($arg_type).to_string()),
                possible_values: None,
                multiple: false,
                group: None,
                requires: Vec::new(),
                conflicts_with: Vec::new(),
                index: None,
            };
            $args.push(arg);
        }
        $crate::__grammar_dsl_parse_args!($args, $($($rest)*)?);
    };
}

/// Internal: Map resource band
#[doc(hidden)]
#[macro_export]
macro_rules! __grammar_dsl_resource {
    (Instant) => {
        $crate::kernel::capability::ResourceBand::Instant
    };
    (Fast) => {
        $crate::kernel::capability::ResourceBand::Fast
    };
    (Medium) => {
        $crate::kernel::capability::ResourceBand::Medium
    };
    (Slow) => {
        $crate::kernel::capability::ResourceBand::Slow
    };
    (Cold) => {
        $crate::kernel::capability::ResourceBand::Cold
    };
}

/// Internal: Map safety profile
#[doc(hidden)]
#[macro_export]
macro_rules! __grammar_dsl_safety {
    () => {
        $crate::kernel::capability::SafetyProfile::AgentSafe
    };
    (AgentSafe) => {
        $crate::kernel::capability::SafetyProfile::AgentSafe
    };
    (HumanReviewRequired) => {
        $crate::kernel::capability::SafetyProfile::HumanReviewRequired
    };
    (InteractiveOnly) => {
        $crate::kernel::capability::SafetyProfile::InteractiveOnly
    };
}

/// Internal: Map stability profile
#[doc(hidden)]
#[macro_export]
macro_rules! __grammar_dsl_stability {
    () => {
        $crate::kernel::capability::StabilityProfile::Stable
    };
    (Stable) => {
        $crate::kernel::capability::StabilityProfile::Stable
    };
    (Preview) => {
        $crate::kernel::capability::StabilityProfile::Preview
    };
    (Experimental) => {
        $crate::kernel::capability::StabilityProfile::Experimental
    };
    (Deprecated) => {
        $crate::kernel::capability::StabilityProfile::Deprecated
    };
    (NonDeterministic) => {
        $crate::kernel::capability::StabilityProfile::NonDeterministic
    };
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use crate::kernel::*;

    #[test]
    fn test_grammar_dsl_basic() {
        let grammar = grammar_dsl! {
            app "test-app" version "1.0.0" {
                noun "file" help "File operations" {
                    verb "read" {
                        capability: Pure,
                        resource: Fast,
                        help: "Read a file"
                    }

                    verb "write" {
                        capability: ReadWriteFS,
                        resource: Medium,
                        safety: HumanReviewRequired,
                        help: "Write a file"
                    }
                }

                noun "network" help "Network operations" {
                    verb "fetch" {
                        capability: Network,
                        resource: Slow,
                        help: "Fetch from URL"
                    }
                }
            }
        };

        // Verify grammar structure
        assert_eq!(grammar.app_name, "test-app");
        assert_eq!(grammar.app_version, Some("1.0.0".to_string()));
        assert_eq!(grammar.nouns().len(), 2);

        // Verify file noun
        let file_noun = grammar.nouns().iter().find(|n| n.name == "file").unwrap();
        assert_eq!(file_noun.verbs.len(), 2);

        // Verify read verb (Pure capability)
        let read_verb = file_noun.verbs.iter().find(|v| v.name == "read").unwrap();
        let read_cap = read_verb.capability.as_ref().unwrap();
        assert_eq!(read_cap.capability_class, capability::CapabilityClass::Pure);
        assert_eq!(read_cap.resource_band, capability::ResourceBand::Fast);

        // Verify write verb (ReadWriteFS with HumanReviewRequired)
        let write_verb = file_noun.verbs.iter().find(|v| v.name == "write").unwrap();
        let write_cap = write_verb.capability.as_ref().unwrap();
        assert_eq!(
            write_cap.capability_class,
            capability::CapabilityClass::ReadWriteFS
        );
        assert_eq!(
            write_cap.safety,
            capability::SafetyProfile::HumanReviewRequired
        );

        // Verify network noun
        let network_noun = grammar.nouns().iter().find(|n| n.name == "network").unwrap();
        assert_eq!(network_noun.verbs.len(), 1);

        let fetch_verb = network_noun.verbs.iter().find(|v| v.name == "fetch").unwrap();
        let fetch_cap = fetch_verb.capability.as_ref().unwrap();
        assert_eq!(
            fetch_cap.capability_class,
            capability::CapabilityClass::Network
        );
        assert_eq!(fetch_cap.resource_band, capability::ResourceBand::Slow);
    }

    #[test]
    fn test_grammar_dsl_stability_profiles() {
        let grammar = grammar_dsl! {
            app "test" version "1.0.0" {
                noun "experimental" help "Experimental features" {
                    verb "beta" {
                        capability: Pure,
                        resource: Fast,
                        stability: Experimental,
                        help: "Beta feature"
                    }

                    verb "deprecated" {
                        capability: Pure,
                        resource: Fast,
                        stability: Deprecated,
                        help: "Old feature"
                    }
                }
            }
        };

        let noun = grammar.nouns().first().unwrap();
        let beta = noun.verbs.iter().find(|v| v.name == "beta").unwrap();
        let deprecated = noun.verbs.iter().find(|v| v.name == "deprecated").unwrap();

        assert_eq!(
            beta.capability.as_ref().unwrap().stability,
            capability::StabilityProfile::Experimental
        );
        assert_eq!(
            deprecated.capability.as_ref().unwrap().stability,
            capability::StabilityProfile::Deprecated
        );
    }
}
