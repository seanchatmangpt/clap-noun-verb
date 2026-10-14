//! Real, Chicago-style, end-to-end proof that the OCEL corpus closes the
//! loop back into a real ggen generation decision -- not merely that the
//! Rust-side projection functions produce plausible-looking output.
//!
//! Requires the real `ggen` binary on `PATH` and a real
//! `~/ggen-marketplace` checkout with `clap-noun-verb-schema-pack` and
//! `ocel-feedback-pack` present (both `#[ignore]`d and skipped with a named
//! reason when either is missing -- never silently mocked).
//!
//! The proof has two real subprocess runs against the SAME composed
//! signals evidence: an ontology containing a command real evidence says
//! is dead is REFUSED by `ocel-feedback-pack`'s gate; the same ontology
//! with that command removed generates cleanly.

use clap_noun_verb::ocel::{
    compute_signals, write_signal_pack, EventAttributeValue, OcelDocument, OcelEvent, Relationship,
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
    packs.join("clap-noun-verb-schema-pack").is_dir() && packs.join("ocel-feedback-pack").is_dir()
}

fn temp_dir(label: &str) -> PathBuf {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let dir = std::env::temp_dir().join(format!("cnv-ocel-loop-test-{label}-{nanos}"));
    std::fs::create_dir_all(&dir).expect("create real temp dir");
    dir
}

fn regards_event(command_id: &str, success: bool) -> OcelEvent {
    OcelEvent {
        id: format!("evt-{command_id}-{success}"),
        event_type: "cli_invocation".to_string(),
        time: chrono::Utc::now().to_rfc3339(),
        attributes: vec![EventAttributeValue {
            name: "success".to_string(),
            value: serde_json::json!(success),
        }],
        relationships: vec![Relationship {
            object_id: command_id.to_string(),
            qualifier: "regards".to_string(),
        }],
    }
}

/// Write a two-command project ontology (`fleet:dead`, always admitted;
/// `fleet:alive`, admitted only when `include_dead` is true) into `dir`.
fn write_project_ontology(dir: &Path, include_dead: bool) {
    let dead_block = if include_dead {
        "cnv:FleetDead\n    a cnv:Command ;\n    cnv:name \"dead\" ;\n    \
         cnv:about \"Never invoked in production per the real merged OCEL corpus.\" ;\n    \
         cnv:belongsToNoun cnv:FleetNoun ;\n    cnv:hasBehavior cnv:FleetDeadBehavior .\n\n\
         cnv:FleetDeadBehavior a cnv:CustomBehavior .\n\n"
    } else {
        ""
    };
    let has_command = if include_dead { "cnv:FleetAlive, cnv:FleetDead" } else { "cnv:FleetAlive" };

    let ontology = format!(
        "@prefix cnv: <https://clap-noun-verb.dev/ontology#> .\n\n\
         cnv:FleetCli\n    a cnv:Cli ;\n    cnv:binaryName \"fleet-cli\" ;\n    \
         cnv:crateName \"fleet-cli\" ;\n    cnv:version \"0.1.0\" ;\n    cnv:edition \"2024\" ;\n    \
         cnv:rustVersion \"1.85\" ;\n    \
         cnv:about \"Real end-to-end OCEL feedback loop test fixture.\" ;\n    \
         cnv:hasNoun cnv:FleetNoun .\n\n\
         cnv:FleetNoun\n    a cnv:Noun ;\n    cnv:name \"fleet\" ;\n    \
         cnv:about \"Fleet management commands.\" ;\n    cnv:hasCommand {has_command} .\n\n\
         cnv:FleetAlive\n    a cnv:Command ;\n    cnv:name \"alive\" ;\n    \
         cnv:about \"Invoked recently and healthy per the real merged OCEL corpus.\" ;\n    \
         cnv:belongsToNoun cnv:FleetNoun ;\n    cnv:hasBehavior cnv:FleetAliveBehavior .\n\n\
         cnv:FleetAliveBehavior a cnv:CustomBehavior .\n\n{dead_block}"
    );
    std::fs::write(dir.join("ontology.ttl"), ontology).expect("write real project ontology.ttl");
}

fn write_project_ggen_toml(dir: &Path, signals_pack_dir: &Path) {
    let packs = ggen_marketplace_packs_dir();
    let toml = format!(
        "[project]\nname = \"fleet-cli-ocel-loop-test\"\n\n\
         [ontology]\nsource = \"ontology.ttl\"\n\n\
         [ontology.prefixes]\ncnv = \"https://clap-noun-verb.dev/ontology#\"\n\
         rdf = \"http://www.w3.org/1999/02/22-rdf-syntax-ns#\"\n\
         rdfs = \"http://www.w3.org/2000/01/rdf-schema#\"\n\
         xsd = \"http://www.w3.org/2001/XMLSchema#\"\n\n\
         [packs]\nclap-noun-verb-schema-pack = {{ path = \"{schema}\" }}\n\
         ocel-feedback-pack = {{ path = \"{feedback}\" }}\n\
         ocel-signals = {{ path = \"{signals}\" }}\n\n\
         [templates]\ndir = \".\"\naggregate_modules = false\n",
        schema = packs.join("clap-noun-verb-schema-pack").display(),
        feedback = packs.join("ocel-feedback-pack").display(),
        signals = signals_pack_dir.display(),
    );
    std::fs::write(dir.join("ggen.toml"), toml).expect("write real project ggen.toml");
}

#[test]
#[ignore = "requires the real `ggen` binary and a real ~/ggen-marketplace checkout with \
            clap-noun-verb-schema-pack and ocel-feedback-pack; see docs/reference/ocel-feedback-loop.md"]
fn real_ggen_refuses_a_command_a_real_ocel_corpus_says_is_dead_and_accepts_it_removed() {
    if !ggen_available() || !packs_available() {
        eprintln!("skipping: ggen binary or ~/ggen-marketplace packs not available");
        return;
    }

    // Arrange: a real merged OCEL corpus where "fleet:dead" was admitted
    // but never once invoked, and "fleet:alive" was invoked and healthy.
    let mut observed = OcelDocument::empty();
    observed.events.push(regards_event("fleet:alive", true));
    observed.events.push(regards_event("fleet:alive", true));
    let admitted = [("fleet", "dead"), ("fleet", "alive")];
    let signals = compute_signals(
        &admitted,
        &observed,
        std::time::Duration::from_secs(60 * 60 * 24 * 30),
        chrono::Utc::now(),
        0.5,
    );
    assert_eq!(signals.len(), 2);

    // Act: write the real, ggen-composable signals pack from those signals.
    let signals_pack_dir = temp_dir("signals-pack");
    write_signal_pack(&signals_pack_dir, &signals).expect("write real signals pack");

    // Arrange: a project ontology admitting BOTH commands, composing the
    // real schema-pack + the real ocel-feedback-pack + this signals pack.
    let project_dir = temp_dir("project-with-dead");
    write_project_ontology(&project_dir, true);
    write_project_ggen_toml(&project_dir, &signals_pack_dir);

    // Act: run the real ggen binary against it.
    let output = Command::new("ggen")
        .arg("sync")
        .arg("run")
        .current_dir(&project_dir)
        .output()
        .expect("spawn real ggen subprocess");

    // Assert: real refusal, citing the real dead command.
    assert!(
        !output.status.success(),
        "expected ggen to refuse generation for a command flagged prune by real OCEL evidence"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("ocel-feedback-pack"), "stderr was: {stderr}");
    assert!(stderr.contains("fleet:dead"), "stderr was: {stderr}");

    // Act: the SAME signals, the SAME gate, but the dead command removed
    // from the ontology -- the negative control.
    let clean_project_dir = temp_dir("project-without-dead");
    write_project_ontology(&clean_project_dir, false);
    write_project_ggen_toml(&clean_project_dir, &signals_pack_dir);

    let clean_output = Command::new("ggen")
        .arg("sync")
        .arg("run")
        .current_dir(&clean_project_dir)
        .output()
        .expect("spawn real ggen subprocess (clean project)");

    // Assert: real success once the dead command is gone.
    assert!(
        clean_output.status.success(),
        "expected clean generation once the dead command is removed; stderr: {}",
        String::from_utf8_lossy(&clean_output.stderr)
    );
    let generated_status = clean_project_dir.join("docs/OCEL_FEEDBACK_STATUS.md");
    assert!(generated_status.is_file(), "expected a real generated status report");
    let status_contents =
        std::fs::read_to_string(&generated_status).expect("read real status report");
    assert!(status_contents.contains("fleet:alive"));
    assert!(status_contents.contains("keep"));

    std::fs::remove_dir_all(&signals_pack_dir).ok();
    std::fs::remove_dir_all(&project_dir).ok();
    std::fs::remove_dir_all(&clean_project_dir).ok();
}

/// Write a single-command project ontology naming `command_name` (e.g.
/// `"flaky"`/`"seasonal"`) under noun `fleet`.
fn write_single_command_ontology(dir: &Path, command_name: &str, about: &str) {
    let pascal = {
        let mut chars = command_name.chars();
        match chars.next() {
            Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            None => String::new(),
        }
    };
    let ontology = format!(
        "@prefix cnv: <https://clap-noun-verb.dev/ontology#> .\n\n\
         cnv:FleetCli\n    a cnv:Cli ;\n    cnv:binaryName \"fleet-cli\" ;\n    \
         cnv:crateName \"fleet-cli\" ;\n    cnv:version \"0.1.0\" ;\n    cnv:edition \"2024\" ;\n    \
         cnv:rustVersion \"1.85\" ;\n    \
         cnv:about \"Real end-to-end OCEL feedback loop test fixture.\" ;\n    \
         cnv:hasNoun cnv:FleetNoun .\n\n\
         cnv:FleetNoun\n    a cnv:Noun ;\n    cnv:name \"fleet\" ;\n    \
         cnv:about \"Fleet management commands.\" ;\n    cnv:hasCommand cnv:Fleet{pascal} .\n\n\
         cnv:Fleet{pascal}\n    a cnv:Command ;\n    cnv:name \"{command_name}\" ;\n    \
         cnv:about \"{about}\" ;\n    cnv:belongsToNoun cnv:FleetNoun ;\n    \
         cnv:hasBehavior cnv:Fleet{pascal}Behavior .\n\n\
         cnv:Fleet{pascal}Behavior a cnv:CustomBehavior .\n"
    );
    std::fs::write(dir.join("ontology.ttl"), ontology)
        .expect("write real single-command ontology.ttl");
}

#[test]
#[ignore = "requires the real `ggen` binary and a real ~/ggen-marketplace checkout with \
            clap-noun-verb-schema-pack and ocel-feedback-pack; see docs/reference/ocel-feedback-loop.md"]
fn real_ggen_refuses_a_command_a_real_ocel_corpus_recommends_hardening() {
    if !ggen_available() || !packs_available() {
        eprintln!("skipping: ggen binary or ~/ggen-marketplace packs not available");
        return;
    }

    // Arrange: "fleet:flaky" ran 4 times, only 1 succeeded (25%) -- below
    // the 50% threshold, all recent, so this is real Harden evidence.
    let mut observed = OcelDocument::empty();
    observed.events.push(regards_event("fleet:flaky", true));
    observed.events.push(regards_event("fleet:flaky", false));
    observed.events.push(regards_event("fleet:flaky", false));
    observed.events.push(regards_event("fleet:flaky", false));
    let admitted = [("fleet", "flaky")];
    let signals = compute_signals(
        &admitted,
        &observed,
        std::time::Duration::from_secs(60 * 60 * 24 * 30),
        chrono::Utc::now(),
        0.5,
    );
    assert_eq!(signals.len(), 1);
    assert_eq!(signals[0].recommendation.as_str(), "harden");

    // Act: write the real signals pack and a project admitting the flaky command.
    let signals_pack_dir = temp_dir("signals-pack-harden");
    write_signal_pack(&signals_pack_dir, &signals).expect("write real signals pack");
    let project_dir = temp_dir("project-flaky");
    write_single_command_ontology(
        &project_dir,
        "flaky",
        "Succeeds only 25% of the time per the real merged OCEL corpus.",
    );
    write_project_ggen_toml(&project_dir, &signals_pack_dir);

    let output = Command::new("ggen")
        .arg("sync")
        .arg("run")
        .current_dir(&project_dir)
        .output()
        .expect("spawn real ggen subprocess");

    // Assert: real refusal, citing the harden gate and the real command.
    assert!(
        !output.status.success(),
        "expected ggen to refuse generation for a command flagged harden by real OCEL evidence"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("harden"), "stderr was: {stderr}");
    assert!(stderr.contains("fleet:flaky"), "stderr was: {stderr}");

    std::fs::remove_dir_all(&signals_pack_dir).ok();
    std::fs::remove_dir_all(&project_dir).ok();
}

#[test]
#[ignore = "requires the real `ggen` binary and a real ~/ggen-marketplace checkout with \
            clap-noun-verb-schema-pack and ocel-feedback-pack; see docs/reference/ocel-feedback-loop.md"]
fn real_ggen_refuses_a_command_a_real_ocel_corpus_recommends_reviewing() {
    if !ggen_available() || !packs_available() {
        eprintln!("skipping: ggen binary or ~/ggen-marketplace packs not available");
        return;
    }

    // Arrange: "fleet:seasonal" succeeded every time it ran, but its one
    // (only) run is 90 days old -- outside the 30-day recency window.
    let mut observed = OcelDocument::empty();
    let old_event = {
        let mut event = regards_event("fleet:seasonal", true);
        event.time = (chrono::Utc::now() - chrono::Duration::days(90)).to_rfc3339();
        event
    };
    observed.events.push(old_event);
    let admitted = [("fleet", "seasonal")];
    let signals = compute_signals(
        &admitted,
        &observed,
        std::time::Duration::from_secs(60 * 60 * 24 * 30),
        chrono::Utc::now(),
        0.5,
    );
    assert_eq!(signals.len(), 1);
    assert_eq!(signals[0].recommendation.as_str(), "review");

    // Act: write the real signals pack and a project admitting the stale command.
    let signals_pack_dir = temp_dir("signals-pack-review");
    write_signal_pack(&signals_pack_dir, &signals).expect("write real signals pack");
    let project_dir = temp_dir("project-seasonal");
    write_single_command_ontology(
        &project_dir,
        "seasonal",
        "Healthy but not exercised in the last 30 days per the real merged OCEL corpus.",
    );
    write_project_ggen_toml(&project_dir, &signals_pack_dir);

    let output = Command::new("ggen")
        .arg("sync")
        .arg("run")
        .current_dir(&project_dir)
        .output()
        .expect("spawn real ggen subprocess");

    // Assert: real refusal, citing the review gate and the real command.
    assert!(
        !output.status.success(),
        "expected ggen to refuse generation for a command flagged review by real OCEL evidence"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("review"), "stderr was: {stderr}");
    assert!(stderr.contains("fleet:seasonal"), "stderr was: {stderr}");

    std::fs::remove_dir_all(&signals_pack_dir).ok();
    std::fs::remove_dir_all(&project_dir).ok();
}
