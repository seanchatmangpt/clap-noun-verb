//! Autonomic CLI Example
//!
//! This example demonstrates the autonomic CLI layer features:
//! - Introspection
//! - Capabilities
//! - Effect metadata
//! - Plane interactions
//! - Guards and budgets
//! - Execution receipts
//! - Structured errors

use clap::Parser;
use clap_noun_verb::autonomic::*;
use clap_noun_verb::{noun, CommandRegistry, Result, VerbArgs, VerbCommand};
use serde::Serialize;

// Business logic
#[derive(Serialize, Debug)]
struct ServiceStatus {
    name: String,
    status: String,
    uptime_seconds: u64,
}

#[derive(Serialize, Debug)]
struct ServiceList {
    services: Vec<ServiceStatus>,
}

fn get_service_status() -> ServiceList {
    ServiceList {
        services: vec![
            ServiceStatus {
                name: "web-server".to_string(),
                status: "running".to_string(),
                uptime_seconds: 3600,
            },
            ServiceStatus {
                name: "database".to_string(),
                status: "running".to_string(),
                uptime_seconds: 7200,
            },
        ],
    }
}

fn restart_service(name: String) -> ServiceStatus {
    ServiceStatus { name, status: "restarting".to_string(), uptime_seconds: 0 }
}

// CLI Layer with Autonomic Metadata

/// Status verb with autonomic metadata
struct StatusVerb;

impl VerbCommand for StatusVerb {
    fn name(&self) -> &'static str {
        "status"
    }

    fn about(&self) -> &'static str {
        "Show status of all services"
    }

    fn run(&self, _args: &VerbArgs) -> Result<()> {
        let status = get_service_status();
        println!("{}", serde_json::to_string_pretty(&status).unwrap_or_default());
        Ok(())
    }
}

impl AutonomicVerbCommand for StatusVerb {
    fn metadata(&self) -> CommandMetadata {
        CommandMetadata::new()
            .with_effects(
                EffectMetadata::new(EffectType::ReadOnly).with_sensitivity(Sensitivity::Low),
            )
            .with_planes(PlaneInteraction::new().observe_read().ontology_read())
            .with_guards(GuardConfig::new().with_max_latency_ms(100))
            .with_output_type("ServiceList")
    }
}

/// Restart verb with autonomic metadata
struct RestartVerb;

impl VerbCommand for RestartVerb {
    fn name(&self) -> &'static str {
        "restart"
    }

    fn about(&self) -> &'static str {
        "Restart a service"
    }

    fn run(&self, args: &VerbArgs) -> Result<()> {
        let service = args.get_one_str("service")?;
        let result = restart_service(service);
        println!("{}", serde_json::to_string_pretty(&result).unwrap_or_default());
        Ok(())
    }

    fn build_command(&self) -> clap::Command {
        clap::Command::new(self.name())
            .about(self.about())
            .arg(
                clap::Arg::new("service")
                    .help("Service name to restart")
                    .required(true)
                    .index(1),
            )
    }
}

impl AutonomicVerbCommand for RestartVerb {
    fn metadata(&self) -> CommandMetadata {
        CommandMetadata::new()
            .with_effects(
                EffectMetadata::new(EffectType::MutateState)
                    .with_sensitivity(Sensitivity::High)
                    .with_idempotent(false)
                    .with_required_role("admin"),
            )
            .with_planes(
                PlaneInteraction::new()
                    .observe_write()
                    .ontology_read()
                    .invariants_check(),
            )
            .with_guards(GuardConfig::new().with_max_latency_ms(500))
            .with_output_type("ServiceStatus")
            .with_argument(
                ArgumentMetadata::new("service", "String")
                    .required()
                    .with_help("Service name to restart")
                    .with_index(0),
            )
            .with_precondition("services status")
    }
}

#[derive(Parser)]
#[command(name = "autonomic-demo")]
#[command(about = "Autonomic CLI demonstration", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Show capabilities
    #[arg(long, global = true)]
    capabilities: bool,

    /// Show introspection data
    #[arg(long, global = true)]
    introspect: bool,

    /// Show introspection for a specific noun
    #[arg(long, global = true)]
    introspect_noun: Option<String>,

    /// Show command graph
    #[arg(long, global = true)]
    graph: bool,
}

#[derive(Parser)]
enum Commands {
    Services(ServicesCommand),
}

#[derive(Parser)]
struct ServicesCommand {
    #[command(subcommand)]
    action: ServicesAction,
}

#[derive(Parser)]
enum ServicesAction {
    Status,
    Restart { service: String },
}

fn main() -> Result<()> {
    // Build the registry
    let registry = CommandRegistry::new()
        .name("autonomic-demo")
        .about("Autonomic CLI demonstration")
        .version("1.0.0")
        .register_noun(noun!("services", "Manage services", [
            StatusVerb,
            RestartVerb,
        ]));

    // Create autonomic CLI wrapper
    let app_metadata = AppMetadata::new("autonomic-demo")
        .with_version("1.0.0")
        .with_about("Autonomic CLI demonstration");
    let autonomic = AutonomicCli::new(registry, env!("CARGO_PKG_VERSION"), app_metadata);

    // Parse CLI args
    let cli = Cli::parse();

    // Handle autonomic flags
    if cli.capabilities {
        let caps = autonomic.capabilities();
        println!("{}", serde_json::to_string_pretty(&caps).unwrap_or_default());
        return Ok(());
    }

    if cli.introspect {
        let introspection = autonomic.introspect();
        println!("{}", serde_json::to_string_pretty(&introspection).unwrap_or_default());
        return Ok(());
    }

    if let Some(noun_name) = cli.introspect_noun {
        if let Some(noun_meta) = autonomic.introspect_noun_by_name(&noun_name) {
            println!("{}", serde_json::to_string_pretty(&noun_meta).unwrap_or_default());
        } else {
            eprintln!("Noun '{}' not found", noun_name);
            std::process::exit(1);
        }
        return Ok(());
    }

    if cli.graph {
        let mut graph = CommandGraph::new();

        // Add nodes for all commands
        graph = graph
            .add_node(
                GraphNode::new("services.status")
                    .with_effect("read_only")
                    .with_metadata("sensitivity", "low"),
            )
            .add_node(
                GraphNode::new("services.restart")
                    .with_effect("mutate_state")
                    .with_metadata("sensitivity", "high"),
            );

        // Add edges for preconditions
        graph = graph.add_edge(GraphEdge::new("services.restart", "services.status", "precondition"));

        println!("{}", serde_json::to_string_pretty(&graph).unwrap_or_default());
        return Ok(());
    }

    // Normal command execution
    match cli.command {
        Some(Commands::Services(services)) => match services.action {
            ServicesAction::Status => {
                let status = get_service_status();
                println!("{}", serde_json::to_string_pretty(&status).unwrap_or_default());
            }
            ServicesAction::Restart { service } => {
                let result = restart_service(service);
                println!("{}", serde_json::to_string_pretty(&result).unwrap_or_default());
            }
        },
        None => {
            eprintln!("No command specified. Use --help for usage information.");
            std::process::exit(1);
        }
    }

    Ok(())
}
