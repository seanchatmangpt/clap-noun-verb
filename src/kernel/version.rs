//! CNV Version Negotiation and Grammar Delta
//!
//! Provides first-class change semantics and negotiation for trillions of agents.
//!
//! # Design
//!
//! Every CNV app can:
//! - Describe what changed between two versions structurally
//! - Negotiate a compatible "view" of the CLI with an agent
//! - Classify changes as breaking, potentially breaking, or safe
//!
//! # Features
//!
//! - **Grammar Delta Model**: Structural diffs between grammar versions
//! - **Change Classification**: Automatic breaking change detection
//! - **Version Negotiation**: Agent compatibility protocol
//! - **Capability-Aware Changes**: Track side-effect and safety changes
//!
//! # Example
//!
//! ```rust,ignore
//! use clap_noun_verb::kernel::{GrammarDelta, VersionNegotiator};
//!
//! // Compare two grammar versions
//! let delta = GrammarDelta::compute(&old_grammar, &new_grammar)?;
//!
//! // Check for breaking changes
//! if delta.has_breaking_changes() {
//!     println!("Warning: Breaking changes detected!");
//! }
//!
//! // Negotiate with an agent
//! let negotiator = VersionNegotiator::new(current_grammar);
//! let response = negotiator.negotiate(&agent_request)?;
//! ```

use crate::kernel::capability::CapabilityContract;
use crate::kernel::grammar::{ArgumentType, GrammarArgument, GrammarModel, GrammarNoun, GrammarVerb};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Change type for grammar elements
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeType {
    /// Element added (non-breaking)
    Added,
    /// Element removed (breaking)
    Removed,
    /// Element renamed (potentially breaking)
    Renamed {
        /// Old name
        old_name: String,
        /// New name
        new_name: String,
    },
    /// Element modified (depends on modification)
    Modified,
}

impl ChangeType {
    /// Check if this change is breaking
    pub fn is_breaking(&self) -> bool {
        matches!(self, Self::Removed | Self::Renamed { .. })
    }

    /// Check if this change is potentially breaking
    pub fn is_potentially_breaking(&self) -> bool {
        matches!(self, Self::Modified)
    }
}

/// Argument change detail
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "change_type", rename_all = "snake_case")]
pub enum ArgumentChange {
    /// Argument added
    Added {
        /// Argument metadata
        arg: GrammarArgument,
        /// Whether it's required (breaking if true)
        breaking: bool,
    },
    /// Argument removed (breaking)
    Removed {
        /// Argument name
        name: String,
    },
    /// Argument renamed (potentially breaking)
    Renamed {
        /// Old name
        old_name: String,
        /// New name
        new_name: String,
    },
    /// Argument requirement changed
    RequirementChanged {
        /// Argument name
        name: String,
        /// Old required status
        was_required: bool,
        /// New required status
        now_required: bool,
    },
    /// Argument type changed (breaking)
    TypeChanged {
        /// Argument name
        name: String,
        /// Old type
        old_type: ArgumentType,
        /// New type
        new_type: ArgumentType,
    },
    /// Default value changed (potentially breaking)
    DefaultChanged {
        /// Argument name
        name: String,
        /// Old default
        old_default: Option<String>,
        /// New default
        new_default: Option<String>,
    },
}

impl ArgumentChange {
    /// Check if this change is breaking
    pub fn is_breaking(&self) -> bool {
        match self {
            Self::Added { breaking, .. } => *breaking,
            Self::Removed { .. } => true,
            Self::TypeChanged { .. } => true,
            Self::RequirementChanged { now_required, .. } => *now_required,
            _ => false,
        }
    }
}

/// Capability change detail
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "change_type", rename_all = "snake_case")]
pub enum CapabilityChange {
    /// Capability added (non-breaking)
    Added {
        /// New capability
        capability: CapabilityContract,
    },
    /// Capability removed (potentially breaking)
    Removed,
    /// Capability class changed
    ClassChanged {
        /// Old class
        old_class: String,
        /// New class
        new_class: String,
        /// Whether more dangerous (breaking)
        more_dangerous: bool,
    },
    /// Safety profile changed
    SafetyChanged {
        /// Old safety
        old_safety: String,
        /// New safety
        new_safety: String,
        /// Whether less safe (breaking for automation)
        less_safe: bool,
    },
    /// Stability changed
    StabilityChanged {
        /// Old stability
        old_stability: String,
        /// New stability
        new_stability: String,
        /// Whether deprecated (breaking)
        deprecated: bool,
    },
}

impl CapabilityChange {
    /// Check if this change is breaking
    pub fn is_breaking(&self) -> bool {
        match self {
            Self::ClassChanged { more_dangerous, .. } => *more_dangerous,
            Self::SafetyChanged { less_safe, .. } => *less_safe,
            Self::StabilityChanged { deprecated, .. } => *deprecated,
            _ => false,
        }
    }
}

/// Verb change detail
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VerbChange {
    /// Change type
    pub change_type: ChangeType,
    /// Verb name
    pub name: String,
    /// Parent noun
    pub noun: String,
    /// Argument changes
    pub argument_changes: Vec<ArgumentChange>,
    /// Capability changes
    pub capability_changes: Vec<CapabilityChange>,
    /// Help text changed
    pub help_changed: bool,
}

impl VerbChange {
    /// Check if this change is breaking
    pub fn is_breaking(&self) -> bool {
        self.change_type.is_breaking()
            || self.argument_changes.iter().any(|c| c.is_breaking())
            || self.capability_changes.iter().any(|c| c.is_breaking())
    }

    /// Check if this change is potentially breaking
    pub fn is_potentially_breaking(&self) -> bool {
        !self.is_breaking()
            && (self.change_type.is_potentially_breaking()
                || !self.argument_changes.is_empty()
                || !self.capability_changes.is_empty())
    }
}

/// Noun change detail
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NounChange {
    /// Change type
    pub change_type: ChangeType,
    /// Noun name
    pub name: String,
    /// Verb changes under this noun
    pub verb_changes: Vec<VerbChange>,
}

impl NounChange {
    /// Check if this change is breaking
    pub fn is_breaking(&self) -> bool {
        self.change_type.is_breaking() || self.verb_changes.iter().any(|c| c.is_breaking())
    }
}

/// Change severity classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeSeverity {
    /// Non-breaking change (safe)
    Safe,
    /// Potentially breaking (caution advised)
    PotentiallyBreaking,
    /// Breaking change (requires migration)
    Breaking,
}

impl ChangeSeverity {
    /// Get severity level (0-2)
    pub fn level(&self) -> u8 {
        match self {
            Self::Safe => 0,
            Self::PotentiallyBreaking => 1,
            Self::Breaking => 2,
        }
    }
}

/// Grammar delta - structural difference between two grammar versions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrammarDelta {
    /// Source grammar version
    pub from_version: String,
    /// Target grammar version
    pub to_version: String,
    /// Noun changes
    pub noun_changes: Vec<NounChange>,
    /// Verb changes (across all nouns)
    pub verb_changes: Vec<VerbChange>,
    /// Overall severity
    pub severity: ChangeSeverity,
}

impl GrammarDelta {
    /// Compute delta between two grammar models
    pub fn compute(from: &GrammarModel, to: &GrammarModel) -> Result<Self, Box<dyn std::error::Error>> {
        let from_version = from.app_version.clone().unwrap_or_else(|| "unknown".to_string());
        let to_version = to.app_version.clone().unwrap_or_else(|| "unknown".to_string());

        let mut noun_changes = Vec::new();
        let mut verb_changes = Vec::new();

        // Build noun maps
        let from_nouns: HashMap<_, _> = from.nouns.iter().map(|n| (&n.name, n)).collect();
        let to_nouns: HashMap<_, _> = to.nouns.iter().map(|n| (&n.name, n)).collect();

        // Find added nouns
        for (name, _noun) in &to_nouns {
            if !from_nouns.contains_key(name) {
                noun_changes.push(NounChange {
                    change_type: ChangeType::Added,
                    name: (*name).clone(),
                    verb_changes: Vec::new(),
                });
            }
        }

        // Find removed nouns
        for (name, _) in &from_nouns {
            if !to_nouns.contains_key(name) {
                noun_changes.push(NounChange {
                    change_type: ChangeType::Removed,
                    name: (*name).clone(),
                    verb_changes: Vec::new(),
                });
            }
        }

        // Find modified nouns
        for (name, from_noun) in &from_nouns {
            if let Some(to_noun) = to_nouns.get(name) {
                let noun_verb_changes = Self::compute_verb_changes(from_noun, to_noun);
                if !noun_verb_changes.is_empty() {
                    noun_changes.push(NounChange {
                        change_type: ChangeType::Modified,
                        name: (*name).clone(),
                        verb_changes: noun_verb_changes.clone(),
                    });
                    verb_changes.extend(noun_verb_changes);
                }
            }
        }

        // Determine overall severity
        let severity = if noun_changes.iter().any(|c| c.is_breaking())
            || verb_changes.iter().any(|c| c.is_breaking())
        {
            ChangeSeverity::Breaking
        } else if !noun_changes.is_empty() || !verb_changes.is_empty() {
            ChangeSeverity::PotentiallyBreaking
        } else {
            ChangeSeverity::Safe
        };

        Ok(Self {
            from_version,
            to_version,
            noun_changes,
            verb_changes,
            severity,
        })
    }

    /// Compute verb changes between two nouns
    fn compute_verb_changes(from_noun: &GrammarNoun, to_noun: &GrammarNoun) -> Vec<VerbChange> {
        let mut changes = Vec::new();

        let from_verbs: HashMap<_, _> = from_noun.verbs.iter().map(|v| (&v.name, v)).collect();
        let to_verbs: HashMap<_, _> = to_noun.verbs.iter().map(|v| (&v.name, v)).collect();

        // Find added verbs
        for (name, verb) in &to_verbs {
            if !from_verbs.contains_key(name) {
                changes.push(VerbChange {
                    change_type: ChangeType::Added,
                    name: (*name).clone(),
                    noun: to_noun.name.clone(),
                    argument_changes: Vec::new(),
                    capability_changes: if verb.capability.is_some() {
                        vec![CapabilityChange::Added {
                            capability: verb.capability.clone().unwrap(),
                        }]
                    } else {
                        Vec::new()
                    },
                    help_changed: false,
                });
            }
        }

        // Find removed verbs
        for (name, _) in &from_verbs {
            if !to_verbs.contains_key(name) {
                changes.push(VerbChange {
                    change_type: ChangeType::Removed,
                    name: (*name).clone(),
                    noun: from_noun.name.clone(),
                    argument_changes: Vec::new(),
                    capability_changes: Vec::new(),
                    help_changed: false,
                });
            }
        }

        // Find modified verbs
        for (name, from_verb) in &from_verbs {
            if let Some(to_verb) = to_verbs.get(name) {
                let arg_changes = Self::compute_argument_changes(from_verb, to_verb);
                let cap_changes = Self::compute_capability_changes(from_verb, to_verb);
                let help_changed = from_verb.help != to_verb.help;

                if !arg_changes.is_empty() || !cap_changes.is_empty() || help_changed {
                    changes.push(VerbChange {
                        change_type: ChangeType::Modified,
                        name: (*name).clone(),
                        noun: from_noun.name.clone(),
                        argument_changes: arg_changes,
                        capability_changes: cap_changes,
                        help_changed,
                    });
                }
            }
        }

        changes
    }

    /// Compute argument changes between two verbs
    fn compute_argument_changes(from_verb: &GrammarVerb, to_verb: &GrammarVerb) -> Vec<ArgumentChange> {
        let mut changes = Vec::new();

        let from_args: HashMap<_, _> = from_verb.arguments.iter().map(|a| (&a.name, a)).collect();
        let to_args: HashMap<_, _> = to_verb.arguments.iter().map(|a| (&a.name, a)).collect();

        // Find added arguments
        for (name, arg) in &to_args {
            if !from_args.contains_key(name) {
                changes.push(ArgumentChange::Added {
                    arg: (*arg).clone(),
                    breaking: arg.required,
                });
            }
        }

        // Find removed arguments
        for (name, _) in &from_args {
            if !to_args.contains_key(name) {
                changes.push(ArgumentChange::Removed {
                    name: (*name).clone(),
                });
            }
        }

        // Find modified arguments
        for (name, from_arg) in &from_args {
            if let Some(to_arg) = to_args.get(name) {
                // Check requirement change
                if from_arg.required != to_arg.required {
                    changes.push(ArgumentChange::RequirementChanged {
                        name: (*name).clone(),
                        was_required: from_arg.required,
                        now_required: to_arg.required,
                    });
                }

                // Check type change
                if from_arg.arg_type != to_arg.arg_type {
                    changes.push(ArgumentChange::TypeChanged {
                        name: (*name).clone(),
                        old_type: from_arg.arg_type.clone(),
                        new_type: to_arg.arg_type.clone(),
                    });
                }

                // Check default change
                if from_arg.default != to_arg.default {
                    changes.push(ArgumentChange::DefaultChanged {
                        name: (*name).clone(),
                        old_default: from_arg.default.clone(),
                        new_default: to_arg.default.clone(),
                    });
                }
            }
        }

        changes
    }

    /// Compute capability changes between two verbs
    fn compute_capability_changes(from_verb: &GrammarVerb, to_verb: &GrammarVerb) -> Vec<CapabilityChange> {
        let mut changes = Vec::new();

        match (&from_verb.capability, &to_verb.capability) {
            (None, Some(cap)) => {
                changes.push(CapabilityChange::Added {
                    capability: cap.clone(),
                });
            }
            (Some(_), None) => {
                changes.push(CapabilityChange::Removed);
            }
            (Some(from_cap), Some(to_cap)) => {
                // Check class change
                if from_cap.capability_class != to_cap.capability_class {
                    changes.push(CapabilityChange::ClassChanged {
                        old_class: format!("{}", from_cap.capability_class),
                        new_class: format!("{}", to_cap.capability_class),
                        more_dangerous: to_cap.capability_class.risk_level()
                            > from_cap.capability_class.risk_level(),
                    });
                }

                // Check safety change
                if from_cap.safety != to_cap.safety {
                    changes.push(CapabilityChange::SafetyChanged {
                        old_safety: format!("{}", from_cap.safety),
                        new_safety: format!("{}", to_cap.safety),
                        less_safe: !to_cap.is_agent_safe() && from_cap.is_agent_safe(),
                    });
                }

                // Check stability change
                if from_cap.stability != to_cap.stability {
                    use crate::kernel::capability::StabilityProfile;
                    changes.push(CapabilityChange::StabilityChanged {
                        old_stability: format!("{}", from_cap.stability),
                        new_stability: format!("{}", to_cap.stability),
                        deprecated: to_cap.stability == StabilityProfile::Deprecated,
                    });
                }
            }
            (None, None) => {}
        }

        changes
    }

    /// Check if there are any breaking changes
    pub fn has_breaking_changes(&self) -> bool {
        matches!(self.severity, ChangeSeverity::Breaking)
    }

    /// Get all breaking changes
    pub fn breaking_changes(&self) -> Vec<String> {
        let mut breaking = Vec::new();

        for noun_change in &self.noun_changes {
            if noun_change.is_breaking() {
                breaking.push(format!("Noun '{}': {:?}", noun_change.name, noun_change.change_type));
            }
        }

        for verb_change in &self.verb_changes {
            if verb_change.is_breaking() {
                breaking.push(format!(
                    "Verb '{}.{}': breaking change",
                    verb_change.noun, verb_change.name
                ));

                for arg_change in &verb_change.argument_changes {
                    if arg_change.is_breaking() {
                        breaking.push(format!("  - Argument change: {:?}", arg_change));
                    }
                }

                for cap_change in &verb_change.capability_changes {
                    if cap_change.is_breaking() {
                        breaking.push(format!("  - Capability change: {:?}", cap_change));
                    }
                }
            }
        }

        breaking
    }

    /// Generate a human-readable change summary
    pub fn summary(&self) -> String {
        let mut lines = Vec::new();

        lines.push(format!(
            "Grammar changes: {} -> {}",
            self.from_version, self.to_version
        ));
        lines.push(format!("Severity: {:?}", self.severity));
        lines.push(String::new());

        if !self.noun_changes.is_empty() {
            lines.push("Noun changes:".to_string());
            for change in &self.noun_changes {
                lines.push(format!("  - {}: {:?}", change.name, change.change_type));
            }
            lines.push(String::new());
        }

        if !self.verb_changes.is_empty() {
            lines.push("Verb changes:".to_string());
            for change in &self.verb_changes {
                lines.push(format!("  - {}.{}: {:?}", change.noun, change.name, change.change_type));

                if !change.argument_changes.is_empty() {
                    lines.push("    Arguments:".to_string());
                    for arg_change in &change.argument_changes {
                        lines.push(format!("      - {:?}", arg_change));
                    }
                }

                if !change.capability_changes.is_empty() {
                    lines.push("    Capabilities:".to_string());
                    for cap_change in &change.capability_changes {
                        lines.push(format!("      - {:?}", cap_change));
                    }
                }
            }
        }

        lines.join("\n")
    }
}

/// Version negotiation request from an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NegotiationRequest {
    /// Agent's known grammar version
    pub known_version: String,
    /// Agent's required capabilities (optional)
    pub required_capabilities: Option<Vec<String>>,
    /// Agent's compatibility level
    pub compatibility_level: CompatibilityLevel,
}

/// Compatibility level for negotiation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompatibilityLevel {
    /// Strict - no breaking changes allowed
    Strict,
    /// Moderate - allow non-breaking changes
    Moderate,
    /// Permissive - allow all changes with warnings
    Permissive,
}

/// Version negotiation response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NegotiationResponse {
    /// Current grammar version
    pub current_version: String,
    /// Delta from requested version
    pub delta: GrammarDelta,
    /// Whether compatible
    pub compatible: bool,
    /// Compatibility warnings
    pub warnings: Vec<String>,
    /// Suggested actions
    pub suggestions: Vec<String>,
}

/// Version negotiator
pub struct VersionNegotiator {
    /// Current grammar
    grammar: GrammarModel,
    /// Historical grammars (version -> grammar)
    history: HashMap<String, GrammarModel>,
}

impl VersionNegotiator {
    /// Create a new version negotiator
    pub fn new(grammar: GrammarModel) -> Self {
        Self {
            grammar,
            history: HashMap::new(),
        }
    }

    /// Add a historical grammar version
    pub fn add_history(&mut self, version: String, grammar: GrammarModel) {
        self.history.insert(version, grammar);
    }

    /// Negotiate with an agent request
    pub fn negotiate(&self, request: &NegotiationRequest) -> Result<NegotiationResponse, Box<dyn std::error::Error>> {
        // Find the requested grammar version
        let from_grammar = self
            .history
            .get(&request.known_version)
            .ok_or_else(|| format!("Unknown version: {}", request.known_version))?;

        // Compute delta
        let delta = GrammarDelta::compute(from_grammar, &self.grammar)?;

        // Determine compatibility based on level
        let compatible = match request.compatibility_level {
            CompatibilityLevel::Strict => !delta.has_breaking_changes(),
            CompatibilityLevel::Moderate => {
                delta.severity != ChangeSeverity::Breaking
                    || delta.breaking_changes().is_empty()
            }
            CompatibilityLevel::Permissive => true,
        };

        // Generate warnings
        let mut warnings = Vec::new();
        if delta.has_breaking_changes() {
            warnings.extend(delta.breaking_changes());
        }

        // Generate suggestions
        let mut suggestions = Vec::new();
        if !compatible {
            suggestions.push("Upgrade to the latest version".to_string());
            suggestions.push("Review breaking changes in the changelog".to_string());
        }

        Ok(NegotiationResponse {
            current_version: self.grammar.app_version.clone().unwrap_or_else(|| "unknown".to_string()),
            delta,
            compatible,
            warnings,
            suggestions,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_change_type_breaking() {
        assert!(ChangeType::Removed.is_breaking());
        assert!(ChangeType::Renamed {
            old_name: "old".to_string(),
            new_name: "new".to_string()
        }
        .is_breaking());
        assert!(!ChangeType::Added.is_breaking());
    }

    #[test]
    fn test_argument_change_breaking() {
        let added_required = ArgumentChange::Added {
            arg: GrammarArgument {
                name: "test".to_string(),
                short: None,
                long: None,
                arg_type: ArgumentType::Named,
                help: None,
                required: true,
                default: None,
                env: None,
                value_name: None,
                possible_values: None,
                multiple: false,
                group: None,
                requires: Vec::new(),
                conflicts_with: Vec::new(),
                index: None,
            },
            breaking: true,
        };
        assert!(added_required.is_breaking());

        let removed = ArgumentChange::Removed {
            name: "test".to_string(),
        };
        assert!(removed.is_breaking());
    }

    #[test]
    fn test_grammar_delta_compute() {
        let v1 = GrammarModel::new("test-app").with_version("1.0.0");
        let v2 = GrammarModel::new("test-app").with_version("2.0.0");

        let delta = GrammarDelta::compute(&v1, &v2).ok().unwrap();
        assert_eq!(delta.from_version, "1.0.0");
        assert_eq!(delta.to_version, "2.0.0");
        assert_eq!(delta.severity, ChangeSeverity::Safe);
    }

    #[test]
    fn test_change_severity_levels() {
        assert_eq!(ChangeSeverity::Safe.level(), 0);
        assert_eq!(ChangeSeverity::PotentiallyBreaking.level(), 1);
        assert_eq!(ChangeSeverity::Breaking.level(), 2);
    }
}
