//! Micro-benchmarks for Wizard v2 performance optimization
//!
//! Measures hot path performance for:
//! - Session creation and state transitions (zero-cost abstractions)
//! - Prompt building and validation
//! - Template rendering with variable substitution
//! - Memory allocation patterns
//!
//! Performance Targets:
//! - Session creation: ≤ 100ns (zero-cost PhantomData)
//! - State transitions: ≤ 10ns (zero-cost PhantomData moves)
//! - Prompt building: ≤ 500ns
//! - Template rendering: ≤ 1µs per variable
//! - Allocation overhead: Minimal with pre-sizing

use clap_noun_verb::wizard::{
    prompt::{PromptBuilder, PromptTemplate},
    session::{Init, SessionBuilder},
    Prompt, WizardSession,
};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::collections::HashMap;

// =============================================================================
// Benchmark 1: Session Creation and State Transitions (Zero-Cost Abstractions)
// =============================================================================

fn bench_session_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("wizard_v2_session_operations");

    // Session creation (target: ≤100ns)
    group.bench_function("session_creation_default", |b| {
        b.iter(|| {
            black_box(WizardSession::new("session-123".to_string()));
        });
    });

    // Session creation with capacity (target: ≤100ns)
    group.bench_function("session_creation_with_capacity", |b| {
        b.iter(|| {
            black_box(WizardSession::with_capacity("session-456".to_string(), 16));
        });
    });

    // State transition: Init → Active (target: ≤10ns - zero-cost)
    group.bench_function("state_transition_init_to_active", |b| {
        b.iter_batched(
            || WizardSession::new("session-789".to_string()),
            |session| black_box(session.start()),
            criterion::BatchSize::SmallInput,
        );
    });

    // State transition: Active → Complete (target: ≤10ns - zero-cost)
    group.bench_function("state_transition_active_to_complete", |b| {
        b.iter_batched(
            || WizardSession::new("session-abc".to_string()).start(),
            |session| black_box(session.complete()),
            criterion::BatchSize::SmallInput,
        );
    });

    // State transition: Active → Paused → Active (target: ≤20ns - zero-cost)
    group.bench_function("state_transition_pause_resume", |b| {
        b.iter_batched(
            || WizardSession::new("session-def".to_string()).start(),
            |session| black_box(session.pause().resume()),
            criterion::BatchSize::SmallInput,
        );
    });

    // History accumulation (with pre-sizing)
    for size in [1, 5, 10, 50].iter() {
        group.bench_with_input(BenchmarkId::new("history_accumulation", size), size, |b, &size| {
            b.iter(|| {
                let mut session = WizardSession::with_capacity("session-ghi".to_string(), size);
                let mut session = session.start();
                for i in 0..size {
                    session.add_interaction(format!("Prompt {}", i), format!("Response {}", i));
                }
                black_box(session);
            });
        });
    }

    group.finish();
}

// =============================================================================
// Benchmark 2: Prompt Building (Allocation Optimization)
// =============================================================================

fn bench_prompt_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("wizard_v2_prompt_operations");

    // Simple prompt creation (target: ≤200ns)
    group.bench_function("prompt_build_simple", |b| {
        b.iter(|| {
            black_box(PromptBuilder::new().text("What is Rust?").build().ok().unwrap());
        });
    });

    // Prompt with system message (target: ≤300ns)
    group.bench_function("prompt_build_with_system", |b| {
        b.iter(|| {
            black_box(
                PromptBuilder::new()
                    .text("Explain zero-cost abstractions")
                    .system("You are a Rust expert")
                    .build()
                    .ok()
                    .unwrap(),
            );
        });
    });

    // Prompt with configuration (target: ≤400ns)
    group.bench_function("prompt_build_with_config", |b| {
        b.iter(|| {
            black_box(
                PromptBuilder::new()
                    .text("Generate CLI code")
                    .system("You are a code generator")
                    .max_tokens(1000)
                    .temperature(0.7)
                    .build()
                    .ok()
                    .unwrap(),
            );
        });
    });

    // Prompt with metadata (varying sizes)
    for metadata_count in [0, 2, 5, 10].iter() {
        group.bench_with_input(
            BenchmarkId::new("prompt_build_with_metadata", metadata_count),
            metadata_count,
            |b, &count| {
                b.iter(|| {
                    let mut builder =
                        PromptBuilder::with_metadata_capacity(count).text("Generate code");

                    for i in 0..count {
                        builder = builder.metadata(format!("key_{}", i), format!("value_{}", i));
                    }

                    black_box(builder.build().ok().unwrap());
                });
            },
        );
    }

    // Prompt getter performance (zero-cost references)
    group.bench_function("prompt_getters", |b| {
        let prompt = PromptBuilder::new()
            .text("Test prompt")
            .system("Test system")
            .max_tokens(100)
            .temperature(0.5)
            .metadata("key", "value")
            .build()
            .ok()
            .unwrap();

        b.iter(|| {
            black_box(prompt.text());
            black_box(prompt.system());
            black_box(prompt.max_tokens());
            black_box(prompt.temperature());
            black_box(prompt.metadata("key"));
            black_box(prompt.all_metadata());
        });
    });

    group.finish();
}

// =============================================================================
// Benchmark 3: Template Rendering (String Operation Optimization)
// =============================================================================

fn bench_template_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("wizard_v2_template_operations");

    // Template creation (variable extraction)
    group.bench_function("template_creation_simple", |b| {
        b.iter(|| {
            black_box(PromptTemplate::new("Hello {{name}}!"));
        });
    });

    // Template with multiple variables
    for var_count in [1, 3, 5, 10].iter() {
        group.bench_with_input(
            BenchmarkId::new("template_creation_multi_var", var_count),
            var_count,
            |b, &count| {
                let template_str =
                    (0..count).map(|i| format!("{{{{var_{}}}}}", i)).collect::<Vec<_>>().join(" ");

                b.iter(|| {
                    black_box(PromptTemplate::new(&template_str));
                });
            },
        );
    }

    // Template rendering (single variable)
    group.bench_function("template_render_single_var", |b| {
        let template = PromptTemplate::new("Hello {{name}}, welcome!");
        let mut values = HashMap::new();
        values.insert("name".to_string(), "Alice".to_string());

        b.iter(|| {
            black_box(template.render(&values).ok().unwrap());
        });
    });

    // Template rendering (multiple variables)
    for var_count in [2, 5, 10, 20].iter() {
        group.bench_with_input(
            BenchmarkId::new("template_render_multi_var", var_count),
            var_count,
            |b, &count| {
                let template_str = (0..count)
                    .map(|i| format!("Var {}: {{{{var_{}}}}}", i, i))
                    .collect::<Vec<_>>()
                    .join(", ");

                let template = PromptTemplate::new(&template_str);

                let mut values = HashMap::new();
                for i in 0..count {
                    values.insert(format!("var_{}", i), format!("Value {}", i));
                }

                b.iter(|| {
                    black_box(template.render(&values).ok().unwrap());
                });
            },
        );
    }

    // Template rendering (varying value lengths)
    for value_len in [10, 50, 100, 500].iter() {
        group.bench_with_input(
            BenchmarkId::new("template_render_var_value_lengths", value_len),
            value_len,
            |b, &len| {
                let template = PromptTemplate::new("Process: {{data}}");
                let mut values = HashMap::new();
                values.insert("data".to_string(), "x".repeat(len));

                b.iter(|| {
                    black_box(template.render(&values).ok().unwrap());
                });
            },
        );
    }

    // Complete template workflow (create + render)
    group.bench_function("template_complete_workflow", |b| {
        let mut values = HashMap::new();
        values.insert("name".to_string(), "Bob".to_string());
        values.insert("action".to_string(), "deploy".to_string());
        values.insert("target".to_string(), "production".to_string());

        b.iter(|| {
            let template = PromptTemplate::new("User {{name}} wants to {{action}} to {{target}}");
            black_box(template.render(&values).ok().unwrap());
        });
    });

    group.finish();
}

// =============================================================================
// Benchmark 4: Memory Allocation Patterns
// =============================================================================

fn bench_memory_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("wizard_v2_memory_patterns");

    // Session builder with varying capacities
    for capacity in [4, 8, 16, 32].iter() {
        group.bench_with_input(
            BenchmarkId::new("session_builder_capacity", capacity),
            capacity,
            |b, &cap| {
                b.iter(|| {
                    black_box(
                        SessionBuilder::new()
                            .session_id(format!("session-{}", cap))
                            .build()
                            .ok()
                            .unwrap(),
                    );
                });
            },
        );
    }

    // Prompt builder with pre-sized metadata
    for capacity in [4, 8, 16].iter() {
        group.bench_with_input(
            BenchmarkId::new("prompt_builder_metadata_capacity", capacity),
            capacity,
            |b, &cap| {
                b.iter(|| {
                    let mut builder =
                        PromptBuilder::with_metadata_capacity(cap).text("Test prompt");

                    for i in 0..cap {
                        builder = builder.metadata(format!("key_{}", i), format!("val_{}", i));
                    }

                    black_box(builder.build().ok().unwrap());
                });
            },
        );
    }

    group.finish();
}

// =============================================================================
// Benchmark 5: End-to-End Workflows
// =============================================================================

fn bench_e2e_workflows(c: &mut Criterion) {
    let mut group = c.benchmark_group("wizard_v2_e2e_workflows");
    group.throughput(Throughput::Elements(1));

    // Complete session workflow with template
    group.bench_function("complete_session_with_template", |b| {
        b.iter(|| {
            // Create session
            let session = WizardSession::with_capacity("session-e2e".to_string(), 5);
            let mut session = session.start();

            // Create template
            let template = PromptTemplate::new("Generate {{type}} for {{name}}");
            let mut values = HashMap::new();
            values.insert("type".to_string(), "CLI".to_string());
            values.insert("name".to_string(), "myapp".to_string());

            // Render template
            let prompt_text = template.render(&values).ok().unwrap();

            // Create prompt
            let prompt = PromptBuilder::new()
                .text(prompt_text)
                .system("You are a code generator")
                .max_tokens(2000)
                .temperature(0.7)
                .build()
                .ok()
                .unwrap();

            // Add to history
            session.add_interaction(prompt.text().to_string(), "Generated code...".to_string());

            // Complete session
            let session = session.complete();
            black_box(session);
        });
    });

    // Multi-turn conversation workflow
    group.bench_function("multi_turn_conversation", |b| {
        b.iter(|| {
            let session = WizardSession::with_capacity("conv-123".to_string(), 10);
            let mut session = session.start();

            for i in 0..5 {
                let prompt =
                    PromptBuilder::new().text(format!("Question {}", i)).build().ok().unwrap();

                session.add_interaction(prompt.text().to_string(), format!("Answer {}", i));
            }

            let session = session.complete();
            black_box(session);
        });
    });

    group.finish();
}

// =============================================================================
// Benchmark Groups
// =============================================================================

criterion_group!(
    benches,
    bench_session_operations,
    bench_prompt_operations,
    bench_template_operations,
    bench_memory_patterns,
    bench_e2e_workflows
);

criterion_main!(benches);
