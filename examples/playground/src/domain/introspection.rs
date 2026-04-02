//! Domain Logic: Autonomic CLI Introspection
//!
//! Pure functions for machine-grade CLI introspection.
//! Enables AI agents to discover and understand CLI capabilities.

use serde::{Deserialize, Serialize};
use super::ontology::{CliCapability, EffectType};

/// Complete CLI introspection response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntrospectionResponse {
    pub cli_name: String,
    pub version: String,
    pub description: String,
    pub nouns: Vec<NounMetadata>,
    pub total_capabilities: usize,
    pub autonomic_features: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NounMetadata {
    pub name: String,
    pub description: String,
    pub verbs: Vec<VerbMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerbMetadata {
    pub name: String,
    pub description: String,
    pub effect: String,
    pub args: Vec<ArgSpec>,
    pub capability_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArgSpec {
    pub name: String,
    pub arg_type: String,
    pub required: bool,
    pub default: Option<String>,
    pub help: String,
}

impl IntrospectionResponse {
    /// Build introspection from capabilities (pure function)
    pub fn from_capabilities(
        cli_name: &str,
        version: &str,
        description: &str,
        capabilities: &[CliCapability],
    ) -> Self {
        // Group capabilities by noun
        let mut noun_map: std::collections::HashMap<String, Vec<&CliCapability>> =
            std::collections::HashMap::new();

        for cap in capabilities {
            noun_map.entry(cap.noun.clone()).or_default().push(cap);
        }

        let nouns: Vec<NounMetadata> = noun_map
            .into_iter()
            .map(|(noun_name, caps)| {
                let verbs: Vec<VerbMetadata> = caps
                    .iter()
                    .map(|cap| VerbMetadata {
                        name: cap.verb.clone(),
                        description: cap.description.clone(),
                        effect: format!("{:?}", cap.effects),
                        args: cap.args.iter().map(|a| ArgSpec {
                            name: a.name.clone(),
                            arg_type: a.arg_type.clone(),
                            required: a.required,
                            default: a.default.clone(),
                            help: format!("{} argument", a.name),
                        }).collect(),
                        capability_id: format!("cnv:{}_{}", cap.noun, cap.verb),
                    })
                    .collect();

                NounMetadata {
                    name: noun_name.clone(),
                    description: format!("{} operations", noun_name),
                    verbs,
                }
            })
            .collect();

        Self {
            cli_name: cli_name.to_string(),
            version: version.to_string(),
            description: description.to_string(),
            total_capabilities: capabilities.len(),
            nouns,
            autonomic_features: vec![
                "introspection".to_string(),
                "rdf_ontology".to_string(),
                "sparql_queries".to_string(),
                "shell_completions".to_string(),
                "man_pages".to_string(),
                "output_formats".to_string(),
                "middleware".to_string(),
                "telemetry".to_string(),
            ],
        }
    }
}

/// Capability contract for effect modeling
#[allow(dead_code)] // FUTURE: Used for agent contract negotiation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContract {
    pub capability_id: String,
    pub effect_type: EffectType,
    pub isolation_level: IsolationLevel,
    pub idempotent: bool,
    pub timeout_ms: Option<u64>,
}

#[allow(dead_code)] // FUTURE: Used for capability sandboxing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IsolationLevel {
    None,
    ReadCommitted,
    Serializable,
}

impl ExecutionContract {
    #[allow(dead_code)] // FUTURE: Used for agent contract negotiation
    pub fn for_capability(cap: &CliCapability) -> Self {
        Self {
            capability_id: format!("cnv:{}_{}", cap.noun, cap.verb),
            effect_type: cap.effects,
            isolation_level: match cap.effects {
                EffectType::ReadOnly => IsolationLevel::None,
                EffectType::Mutating => IsolationLevel::ReadCommitted,
                EffectType::Idempotent => IsolationLevel::None,
            },
            idempotent: cap.effects == EffectType::Idempotent || cap.effects == EffectType::ReadOnly,
            timeout_ms: Some(30000),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::ontology::build_playground_ontology;

    #[test]
    fn test_introspection_from_capabilities() {
        let caps = build_playground_ontology();
        let response = IntrospectionResponse::from_capabilities(
            "playground",
            "1.0.0",
            "Domain-separated exemplar",
            &caps,
        );
        assert_eq!(response.cli_name, "playground");
        assert!(response.nouns.len() >= 3);
        assert!(response.autonomic_features.contains(&"introspection".to_string()));
    }

    #[test]
    fn test_execution_contract() {
        let cap = CliCapability::mutating("config", "set", "Set config");
        let contract = ExecutionContract::for_capability(&cap);
        assert_eq!(contract.effect_type, EffectType::Mutating);
        assert!(!contract.idempotent);
    }
}
