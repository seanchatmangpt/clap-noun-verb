//! Real, Chicago-style integration test: `wrap()` a real fixture script with
//! its hand-written manifest, run a real admitted invocation through the
//! real `Gateway`/`ProcessExecutor` path, and assert the real captured
//! stdout/exit-code plus a real OCEL event on disk.

use clap_noun_verb_any::wrap;
use clap_noun_verb_deploy::{AdmitValidated, Gateway};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// Serializes every test in this binary that mutates the process-wide
/// `CLAP_NOUN_VERB_OCEL_PATH` env var, mirroring the lock/scoping pattern
/// `clap-noun-verb-deploy`'s own OCEL tests use in `tests/deploy.rs` (env
/// var mutation must be serialized across tests in one process; the value
/// itself is a real file on disk, not a mock).
static OCEL_ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn fixture_paths() -> (PathBuf, PathBuf) {
    let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    (fixtures.join("greet.sh"), fixtures.join("cnv-any.json"))
}

fn unique_ocel_dir(label: &str) -> PathBuf {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    std::env::temp_dir().join(format!("cnv-any-wrap-ocel-{label}-{nanos}"))
}

#[test]
fn wrap_runs_real_fixture_through_real_gateway_and_records_ocel_parity() {
    // Arrange: a real, hand-written manifest wrapping a real trivial script.
    let (executable, manifest) = fixture_paths();
    let wrapped = wrap(executable.into_os_string(), &manifest).expect("wrap real fixture script");
    assert_eq!(wrapped.deploy().schema().name, "greet-fixture");
    assert_eq!(wrapped.deploy().schema().commands.len(), 1);
    assert!(wrapped.deploy().schema().commands[0].callable);

    let mut arguments = serde_json::Map::new();
    arguments.insert("name".to_owned(), serde_json::json!("World"));
    let invocation = wrapped
        .deploy()
        .schema()
        .build_invocation("greet", &arguments)
        .expect("manifest admits a validated invocation for its one callable command");
    assert_eq!(invocation.args, vec!["greet".to_owned(), "World".to_owned()]);

    // Arrange: scope the OCEL path this test's OCEL event lands in, the same
    // way clap-noun-verb-deploy's own OCEL tests scope
    // `CLAP_NOUN_VERB_OCEL_PATH` (guard the process-wide env var, use a
    // unique temp dir, clean up afterward).
    let _guard = OCEL_ENV_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let dir = unique_ocel_dir("greet");
    let ocel_path = dir.join("ocel.json");
    std::env::set_var("CLAP_NOUN_VERB_OCEL_PATH", &ocel_path);

    // Act: the real Gateway/ProcessExecutor/OcelExecutor path, mirroring how
    // clap-noun-verb-deploy's own tests/deploy.rs exercises Gateway.
    let (_deploy, executor) = wrapped.into_parts();
    let gateway = Gateway::new("greet-fixture", executor, AdmitValidated);
    let record = gateway.execute(invocation).expect("admitted invocation executes for real");

    std::env::remove_var("CLAP_NOUN_VERB_OCEL_PATH");

    // Assert: real captured stdout and exit code from the real subprocess.
    assert_eq!(record.execution.stdout, "Hello, World!\n");
    assert_eq!(record.execution.exit_code, 0);
    assert!(record.execution.success());
    assert!(record.verify_integrity());

    // Assert: a real OCEL event was appended, using the manifest's noun/verb.
    let document =
        clap_noun_verb::ocel::read_document(&ocel_path).expect("read real OCEL document");
    assert_eq!(document.events.len(), 1);
    assert_eq!(document.events[0].event_type, "cli_invocation");
    let noun = document.events[0]
        .attributes
        .iter()
        .find(|attribute| attribute.name == "noun")
        .map(|attribute| attribute.value.clone());
    let verb = document.events[0]
        .attributes
        .iter()
        .find(|attribute| attribute.name == "verb")
        .map(|attribute| attribute.value.clone());
    assert_eq!(noun, Some(serde_json::json!("greet")));
    assert_eq!(verb, Some(serde_json::json!("greet")));
    let success = document.events[0]
        .attributes
        .iter()
        .find(|attribute| attribute.name == "success")
        .map(|attribute| attribute.value.clone());
    assert_eq!(success, Some(serde_json::json!(true)));

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn wrap_refuses_invocation_outside_admitted_command_before_executor_boundary() {
    // Arrange
    let (executable, manifest) = fixture_paths();
    let wrapped = wrap(executable.into_os_string(), &manifest).expect("wrap real fixture script");

    // Act: an unknown tool name is refused by the schema before any process
    // is ever spawned.
    let error = wrapped
        .deploy()
        .schema()
        .build_invocation("not-a-real-tool", &serde_json::Map::new())
        .expect_err("unknown tool must be refused");

    // Assert
    assert!(matches!(
        error,
        clap_noun_verb_deploy::InvocationBuildError::UnknownTool(name) if name == "not-a-real-tool"
    ));
}

#[test]
fn wrap_refuses_a_shape_invalid_manifest_before_any_process_is_ever_spawned() {
    // Arrange: a real, syntactically valid CliSchema JSON with a real
    // shape error (two arguments sharing the same id) -- exercises the
    // exact same clap_noun_verb_any::doctor::schema_shape_errors check
    // doctor_integration.rs verifies standalone, now wired as a hard
    // refusal inside wrap() itself.
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let manifest = std::env::temp_dir().join(format!("cnv-any-invalid-shape-{nanos}.json"));
    std::fs::write(
        &manifest,
        r#"{
            "name": "broken-fixture",
            "about": null,
            "commands": [
                {
                    "path": ["greet"],
                    "about": "Broken: duplicate argument id",
                    "arguments": [
                        {"id": "name", "long": null, "short": null, "required": true, "positional": true, "kind": "string", "behavior": "value"},
                        {"id": "name", "long": "verbose", "short": null, "required": false, "positional": false, "kind": "boolean", "behavior": "set_true"}
                    ],
                    "callable": true
                }
            ]
        }"#,
    )
    .expect("write real invalid-shape manifest");

    let (executable, _) = fixture_paths();

    // Act: wrap() must refuse before any process is ever spawned -- there
    // is no way to observe a spawned process here (Wrapped is never
    // constructed), which is itself the proof: only an Err ever returns.
    let result = wrap(executable.into_os_string(), &manifest);

    // Assert
    let error = result.expect_err("a shape-invalid manifest must be refused by wrap() itself");
    assert!(matches!(error, clap_noun_verb_any::WrapError::InvalidShape(_)));
    assert!(error.to_string().contains("duplicate argument id 'name'"), "error was: {error}");

    std::fs::remove_file(&manifest).ok();
}
