//! Agent CLI Builder - Dynamic Command Composition Without Macros
//!
//! This example demonstrates how MCP agents can programmatically create
//! and execute CLIs at runtime without compile-time macros.
//!
//! Workflow:
//! 1. Agent defines commands and handlers dynamically
//! 2. Commands are registered with the builder
//! 3. CLI is built and ready for execution
//! 4. Agents can execute commands and get structured results

use clap_noun_verb::agent_cli::{
    AgentCliBuilder, CommandArgs, CommandHandler, CommandMetadata, AgentResult,
};
use std::sync::Arc;

// ============================================================================
// Example Command Handlers
// ============================================================================

/// Handler for listing items
struct ListHandler;

impl CommandHandler for ListHandler {
    fn execute(&self, _args: &CommandArgs) -> AgentResult<serde_json::Value> {
        Ok(serde_json::json!({
            "status": "success",
            "command": "list",
            "items": [
                {"id": 1, "name": "Item One"},
                {"id": 2, "name": "Item Two"},
                {"id": 3, "name": "Item Three"},
            ]
        }))
    }

    fn metadata(&self) -> CommandMetadata {
        CommandMetadata {
            name: "list".to_string(),
            description: "List all available items".to_string(),
            arguments: vec![],
            requires_args: false,
        }
    }
}

/// Handler for showing item details
struct ShowHandler;

impl CommandHandler for ShowHandler {
    fn execute(&self, args: &CommandArgs) -> AgentResult<serde_json::Value> {
        let id = args.first_positional().unwrap_or("unknown");

        Ok(serde_json::json!({
            "status": "success",
            "command": "show",
            "item": {
                "id": id,
                "name": format!("Item {}", id),
                "description": "Detailed information about the item",
                "created_at": "2024-01-15T10:30:00Z"
            }
        }))
    }

    fn metadata(&self) -> CommandMetadata {
        CommandMetadata {
            name: "show".to_string(),
            description: "Show details for a specific item".to_string(),
            arguments: vec![],
            requires_args: true,
        }
    }
}

/// Handler for creating items
struct CreateHandler;

impl CommandHandler for CreateHandler {
    fn execute(&self, args: &CommandArgs) -> AgentResult<serde_json::Value> {
        let name = args.get("name").unwrap_or("Unnamed");

        Ok(serde_json::json!({
            "status": "success",
            "command": "create",
            "created_item": {
                "id": 42,
                "name": name,
                "created_at": "2024-01-15T10:30:00Z"
            }
        }))
    }

    fn metadata(&self) -> CommandMetadata {
        CommandMetadata {
            name: "create".to_string(),
            description: "Create a new item".to_string(),
            arguments: vec![],
            requires_args: false,
        }
    }
}

/// Handler for status information
struct StatusHandler;

impl CommandHandler for StatusHandler {
    fn execute(&self, _args: &CommandArgs) -> AgentResult<serde_json::Value> {
        Ok(serde_json::json!({
            "status": "success",
            "command": "status",
            "system": {
                "uptime_seconds": 3600,
                "version": "1.0.0",
                "health": "healthy",
                "active_commands": 0
            }
        }))
    }

    fn metadata(&self) -> CommandMetadata {
        CommandMetadata {
            name: "status".to_string(),
            description: "Show system status".to_string(),
            arguments: vec![],
            requires_args: false,
        }
    }
}

// ============================================================================
// Main Example
// ============================================================================

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!();
    println!("+═══════════════════════════════════════════════════════════+");
    println!("|        AGENT CLI BUILDER - DYNAMIC COMPOSITION             |");
    println!("|       Building CLIs at Runtime Without Macros              |");
    println!("+═══════════════════════════════════════════════════════════+\n");

    // ========================================================================
    // PHASE 1: CREATE AGENT CLI BUILDER
    // ========================================================================

    println!("PHASE 1: Initialize Agent CLI Builder");
    println!("─────────────────────────────────────\n");

    let mut cli_builder = AgentCliBuilder::new("agent-cli", "Agent-generated CLI tool");
    cli_builder = cli_builder.version("1.0.0");

    println!("  [AGENT] Creating CLI builder...");
    println!("    Name: agent-cli");
    println!("    Version: 1.0.0");
    println!("    Status: Ready for command registration\n");

    // ========================================================================
    // PHASE 2: REGISTER HANDLERS DYNAMICALLY
    // ========================================================================

    println!("PHASE 2: Register Command Handlers");
    println!("──────────────────────────────────\n");

    cli_builder.register_command(
        "list",
        "List all items",
        Arc::new(ListHandler),
    )?;

    cli_builder.register_command(
        "show",
        "Show item details",
        Arc::new(ShowHandler),
    )?;

    cli_builder.register_command(
        "create",
        "Create new item",
        Arc::new(CreateHandler),
    )?;

    cli_builder.register_command(
        "status",
        "Check system status",
        Arc::new(StatusHandler),
    )?;

    println!("  [AGENT] Registering command handlers...");
    println!("    ✓ list   - List all items");
    println!("    ✓ show   - Show item details");
    println!("    ✓ create - Create new item");
    println!("    ✓ status - Check system status\n");

    // ========================================================================
    // PHASE 3: BUILD THE CLI
    // ========================================================================

    println!("PHASE 3: Build Agent CLI");
    println!("────────────────────────\n");

    println!("  [AGENT] Building CLI from registered handlers...");
    let cli = cli_builder.build()?;

    println!("  [AGENT] CLI built successfully!");
    println!("    Total commands: {}", cli.commands().len());
    println!("    CLI Name: {}", cli.name());
    println!("    CLI Description: {}", cli.description());
    println!();

    // ========================================================================
    // PHASE 4: DISPLAY HELP AND COMMAND INFO
    // ========================================================================

    println!("PHASE 4: CLI Introspection");
    println!("──────────────────────────\n");

    println!("  [AGENT] Available Commands:");
    for cmd in cli.commands() {
        if let Some(info) = cli.command_info(cmd) {
            println!("    • {:<10} - {}", cmd, info.description);
        }
    }
    println!();

    println!("  [AGENT] Help Text:");
    println!("{}", cli.help());

    // ========================================================================
    // PHASE 5: EXECUTE COMMANDS
    // ========================================================================

    println!("PHASE 5: Command Execution");
    println!("──────────────────────────\n");

    // Execute: list
    println!("  [EXECUTION] Running 'list' command...");
    let result = cli.execute("list", CommandArgs::new())?;
    println!("  Result: {}", serde_json::to_string_pretty(&result)?);
    println!();

    // Execute: show
    println!("  [EXECUTION] Running 'show' command with ID=5...");
    let args = CommandArgs::new().with_positional("5");
    let result = cli.execute("show", args)?;
    println!("  Result: {}", serde_json::to_string_pretty(&result)?);
    println!();

    // Execute: create
    println!("  [EXECUTION] Running 'create' command...");
    let args = CommandArgs::new().with_arg("name", "New Item");
    let result = cli.execute("create", args)?;
    println!("  Result: {}", serde_json::to_string_pretty(&result)?);
    println!();

    // Execute: status
    println!("  [EXECUTION] Running 'status' command...");
    let result = cli.execute("status", CommandArgs::new())?;
    println!("  Result: {}", serde_json::to_string_pretty(&result)?);
    println!();

    // ========================================================================
    // PHASE 6: DEMONSTRATE DISCOVERY AND INTROSPECTION
    // ========================================================================

    println!("PHASE 6: Agent Discovery & Introspection");
    println!("───────────────────────────────────────\n");

    println!("  [AGENT DISCOVERY] Querying CLI capabilities...");
    println!("    Available commands: {}", cli.commands().join(", "));
    println!();

    println!("  [AGENT INTROSPECTION] Analyzing 'list' command...");
    if let Some(info) = cli.command_info("list") {
        println!("    Name: {}", info.name);
        println!("    Description: {}", info.description);
        println!("    Arguments: {:?}", info.arguments);
        println!("    Requires args: {}", info.requires_args);
    }
    println!();

    // ========================================================================
    // RESULTS SUMMARY
    // ========================================================================

    println!("+═══════════════════════════════════════════════════════════+");
    println!("|                  EXECUTION SUMMARY                        |");
    println!("+═══════════════════════════════════════════════════════════+\n");

    println!("Agent CLI Creation Workflow:");
    println!("  1. ✓ Created builder with name and version");
    println!("  2. ✓ Registered 4 command handlers dynamically");
    println!("  3. ✓ Built CLI from registered handlers");
    println!("  4. ✓ Displayed CLI introspection and help");
    println!("  5. ✓ Executed all commands successfully");
    println!("  6. ✓ Demonstrated discovery and introspection\n");

    println!("Key Features Demonstrated:");
    println!("  • No macros required - fully runtime-driven");
    println!("  • Type-safe handler registration");
    println!("  • JSON output for agent consumption");
    println!("  • Metadata available for discovery");
    println!("  • Handler-based execution model");
    println!("  • Composable from multiple sources\n");

    println!("Agent CLI Status:");
    println!("  • Builder initialized: ✓");
    println!("  • Handlers registered: ✓");
    println!("  • CLI built: ✓");
    println!("  • Commands executed: ✓");
    println!("  • Discovery working: ✓\n");

    println!("+═══════════════════════════════════════════════════════════+");
    println!("|      AGENT CLI BUILDER EXAMPLE - SUCCESS                  |");
    println!("|    CLIs can now be created dynamically by agents!         |");
    println!("+═══════════════════════════════════════════════════════════+");

    Ok(())
}
