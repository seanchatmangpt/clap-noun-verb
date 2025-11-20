//! Tests for autonomic CLI features

use clap_noun_verb::autonomic::*;
use clap_noun_verb::{noun, CommandRegistry, Result, VerbArgs, VerbCommand};

// Test verb implementations
struct ReadOnlyVerb;

impl VerbCommand for ReadOnlyVerb {
    fn name(&self) -> &'static str {
        "status"
    }

    fn about(&self) -> &'static str {
        "Show status"
    }

    fn run(&self, _args: &VerbArgs) -> Result<()> {
        Ok(())
    }
}

impl AutonomicVerbCommand for ReadOnlyVerb {
    fn metadata(&self) -> CommandMetadata {
        CommandMetadata::new()
            .with_effects(EffectMetadata::new(EffectType::ReadOnly))
            .with_planes(PlaneInteraction::new().observe_read())
            .with_guards(GuardConfig::new().with_max_latency_ms(100))
    }
}

struct MutatingVerb;

impl VerbCommand for MutatingVerb {
    fn name(&self) -> &'static str {
        "restart"
    }

    fn about(&self) -> &'static str {
        "Restart service"
    }

    fn run(&self, _args: &VerbArgs) -> Result<()> {
        Ok(())
    }
}

impl AutonomicVerbCommand for MutatingVerb {
    fn metadata(&self) -> CommandMetadata {
        CommandMetadata::new()
            .with_effects(
                EffectMetadata::new(EffectType::MutateState).with_sensitivity(Sensitivity::High),
            )
            .with_planes(PlaneInteraction::new().observe_write().invariants_check())
            .with_guards(GuardConfig::new().with_max_latency_ms(500))
    }
}

#[test]
fn test_capabilities() {
    let registry = CommandRegistry::new().name("test-app").about("Test").register_noun(noun!(
        "services",
        "Services",
        [ReadOnlyVerb, MutatingVerb,]
    ));

    let app_metadata = AppMetadata::new("test-app").with_version("1.0.0").with_about("Test");
    let autonomic = AutonomicCli::new(registry, "3.8.0", app_metadata);

    let caps = autonomic.capabilities();
    assert_eq!(caps.cli_version, "3.8.0");
    assert_eq!(caps.schema_version, SCHEMA_VERSION);
    assert!(caps.features.contains(&"introspect".to_string()));
    assert!(caps.features.contains(&"capabilities".to_string()));
}

#[test]
fn test_introspection() {
    let registry = CommandRegistry::new().name("test-app").about("Test").register_noun(noun!(
        "services",
        "Services",
        [ReadOnlyVerb, MutatingVerb,]
    ));

    let app_metadata = AppMetadata::new("test-app").with_version("1.0.0").with_about("Test");
    let autonomic = AutonomicCli::new(registry, "3.8.0", app_metadata);

    let introspection = autonomic.introspect();
    assert_eq!(introspection.cli_version, "3.8.0");
    assert_eq!(introspection.nouns.len(), 1);

    let noun = &introspection.nouns[0];
    assert_eq!(noun.name, "services");
    assert_eq!(noun.verbs.len(), 2);
}

#[test]
fn test_effect_metadata() {
    let effect = EffectMetadata::new(EffectType::ReadOnly);
    assert!(!effect.is_high_risk());

    let effect =
        EffectMetadata::new(EffectType::MutateSecurity).with_sensitivity(Sensitivity::Critical);
    assert!(effect.is_high_risk());
}

#[test]
fn test_plane_interactions() {
    let planes = PlaneInteraction::new().observe_read().ontology_read().invariants_check();

    assert!(planes.interacts_with(Plane::Observations));
    assert!(planes.interacts_with(Plane::Ontology));
    assert!(planes.interacts_with(Plane::Invariants));
    assert!(!planes.interacts_with(Plane::Overlays));
}

#[test]
fn test_plane_interaction_from_str() {
    let planes = PlaneInteraction::from_str("O_read,Σ_read,Q_check");

    assert!(planes.interacts_with(Plane::Observations));
    assert!(planes.interacts_with(Plane::Ontology));
    assert!(planes.interacts_with(Plane::Invariants));
}

#[test]
fn test_guard_config() {
    let guard = GuardConfig::new().with_max_latency_ms(100).with_max_memory_kb(1024);

    assert_eq!(guard.max_latency_ms, Some(100));
    assert_eq!(guard.max_memory_kb, Some(1024));
    assert!(guard.has_guards());
}

#[test]
fn test_guard_result() {
    let result = GuardResult::within_budget(50, 100);
    assert_eq!(result.status, GuardStatus::WithinBudget);
    assert!(!result.is_violated());

    let result = GuardResult::exceeded_budget(150, 100);
    assert_eq!(result.status, GuardStatus::ExceededBudget);
    assert!(result.is_violated());
}

#[test]
fn test_structured_error() {
    let error = StructuredError::invalid_input("Invalid argument");
    assert_eq!(error.kind, ErrorKind::InvalidInput);

    let error = StructuredError::deadline_exceeded(100, 150);
    assert_eq!(error.kind, ErrorKind::DeadlineExceeded);
    assert!(error.details.contains_key("deadline_ms"));
    assert!(error.details.contains_key("actual_ms"));
}

#[test]
fn test_execution_receipt() {
    let receipt = ExecutionReceipt::new("services status")
        .with_duration_ms(50)
        .with_guard(GuardResult::within_budget(50, 100));

    assert_eq!(receipt.command, "services status");
    assert_eq!(receipt.duration_ms, 50);
    assert!(receipt.success);
}

#[test]
fn test_command_graph() {
    let graph = CommandGraph::new()
        .add_node(GraphNode::new("services.status").with_effect("read_only"))
        .add_node(GraphNode::new("services.restart").with_effect("mutate_state"))
        .add_edge(GraphEdge::new("services.restart", "services.status", "precondition"));

    assert_eq!(graph.nodes.len(), 2);
    assert_eq!(graph.edges.len(), 1);
}

#[test]
fn test_introspection_json_serialization() {
    let registry = CommandRegistry::new().name("test-app").about("Test").register_noun(noun!(
        "services",
        "Services",
        [ReadOnlyVerb,]
    ));

    let app_metadata = AppMetadata::new("test-app").with_version("1.0.0");
    let autonomic = AutonomicCli::new(registry, "3.8.0", app_metadata);

    let json = autonomic.introspect_json();
    assert!(json.is_ok());
    assert!(json.unwrap_or_default().contains("\"cli_version\""));
}
