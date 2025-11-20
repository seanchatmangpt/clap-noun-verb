//! Comprehensive Integration Layer Example
//!
//! Demonstrates:
//! - Middleware executor with type-state pattern
//! - Custom domain-specific middlewares
//! - Production exporters (Datadog, Elasticsearch)
//! - Plugin configuration and dependency resolution
//!
//! Run with: cargo run --example integration_layer_example

use clap_noun_verb::integration::exporters::PlatformExporter;
use clap_noun_verb::integration::{
    CommandExecutor, DatadogExporter, DynamicCachingMiddleware, ElasticsearchExporter,
    ExecutionContext, ObservabilityMiddleware, PluginDependencyGraph, SmartRetryMiddleware,
};
use clap_noun_verb::middleware::MiddlewarePipeline;
use clap_noun_verb::telemetry::{
    ConsoleExporter, MetricsCollector, MetricsExporter, TracingCollector,
};
use std::sync::{Arc, Mutex};

fn main() -> clap_noun_verb::Result<()> {
    println!("╔═════════════════════════════════════════════════════════════╗");
    println!("║  Integration Layer - Advanced Middleware & Observability   ║");
    println!("╚═════════════════════════════════════════════════════════════╝\n");

    // =====================================================================
    // Part 1: Middleware Executor with Type-State Pattern
    // =====================================================================
    println!("📋 Part 1: Middleware Executor with Type-State Pattern\n");

    // Create middleware pipeline
    let pipeline = MiddlewarePipeline::new().add(Box::new(LoggingMiddleware::new().verbose()));

    // Create execution context
    let context = ExecutionContext::new("database-query")
        .with_arg("--table")
        .with_arg("users")
        .with_requester("admin");

    println!("   Creating command executor...");
    println!("   Command: {}", context.command());
    println!("   Args: {:?}", context.args());
    println!("   Trace ID: {}\n", context.trace_id());

    // Type-state pattern: PreExecution → Executing → PostExecution
    match CommandExecutor::new(pipeline)
        .with_context(context)
        .execute_pre()
        .map_err(|e| {
            println!("   ✗ Pre-execution failed: {}", e);
            e
        })
        .and_then(|executor| {
            println!("   ✓ Pre-execution phase passed");
            println!("   Executing command...\n");

            executor.execute_command(|| {
                // Simulate command execution
                std::thread::sleep(std::time::Duration::from_millis(50));
                Ok("Query returned 42 rows".to_string())
            })
        })
        .map_err(|e| {
            println!("   ✗ Command execution failed: {}", e);
            e
        })
        .and_then(|executor| {
            executor.execute_post().map_err(|e| {
                println!("   ✗ Post-execution failed: {}", e);
                e
            })
        }) {
        Ok(executor) => {
            let ctx = executor.into_context();
            println!("   ✓ Execution completed successfully");
            println!("   Duration: {}ms", ctx.duration_ms().unwrap_or(0));
            println!("   Result: {}\n", ctx.result().unwrap_or("N/A"));
        }
        Err(e) => println!("   Error: {}\n", e),
    }

    // =====================================================================
    // Part 2: Custom Domain-Specific Middlewares
    // =====================================================================
    println!("🛠️  Part 2: Custom Domain-Specific Middlewares\n");

    println!("   1. Dynamic Caching Middleware");
    let cache_mw = DynamicCachingMiddleware::with_ttl(600);
    println!("      ✓ Created with 600-second TTL\n");

    println!("   2. Smart Retry Middleware");
    let retry_mw = SmartRetryMiddleware::new().with_max_retries(5).with_base_backoff(100);
    println!("      ✓ Configured: {} retries, {} base backoff", 5, 100);
    println!("      Backoff times (exponential):");
    for attempt in 0..4 {
        let backoff = retry_mw.calculate_backoff(attempt);
        println!("        Attempt {}: {}ms", attempt, backoff);
    }
    println!();

    // =====================================================================
    // Part 3: Observability - Metrics and Telemetry
    // =====================================================================
    println!("📊 Part 3: Observability - Metrics and Tracing\n");

    let metrics = Arc::new(MetricsCollector::new());
    let tracing = Arc::new(Mutex::new(TracingCollector::new()));

    // Record some metrics
    metrics.record_command_execution("list_users", 125).ok();
    metrics.record_command_execution("create_user", 450).ok();
    metrics.record_command_execution("delete_user", 75).ok();
    metrics.record_command_error("invalid_query", "Syntax error").ok();

    println!("   Metrics recorded:");
    println!("      Commands executed: {}", metrics.command_count());
    println!("      Errors: {}", metrics.error_count());
    if let Some(mean) = metrics.execution_times().mean() {
        println!("      Mean execution time: {:.1}ms", mean);
    }
    println!();

    // Test ObservabilityMiddleware
    let obs_mw = ObservabilityMiddleware::new(metrics.clone(), tracing.clone());
    println!("   ✓ ObservabilityMiddleware configured\n");

    // =====================================================================
    // Part 4: Production Exporters
    // =====================================================================
    println!("📤 Part 4: Production Exporters\n");

    let metrics_for_export = MetricsCollector::new();
    metrics_for_export.record_command_execution("api_call", 200).ok();
    metrics_for_export.record_command_error("timeout", "Request timeout").ok();

    println!("   1. Console Exporter");
    let console_exporter = ConsoleExporter::new();
    match console_exporter.export(&metrics_for_export) {
        Ok(output) => {
            for line in output.lines().take(4) {
                println!("      {}", line);
            }
        }
        Err(e) => println!("      Error: {}", e),
    }
    println!();

    println!("   2. Datadog Exporter");
    let dd_exporter = DatadogExporter::new("your-api-key").with_site("us1");
    match dd_exporter.export(&metrics_for_export) {
        Ok(output) => {
            let lines: Vec<&str> = output.lines().collect();
            println!("      {}", lines.first().unwrap_or(&""));
        }
        Err(e) => println!("      Error: {}", e),
    }
    println!();

    println!("   3. Elasticsearch Exporter");
    let es_exporter = ElasticsearchExporter::new(vec!["http://localhost:9200".to_string()])
        .with_index_prefix("app-logs");
    match es_exporter.export(&metrics_for_export) {
        Ok(output) => {
            let lines: Vec<&str> = output.lines().collect();
            println!("      {}", lines.first().unwrap_or(&""));
        }
        Err(e) => println!("      Error: {}", e),
    }
    println!();

    // =====================================================================
    // Part 5: Plugin Configuration and Dependency Resolution
    // =====================================================================
    println!("🔌 Part 5: Plugin Configuration and Dependency Resolution\n");

    // Create plugin graph
    let mut graph = PluginDependencyGraph::new();

    // Simulate parsing manifests
    let mut plugin_core =
        clap_noun_verb::integration::config::PluginConfig::new("core", "1.0.0", "lib/core.so");

    let mut plugin_logging = clap_noun_verb::integration::config::PluginConfig::new(
        "logging",
        "1.0.0",
        "lib/logging.so",
    );
    plugin_logging.dependencies = vec!["core".to_string()];

    let mut plugin_metrics = clap_noun_verb::integration::config::PluginConfig::new(
        "metrics",
        "1.0.0",
        "lib/metrics.so",
    );
    plugin_metrics.dependencies = vec!["core".to_string(), "logging".to_string()];

    graph.add_plugin(&plugin_core);
    graph.add_plugin(&plugin_logging);
    graph.add_plugin(&plugin_metrics);

    println!("   Plugin dependency graph created");
    println!("      Total plugins: {}", graph.node_count());
    println!("      Plugins: {:?}\n", graph.plugins());

    // Resolve load order
    match graph.resolve() {
        Ok(load_order) => {
            println!("   Load order (topological sort):");
            for (idx, plugin) in load_order.iter().enumerate() {
                println!("      {}. {}", idx + 1, plugin);
            }
        }
        Err(e) => println!("      Error: {}", e),
    }
    println!();

    // =====================================================================
    // Summary
    // =====================================================================
    println!("╔═════════════════════════════════════════════════════════════╗");
    println!("║                      Integration Summary                    ║");
    println!("╚═════════════════════════════════════════════════════════════╝\n");

    println!("✨ Demonstrated Features:");
    println!("   1. Type-state middleware executor (PreExecution → Executing → PostExecution)");
    println!("   2. Custom domain-specific middlewares with composition");
    println!("   3. Metrics collection and calculation (mean, percentiles)");
    println!("   4. Multiple observability exporters (Console, Datadog, Elasticsearch)");
    println!("   5. Plugin configuration with dependency resolution");
    println!("   6. Topological sort for plugin load order");
    println!();

    println!("🎯 Advanced Rust Patterns Used:");
    println!("   • Type-state pattern (compile-time phase verification)");
    println!("   • Phantom types (zero-cost abstractions)");
    println!("   • Const generics (compile-time configuration)");
    println!("   • Trait objects (dynamic dispatch)");
    println!("   • Graph algorithms (topological sort)");
    println!();

    println!("📚 Next Steps:");
    println!("   • Integrate with actual Datadog/Elasticsearch instances");
    println!("   • Add custom middlewares for your domain");
    println!("   • Load plugins from manifest files");
    println!("   • Export metrics continuously to observability platforms");
    println!();

    Ok(())
}
