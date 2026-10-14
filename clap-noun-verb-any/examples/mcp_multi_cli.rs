//! Real MCP server example serving 3 different `cnv-any`-wrapped CLIs
//! (item #20 of the 25-prompt closure pass) from one stdio MCP endpoint:
//! `greet.sh` (a real greeting script), `calc.sh` (integer addition), and
//! `list-fruits.sh` (a repeated-flag joiner) -- all three real fixture
//! scripts already proven individually in `tests/wrap_integration.rs` /
//! `tests/five_more_fixtures.rs`.
//!
//! Run it:
//! ```sh
//! cargo run -p clap-noun-verb-any --example mcp_multi_cli
//! ```
//! then send newline-delimited JSON-RPC 2.0 requests on stdin, e.g.:
//! ```json
//! {"jsonrpc":"2.0","id":1,"method":"tools/list"}
//! {"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"greet","arguments":{"name":"World"}}}
//! {"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"add","arguments":{"a":"2","b":"3"}}}
//! {"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"list","arguments":{"item":["apple","banana"]}}}
//! ```

use clap_noun_verb_any::{merge_schemas, wrap, MultiExecutor};
use clap_noun_verb_deploy::mcp::McpServer;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fixtures = fixtures_dir();

    let greet = wrap(fixtures.join("greet.sh").into_os_string(), &fixtures.join("cnv-any.json"))?;
    let calc = wrap(fixtures.join("calc.sh").into_os_string(), &fixtures.join("calc.json"))?;
    let list_fruits =
        wrap(fixtures.join("list-fruits.sh").into_os_string(), &fixtures.join("list-fruits.json"))?;

    let merged_schema = merge_schemas(
        "multi-cli-demo",
        Some("One MCP server serving 3 real wrapped foreign-binary CLIs".to_owned()),
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

    let server = McpServer::new("multi-cli-demo", env!("CARGO_PKG_VERSION"), merged_schema, multi);

    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    server.serve_stdio(stdin.lock(), stdout.lock())?;
    Ok(())
}

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}
