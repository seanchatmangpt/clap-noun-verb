//! Real, end-to-end proof for the `mcp_multi_cli` example (item #20): one
//! `McpServer` wired with a real `MultiExecutor` over 3 real wrapped
//! fixture scripts (`greet.sh`, `calc.sh`, `list-fruits.sh`), driven
//! through the exact JSON-RPC 2.0 `handle()` entry point the real
//! `serve_stdio` loop uses -- `tools/list` legitimately lists all 3
//! wrapped tools, and `tools/call` routes each call to the right process.

use clap_noun_verb_any::{merge_schemas, wrap, MultiExecutor};
use clap_noun_verb_deploy::mcp::McpServer;
use std::path::PathBuf;

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

/// Build a real MCP 2026-07-28 request envelope: every request must carry
/// the `_meta` protocol-version + client-capabilities block `validate_request_meta`
/// requires (see `clap-noun-verb-deploy/src/mcp.rs`).
fn request(id: i64, method: &str, params: Option<serde_json::Value>) -> serde_json::Value {
    let mut request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": method,
        "_meta": {
            "io.modelcontextprotocol/protocolVersion": "2026-07-28",
            "io.modelcontextprotocol/clientCapabilities": {}
        }
    });
    if let Some(params) = params {
        request["params"] = params;
    }
    request
}

fn build_server() -> McpServer<MultiExecutor> {
    let fixtures = fixtures_dir();
    let greet = wrap(fixtures.join("greet.sh").into_os_string(), &fixtures.join("cnv-any.json"))
        .expect("wrap real greet.sh");
    let calc = wrap(fixtures.join("calc.sh").into_os_string(), &fixtures.join("calc.json"))
        .expect("wrap real calc.sh");
    let list_fruits =
        wrap(fixtures.join("list-fruits.sh").into_os_string(), &fixtures.join("list-fruits.json"))
            .expect("wrap real list-fruits.sh");

    let merged_schema = merge_schemas(
        "multi-cli-demo",
        Some("3 real wrapped foreign-binary CLIs behind one MCP server".to_owned()),
        &[
            greet.deploy().schema().clone(),
            calc.deploy().schema().clone(),
            list_fruits.deploy().schema().clone(),
        ],
    );

    let (_greet_deploy, greet_executor) = greet.into_parts();
    let (_calc_deploy, calc_executor) = calc.into_parts();
    let (_list_deploy, list_executor) = list_fruits.into_parts();

    let mut multi = MultiExecutor::new();
    multi.add("greet", greet_executor);
    multi.add("add", calc_executor);
    multi.add("list", list_executor);

    McpServer::new("multi-cli-demo", "0.0.0-test", merged_schema, multi)
}

#[test]
fn tools_list_admits_all_three_wrapped_targets_tools() {
    let server = build_server();
    let request = request(1, "tools/list", None);

    let response = server.handle(&request).expect("real handle() call").expect("a real response");
    let tool_names: Vec<&str> = response["result"]["tools"]
        .as_array()
        .expect("tools array")
        .iter()
        .map(|tool| tool["name"].as_str().expect("tool name"))
        .collect();

    assert_eq!(tool_names, vec!["add", "greet", "list"]);
}

#[test]
fn tools_call_routes_greet_through_the_real_greet_sh_process() {
    let server = build_server();
    let request = request(
        2,
        "tools/call",
        Some(serde_json::json!({"name": "greet", "arguments": {"name": "World"}})),
    );

    let response = server.handle(&request).expect("real handle() call").expect("a real response");
    assert_eq!(response["result"]["isError"], serde_json::json!(false));
    assert_eq!(response["result"]["content"][0]["text"], serde_json::json!("Hello, World!\n"));
}

#[test]
fn tools_call_routes_add_through_the_real_calc_sh_process() {
    let server = build_server();
    let request = request(
        3,
        "tools/call",
        Some(serde_json::json!({"name": "add", "arguments": {"a": "10", "b": "32"}})),
    );

    let response = server.handle(&request).expect("real handle() call").expect("a real response");
    assert_eq!(response["result"]["isError"], serde_json::json!(false));
    assert_eq!(response["result"]["content"][0]["text"], serde_json::json!("42\n"));
}

#[test]
fn tools_call_routes_list_through_the_real_list_fruits_sh_process() {
    let server = build_server();
    let request = request(
        4,
        "tools/call",
        Some(serde_json::json!({"name": "list", "arguments": {"item": ["apple", "banana"]}})),
    );

    let response = server.handle(&request).expect("real handle() call").expect("a real response");
    assert_eq!(response["result"]["isError"], serde_json::json!(false));
    assert_eq!(response["result"]["content"][0]["text"], serde_json::json!("apple,banana\n"));
}
