//! Advanced v4.3 Features Example
//!
//! Demonstrates all new features:
//! - Phase 7: Advanced clap integration (enum dispatch, value parsers, completions)
//! - Feature 1: Plugin system with built-in plugins
//! - Feature 4: Middleware pipeline with multiple middlewares
//! - Feature 5: Telemetry and observability
//!
//! Run with: cargo run --example advanced_features_v4_3

use clap_noun_verb::clap::{
    CommandContext, CompletionGenerator, EnumCommand, EnumDispatcher, Shell, ValidatedPort,
    ValidatedUrl,
};
use clap_noun_verb::middleware::{
    LoggingMiddleware, Middleware, MiddlewarePipeline, MiddlewareRequest, MiddlewareResponse,
};
use clap_noun_verb::plugin::{
    AliasPlugin, HelpPlugin, HistoryPlugin, Plugin, PluginCapability, PluginRegistry,
};
use clap_noun_verb::telemetry::{
    ConsoleExporter, MetricsCollector, MetricsExporter, TelemetryCollector,
};

/// Example: Advanced Server Command (Phase 7 - Enum Dispatch)
struct ServerCommand;

impl EnumCommand for ServerCommand {
    fn execute(&self) -> clap_noun_verb::Result<String> {
        Ok("Server command executed".to_string())
    }
}

fn main() -> clap_noun_verb::Result<()> {
    println!("═══════════════════════════════════════════════════════════════");
    println!("   clap-noun-verb v4.3 Advanced Features Demo");
    println!("═══════════════════════════════════════════════════════════════\n");

    // =====================================================================
    // Phase 7: Advanced clap Integration
    // =====================================================================
    println!("📦 Phase 7: Advanced clap Integration\n");
    println!("   ▸ Enum-based command dispatch");

    let dispatcher = EnumDispatcher::new("server").with_description("Server management commands");
    println!("     {}\n", dispatcher);

    let context = CommandContext::new("server").with_arg("--port").with_arg("8080");
    println!("   ▸ Command context: {}", context.full_path());
    println!("     Args: {:?}\n", context.args());

    println!("   ▸ Custom value parsers");
    match ValidatedUrl::new("https://example.com") {
        Ok(url) => println!("     ✓ Valid URL: {}", url),
        Err(e) => println!("     ✗ Invalid URL: {}", e),
    }

    match ValidatedPort::new(8080) {
        Ok(port) => println!("     ✓ Valid port: {}\n", port.port()),
        Err(e) => println!("     ✗ Invalid port: {}\n", e),
    }

    println!("   ▸ Shell completion generation");
    let generator = CompletionGenerator::new("myapp")
        .with_command("start")
        .with_command("stop")
        .with_option("--help")
        .with_option("--version");

    match generator.generate(Shell::Bash) {
        Ok(script) => {
            let lines: Vec<&str> = script.lines().take(3).collect();
            println!("     Bash completion ({} lines):", script.lines().count());
            for line in lines {
                println!("       {}", line);
            }
        }
        Err(e) => println!("     Error: {}", e),
    }
    println!();

    // =====================================================================
    // Feature 1: Plugin System
    // =====================================================================
    println!("🔌 Feature 1: Plugin System\n");
    println!("   ▸ Plugin registry and management");

    let mut registry = PluginRegistry::new();

    let help_plugin = HelpPlugin::new();
    println!("     Registering: {}", help_plugin.status());
    registry.register(Box::new(help_plugin)).ok();

    let history_plugin = HistoryPlugin::new();
    println!("     Registering: {}", history_plugin.status());
    registry.register(Box::new(history_plugin)).ok();

    let alias_plugin = AliasPlugin::new();
    println!("     Registering: {}", alias_plugin.status());
    registry.register(Box::new(alias_plugin)).ok();

    println!("\n   ▸ Loaded plugins ({}):", registry.count());
    for name in registry.list_all() {
        if let Some(plugin) = registry.get(name) {
            println!(
                "     • {} v{} - Capabilities: {:?}",
                plugin.name(),
                plugin.version(),
                plugin.capabilities()
            );
        }
    }

    println!("\n   ▸ Plugin discovery by capability:");
    let cmd_plugins = registry.find_by_capability(PluginCapability::Command);
    println!("     Command plugins: {}", cmd_plugins.len());
    let hook_plugins = registry.find_by_capability(PluginCapability::Hook);
    println!("     Hook plugins: {}\n", hook_plugins.len());

    // =====================================================================
    // Feature 4: Middleware Pipeline
    // =====================================================================
    println!("⚙️  Feature 4: Middleware Pipeline\n");
    println!("   ▸ Building middleware pipeline");

    let pipeline = MiddlewarePipeline::new().add(Box::new(LoggingMiddleware::new()));

    println!("     Pipeline: {}", pipeline);
    println!("     Middleware count: {}", pipeline.len());
    println!("     Middlewares: {:?}\n", pipeline.middleware_names());

    println!("   ▸ Executing middleware request");
    let request = MiddlewareRequest::new("execute").with_arg("--verbose").with_requester("admin");

    match pipeline.execute_before(&request) {
        Ok(_) => println!("     ✓ Pre-execution phase passed"),
        Err(e) => println!("     ✗ Pre-execution phase failed: {}", e),
    }

    let response = MiddlewareResponse::success("Command completed successfully");
    match pipeline.execute_after(&response) {
        Ok(_) => println!("     ✓ Post-execution phase passed\n"),
        Err(e) => println!("     ✗ Post-execution phase failed: {}\n", e),
    }

    // =====================================================================
    // Feature 5: Telemetry & Observability
    // =====================================================================
    println!("📊 Feature 5: Telemetry & Observability\n");
    println!("   ▸ Metrics collection");

    let metrics = MetricsCollector::new();
    metrics.record_command_execution("start", 150).ok();
    metrics.record_command_execution("stop", 75).ok();
    metrics.record_command_execution("restart", 225).ok();
    metrics.record_command_error("invalid", "Unknown command").ok();

    println!("     Total executions: {}", metrics.command_count());
    println!("     Total errors: {}", metrics.error_count());
    if let Some(mean) = metrics.execution_times().mean() {
        println!("     Average execution time: {:.1}ms", mean);
    }

    println!("\n   ▸ Telemetry collector");
    let mut telemetry = TelemetryCollector::new();
    println!("     Status: {}", telemetry);

    println!("\n   ▸ Metrics export (Console)");
    let exporter = ConsoleExporter::new();
    match exporter.export(telemetry.metrics()) {
        Ok(output) => {
            for line in output.lines().take(5) {
                println!("     {}", line);
            }
        }
        Err(e) => println!("     Error: {}", e),
    }
    println!();

    println!("   ▸ Distributed tracing");
    let span = telemetry.span("database_query").build("trace_001");
    println!("     Span ID: {}", span.id());
    println!("     Trace ID: {}", span.trace_id());
    println!("     Status: {:?}\n", span.status());

    // =====================================================================
    // Summary
    // =====================================================================
    println!("═══════════════════════════════════════════════════════════════");
    println!("   Summary: All v4.3 features demonstrated successfully!");
    println!("═══════════════════════════════════════════════════════════════");
    println!();
    println!("✨ What was demonstrated:");
    println!("   1. Phase 7: Advanced clap with enum dispatch, parsers, completions");
    println!("   2. Feature 1: Plugin system with registry and discovery");
    println!("   3. Feature 4: Middleware pipeline with logging and processing");
    println!("   4. Feature 5: Telemetry with metrics, tracing, and exporters");
    println!();
    println!("📚 Next steps:");
    println!("   • Integrate middleware into command execution flow");
    println!("   • Create custom middlewares for your use case");
    println!("   • Configure plugins via manifest files");
    println!("   • Export metrics to observability platforms");
    println!();

    Ok(())
}
