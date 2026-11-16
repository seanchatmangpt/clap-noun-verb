//! Real, Chicago-style integration tests wrapping 5 more distinct real
//! executables (item #7 of the 25-prompt closure pass) -- each a genuinely
//! runnable `/bin/sh` script exercising a different `ArgumentKind`/
//! `ArgumentBehavior` combination the real `CliSchema` supports: a string
//! positional + boolean flag (`word-count.sh`), two integer positionals
//! (`calc.sh`), a repeated `--item` array/append flag (`list-fruits.sh`),
//! a real non-zero exit code path (`status-check.sh`), and a short-flag
//! integer value (`repeat.sh`). Every test runs the real `wrap()` ->
//! `Gateway::execute` -> real subprocess -> real OCEL event path, exactly
//! like `wrap_integration.rs`'s `greet.sh` fixture.

use clap_noun_verb_any::wrap;
use clap_noun_verb_deploy::{AdmitValidated, Gateway};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// Serializes every test in this binary that mutates the process-wide
/// `CLAP_NOUN_VERB_OCEL_PATH` env var (same discipline as
/// `wrap_integration.rs`'s `OCEL_ENV_LOCK`).
static OCEL_ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn unique_ocel_dir(label: &str) -> PathBuf {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    std::env::temp_dir().join(format!("cnv-any-five-fixtures-ocel-{label}-{nanos}"))
}

/// Wrap `script`/`manifest`, execute `tool` with `arguments` through the
/// real `Gateway`, and return the real captured `Execution` plus the noun
/// recorded on the resulting real OCEL event.
fn wrap_and_execute(
    script: &str,
    manifest: &str,
    tool: &str,
    arguments: serde_json::Map<String, serde_json::Value>,
    ocel_label: &str,
) -> (clap_noun_verb_deploy::Execution, String) {
    let fixtures = fixtures_dir();
    let wrapped = wrap(fixtures.join(script).into_os_string(), &fixtures.join(manifest))
        .unwrap_or_else(|e| panic!("wrap real fixture {script}: {e}"));

    let invocation = wrapped
        .deploy()
        .schema()
        .build_invocation(tool, &arguments)
        .unwrap_or_else(|e| panic!("build real invocation for {tool}: {e}"));

    let _guard = OCEL_ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let dir = unique_ocel_dir(ocel_label);
    let ocel_path = dir.join("ocel.json");
    std::env::set_var("CLAP_NOUN_VERB_OCEL_PATH", &ocel_path);

    let (_deploy, executor) = wrapped.into_parts();
    let gateway = Gateway::new(ocel_label, executor, AdmitValidated);
    let record = gateway.execute(invocation).unwrap_or_else(|e| panic!("real execution: {e}"));

    std::env::remove_var("CLAP_NOUN_VERB_OCEL_PATH");

    let document =
        clap_noun_verb::ocel::read_document(&ocel_path).expect("read real OCEL document");
    assert_eq!(document.events.len(), 1);
    let noun = document.events[0]
        .attributes
        .iter()
        .find(|a| a.name == "noun")
        .and_then(|a| a.value.as_str())
        .unwrap_or_default()
        .to_owned();

    std::fs::remove_dir_all(&dir).ok();
    (record.execution, noun)
}

#[test]
fn word_count_sh_counts_words_and_honors_verbose_flag() {
    let mut arguments = serde_json::Map::new();
    arguments.insert("text".to_owned(), serde_json::json!("the quick brown fox"));
    arguments.insert("verbose".to_owned(), serde_json::json!(true));

    let (execution, noun) =
        wrap_and_execute("word-count.sh", "word-count.json", "count", arguments, "word-count");

    assert_eq!(execution.exit_code, 0);
    assert_eq!(execution.stdout, "word count for 'the quick brown fox': 4\n");
    assert_eq!(noun, "count");
}

#[test]
fn calc_sh_adds_two_real_integers() {
    let mut arguments = serde_json::Map::new();
    arguments.insert("a".to_owned(), serde_json::json!("17"));
    arguments.insert("b".to_owned(), serde_json::json!("25"));

    let (execution, noun) = wrap_and_execute("calc.sh", "calc.json", "add", arguments, "calc");

    assert_eq!(execution.exit_code, 0);
    assert_eq!(execution.stdout, "42\n");
    assert_eq!(noun, "add");
}

#[test]
fn list_fruits_sh_joins_repeated_append_values() {
    let mut arguments = serde_json::Map::new();
    arguments.insert("item".to_owned(), serde_json::json!(["apple", "banana", "cherry"]));

    let (execution, noun) =
        wrap_and_execute("list-fruits.sh", "list-fruits.json", "list", arguments, "list-fruits");

    assert_eq!(execution.exit_code, 0);
    assert_eq!(execution.stdout, "apple,banana,cherry\n");
    assert_eq!(noun, "list");
}

#[test]
fn status_check_sh_real_failure_path_surfaces_nonzero_exit_and_stderr() {
    let mut arguments = serde_json::Map::new();
    arguments.insert("fail".to_owned(), serde_json::json!(true));

    let (execution, noun) =
        wrap_and_execute("status-check.sh", "status-check.json", "check", arguments, "status-fail");

    assert_eq!(execution.exit_code, 1);
    assert!(!execution.success());
    assert_eq!(execution.stderr, "simulated failure\n");
    assert_eq!(noun, "check");
}

#[test]
fn status_check_sh_real_success_path_is_distinct_from_the_failure_path() {
    let arguments = serde_json::Map::new();

    let (execution, noun) =
        wrap_and_execute("status-check.sh", "status-check.json", "check", arguments, "status-ok");

    assert_eq!(execution.exit_code, 0);
    assert!(execution.success());
    assert_eq!(execution.stdout, "ok\n");
    assert_eq!(noun, "check");
}

#[test]
fn repeat_sh_short_flag_integer_value_repeats_a_real_number_of_times() {
    let mut arguments = serde_json::Map::new();
    arguments.insert("count".to_owned(), serde_json::json!("5"));

    let (execution, noun) =
        wrap_and_execute("repeat.sh", "repeat.json", "bang", arguments, "repeat");

    assert_eq!(execution.exit_code, 0);
    assert_eq!(execution.stdout, "!!!!!\n");
    assert_eq!(noun, "bang");
}
