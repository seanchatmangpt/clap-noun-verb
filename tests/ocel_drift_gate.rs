//! Real, subprocess-level proof that `ocel-drift-pack`'s coverage-floor
//! gate closes `clap_noun_verb::ocel::drift_report`'s coverage ratio into
//! a real ggen generation decision -- complementing
//! `tests/ocel_feedback_loop.rs`'s per-command prune proof with a
//! project-wide aggregate-coverage proof.

use clap_noun_verb::ocel::{
    drift_report, write_drift_pack, EventAttributeValue, OcelDocument, OcelEvent, Relationship,
};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn ggen_marketplace_packs_dir() -> PathBuf {
    PathBuf::from(std::env::var("HOME").unwrap_or_default()).join("ggen-marketplace/packs")
}

fn ggen_available() -> bool {
    Command::new("ggen").arg("--version").output().map(|o| o.status.success()).unwrap_or(false)
}

fn packs_available() -> bool {
    let packs = ggen_marketplace_packs_dir();
    packs.join("clap-noun-verb-schema-pack").is_dir() && packs.join("ocel-drift-pack").is_dir()
}

fn temp_dir(label: &str) -> PathBuf {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let dir = std::env::temp_dir().join(format!("cnv-drift-gate-test-{label}-{nanos}"));
    std::fs::create_dir_all(&dir).expect("create real temp dir");
    dir
}

fn write_project(dir: &Path, drift_pack_dir: &Path) {
    let packs = ggen_marketplace_packs_dir();
    let ontology = "@prefix cnv: <https://clap-noun-verb.dev/ontology#> .\n\n\
         cnv:FleetCli\n    a cnv:Cli ;\n    cnv:binaryName \"fleet-cli\" ;\n    \
         cnv:crateName \"fleet-cli\" ;\n    cnv:version \"0.1.0\" ;\n    cnv:edition \"2024\" ;\n    \
         cnv:rustVersion \"1.85\" ;\n    \
         cnv:about \"Real drift-gate test fixture.\" ;\n    \
         cnv:hasNoun cnv:FleetNoun .\n\n\
         cnv:FleetNoun\n    a cnv:Noun ;\n    cnv:name \"fleet\" ;\n    \
         cnv:about \"Fleet commands.\" ;\n    cnv:hasCommand cnv:FleetAlive, cnv:FleetDead .\n\n\
         cnv:FleetAlive\n    a cnv:Command ;\n    cnv:name \"alive\" ;\n    \
         cnv:about \"Exercised.\" ;\n    cnv:belongsToNoun cnv:FleetNoun ;\n    \
         cnv:hasBehavior cnv:FleetAliveBehavior .\n\n\
         cnv:FleetAliveBehavior a cnv:CustomBehavior .\n\n\
         cnv:FleetDead\n    a cnv:Command ;\n    cnv:name \"dead\" ;\n    \
         cnv:about \"Never exercised.\" ;\n    cnv:belongsToNoun cnv:FleetNoun ;\n    \
         cnv:hasBehavior cnv:FleetDeadBehavior .\n\n\
         cnv:FleetDeadBehavior a cnv:CustomBehavior .\n";
    std::fs::write(dir.join("ontology.ttl"), ontology).expect("write real project ontology.ttl");

    let toml = format!(
        "[project]\nname = \"fleet-cli-drift-gate-test\"\n\n\
         [ontology]\nsource = \"ontology.ttl\"\n\n\
         [ontology.prefixes]\ncnv = \"https://clap-noun-verb.dev/ontology#\"\n\
         rdf = \"http://www.w3.org/1999/02/22-rdf-syntax-ns#\"\n\
         rdfs = \"http://www.w3.org/2000/01/rdf-schema#\"\n\
         xsd = \"http://www.w3.org/2001/XMLSchema#\"\n\n\
         [packs]\nclap-noun-verb-schema-pack = {{ path = \"{schema}\" }}\n\
         ocel-drift-pack = {{ path = \"{drift}\" }}\n\
         ocel-drift-data = {{ path = \"{data}\" }}\n\n\
         [templates]\ndir = \".\"\naggregate_modules = false\n",
        schema = packs.join("clap-noun-verb-schema-pack").display(),
        drift = packs.join("ocel-drift-pack").display(),
        data = drift_pack_dir.display(),
    );
    std::fs::write(dir.join("ggen.toml"), toml).expect("write real project ggen.toml");
}

#[test]
#[ignore = "requires the real `ggen` binary and a real ~/ggen-marketplace checkout with \
            clap-noun-verb-schema-pack and ocel-drift-pack; see docs/reference/ocel-feedback-loop.md"]
fn real_ggen_refuses_when_real_coverage_falls_below_the_required_floor_and_accepts_above_it() {
    if !ggen_available() || !packs_available() {
        eprintln!("skipping: ggen binary or ~/ggen-marketplace packs not available");
        return;
    }

    // Arrange: a real corpus where only 1 of 2 admitted commands was ever
    // exercised -- 50% coverage.
    let mut observed = OcelDocument::empty();
    observed.events.push(OcelEvent {
        id: "evt-alive".to_string(),
        event_type: "cli_invocation".to_string(),
        time: chrono::Utc::now().to_rfc3339(),
        attributes: vec![EventAttributeValue {
            name: "success".to_string(),
            value: serde_json::json!(true),
        }],
        relationships: vec![Relationship {
            object_id: "fleet:alive".to_string(),
            qualifier: "regards".to_string(),
        }],
    });
    let admitted = [("fleet", "alive"), ("fleet", "dead")];
    let report = drift_report(&admitted, &observed);
    assert!((report.coverage_ratio - 0.5).abs() < f64::EPSILON);

    // Act: require an 80% floor -- 50% real coverage must be refused.
    let drift_pack_dir = temp_dir("drift-pack-strict");
    write_drift_pack(&drift_pack_dir, &report, 0.8).expect("write real drift pack");
    let strict_project = temp_dir("project-strict");
    write_project(&strict_project, &drift_pack_dir);

    let strict_output = Command::new("ggen")
        .arg("sync")
        .arg("run")
        .current_dir(&strict_project)
        .output()
        .expect("spawn real ggen subprocess");

    assert!(
        !strict_output.status.success(),
        "expected ggen to refuse generation: real coverage (0.5) is below the required floor (0.8)"
    );
    let stderr = String::from_utf8_lossy(&strict_output.stderr);
    assert!(stderr.contains("ocel-drift-pack"), "stderr was: {stderr}");

    // Act: the SAME real coverage, but a lenient 0.3 floor -- must accept.
    let lenient_drift_pack_dir = temp_dir("drift-pack-lenient");
    write_drift_pack(&lenient_drift_pack_dir, &report, 0.3).expect("write real drift pack");
    let lenient_project = temp_dir("project-lenient");
    write_project(&lenient_project, &lenient_drift_pack_dir);

    let lenient_output = Command::new("ggen")
        .arg("sync")
        .arg("run")
        .current_dir(&lenient_project)
        .output()
        .expect("spawn real ggen subprocess");

    assert!(
        lenient_output.status.success(),
        "expected clean generation: real coverage (0.5) meets the lenient floor (0.3); stderr: {}",
        String::from_utf8_lossy(&lenient_output.stderr)
    );
    let status_report = lenient_project.join("docs/DRIFT_STATUS.md");
    assert!(status_report.is_file(), "expected a real generated drift status report");

    std::fs::remove_dir_all(&drift_pack_dir).ok();
    std::fs::remove_dir_all(&strict_project).ok();
    std::fs::remove_dir_all(&lenient_drift_pack_dir).ok();
    std::fs::remove_dir_all(&lenient_project).ok();
}
