//! Swarm-Native CLI Example (2027 Vision)
//!
//! This example demonstrates all swarm-native features:
//! - Stable capability IDs
//! - Multi-agent tenancy and identity
//! - Policy hooks and governance
//! - Input/output schemas for composition
//! - Extended effect declarations
//! - Capability versioning
//! - Streaming and sessions (simulated)
//!
//! Run with:
//! ```
//! cargo run --example swarm_native_2027 -- --introspect
//! cargo run --example swarm_native_2027 -- --capabilities
//! cargo run --example swarm_native_2027 -- services status
//! ```

use clap::Parser;
use clap_noun_verb::autonomic::*;
use clap_noun_verb::{noun, CommandRegistry, Result, VerbArgs, VerbCommand};
use serde::Serialize;

// Business logic types
#[derive(Serialize, Debug)]
struct ServiceStatus {
    service_id: String,
    status: String,
    uptime_seconds: u64,
    cpu_percent: f32,
    memory_mb: u64,
}

#[derive(Serialize, Debug)]
struct ServiceList {
    services: Vec<ServiceStatus>,
    total_count: usize,
}

#[derive(Serialize, Debug)]
struct DeploymentResult {
    deployment_id: String,
    service: String,
    version: String,
    status: String,
    deployed_at: String,
}

// Business logic functions
fn get_service_status() -> ServiceList {
    ServiceList {
        services: vec![
            ServiceStatus {
                service_id: "svc-001".to_string(),
                status: "running".to_string(),
                uptime_seconds: 86400,
                cpu_percent: 12.5,
                memory_mb: 512,
            },
            ServiceStatus {
                service_id: "svc-002".to_string(),
                status: "running".to_string(),
                uptime_seconds: 43200,
                cpu_percent: 8.2,
                memory_mb: 256,
            },
        ],
        total_count: 2,
    }
}

fn deploy_service(service: String, version: String) -> DeploymentResult {
    DeploymentResult {
        deployment_id: uuid::Uuid::new_v4().to_string(),
        service,
        version,
        status: "deploying".to_string(),
        deployed_at: chrono::Utc::now().to_rfc3339(),
    }
}

// CLI Layer with Full Swarm-Native Metadata

/// Status verb - Read-only, safe for all agents
struct StatusVerb;

impl VerbCommand for StatusVerb {
    fn name(&self) -> &'static str {
        "status"
    }

    fn about(&self) -> &'static str {
        "Show status of all services (swarm-safe, read-only)"
    }

    fn run(&self, _args: &VerbArgs) -> Result<()> {
        let status = get_service_status();
        println!("{}", serde_json::to_string_pretty(&status).unwrap_or_default());
        Ok(())
    }
}

impl AutonomicVerbCommand for StatusVerb {
    fn metadata(&self) -> CommandMetadata {
        // Input/output schemas for composition
        let input_schema = InputSchema::new(); // No required inputs

        let output_schema = OutputSchema::new(
            TypeSchema::object({
                let mut props = std::collections::HashMap::new();
                props.insert("services".to_string(), TypeSchema::array(TypeSchema::reference("ServiceStatus")));
                props.insert("total_count".to_string(), TypeSchema::number());
                props
            })
        );

        let composition = CompositionMetadata::new(input_schema, output_schema)
            .produces(Resource::new("observations", "service-status"));

        CommandMetadata::new()
            .with_effects(
                EffectMetadata::new(EffectType::ReadOnly)
                    .with_sensitivity(Sensitivity::Low)
                    .with_isolation(IsolationRequirement::Shared),
            )
            .with_planes(
                PlaneInteraction::new()
                    .observe_read()
                    .ontology_read()
            )
            .with_guards(GuardConfig::new().with_max_latency_ms(100))
            .with_output_type("ServiceList")
            .with_composition(composition)
    }
}

/// Deploy verb - Mutates state, requires elevated permissions
struct DeployVerb;

impl VerbCommand for DeployVerb {
    fn name(&self) -> &'static str {
        "deploy"
    }

    fn about(&self) -> &'static str {
        "Deploy a new service version (requires approval for production)"
    }

    fn run(&self, args: &VerbArgs) -> Result<()> {
        let service = args.get_one_str("service")?;
        let version = args.get_one_str("version")?;

        let result = deploy_service(service, version);
        println!("{}", serde_json::to_string_pretty(&result).unwrap_or_default());
        Ok(())
    }

    fn build_command(&self) -> clap::Command {
        clap::Command::new(self.name())
            .about(self.about())
            .arg(
                clap::Arg::new("service")
                    .help("Service name to deploy")
                    .required(true)
                    .index(1),
            )
            .arg(
                clap::Arg::new("version")
                    .help("Version to deploy (semver)")
                    .required(true)
                    .index(2),
            )
    }
}

impl AutonomicVerbCommand for DeployVerb {
    fn metadata(&self) -> CommandMetadata {
        // Input schema
        let input_schema = InputSchema::new()
            .with_required("service", TypeSchema::string())
            .with_required("version", TypeSchema::string());

        // Output schema
        let output_schema = OutputSchema::new(TypeSchema::reference("DeploymentResult"));

        let composition = CompositionMetadata::new(input_schema, output_schema)
            .consumes(Resource::new("configuration", "service-manifest")
                .with_schema(TypeSchema::object(std::collections::HashMap::new())))
            .produces(Resource::new("deployment", "deployment-record")
                .with_schema(TypeSchema::reference("DeploymentResult")));

        CommandMetadata::new()
            .with_effects(
                EffectMetadata::new(EffectType::MutateState)
                    .with_sensitivity(Sensitivity::High)
                    .with_idempotent(false)
                    .with_required_role("deployer")
                    .with_isolation(IsolationRequirement::Isolated)
                    .supports_dry_run(), // Supports plan-only mode
            )
            .with_planes(
                PlaneInteraction::new()
                    .observe_write()
                    .ontology_read()
                    .invariants_check()
            )
            .with_guards(GuardConfig::new().with_max_latency_ms(5000))
            .with_output_type("DeploymentResult")
            .with_composition(composition)
            .with_argument(
                ArgumentMetadata::new("service", "String")
                    .required()
                    .with_help("Service name to deploy")
                    .with_index(0),
            )
            .with_argument(
                ArgumentMetadata::new("version", "String")
                    .required()
                    .with_help("Version to deploy (semver)")
                    .with_index(1),
            )
            .with_precondition("services status")
    }
}

#[derive(Parser)]
#[command(name = "swarm-cli")]
#[command(about = "Swarm-Native CLI (2027 Vision)", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Show capabilities with stable IDs
    #[arg(long, global = true)]
    capabilities: bool,

    /// Show introspection data with schemas
    #[arg(long, global = true)]
    introspect: bool,

    /// Show capability changelog
    #[arg(long, global = true)]
    changelog: bool,

    /// Simulate agent context
    #[arg(long, global = true)]
    agent_id: Option<String>,

    /// Simulate tenant context
    #[arg(long, global = true)]
    tenant_id: Option<String>,

    /// Dry-run mode (plan only)
    #[arg(long, global = true)]
    dry_run: bool,
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
    Deploy {
        service: String,
        version: String,
    },
}

fn main() -> Result<()> {
    // Build the registry
    let registry = CommandRegistry::new()
        .name("swarm-cli")
        .about("Swarm-Native CLI demonstration for 2027")
        .version("2.0.0")
        .register_noun(noun!("services", "Manage services with swarm-native features", [
            StatusVerb,
            DeployVerb,
        ]));

    // Create autonomic CLI wrapper
    let app_metadata = AppMetadata::new("swarm-cli")
        .with_version("2.0.0")
        .with_about("Swarm-Native CLI demonstration for 2027");

    let autonomic = AutonomicCli::new(registry, "2.0.0", app_metadata);

    // Parse CLI args
    let cli = Cli::parse();

    // Handle swarm-native flags
    if cli.capabilities {
        let caps = autonomic.capabilities();
        println!("=== SWARM-NATIVE CAPABILITIES (2027) ===\n");
        println!("{}", serde_json::to_string_pretty(&caps).unwrap_or_default());
        println!("\nSupported features:");
        for feature in SUPPORTED_FEATURES {
            println!("  - {}", feature);
        }
        return Ok(());
    }

    if cli.introspect {
        let introspection = autonomic.introspect();
        println!("=== SWARM-NATIVE INTROSPECTION (2027) ===\n");
        println!("Schema version: {}", SCHEMA_VERSION);
        println!("\n{}", serde_json::to_string_pretty(&introspection).unwrap_or_default());
        return Ok(());
    }

    if cli.changelog {
        println!("=== CAPABILITY CHANGELOG ===\n");

        let changelog = CapabilityChangelog::new("2.0.0")
            .add_change(
                CapabilityChange::new(
                    CapabilityId::from_path("services.status"),
                    ChangeType::Extension,
                    "2.0.0",
                    "Added composition metadata for swarm workflows",
                )
            )
            .add_change(
                CapabilityChange::new(
                    CapabilityId::from_path("services.deploy"),
                    ChangeType::Addition,
                    "2.0.0",
                    "New deploy verb with multi-agent tenancy support",
                )
            );

        println!("{}", serde_json::to_string_pretty(&changelog).unwrap_or_default());
        return Ok(());
    }

    // Demonstrate multi-agent context
    if let (Some(agent_id), Some(tenant_id)) = (cli.agent_id.clone(), cli.tenant_id.clone()) {
        let agent = AgentIdentity::new(agent_id, "code-assistant")
            .with_version("1.0.0");

        let tenant = TenantIdentity::new(tenant_id)
            .with_environment("production");

        let context = InvocationContext::new(agent, tenant)
            .with_qos(QoSHints::new().with_priority(PriorityClass::High));

        println!("=== INVOCATION CONTEXT ===");
        println!("{}\n", serde_json::to_string_pretty(&context).unwrap_or_default());
    }

    // Demonstrate policy evaluation
    if cli.dry_run {
        println!("=== DRY-RUN MODE (Policy Evaluation) ===\n");

        let policy_engine = RuleBasedPolicyEngine::new("production-policy");

        let agent = AgentIdentity::anonymous();
        let tenant = TenantIdentity::default_tenant();
        let ctx = InvocationContext::new(agent, tenant);

        let request = PolicyRequest::new(
            ctx,
            "services",
            "status",
            std::collections::HashMap::new(),
            EffectMetadata::new(EffectType::ReadOnly),
        ).dry_run();

        match policy_engine.evaluate(&request) {
            Ok(result) => {
                println!("Policy decision: {}", if result.is_allowed() { "✓ ALLOW" } else { "✗ DENY" });
                println!("{}", serde_json::to_string_pretty(&result).unwrap_or_default());
            }
            Err(e) => eprintln!("Policy evaluation error: {}", e),
        }
        return Ok(());
    }

    // Normal command execution
    match cli.command {
        Some(Commands::Services(services)) => match services.action {
            ServicesAction::Status => {
                let status = get_service_status();

                // Create an execution receipt
                let receipt = ExecutionReceipt::new("services.status")
                    .with_duration_ms(15)
                    .with_guard(GuardResult::within_budget(15, 100))
                    .with_result_hash(
                        ExecutionReceipt::compute_hash(&status).unwrap_or_default()
                    );

                println!("=== EXECUTION RESULT ===");
                println!("{}\n", serde_json::to_string_pretty(&status).unwrap_or_default());

                println!("=== EXECUTION RECEIPT ===");
                println!("{}", serde_json::to_string_pretty(&receipt).unwrap_or_default());
            }
            ServicesAction::Deploy { service, version } => {
                let result = deploy_service(service.clone(), version.clone());

                // Create an execution receipt for a mutating operation
                let receipt = ExecutionReceipt::new("services.deploy")
                    .with_duration_ms(450)
                    .with_guard(GuardResult::within_budget(450, 5000));

                println!("=== DEPLOYMENT INITIATED ===");
                println!("{}\n", serde_json::to_string_pretty(&result).unwrap_or_default());

                println!("=== EXECUTION RECEIPT ===");
                println!("{}", serde_json::to_string_pretty(&receipt).unwrap_or_default());
            }
        },
        None => {
            eprintln!("No command specified. Try:");
            eprintln!("  --capabilities    Show swarm-native capabilities");
            eprintln!("  --introspect      Show full introspection data");
            eprintln!("  --changelog       Show capability changelog");
            eprintln!("  services status   Get service status");
            eprintln!("  services deploy <service> <version>   Deploy a service");
            std::process::exit(1);
        }
    }

    Ok(())
}
