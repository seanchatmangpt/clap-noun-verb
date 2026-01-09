//! Benchmarks for MCP Agent CLI Creation
//!
//! Measures real end-to-end performance for each JTBD (Job To Be Done):
//! 1. Builder Initialization - Agent discovers and creates CLI builder
//! 2. Command Registration - Agent registers commands dynamically
//! 3. CLI Building - Agent finalizes the CLI configuration
//! 4. Command Execution - Agent executes a registered command
//! 5. Command Discovery - Agent discovers existing commands
//! 6. Command Chaining - Agent executes multiple commands in sequence
//!
//! Performance SLOs: ≤100ms end-to-end CLI execution

use clap_noun_verb::agent_cli::{
    AgentCliBuilder, AgentResult, CommandArgs, CommandHandler, CommandMetadata,
};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use serde_json::json;
use std::sync::Arc;

// ============================================================================
// Test Handlers
// ============================================================================

/// Simple echo handler for benchmarks
#[derive(Clone)]
struct EchoHandler {
    message: String,
}

impl EchoHandler {
    fn new(message: &str) -> Arc<Self> {
        Arc::new(Self { message: message.to_string() })
    }
}

impl CommandHandler for EchoHandler {
    fn execute(&self, args: &CommandArgs) -> AgentResult<serde_json::Value> {
        Ok(json!({
            "output": self.message,
            "args_count": args.len(),
            "is_empty": args.is_empty(),
        }))
    }

    fn metadata(&self) -> CommandMetadata {
        CommandMetadata {
            name: "echo".to_string(),
            description: "Echo test handler".to_string(),
            arguments: Vec::new(),
            requires_args: false,
        }
    }
}

/// Counter handler that increments
#[derive(Clone)]
struct CounterHandler;

impl CommandHandler for CounterHandler {
    fn execute(&self, _args: &CommandArgs) -> AgentResult<serde_json::Value> {
        Ok(json!({"count": 42}))
    }

    fn metadata(&self) -> CommandMetadata {
        CommandMetadata {
            name: "count".to_string(),
            description: "Counter handler".to_string(),
            arguments: Vec::new(),
            requires_args: false,
        }
    }
}

// ============================================================================
// JTBD 1: Builder Initialization
// ============================================================================

fn jtbd_builder_initialization(c: &mut Criterion) {
    c.bench_function("jtbd_1_builder_initialization", |b| {
        b.iter(|| {
            AgentCliBuilder::new(black_box("test-cli"), black_box("Test CLI for agent discovery"))
        });
    });
}

// ============================================================================
// JTBD 2: Command Registration
// ============================================================================

fn jtbd_command_registration(c: &mut Criterion) {
    let mut group = c.benchmark_group("jtbd_2_command_registration");

    // Single command registration
    group.bench_function("register_single_command", |b| {
        b.iter(|| {
            let mut builder = AgentCliBuilder::new("test-cli", "Test CLI");
            let handler = EchoHandler::new("test");
            builder.register_command("echo", "Echo command", handler).ok();
        });
    });

    // Register 5 commands
    group.bench_with_input(BenchmarkId::from_parameter("register_5_commands"), &5, |b, &n| {
        b.iter(|| {
            let mut builder = AgentCliBuilder::new("test-cli", "Test CLI");
            for i in 0..n {
                let handler = EchoHandler::new(&format!("handler_{}", i));
                let cmd_name = format!("cmd_{}", i);
                builder.register_command(&cmd_name, &format!("Command {}", i), handler).ok();
            }
        });
    });

    // Register 20 commands
    group.bench_with_input(BenchmarkId::from_parameter("register_20_commands"), &20, |b, &n| {
        b.iter(|| {
            let mut builder = AgentCliBuilder::new("test-cli", "Test CLI");
            for i in 0..n {
                let handler = EchoHandler::new(&format!("handler_{}", i));
                let cmd_name = format!("cmd_{}", i);
                builder.register_command(&cmd_name, &format!("Command {}", i), handler).ok();
            }
        });
    });

    group.finish();
}

// ============================================================================
// JTBD 3: CLI Building
// ============================================================================

fn jtbd_cli_building(c: &mut Criterion) {
    let mut group = c.benchmark_group("jtbd_3_cli_building");

    // Build with 1 command
    group.bench_with_input(BenchmarkId::from_parameter("build_1_command"), &1, |b, &n| {
        b.iter_batched(
            || {
                let mut builder = AgentCliBuilder::new("test-cli", "Test CLI");
                for i in 0..n {
                    let handler = EchoHandler::new(&format!("handler_{}", i));
                    let cmd_name = format!("cmd_{}", i);
                    builder.register_command(&cmd_name, &format!("Command {}", i), handler).ok();
                }
                builder
            },
            |builder| builder.build(),
            criterion::BatchSize::SmallInput,
        );
    });

    // Build with 5 commands
    group.bench_with_input(BenchmarkId::from_parameter("build_5_commands"), &5, |b, &n| {
        b.iter_batched(
            || {
                let mut builder = AgentCliBuilder::new("test-cli", "Test CLI");
                for i in 0..n {
                    let handler = EchoHandler::new(&format!("handler_{}", i));
                    let cmd_name = format!("cmd_{}", i);
                    builder.register_command(&cmd_name, &format!("Command {}", i), handler).ok();
                }
                builder
            },
            |builder| builder.build(),
            criterion::BatchSize::SmallInput,
        );
    });

    // Build with 20 commands
    group.bench_with_input(BenchmarkId::from_parameter("build_20_commands"), &20, |b, &n| {
        b.iter_batched(
            || {
                let mut builder = AgentCliBuilder::new("test-cli", "Test CLI");
                for i in 0..n {
                    let handler = EchoHandler::new(&format!("handler_{}", i));
                    let cmd_name = format!("cmd_{}", i);
                    builder.register_command(&cmd_name, &format!("Command {}", i), handler).ok();
                }
                builder
            },
            |builder| builder.build(),
            criterion::BatchSize::SmallInput,
        );
    });

    group.finish();
}

// ============================================================================
// JTBD 4: Command Execution
// ============================================================================

fn jtbd_command_execution(c: &mut Criterion) {
    let mut group = c.benchmark_group("jtbd_4_command_execution");

    // Execute with no arguments
    group.bench_function("execute_no_args", |b| {
        b.iter_batched(
            || {
                let mut builder = AgentCliBuilder::new("test-cli", "Test CLI");
                let handler = EchoHandler::new("test");
                builder.register_command("echo", "Echo command", handler).ok();
                builder.build().ok().unwrap()
            },
            |cli| {
                let args = CommandArgs::new();
                cli.execute("echo", args)
            },
            criterion::BatchSize::SmallInput,
        );
    });

    // Execute with named arguments
    group.bench_function("execute_with_named_args", |b| {
        b.iter_batched(
            || {
                let mut builder = AgentCliBuilder::new("test-cli", "Test CLI");
                let handler = EchoHandler::new("test");
                builder.register_command("echo", "Echo command", handler).ok();
                builder.build().ok().unwrap()
            },
            |cli| {
                let args = CommandArgs::new().with_arg("key1", "value1").with_arg("key2", "value2");
                cli.execute("echo", args)
            },
            criterion::BatchSize::SmallInput,
        );
    });

    // Execute with positional arguments
    group.bench_function("execute_with_positional_args", |b| {
        b.iter_batched(
            || {
                let mut builder = AgentCliBuilder::new("test-cli", "Test CLI");
                let handler = EchoHandler::new("test");
                builder.register_command("echo", "Echo command", handler).ok();
                builder.build().ok().unwrap()
            },
            |cli| {
                let args = CommandArgs::new().with_positional("arg1").with_positional("arg2");
                cli.execute("echo", args)
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.finish();
}

// ============================================================================
// JTBD 5: Command Discovery
// ============================================================================

fn jtbd_command_discovery(c: &mut Criterion) {
    let mut group = c.benchmark_group("jtbd_5_command_discovery");

    // Discover commands in small CLI (5 commands)
    group.bench_with_input(BenchmarkId::from_parameter("discover_5_commands"), &5, |b, &n| {
        b.iter_batched(
            || {
                let mut builder = AgentCliBuilder::new("test-cli", "Test CLI");
                for i in 0..n {
                    let handler = EchoHandler::new(&format!("handler_{}", i));
                    let cmd_name = format!("cmd_{}", i);
                    builder.register_command(&cmd_name, &format!("Command {}", i), handler).ok();
                }
                builder.build().ok().unwrap()
            },
            |cli| {
                let cmd_list = cli.commands();
                for cmd in cmd_list {
                    let _ = cli.command_info(cmd);
                }
            },
            criterion::BatchSize::SmallInput,
        );
    });

    // Discover commands in large CLI (20 commands)
    group.bench_with_input(BenchmarkId::from_parameter("discover_20_commands"), &20, |b, &n| {
        b.iter_batched(
            || {
                let mut builder = AgentCliBuilder::new("test-cli", "Test CLI");
                for i in 0..n {
                    let handler = EchoHandler::new(&format!("handler_{}", i));
                    let cmd_name = format!("cmd_{}", i);
                    builder.register_command(&cmd_name, &format!("Command {}", i), handler).ok();
                }
                builder.build().ok().unwrap()
            },
            |cli| {
                let cmd_list = cli.commands();
                for cmd in cmd_list {
                    let _ = cli.command_info(cmd);
                }
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.finish();
}

// ============================================================================
// JTBD 6: Command Chaining
// ============================================================================

fn jtbd_command_chaining(c: &mut Criterion) {
    let mut group = c.benchmark_group("jtbd_6_command_chaining");

    // Chain 2 commands
    group.bench_with_input(BenchmarkId::from_parameter("chain_2_commands"), &2, |b, &n| {
        b.iter_batched(
            || {
                let mut builder = AgentCliBuilder::new("test-cli", "Test CLI");
                for i in 0..n {
                    let handler = EchoHandler::new(&format!("handler_{}", i));
                    let cmd_name = format!("cmd_{}", i);
                    builder.register_command(&cmd_name, &format!("Command {}", i), handler).ok();
                }
                builder.build().ok().unwrap()
            },
            |cli| {
                // Execute first command
                let args = CommandArgs::new();
                let result1 = cli.execute("cmd_0", args);

                // Execute second command (simulating chain)
                let args2 = CommandArgs::new();
                let result2 = cli.execute("cmd_1", args2);

                (result1, result2)
            },
            criterion::BatchSize::SmallInput,
        );
    });

    // Chain 5 commands
    group.bench_with_input(BenchmarkId::from_parameter("chain_5_commands"), &5, |b, &n| {
        b.iter_batched(
            || {
                let mut builder = AgentCliBuilder::new("test-cli", "Test CLI");
                for i in 0..n {
                    let handler = EchoHandler::new(&format!("handler_{}", i));
                    let cmd_name = format!("cmd_{}", i);
                    builder.register_command(&cmd_name, &format!("Command {}", i), handler).ok();
                }
                builder.build().ok().unwrap()
            },
            |cli| {
                for i in 0..n {
                    let cmd_name = format!("cmd_{}", i);
                    let args = CommandArgs::new();
                    let _ = cli.execute(&cmd_name, args);
                }
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.finish();
}

// ============================================================================
// End-to-End Workflow Benchmark
// ============================================================================

fn end_to_end_workflow(c: &mut Criterion) {
    c.bench_function("end_to_end_complete_workflow", |b| {
        b.iter(|| {
            // 1. Initialize builder
            let mut builder = AgentCliBuilder::new("e2e-cli", "End-to-end test CLI");

            // 2. Register commands
            for i in 0..5 {
                let handler = EchoHandler::new(&format!("handler_{}", i));
                let cmd_name = format!("cmd_{}", i);
                builder.register_command(&cmd_name, &format!("Command {}", i), handler).ok();
            }

            // 3. Build CLI
            if let Ok(cli) = builder.build() {
                // 4. Discover commands
                let _ = cli.commands();

                // 5. Execute multiple commands
                for i in 0..3 {
                    let args =
                        CommandArgs::new().with_arg(&format!("key_{}", i), &format!("value_{}", i));
                    let _ = cli.execute(&format!("cmd_{}", i), args);
                }
            }
        });
    });
}

// ============================================================================
// Batch Operations Benchmark
// ============================================================================

fn batch_operations(c: &mut Criterion) {
    c.bench_function("batch_register_10_commands_at_once", |b| {
        b.iter(|| {
            let mut builder = AgentCliBuilder::new("batch-cli", "Batch registration CLI");

            let commands: Vec<_> = (0..10)
                .map(|i| {
                    (
                        format!("cmd_{}", i),
                        format!("Command {}", i),
                        EchoHandler::new(&format!("handler_{}", i)) as Arc<dyn CommandHandler>,
                    )
                })
                .collect();

            builder.register_commands(commands).ok();
            builder.build().ok();
        });
    });
}

// ============================================================================
// Convenience Methods Benchmark
// ============================================================================

fn convenience_methods(c: &mut Criterion) {
    let mut group = c.benchmark_group("convenience_methods");

    group.bench_function("command_args_contains_operation", |b| {
        b.iter(|| {
            let args = CommandArgs::new().with_arg("key1", "value1").with_arg("key2", "value2");

            black_box(args.contains("key1"));
            black_box(args.contains("key2"));
            black_box(args.contains("key3"));
        });
    });

    group.bench_function("command_args_len_operation", |b| {
        b.iter(|| {
            let args = CommandArgs::new()
                .with_arg("key1", "value1")
                .with_arg("key2", "value2")
                .with_positional("pos1")
                .with_positional("pos2");

            black_box(args.len());
        });
    });

    group.bench_function("command_args_get_all_positional", |b| {
        b.iter(|| {
            let args = CommandArgs::new()
                .with_positional("arg1")
                .with_positional("arg2")
                .with_positional("arg3");

            black_box(args.get_all_positional());
        });
    });

    group.finish();
}

// ============================================================================
// Benchmark Groups
// ============================================================================

criterion_group!(
    benches,
    jtbd_builder_initialization,
    jtbd_command_registration,
    jtbd_cli_building,
    jtbd_command_execution,
    jtbd_command_discovery,
    jtbd_command_chaining,
    end_to_end_workflow,
    batch_operations,
    convenience_methods,
);

criterion_main!(benches);
