//! End-to-end benchmark: Agent generates complete noun-verb CLI
//!
//! Measures actual time from empty state to fully built and tested CLI
//! Parameters: noun_count, verb_count (e.g., 8 nouns × 8 verbs = 64 commands)

use clap_noun_verb::agent_cli::{
    AgentCliBuilder, AgentResult, CommandArgs, CommandHandler, CommandMetadata,
};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;

// ============================================================================
// Dynamic Handler Factory
// ============================================================================

/// Factory for creating handlers dynamically
struct HandlerFactory;

impl HandlerFactory {
    /// Create a new handler for a noun-verb pair
    fn create(noun: &str, verb: &str) -> Arc<dyn CommandHandler> {
        Arc::new(DynamicHandler { noun: noun.to_string(), verb: verb.to_string() })
    }
}

/// Dynamically created handler for any noun-verb pair
#[derive(Clone)]
struct DynamicHandler {
    noun: String,
    verb: String,
}

impl CommandHandler for DynamicHandler {
    fn execute(&self, args: &CommandArgs) -> AgentResult<serde_json::Value> {
        Ok(json!({
            "noun": self.noun,
            "verb": self.verb,
            "arg_count": args.len(),
            "status": "executed"
        }))
    }

    fn metadata(&self) -> CommandMetadata {
        CommandMetadata {
            name: format!("{}-{}", self.noun, self.verb),
            description: format!("Execute {} on {}", self.verb, self.noun),
            arguments: Vec::new(),
            requires_args: false,
        }
    }
}

// ============================================================================
// Agent CLI Generation Benchmark
// ============================================================================

/// Benchmark: Agent generates CLI with N nouns × M verbs
fn agent_generates_noun_verb_cli(c: &mut Criterion) {
    let mut group = c.benchmark_group("agent_cli_generation");
    group.sample_size(10); // Fewer samples since each iteration is slower

    // Test case: 8 nouns × 8 verbs = 64 total commands
    let noun_count = black_box(8);
    let verb_count = black_box(8);

    group.bench_with_input(
        BenchmarkId::new(
            "generate_build_execute",
            format!("{}x{}_commands", noun_count, verb_count),
        ),
        &(noun_count, verb_count),
        |b, &(nouns, verbs)| {
            b.iter(|| {
                // 1. Agent creates builder
                let mut builder = AgentCliBuilder::new(
                    "agent-generated-cli",
                    "CLI generated dynamically by agent",
                );

                // 2. Agent generates and registers commands
                for n in 0..nouns {
                    for v in 0..verbs {
                        let noun_name = format!("noun{}", n);
                        let verb_name = format!("verb{}", v);
                        let cmd_name = format!("{}-{}", noun_name, verb_name);
                        let description = format!("Action: {} on {}", verb_name, noun_name);

                        let handler = HandlerFactory::create(&noun_name, &verb_name);
                        builder.register_command(&cmd_name, &description, handler).ok();
                    }
                }

                // 3. Agent builds CLI
                let cli = builder.build().ok();

                // 4. Agent discovers all commands
                if let Some(cli) = cli {
                    let all_commands = cli.commands();

                    // 5. Agent executes sample commands (round-robin)
                    for (idx, cmd_name) in all_commands.iter().enumerate() {
                        if idx % 8 == 0 {
                            // Execute every 8th command
                            let args = CommandArgs::new()
                                .with_arg("execution_id", format!("exec_{}", idx));
                            let _ = cli.execute(cmd_name, args);
                        }
                    }
                }
            });
        },
    );

    group.finish();
}

// ============================================================================
// Scaling Benchmark - Test with different sizes
// ============================================================================

/// Benchmark CLI generation with different noun/verb combinations
fn agent_cli_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("agent_cli_scaling");
    group.sample_size(10);

    // Test multiple configurations
    let configs = vec![
        (2, 2),   // 4 commands
        (4, 4),   // 16 commands
        (8, 8),   // 64 commands (target scenario)
        (10, 10), // 100 commands
    ];

    for (noun_count, verb_count) in configs {
        group.bench_with_input(
            BenchmarkId::new("full_workflow", format!("{}x{}_commands", noun_count, verb_count)),
            &(noun_count, verb_count),
            |b, &(nouns, verbs)| {
                b.iter(|| {
                    let mut builder =
                        AgentCliBuilder::new("scaling-test-cli", "CLI for scaling tests");

                    for n in 0..nouns {
                        for v in 0..verbs {
                            let noun_name = format!("noun{}", n);
                            let verb_name = format!("verb{}", v);
                            let cmd_name = format!("{}-{}", noun_name, verb_name);

                            let handler = HandlerFactory::create(&noun_name, &verb_name);
                            builder
                                .register_command(
                                    &cmd_name,
                                    &format!("{} {}", verb_name, noun_name),
                                    handler,
                                )
                                .ok();
                        }
                    }

                    if let Ok(cli) = builder.build() {
                        let cmds = cli.commands();

                        // Execute sample
                        for (idx, cmd) in cmds.iter().enumerate() {
                            if idx < 3 {
                                let _ = cli.execute(cmd, CommandArgs::new());
                            }
                        }
                    }
                });
            },
        );
    }

    group.finish();
}

// ============================================================================
// Batch Generation - Simulate agent creating multiple CLIs
// ============================================================================

/// Benchmark creating 10 separate CLIs with 8×8 commands each
fn agent_batch_cli_generation(c: &mut Criterion) {
    c.bench_function("batch_generate_10_separate_clis", |b| {
        b.iter(|| {
            let mut clis = Vec::new();

            // Agent creates 10 separate CLIs
            for cli_num in 0..10 {
                let mut builder = AgentCliBuilder::new(
                    &format!("cli-{}", cli_num),
                    &format!("Generated CLI #{}", cli_num),
                );

                // Each CLI gets 8×8 commands
                for n in 0..8 {
                    for v in 0..8 {
                        let noun_name = format!("noun{}", n);
                        let verb_name = format!("verb{}", v);
                        let cmd_name = format!("{}-{}", noun_name, verb_name);

                        let handler = HandlerFactory::create(&noun_name, &verb_name);
                        builder
                            .register_command(
                                &cmd_name,
                                &format!("{} {}", verb_name, noun_name),
                                handler,
                            )
                            .ok();
                    }
                }

                if let Ok(cli) = builder.build() {
                    clis.push(cli);
                }
            }

            // Execute one command from each CLI
            for (_idx, cli) in clis.iter().enumerate() {
                if let Some(first_cmd) = cli.commands().first() {
                    let _ = cli.execute(first_cmd, CommandArgs::new());
                }
            }
        });
    });
}

// ============================================================================
// Complex Workflow - Simulating real agent coordination
// ============================================================================

/// Benchmark realistic agent workflow: build → discover → execute → analyze
fn agent_realistic_workflow(c: &mut Criterion) {
    c.bench_function("realistic_agent_workflow_8x8", |b| {
        b.iter(|| {
            // Phase 1: CLI Generation
            let mut builder = AgentCliBuilder::new(
                "realistic-cli",
                "CLI built by agent following realistic workflow",
            );

            let mut command_map: HashMap<String, String> = HashMap::new();

            for n in 0..8 {
                for v in 0..8 {
                    let noun_name = format!("noun{}", n);
                    let verb_name = format!("verb{}", v);
                    let cmd_name = format!("{}-{}", noun_name, verb_name);

                    let handler = HandlerFactory::create(&noun_name, &verb_name);
                    builder
                        .register_command(
                            &cmd_name,
                            &format!("{} on {}", verb_name, noun_name),
                            handler,
                        )
                        .ok();

                    command_map.insert(cmd_name, format!("{}:{}", n, v));
                }
            }

            if let Ok(cli) = builder.build() {
                // Phase 2: Discovery
                let discovered_commands = cli.commands();

                // Phase 3: Selective Execution (every noun×verb combination once)
                let mut execution_results = Vec::new();
                for (idx, cmd_name) in discovered_commands.iter().enumerate() {
                    let args = CommandArgs::new()
                        .with_arg("index", format!("{}", idx))
                        .with_arg("command", cmd_name.to_string());

                    if let Ok(result) = cli.execute(cmd_name, args) {
                        execution_results.push(result);
                    }
                }

                // Phase 4: Analysis (simulate agent analyzing results)
                let total_executions = execution_results.len();
                let _summary = json!({
                    "total_commands": discovered_commands.len(),
                    "executed": total_executions,
                    "success_rate": if total_executions > 0 { 1.0 } else { 0.0 }
                });
            }
        });
    });
}

// ============================================================================
// Incremental Growth - Simulate agent adding commands dynamically
// ============================================================================

/// Benchmark simulating agent progressively building CLI
fn agent_incremental_cli_growth(c: &mut Criterion) {
    c.bench_function("incremental_growth_8x8_total", |b| {
        b.iter(|| {
            let mut builder =
                AgentCliBuilder::new("incremental-cli", "CLI built incrementally by agent");

            // Simulate agent adding 1 noun at a time
            for n in 0..8 {
                // For each noun, add all 8 verbs
                for v in 0..8 {
                    let noun_name = format!("noun{}", n);
                    let verb_name = format!("verb{}", v);
                    let cmd_name = format!("{}-{}", noun_name, verb_name);

                    let handler = HandlerFactory::create(&noun_name, &verb_name);
                    builder
                        .register_command(
                            &cmd_name,
                            &format!("{} {}", verb_name, noun_name),
                            handler,
                        )
                        .ok();
                }

                // After each noun, agent could validate/check
                let cmd_count = ((n + 1) * 8) as usize;
                // Simulate validation check
                let _ = cmd_count;
            }

            // Final build and execution
            if let Ok(cli) = builder.build() {
                let commands = cli.commands();
                for cmd in commands.iter().take(3) {
                    let _ = cli.execute(cmd, CommandArgs::new());
                }
            }
        });
    });
}

// ============================================================================
// Benchmark Groups
// ============================================================================

criterion_group!(
    benches,
    agent_generates_noun_verb_cli,
    agent_cli_scaling,
    agent_batch_cli_generation,
    agent_realistic_workflow,
    agent_incremental_cli_growth,
);

criterion_main!(benches);
