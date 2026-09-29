//! Real integration tests for `clap_noun_verb_any::doctor::diagnose`,
//! against the real `tests/fixtures/greet.sh`/`cnv-any.json` pair this
//! crate's other tests already use.

use clap_noun_verb_any::doctor::diagnose;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn temp_manifest(label: &str, contents: &str) -> PathBuf {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let path = std::env::temp_dir().join(format!("cnv-any-doctor-test-{label}-{nanos}.json"));
    std::fs::write(&path, contents).expect("write real temp manifest");
    path
}

#[test]
fn a_real_healthy_fixture_pairing_reports_no_problems() {
    // Arrange: the crate's own real, already-verified fixture pair.
    let executable = fixtures_dir().join("greet.sh");
    let manifest = fixtures_dir().join("cnv-any.json");

    // Act
    let report = diagnose(&executable, &manifest);

    // Assert
    assert!(
        report.is_healthy(),
        "the real, already-working fixture pairing must report zero errors: {:?}",
        report.findings
    );
    assert_eq!(report.error_count(), 0);
}

#[test]
fn a_nonexistent_executable_is_reported_as_a_real_error() {
    // Arrange
    let executable = Path::new("/this/path/really/does/not/exist/anywhere");
    let manifest = fixtures_dir().join("cnv-any.json");

    // Act
    let report = diagnose(executable, manifest.as_path());

    // Assert
    assert!(!report.is_healthy());
    assert!(report.findings.iter().any(|f| f.message.contains("does not exist")));
}

#[test]
fn a_non_executable_regular_file_is_reported_as_a_real_error() {
    // Arrange: the manifest JSON file itself is a real, existing,
    // non-executable regular file.
    let executable = fixtures_dir().join("cnv-any.json");
    let manifest = fixtures_dir().join("cnv-any.json");

    // Act
    let report = diagnose(&executable, &manifest);

    // Assert
    #[cfg(unix)]
    assert!(
        report.findings.iter().any(|f| f.message.contains("not executable")),
        "findings were: {:?}",
        report.findings
    );
}

#[test]
fn a_manifest_that_does_not_parse_is_reported_as_a_real_error() {
    // Arrange: real, syntactically invalid JSON on disk.
    let executable = fixtures_dir().join("greet.sh");
    let manifest = temp_manifest("invalid-json", "{ not valid json");

    // Act
    let report = diagnose(&executable, &manifest);

    // Assert
    assert!(!report.is_healthy());
    assert!(report.findings.iter().any(|f| f.message.contains("not valid CliSchema JSON")));

    std::fs::remove_file(&manifest).ok();
}

#[test]
fn duplicate_argument_ids_are_reported_as_a_real_error() {
    // Arrange: a real manifest with two arguments sharing the same id.
    let manifest = temp_manifest(
        "duplicate-ids",
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
    );
    let executable = fixtures_dir().join("greet.sh");

    // Act
    let report = diagnose(&executable, &manifest);

    // Assert
    assert!(!report.is_healthy());
    assert!(report.findings.iter().any(|f| f.message.contains("duplicate argument id 'name'")));

    std::fs::remove_file(&manifest).ok();
}

#[test]
fn a_positional_argument_declaring_a_long_flag_is_reported_as_a_real_warning() {
    // Arrange: a real, internally-inconsistent manifest -- a positional
    // argument that also names a long flag it can never actually receive.
    let manifest = temp_manifest(
        "positional-with-flag",
        r#"{
            "name": "broken-fixture",
            "about": null,
            "commands": [
                {
                    "path": ["greet"],
                    "about": "Positional arg with an unreachable long flag",
                    "arguments": [
                        {"id": "name", "long": "name", "short": null, "required": true, "positional": true, "kind": "string", "behavior": "value"}
                    ],
                    "callable": true
                }
            ]
        }"#,
    );
    let executable = fixtures_dir().join("greet.sh");

    // Act
    let report = diagnose(&executable, &manifest);

    // Assert: a warning, not an error -- this is unusual but not
    // deployment-blocking.
    assert!(report.is_healthy(), "an unreachable-flag warning must not be an error");
    assert!(report.warning_count() >= 1);
    assert!(report
        .findings
        .iter()
        .any(|f| f.message.contains("positional but also declares a long/short flag")));

    std::fs::remove_file(&manifest).ok();
}

#[test]
fn duplicate_command_paths_are_reported_as_a_real_error() {
    // Arrange: two commands with the exact same path -- an ambiguous tool
    // name Gateway::execute could never disambiguate.
    let manifest = temp_manifest(
        "duplicate-path",
        r#"{
            "name": "broken-fixture",
            "about": null,
            "commands": [
                {"path": ["greet"], "about": "First", "arguments": [], "callable": true},
                {"path": ["greet"], "about": "Second, same path", "arguments": [], "callable": true}
            ]
        }"#,
    );
    let executable = fixtures_dir().join("greet.sh");

    // Act
    let report = diagnose(&executable, &manifest);

    // Assert
    assert!(!report.is_healthy());
    assert!(report.findings.iter().any(|f| f.message.contains("duplicate command path")));

    std::fs::remove_file(&manifest).ok();
}
