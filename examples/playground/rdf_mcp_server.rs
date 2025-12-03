//! RDF-powered MCP server example
//!
//! Demonstrates running the RDF control layer as an MCP server
//! that agents can introspect and validate command invocations against.

use clap_noun_verb::rdf::{OntologyBuilder, RdfMcpHandler};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Build an ontology with some sample commands
    let mut builder = OntologyBuilder::new();
    builder
        .add_command("services-status", "services", "status", "Get service status")
        .expect("Failed to add services-status command");
    builder
        .add_command("config-show", "config", "show", "Display configuration")
        .expect("Failed to add config-show command");
    builder
        .add_command("logs-tail", "logs", "tail", "Stream recent logs")
        .expect("Failed to add logs-tail command");

    let ontology = Arc::new(builder.build().expect("Failed to build ontology"));

    // Create the RDF MCP handler
    let handler = RdfMcpHandler::new(ontology);

    // For now, demonstrate the handler capabilities
    println!("RDF MCP Server Example");
    println!("======================\n");

    // Show server info
    let info = handler.get_server_info();
    println!("Server: {}", info.server_info.name);
    println!("Version: {}", info.server_info.version);
    println!("Instructions: {:?}", info.instructions);
    println!();

    // Demonstrate command discovery
    println!("Discovering commands with intent 'service':");
    match handler.discover_commands("service") {
        Ok(result) => {
            println!("Found {} commands:", result.count);
            for cmd in result.commands {
                println!("  - {}", cmd);
            }
        }
        Err(e) => println!("Error: {}", e),
    }
    println!();

    // Demonstrate invocation validation
    println!("Validating invocations:");
    let commands = vec!["services-status", "config-show", "invalid-command"];
    for cmd in commands {
        match handler.validate_invocation(cmd, &None) {
            Ok(result) => {
                println!("  {} => {} ({})", cmd, result.valid, result.message);
            }
            Err(e) => println!("  {} => Error: {}", cmd, e),
        }
    }
    println!();

    // Demonstrate SPARQL query execution
    println!("Executing SPARQL query:");
    match handler.execute_sparql("SELECT ?subject WHERE { ?subject ?predicate ?object }") {
        Ok(result) => {
            println!("  Results: {}", serde_json::to_string_pretty(&result.results)?);
        }
        Err(e) => println!("  Error: {}", e),
    }
    println!();

    // Demonstrate receipt recording
    println!("Recording execution receipt:");
    match handler.record_receipt("services-status", 0) {
        Ok(result) => {
            println!("  Receipt ID: {}", result.receipt_id);
            println!("  Command: {}", result.command);
        }
        Err(e) => println!("  Error: {}", e),
    }

    println!("\nRDF MCP Server is ready. In production, would run stdio transport server.");
    println!("See rmcp documentation for full server setup with stdio transport.");

    Ok(())
}
