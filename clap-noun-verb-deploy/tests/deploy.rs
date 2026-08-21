use clap_noun_verb::{Arg, ArgAction, Command};
use clap_noun_verb_deploy::{ArgumentKind, CliSchema, InvocationBuildError};
use serde_json::{json, Map};

/// Serializes every test in this binary that mutates the process-wide
/// `CLAP_NOUN_VERB_OCEL_PATH` env var (shared across the `mcp` and `http`
/// submodules below, since `cargo test` runs all tests in this binary as
/// threads in one process, not separate processes -- Chicago style: real
/// files, real env var, no mocks, but env var mutation must be serialized).
static OCEL_ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

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
    fn lists_ocel_resource_for_discovery() {
        let response = server()
            .handle(&request(6, "resources/list", json!({})))
            .expect("handled")
            .expect("response");
        assert_eq!(response["result"]["resources"][0]["uri"], "clap-noun-verb://ocel");
        assert_eq!(response["result"]["resources"][0]["mimeType"], "application/json");
    }

    #[test]
    fn reads_spec_shaped_ocel_resource() {
        let _guard = super::OCEL_ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let dir = std::env::temp_dir().join(format!(
            "cnv-deploy-mcp-ocel-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let path = dir.join("ocel.json");
        std::env::set_var("CLAP_NOUN_VERB_OCEL_PATH", &path);

        let response = server()
            .handle(&request(7, "resources/read", json!({"uri": "clap-noun-verb://ocel"})))
            .expect("handled")
            .expect("response");

        std::env::remove_var("CLAP_NOUN_VERB_OCEL_PATH");
        std::fs::remove_dir_all(&dir).ok();

        let text = response["result"]["contents"][0]["text"].as_str().expect("text content");
        let document: serde_json::Value =
            serde_json::from_str(text).expect("valid OCEL 2.0 JSON body");
        assert!(document["objectTypes"].is_array());
        assert!(document["eventTypes"].is_array());
        assert!(document["objects"].is_array());
        assert!(document["events"].is_array());
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

    /// `record_invocation`'s fallback path (`TMPDIR`/`clap-noun-verb-ocel.json`)
    /// is process-global and not overridable via env var, so a stray file left
    /// there by an unrelated prior run (this repo's own OCEL test suite
    /// exercises that exact fallback) would otherwise leak into an
    /// `/ocel`-route test whose primary path has not been written yet.
    /// Clearing it makes "no invocations yet" actually mean zero events.
    fn clear_global_ocel_fallback() {
        std::fs::remove_file(clap_noun_verb::ocel::fallback_path()).ok();
    }

    fn ocel_temp_path(label: &str) -> (std::path::PathBuf, std::path::PathBuf) {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let dir = std::env::temp_dir().join(format!("cnv-deploy-http-ocel-{label}-{nanos}"));
        (dir.clone(), dir.join("ocel.json"))
    }

    #[test]
    fn ocel_route_returns_spec_shaped_empty_document_before_any_invocation() {
        let _guard = OCEL_ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        clear_global_ocel_fallback();
        let (dir, path) = ocel_temp_path("empty");
        std::env::set_var("CLAP_NOUN_VERB_OCEL_PATH", &path);

        let response =
            HttpServer::new(schema(), EchoExecutor).handle("GET", "/ocel", b"").expect("response");

        std::env::remove_var("CLAP_NOUN_VERB_OCEL_PATH");
        std::fs::remove_dir_all(&dir).ok();

        assert_eq!(response.status, 200);
        assert_eq!(response.content_type, "application/json");
        let document: serde_json::Value =
            serde_json::from_str(&response.body).expect("valid OCEL 2.0 JSON body");
        assert!(document["objectTypes"].is_array());
        assert!(document["eventTypes"].is_array());
        assert!(document["objects"].as_array().expect("objects array").is_empty());
        assert!(document["events"].as_array().expect("events array").is_empty());
    }

    /// An `Executor` that mirrors what a real self-exec'd `clap-noun-verb`
    /// binary does per phase 1: write its own OCEL 2.0 event as part of
    /// dispatch. This is what makes the OCEL file this test reads through
    /// `GET /ocel` grow -- exactly the "child process already writes its own
    /// OCEL event" seam this task builds on top of, exercised without an
    /// actual subprocess so the test stays fast and self-contained.
    struct WritingExecutor;

    impl Executor for WritingExecutor {
        type Error = Infallible;

        fn execute(&self, invocation: &Invocation) -> Result<Execution, Self::Error> {
            clap_noun_verb::ocel::record_invocation("user", "create", true, 1);
            Ok(Execution { exit_code: 0, stdout: invocation.args.join(" "), stderr: String::new() })
        }
    }

    fn raw_http_request(
        addr: std::net::SocketAddr,
        method: &str,
        path: &str,
        body: &str,
    ) -> String {
        use std::io::{Read, Write};
        let mut stream = std::net::TcpStream::connect(addr).expect("connect to real HTTP server");
        let request = format!(
            "{method} {path} HTTP/1.1\r\nHost: localhost\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        stream.write_all(request.as_bytes()).expect("write real HTTP request");
        let mut response = String::new();
        stream.read_to_string(&mut response).expect("read real HTTP response");
        response
    }

    #[test]
    fn ocel_route_grows_after_a_real_admitted_invocation_over_real_tcp() {
        let _guard = OCEL_ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        clear_global_ocel_fallback();
        let (dir, path) = ocel_temp_path("grows");
        std::env::set_var("CLAP_NOUN_VERB_OCEL_PATH", &path);

        let listener =
            std::net::TcpListener::bind("127.0.0.1:0").expect("bind real ephemeral TCP port");
        let addr = listener.local_addr().expect("real local address");
        let server = HttpServer::new(schema(), WritingExecutor);
        std::thread::spawn(move || {
            let _ = server.serve(listener);
        });

        let before = raw_http_request(addr, "GET", "/ocel", "");
        let before_body = before.split("\r\n\r\n").nth(1).expect("real response body");
        let before_document: serde_json::Value =
            serde_json::from_str(before_body).expect("valid OCEL 2.0 JSON body before invocation");
        let before_count = before_document["events"].as_array().expect("events array").len();

        let invoke_body = r#"{"tool":"user__create","arguments":{"name":"Ada"}}"#;
        let invoke_response = raw_http_request(addr, "POST", "/invoke", invoke_body);
        assert!(invoke_response.contains("200"), "invocation must be admitted: {invoke_response}");

        let after = raw_http_request(addr, "GET", "/ocel", "");
        let after_body = after.split("\r\n\r\n").nth(1).expect("real response body");
        let after_document: serde_json::Value =
            serde_json::from_str(after_body).expect("valid OCEL 2.0 JSON body after invocation");
        let after_count = after_document["events"].as_array().expect("events array").len();

        std::env::remove_var("CLAP_NOUN_VERB_OCEL_PATH");
        std::fs::remove_dir_all(&dir).ok();

        assert!(
            after_count > before_count,
            "OCEL document must grow a real event: before={before_count} after={after_count}"
        );
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

#[cfg(feature = "kubernetes")]
#[test]
fn adds_writable_tmp_empty_dir_when_root_filesystem_is_read_only() {
    use clap_noun_verb_deploy::kubernetes::KubernetesConfig;

    let mut config = KubernetesConfig::new("demo", "ghcr.io/example/demo:sha-123");
    config.read_only_root_filesystem = true;
    let rendered = config.render().expect("valid Kubernetes projection");

    assert!(rendered.contains("readOnlyRootFilesystem: true"));
    assert!(rendered.contains("volumeMounts:\n        - name: tmp\n          mountPath: /tmp"));
    assert!(rendered.contains("volumes:\n      - name: tmp\n        emptyDir: {}"));
}

#[cfg(feature = "kubernetes")]
#[test]
fn omits_tmp_empty_dir_when_root_filesystem_is_already_writable() {
    use clap_noun_verb_deploy::kubernetes::KubernetesConfig;

    let mut config = KubernetesConfig::new("demo", "ghcr.io/example/demo:sha-123");
    config.read_only_root_filesystem = false;
    let rendered = config.render().expect("valid Kubernetes projection");

    assert!(rendered.contains("readOnlyRootFilesystem: false"));
    assert!(!rendered.contains("volumeMounts:"));
    assert!(!rendered.contains("emptyDir"));
}

#[cfg(feature = "kubernetes")]
#[test]
fn renders_hardened_cronjob_projection_deterministically() {
    use clap_noun_verb_deploy::kubernetes::CronJobConfig;

    let mut config =
        CronJobConfig::new("nightly-report", "ghcr.io/example/demo:sha-123", "0 3 * * *");
    config.args = vec!["report".into(), "generate".into()];
    let first = config.render().expect("valid CronJob projection");
    assert_eq!(first, config.render().expect("deterministic projection"));
    assert!(first.contains("kind: CronJob"));
    assert!(first.contains("schedule: \"0 3 * * *\""));
    assert!(first.contains("concurrencyPolicy: Forbid"));
    assert!(first.contains("restartPolicy: Never"));
    assert!(first.contains("readOnlyRootFilesystem: true"));
    assert!(first.contains("runAsNonRoot: true"));
    assert!(first.contains("automountServiceAccountToken: false"));
    assert!(first.contains("type: RuntimeDefault"));
    assert!(first.contains("args: [\"report\", \"generate\"]"));
}

#[cfg(feature = "kubernetes")]
#[test]
fn cronjob_refuses_a_malformed_schedule_before_rendering_any_yaml() {
    use clap_noun_verb_deploy::kubernetes::{CronJobConfig, KubernetesRenderError};

    let config =
        CronJobConfig::new("nightly-report", "ghcr.io/example/demo:sha-123", "not-a-cron-schedule");
    let error = config.render().expect_err("a malformed schedule must be refused");
    assert!(matches!(error, KubernetesRenderError::InvalidField { field: "schedule", .. }));
}

#[cfg(feature = "kubernetes")]
#[test]
fn cronjob_adds_writable_tmp_empty_dir_when_root_filesystem_is_read_only() {
    use clap_noun_verb_deploy::kubernetes::CronJobConfig;

    let mut config =
        CronJobConfig::new("nightly-report", "ghcr.io/example/demo:sha-123", "0 3 * * *");
    config.read_only_root_filesystem = true;
    let rendered = config.render().expect("valid CronJob projection");

    assert!(rendered.contains("readOnlyRootFilesystem: true"));
    assert!(rendered
        .contains("volumeMounts:\n                - name: tmp\n                  mountPath: /tmp"));
    assert!(rendered.contains("volumes:\n              - name: tmp\n                emptyDir: {}"));
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
