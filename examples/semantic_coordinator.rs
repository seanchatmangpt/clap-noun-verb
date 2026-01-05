//! Semantic Agent Coordinator CLI Example
//!
//! Demonstrates the complete semantic agent coordinator system with:
//! - Type-state machine for agent lifecycle
//! - Semantic discovery via RDF/SPARQL
//! - Swarm coordination with Byzantine fault tolerance
//! - MAPE-K autonomic loops
//!
//! # Usage
//!
//! ```bash
//! # Register agent
//! cargo run --example semantic_coordinator --all-features -- agent register agent-001 nlp,vision
//!
//! # Discover agents
//! cargo run --example semantic_coordinator --all-features -- agent discover nlp
//!
//! # Execute task via auction
//! cargo run --example semantic_coordinator --all-features -- agent execute task-001 nlp
//!
//! # Check swarm health
//! cargo run --example semantic_coordinator --all-features -- agent health
//!
//! # Introspect agent semantics
//! cargo run --example semantic_coordinator --all-features -- agent introspect agent-001
//!
//! # Trigger swarm coordination
//! cargo run --example semantic_coordinator --all-features -- swarm coordinate
//!
//! # Trigger autonomic tuning
//! cargo run --example semantic_coordinator --all-features -- autonomic tune
//! ```

use clap::{Parser, Subcommand};
use clap_noun_verb::agents::*;
use serde_json::json;

#[derive(Parser)]
#[command(name = "semantic-coordinator")]
#[command(about = "Semantic Agent Coordinator CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Agent lifecycle and discovery commands
    Agent {
        #[command(subcommand)]
        action: AgentCommands,
    },

    /// Swarm coordination commands
    Swarm {
        #[command(subcommand)]
        action: SwarmCommands,
    },

    /// Autonomic loop commands
    Autonomic {
        #[command(subcommand)]
        action: AutonomicCommands,
    },
}

#[derive(Subcommand)]
enum AgentCommands {
    /// Register agent with capabilities
    Register {
        /// Agent ID
        agent_id: String,

        /// Comma-separated capabilities
        capabilities: String,
    },

    /// Discover agents by capability
    Discover {
        /// Capability to search for
        capability: String,
    },

    /// Execute task via auction
    Execute {
        /// Task ID
        task_id: String,

        /// Required capability
        capability: String,
    },

    /// Check swarm health
    Health,

    /// Introspect agent semantics (RDF export)
    Introspect {
        /// Agent ID
        agent_id: String,
    },
}

#[derive(Subcommand)]
enum SwarmCommands {
    /// Trigger swarm coordination cycle
    Coordinate,

    /// Show Byzantine detection status
    Byzantine {
        /// Agent ID
        agent_id: String,
    },
}

#[derive(Subcommand)]
enum AutonomicCommands {
    /// Trigger autonomic tuning cycle
    Tune,

    /// Show current metrics
    Metrics,
}

// Global state (in production, this would be persistent)
struct CoordinatorState {
    swarm: SwarmCoordinator,
    #[cfg(feature = "rdf")]
    semantic: SemanticDiscovery,
    #[cfg(feature = "autonomic")]
    autonomic: AutonomicLoop,
}

impl Default for CoordinatorState {
    fn default() -> Self {
        Self::new()
    }
}

impl CoordinatorState {
    fn new() -> Self {
        Self {
            swarm: SwarmCoordinator::new(),
            #[cfg(feature = "rdf")]
            semantic: SemanticDiscovery::new(),
            #[cfg(feature = "autonomic")]
            autonomic: AutonomicLoop::new(),
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let mut state = CoordinatorState::new();

    match cli.command {
        Commands::Agent { action } => handle_agent_command(action, &mut state)?,
        Commands::Swarm { action } => handle_swarm_command(action, &mut state)?,
        Commands::Autonomic { action } => handle_autonomic_command(action, &mut state)?,
    }

    Ok(())
}

fn handle_agent_command(
    command: AgentCommands,
    state: &mut CoordinatorState,
) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        AgentCommands::Register { agent_id, capabilities } => {
            let caps: Vec<String> = capabilities.split(',').map(|s| s.trim().to_string()).collect();

            // Create agent with type-state machine
            let agent = AgentState::<Unregistered>::new(agent_id.clone())
                .register(caps.clone())
                .verify(b"proof-data")
                .trust(0.8)?;

            // Register with swarm
            let agent_info = AgentInfo::new(&agent_id, caps.clone());
            state.swarm.register_agent(agent_info);

            // Register semantically (if RDF feature enabled)
            #[cfg(feature = "rdf")]
            {
                let semantic_caps: Vec<Capability> =
                    caps.iter().map(|c| Capability::new(c, format!("{} capability", c))).collect();

                state.semantic.register_agent(&agent_id, semantic_caps);
            }

            let output = json!({
                "status": "success",
                "agent_id": agent_id,
                "trust_score": agent.get_trust_score(),
                "message": "Agent registered successfully"
            });

            println!("{}", serde_json::to_string_pretty(&output)?);
        }

        AgentCommands::Discover { capability } => {
            #[cfg(feature = "rdf")]
            {
                let query =
                    SparqlQueryBuilder::new().select_agents_with_capability(&capability).build();

                let results = state.semantic.query(&query)?;

                let output = json!({
                    "status": "success",
                    "capability": capability,
                    "agents": results,
                    "count": results.len()
                });

                println!("{}", serde_json::to_string_pretty(&output)?);
            }

            #[cfg(not(feature = "rdf"))]
            {
                eprintln!("RDF feature not enabled. Build with --features rdf");
                std::process::exit(1);
            }
        }

        AgentCommands::Execute { task_id, capability } => {
            let auction = TaskAuction::new(task_id.clone(), vec![capability.clone()], 1.0);
            let winner = state.swarm.run_auction(auction)?;

            // Simulate task execution (success for demo)
            state.swarm.record_task_result(&winner, true)?;

            let output = json!({
                "status": "success",
                "task_id": task_id,
                "winner": winner,
                "capability": capability,
                "result": "completed"
            });

            println!("{}", serde_json::to_string_pretty(&output)?);
        }

        AgentCommands::Health => {
            let metrics = state.swarm.health_metrics();

            let output = json!({
                "status": "success",
                "health": {
                    "total_agents": metrics.total_agents,
                    "trusted_agents": metrics.trusted_agents,
                    "average_trust": format!("{:.3}", metrics.average_trust),
                    "health_score": if metrics.total_agents > 0 {
                        metrics.trusted_agents as f64 / metrics.total_agents as f64
                    } else {
                        0.0
                    }
                }
            });

            println!("{}", serde_json::to_string_pretty(&output)?);
        }

        AgentCommands::Introspect { agent_id } => {
            #[cfg(feature = "rdf")]
            {
                let triples = state.semantic.get_agent_triples(&agent_id);

                let output = json!({
                    "status": "success",
                    "agent_id": agent_id,
                    "rdf_triples": triples,
                    "count": triples.len()
                });

                println!("{}", serde_json::to_string_pretty(&output)?);
            }

            #[cfg(not(feature = "rdf"))]
            {
                eprintln!("RDF feature not enabled. Build with --features rdf");
                std::process::exit(1);
            }
        }
    }

    Ok(())
}

fn handle_swarm_command(
    command: SwarmCommands,
    state: &mut CoordinatorState,
) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        SwarmCommands::Coordinate => {
            let metrics = state.swarm.health_metrics();

            let output = json!({
                "status": "success",
                "message": "Swarm coordination cycle completed",
                "metrics": {
                    "total_agents": metrics.total_agents,
                    "trusted_agents": metrics.trusted_agents,
                    "average_trust": format!("{:.3}", metrics.average_trust)
                }
            });

            println!("{}", serde_json::to_string_pretty(&output)?);
        }

        SwarmCommands::Byzantine { agent_id } => {
            let is_byzantine = state.swarm.check_byzantine(&agent_id, true);

            let output = json!({
                "status": "success",
                "agent_id": agent_id,
                "is_byzantine": is_byzantine,
                "message": if is_byzantine {
                    "Agent detected as potentially Byzantine"
                } else {
                    "Agent appears trustworthy"
                }
            });

            println!("{}", serde_json::to_string_pretty(&output)?);
        }
    }

    Ok(())
}

fn handle_autonomic_command(
    command: AutonomicCommands,
    state: &mut CoordinatorState,
) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "autonomic")]
    {
        match command {
            AutonomicCommands::Tune => {
                // Simulate some metrics
                state.autonomic.record_metric("response_time_ms", 120.0);
                state.autonomic.record_metric("cpu_usage_pct", 65.0);
                state.autonomic.record_metric("memory_usage_mb", 512.0);

                // Run MAPE-K cycle
                let action_count = state.autonomic.run_cycle()?;

                let output = json!({
                    "status": "success",
                    "message": "Autonomic tuning cycle completed",
                    "actions_executed": action_count,
                    "phase": format!("{:?}", state.autonomic.current_phase())
                });

                println!("{}", serde_json::to_string_pretty(&output)?);
            }

            AutonomicCommands::Metrics => {
                let output = json!({
                    "status": "success",
                    "message": "Current autonomic metrics",
                    "phase": format!("{:?}", state.autonomic.current_phase())
                });

                println!("{}", serde_json::to_string_pretty(&output)?);
            }
        }
    }

    #[cfg(not(feature = "autonomic"))]
    {
        eprintln!("Autonomic feature not enabled. Build with --features autonomic");
        std::process::exit(1);
    }

    Ok(())
}
