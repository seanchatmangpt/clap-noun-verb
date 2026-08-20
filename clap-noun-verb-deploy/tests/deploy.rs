use clap_noun_verb::{Arg, ArgAction, Command};
use clap_noun_verb_deploy::{ArgumentKind, CliSchema, InvocationBuildError};
use serde_json::{json, Map};

fn schema() -> CliSchema {
    let command = Command::new("demo").subcommand(
        Command::new("user").subcommand(
            Command::new("create")
                .about("Create a user")
                .arg(Arg::new("name").long("name").required(true))
                .arg(Arg::new("admin").long("admin").action(ArgAction::SetTrue))
                .arg(Arg::new("tag").long("tag").action(ArgAction::Append)),
        ),
    );
    CliSchema::from_command(&command)
}

#[test]
fn derives_callable_tool_and_argument_types() {
    let schema = schema();
    let tools = schema.tools();
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].name, "user__create");
    let command = schema.commands.iter().find(|command| command.callable).expect("leaf command");
    assert_eq!(command.arguments[0].kind, ArgumentKind::String);
    assert_eq!(command.arguments[1].kind, ArgumentKind::Boolean);
    assert_eq!(command.arguments[2].kind, ArgumentKind::Array);
}

#[test]
fn manufactures_validated_argv_without_actuation() {
    let mut arguments = Map::new();
    arguments.insert("name".into(), json!("Ada"));
    arguments.insert("admin".into(), json!(true));
    arguments.insert("tag".into(), json!(["math", "ops"]));
    let invocation =
        schema().build_invocation("user__create", &arguments).expect("admitted invocation");
    assert_eq!(
        invocation.args,
        ["user", "create", "--name", "Ada", "--admin", "--tag", "math", "--tag", "ops"]
    );
}

#[test]
fn refuses_unknown_arguments_before_executor_boundary() {
    let mut arguments = Map::new();
    arguments.insert("name".into(), json!("Ada"));
    arguments.insert("shell".into(), json!("/bin/sh"));
    let error = schema()
        .build_invocation("user__create", &arguments)
        .expect_err("unknown argument must be refused");
    assert!(matches!(error, InvocationBuildError::UnknownArgument { .. }));
}

#[cfg(feature = "mcp")]
mod mcp {
    use super::*;
    use clap_noun_verb_deploy::mcp::{McpServer, MCP_PROTOCOL_VERSION};
    use clap_noun_verb_deploy::{Execution, Executor, Invocation};
    use std::convert::Infallible;

    #[derive(Default)]
    struct RecordingExecutor;

    impl Executor for RecordingExecutor {
        type Error = Infallible;

        fn execute(&self, _invocation: &Invocation) -> Result<Execution, Self::Error> {
            Ok(Execution { exit_code: 0, stdout: "created".into(), stderr: String::new() })
        }
    }

    fn server() -> McpServer<RecordingExecutor> {
        McpServer::new("demo", "1.0.0", schema(), RecordingExecutor)
    }

    fn request(id: u64, method: &str, params: serde_json::Value) -> serde_json::Value {
        json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params,
            "_meta": {
                "io.modelcontextprotocol/protocolVersion": MCP_PROTOCOL_VERSION,
                "io.modelcontextprotocol/clientCapabilities": {},
                "io.modelcontextprotocol/clientInfo": {"name": "test-client", "version": "1.0.0"}
            }
        })
    }

    #[test]
    fn discovers_stateless_modern_protocol() {
        let response = server()
            .handle(&request(1, "server/discover", json!({})))
            .expect("handled")
            .expect("response");
        assert_eq!(response["result"]["supportedVersions"][0], MCP_PROTOCOL_VERSION);
        assert_eq!(response["result"]["resultType"], "complete");
        assert_eq!(
            response["result"]["_meta"]["io.modelcontextprotocol/serverInfo"]["name"],
            "demo"
        );
    }

    #[test]
    fn lists_cli_leaf_as_cacheable_mcp_tool() {
        let response = server()
            .handle(&request(2, "tools/list", json!({})))
            .expect("handled")
            .expect("response");
        assert_eq!(response["result"]["tools"][0]["name"], "user__create");
        assert_eq!(response["result"]["resultType"], "complete");
        assert_eq!(response["result"]["cacheScope"], "public");
        assert!(response["result"]["ttlMs"].as_u64().is_some());
    }

    #[test]
    fn successful_tool_call_is_complete() {
        let response = server()
            .handle(&request(
                3,
                "tools/call",
                json!({"name":"user__create","arguments":{"name":"Ada"}}),
            ))
            .expect("handled")
            .expect("response");
        assert_eq!(response["result"]["resultType"], "complete");
        assert_eq!(response["result"]["content"][0]["text"], "created");
    }

    #[test]
    fn refuses_missing_modern_protocol_envelope() {
        let response = server()
            .handle(&json!({"jsonrpc":"2.0","id":4,"method":"tools/list"}))
            .expect("handled")
            .expect("response");
        assert_eq!(response["error"]["code"], -32600);
    }

    #[test]
    fn initialize_is_not_available_in_modern_era() {
        let response = server()
            .handle(&request(5, "initialize", json!({})))
            .expect("handled")
            .expect("response");
        assert_eq!(response["error"]["code"], -32601);
    }

    #[test]
    fn refuses_invalid_call_as_json_rpc_invalid_params() {
        let response = server()
            .handle(&request(
                6,
                "tools/call",
                json!({"name":"user__create","arguments":{"shell":"/bin/sh"}}),
            ))
            .expect("handled")
            .expect("response");
        assert_eq!(response["error"]["code"], -32602);
    }
}

#[cfg(feature = "http")]
mod http {
    use super::*;
    use clap_noun_verb_deploy::http::HttpServer;
    use clap_noun_verb_deploy::{Execution, Executor, Invocation};
    use std::convert::Infallible;

    struct EchoExecutor;

    impl Executor for EchoExecutor {
        type Error = Infallible;

        fn execute(&self, invocation: &Invocation) -> Result<Execution, Self::Error> {
            Ok(Execution { exit_code: 0, stdout: invocation.args.join(" "), stderr: String::new() })
        }
    }

    #[test]
    fn invoke_uses_same_schema_admission_as_mcp() {
        let response = HttpServer::new(schema(), EchoExecutor)
            .handle("POST", "/invoke", br#"{"tool":"user__create","arguments":{"name":"Ada"}}"#)
            .expect("response");
        assert_eq!(response.status, 200);
        assert!(response.body.contains("user create --name Ada"));
        assert!(response.body.contains("fingerprint"));
    }
}

#[cfg(feature = "kubernetes")]
#[test]
fn renders_hardened_kubernetes_projection_deterministically() {
    use clap_noun_verb_deploy::kubernetes::KubernetesConfig;

    let mut config = KubernetesConfig::new("demo", "ghcr.io/example/demo:sha-123");
    config.args = vec!["serve".into(), "http".into()];
    let first = config.render().expect("valid Kubernetes projection");
    assert_eq!(first, config.render().expect("deterministic projection"));
    assert!(first.contains("kind: Deployment"));
    assert!(first.contains("readOnlyRootFilesystem: true"));
    assert!(first.contains("runAsNonRoot: true"));
    assert!(first.contains("automountServiceAccountToken: false"));
    assert!(first.contains("type: RuntimeDefault"));
}

#[cfg(feature = "container")]
#[test]
fn renders_locked_multi_stage_container_projection() {
    use clap_noun_verb_deploy::container::ContainerConfig;

    let mut config = ContainerConfig::new("my-cli", "my-cli");
    config.args = vec!["serve".into(), "http".into()];
    let dockerfile = config.render_dockerfile().expect("valid container projection");
    assert!(dockerfile.contains("cargo build --release --locked -p my-cli"));
    assert!(dockerfile.contains("ENTRYPOINT [\"my-cli\", \"serve\", \"http\"]"));
}
