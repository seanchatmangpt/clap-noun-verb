//! Real end-to-end proof that `RegistryExecutor` closes the loop: a real
//! `clap_noun_verb::cli::CommandRegistry` verb, served over a real
//! `McpServer`, dispatched entirely in-process (no subprocess) -- the
//! previously-missing execution half of "wire clap-noun-verb-deploy into
//! the main CLI" (deploy already had the schema-projection half via
//! `Deploy::from_command`).

#[cfg(feature = "mcp")]
#[test]
fn mcp_server_serves_a_real_registry_verb_via_registry_executor_in_process() {
    use clap_noun_verb::cli::CommandRegistry;
    use clap_noun_verb::logic::{HandlerInput, HandlerOutput};
    use clap_noun_verb_deploy::mcp::McpServer;
    use clap_noun_verb_deploy::registry_executor::RegistryExecutor;
    use clap_noun_verb_deploy::CliSchema;

    // Arrange: a real verb on the real, process-wide registry.
    CommandRegistry::register_noun("mcp_e2e_probe_noun", "Real end-to-end MCP probe noun");
    CommandRegistry::register_verb(
        "mcp_e2e_probe_noun",
        "status",
        "Reports a real status payload",
        |_input: HandlerInput| -> clap_noun_verb::Result<HandlerOutput> {
            HandlerOutput::from_data(serde_json::json!({"healthy": true}))
        },
    );

    let registry_lock = CommandRegistry::get();
    let registry = registry_lock.lock().unwrap_or_else(|e| e.into_inner());
    let command = registry.build_command();
    drop(registry);
    let schema = CliSchema::from_command(&command);

    let server = McpServer::new("registry-executor-e2e", "0.0.0-test", schema, RegistryExecutor);

    // Act: a real MCP 2026-07-28 tools/call request, handled entirely
    // in-process through RegistryExecutor -> CommandRegistry::execute_single_step.
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {"name": "mcp_e2e_probe_noun__status", "arguments": {}},
        "_meta": {
            "io.modelcontextprotocol/protocolVersion": "2026-07-28",
            "io.modelcontextprotocol/clientCapabilities": {}
        }
    });
    let response = server.handle(&request).expect("real handle() call").expect("a real response");

    // Assert: the real handler's real output round-tripped through MCP.
    assert_eq!(response["result"]["isError"], serde_json::json!(false));
    let text =
        response["result"]["content"][0]["text"].as_str().expect("real text content").to_owned();
    let parsed: serde_json::Value = serde_json::from_str(&text).expect("real JSON payload");
    assert_eq!(parsed["healthy"], serde_json::json!(true));
}
