//! Comprehensive benchmarks for the Wizard Package (Agent CLI Builder)
//!
//! Performance SLO Targets:
//! - Session initialization: ≤ 100ms
//! - Prompt processing: ≤ 5 seconds
//! - Memory per session: ≤ 50MB
//! - Compilation time: ≤ 2 seconds incremental
//! - Test execution: ≤ 10 seconds unit, ≤ 30 seconds integration
//!
//! Benchmarks:
//! 1. Session Creation - AgentCliBuilder instantiation
//! 2. Command Registration - Single and batch operations
//! 3. CLI Building - Builder to AgentCli conversion
//! 4. Command Execution - Runtime command dispatch
//! 5. Concurrent Sessions - Multiple builder instances
//! 6. Memory Profiling - Allocation patterns and usage

use clap_noun_verb::agent_cli::{
    AgentCliBuilder, AgentResult, CommandArgs, CommandHandler, CommandMetadata,
};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use serde_json::Value;
use std::sync::Arc;

// =============================================================================
// Mock Command Handlers
// =============================================================================

/// Simple no-op command handler for benchmarking
#[derive(Debug, Clone)]
struct NoOpHandler {
    name: String,
}

impl CommandHandler for NoOpHandler {
    fn execute(&self, _args: &CommandArgs) -> AgentResult<Value> {
        Ok(serde_json::json!({"status": "success", "command": self.name}))
    }

    fn metadata(&self) -> CommandMetadata {
        CommandMetadata {
            name: self.name.clone(),
            description: format!("No-op handler for {}", self.name),
            arguments: vec![],
            requires_args: false,
        }
    }
}

/// Complex handler that simulates argument processing
#[derive(Debug, Clone)]
struct ProcessingHandler {
    name: String,
}

impl CommandHandler for ProcessingHandler {
    fn execute(&self, args: &CommandArgs) -> AgentResult<Value> {
        // Simulate some processing work
        let mut result = serde_json::json!({
            "status": "success",
            "command": self.name,
            "args_count": args.len(),
        });

        // Process named arguments
        for (key, value) in &args.values {
            result[key] = serde_json::json!(value);
        }

        Ok(result)
    }

    fn metadata(&self) -> CommandMetadata {
        CommandMetadata {
            name: self.name.clone(),
            description: format!("Processing handler for {}", self.name),
            arguments: vec![],
            requires_args: false,
        }
    }
}

// =============================================================================
// Benchmark 1: Session Creation (SLO: ≤100ms)
// =============================================================================

fn bench_session_initialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("wizard_session_initialization");

    group.bench_function("empty_builder", |b| {
        b.iter(|| {
            black_box(AgentCliBuilder::new("test-cli", "Test CLI application"));
        });
    });

    group.bench_function("builder_with_version", |b| {
        b.iter(|| {
            black_box(AgentCliBuilder::new("test-cli", "Test CLI application").version("1.0.0"));
        });
    });

    group.finish();
}

// =============================================================================
// Benchmark 2: Command Registration (Single & Batch)
// =============================================================================

fn bench_command_registration(c: &mut Criterion) {
    let mut group = c.benchmark_group("wizard_command_registration");

    // Single command registration
    group.bench_function("register_single_noop", |b| {
        b.iter(|| {
            let mut builder = AgentCliBuilder::new("test-cli", "Test CLI");
            let handler = Arc::new(NoOpHandler { name: "test".to_string() });
            black_box(builder.register_command("test", "Test command", handler));
        });
    });

    // Batch command registration with varying sizes
    for size in [1, 5, 10, 50, 100].iter() {
        group.bench_with_input(BenchmarkId::new("batch_registration", size), size, |b, &size| {
            b.iter(|| {
                let mut builder = AgentCliBuilder::new("test-cli", "Test CLI");
                for i in 0..size {
                    let handler = Arc::new(NoOpHandler { name: format!("command_{}", i) });
                    let _ = builder.register_command(
                        format!("cmd_{}", i),
                        format!("Command {}", i),
                        handler,
                    );
                }
                black_box(builder);
            });
        });
    }

    group.finish();
}

// =============================================================================
// Benchmark 3: CLI Building (Builder → AgentCli)
// =============================================================================

fn bench_cli_building(c: &mut Criterion) {
    let mut group = c.benchmark_group("wizard_cli_building");

    // Build with varying command counts
    for size in [1, 10, 50, 100].iter() {
        group.bench_with_input(BenchmarkId::new("build_with_commands", size), size, |b, &size| {
            b.iter_batched(
                || {
                    let mut builder = AgentCliBuilder::new("test-cli", "Test CLI");
                    for i in 0..size {
                        let handler = Arc::new(NoOpHandler { name: format!("command_{}", i) });
                        let _ = builder.register_command(
                            format!("cmd_{}", i),
                            format!("Command {}", i),
                            handler,
                        );
                    }
                    builder
                },
                |builder| black_box(builder.build()),
                criterion::BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

// =============================================================================
// Benchmark 4: Command Execution (Runtime Dispatch)
// =============================================================================

fn bench_command_execution(c: &mut Criterion) {
    let mut group = c.benchmark_group("wizard_command_execution");

    // Setup: Build a CLI with multiple commands
    let mut builder = AgentCliBuilder::new("test-cli", "Test CLI");

    for i in 0..10 {
        let handler = Arc::new(NoOpHandler { name: format!("command_{}", i) });
        let _ = builder.register_command(format!("cmd_{}", i), format!("Command {}", i), handler);
    }

    let cli = builder.build().expect("Failed to build CLI");

    // Execute single command (no arguments)
    group.bench_function("execute_noop_no_args", |b| {
        b.iter(|| {
            let args = CommandArgs::new();
            black_box(cli.execute("cmd_0", args));
        });
    });

    // Execute with arguments
    group.bench_function("execute_noop_with_args", |b| {
        b.iter(|| {
            let args = CommandArgs::new()
                .with_arg("key1", "value1")
                .with_arg("key2", "value2")
                .with_positional("pos1");
            black_box(cli.execute("cmd_0", args));
        });
    });

    // Execute processing handler
    let mut builder = AgentCliBuilder::new("test-cli", "Test CLI");
    let handler = Arc::new(ProcessingHandler { name: "process".to_string() });
    let _ = builder.register_command("process", "Process command", handler);
    let cli = builder.build().expect("Failed to build CLI");

    group.bench_function("execute_processing_with_args", |b| {
        b.iter(|| {
            let args = CommandArgs::new()
                .with_arg("key1", "value1")
                .with_arg("key2", "value2")
                .with_arg("key3", "value3")
                .with_positional("pos1")
                .with_positional("pos2");
            black_box(cli.execute("process", args));
        });
    });

    group.finish();
}

// =============================================================================
// Benchmark 5: Concurrent Session Handling
// =============================================================================

fn bench_concurrent_sessions(c: &mut Criterion) {
    let mut group = c.benchmark_group("wizard_concurrent_sessions");

    // Create multiple independent sessions
    for session_count in [1, 5, 10, 20].iter() {
        group.bench_with_input(
            BenchmarkId::new("parallel_sessions", session_count),
            session_count,
            |b, &count| {
                b.iter(|| {
                    let handles: Vec<_> = (0..count)
                        .map(|i| {
                            std::thread::spawn(move || {
                                let mut builder = AgentCliBuilder::new(
                                    format!("cli-{}", i),
                                    format!("CLI {}", i),
                                );

                                for j in 0..10 {
                                    let handler =
                                        Arc::new(NoOpHandler { name: format!("cmd_{}", j) });
                                    let _ = builder.register_command(
                                        format!("cmd_{}", j),
                                        format!("Command {}", j),
                                        handler,
                                    );
                                }

                                let cli = builder.build().expect("Failed to build");
                                let args = CommandArgs::new();
                                let _ = cli.execute("cmd_0", args);
                            })
                        })
                        .collect();

                    for handle in handles {
                        let _ = handle.join();
                    }
                });
            },
        );
    }

    group.finish();
}

// =============================================================================
// Benchmark 6: Full Workflow (End-to-End)
// =============================================================================

fn bench_full_workflow(c: &mut Criterion) {
    let mut group = c.benchmark_group("wizard_full_workflow");
    group.throughput(Throughput::Elements(1));

    // Complete workflow: create → register → build → execute
    group.bench_function("complete_workflow_10_commands", |b| {
        b.iter(|| {
            // Create builder
            let mut builder = AgentCliBuilder::new("test-cli", "Test CLI").version("1.0.0");

            // Register 10 commands
            for i in 0..10 {
                let handler = Arc::new(NoOpHandler { name: format!("command_{}", i) });
                let _ = builder.register_command(
                    format!("cmd_{}", i),
                    format!("Command {}", i),
                    handler,
                );
            }

            // Build CLI
            let cli = builder.build().expect("Failed to build CLI");

            // Execute command
            let args = CommandArgs::new();
            let result = cli.execute("cmd_0", args);

            black_box(result);
        });
    });

    // Large-scale workflow (100 commands)
    group.bench_function("complete_workflow_100_commands", |b| {
        b.iter(|| {
            let mut builder = AgentCliBuilder::new("test-cli", "Test CLI").version("1.0.0");

            for i in 0..100 {
                let handler = Arc::new(NoOpHandler { name: format!("command_{}", i) });
                let _ = builder.register_command(
                    format!("cmd_{}", i),
                    format!("Command {}", i),
                    handler,
                );
            }

            let cli = builder.build().expect("Failed to build CLI");
            let args = CommandArgs::new();
            let result = cli.execute("cmd_0", args);

            black_box(result);
        });
    });

    group.finish();
}

// =============================================================================
// Benchmark 7: Memory and Allocation Patterns
// =============================================================================

fn bench_memory_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("wizard_memory_patterns");

    // Measure builder memory footprint growth
    group.bench_function("builder_memory_growth", |b| {
        b.iter(|| {
            let mut builders = Vec::new();

            for i in 0..10 {
                let mut builder = AgentCliBuilder::new(format!("cli-{}", i), format!("CLI {}", i));

                for j in 0..50 {
                    let handler = Arc::new(NoOpHandler { name: format!("cmd_{}", j) });
                    let _ = builder.register_command(
                        format!("cmd_{}", j),
                        format!("Command {}", j),
                        handler,
                    );
                }

                builders.push(builder);
            }

            black_box(builders);
        });
    });

    // Measure CLI memory footprint
    group.bench_function("cli_memory_footprint", |b| {
        b.iter(|| {
            let mut clis = Vec::new();

            for i in 0..10 {
                let mut builder = AgentCliBuilder::new(format!("cli-{}", i), format!("CLI {}", i));

                for j in 0..50 {
                    let handler = Arc::new(NoOpHandler { name: format!("cmd_{}", j) });
                    let _ = builder.register_command(
                        format!("cmd_{}", j),
                        format!("Command {}", j),
                        handler,
                    );
                }

                let cli = builder.build().expect("Failed to build");
                clis.push(cli);
            }

            black_box(clis);
        });
    });

    group.finish();
}

// =============================================================================
// Benchmark Groups
// =============================================================================

criterion_group!(
    benches,
    bench_session_initialization,
    bench_command_registration,
    bench_cli_building,
    bench_command_execution,
    bench_concurrent_sessions,
    bench_full_workflow,
    bench_memory_patterns
);

criterion_main!(benches);
