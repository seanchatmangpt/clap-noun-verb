// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Integration test: real verbs dispatched through the real
//! `CommandRegistry::execute_verb` seam must produce a real, spec-shaped OCEL
//! 2.0 JSON document on disk automatically, with no per-verb opt-in.

use clap_noun_verb::cli::CommandRegistry;
use clap_noun_verb::logic::{HandlerContext, HandlerInput, HandlerOutput};
use clap_noun_verb::ocel;
use std::collections::HashMap;
use std::fs;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

// Serializes tests in this binary that set the shared OCEL_PATH env var so
// they don't race each other (Chicago style: real env, real files, no mocks).
static ENV_LOCK: Mutex<()> = Mutex::new(());

fn temp_ocel_path(label: &str) -> std::path::PathBuf {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let dir = std::env::temp_dir().join(format!("cnv-ocel-dispatch-{label}-{nanos}"));
    fs::create_dir_all(&dir).expect("create temp test dir");
    dir.join("ocel.json")
}

#[test]
fn two_real_dispatched_verbs_produce_a_spec_shaped_ocel_document() {
    // Arrange: point the OCEL log at an isolated temp path, then register two
    // distinct real verbs on the real, globally-shared CommandRegistry.
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let path = temp_ocel_path("two-verbs");
    std::env::set_var("CLAP_NOUN_VERB_OCEL_PATH", &path);

    CommandRegistry::register_verb(
        "ocel_it_widgets",
        "list",
        "List widgets",
        |_input: HandlerInput| -> clap_noun_verb::Result<HandlerOutput> {
            Ok(HandlerOutput::from_data(serde_json::json!({"widgets": []}))?)
        },
    );
    CommandRegistry::register_verb(
        "ocel_it_widgets",
        "create",
        "Create a widget",
        |_input: HandlerInput| -> clap_noun_verb::Result<HandlerOutput> {
            Ok(HandlerOutput::from_data(serde_json::json!({"created": true}))?)
        },
    );

    // Act: dispatch both verbs through the real registry seam, exactly as the
    // CLI entry point does -- no OCEL-specific code required from the caller.
    let registry_lock = CommandRegistry::get();
    let registry = registry_lock.lock().unwrap_or_else(|e| e.into_inner());
    let input = HandlerInput {
        args: HashMap::new(),
        args_multi: HashMap::new(),
        opts: HashMap::new(),
        context: HandlerContext::new("list").with_noun("ocel_it_widgets"),
    };
    let list_result = registry.execute_verb("ocel_it_widgets", "list", input);
    assert!(list_result.is_ok(), "list verb should dispatch successfully");

    let input = HandlerInput {
        args: HashMap::new(),
        args_multi: HashMap::new(),
        opts: HashMap::new(),
        context: HandlerContext::new("create").with_noun("ocel_it_widgets"),
    };
    let create_result = registry.execute_verb("ocel_it_widgets", "create", input);
    assert!(create_result.is_ok(), "create verb should dispatch successfully");
    drop(registry);

    std::env::remove_var("CLAP_NOUN_VERB_OCEL_PATH");

    // Assert: a real OCEL 2.0 document was written automatically.
    let value: serde_json::Value = {
        let contents = fs::read_to_string(&path).expect("OCEL log must exist after dispatch");
        serde_json::from_str(&contents).expect("OCEL log must be valid JSON")
    };

    // Top-level spec shape.
    let obj = value.as_object().expect("top-level OCEL object");
    assert!(obj.contains_key("objectTypes"));
    assert!(obj.contains_key("eventTypes"));
    assert!(obj.contains_key("objects"));
    assert!(obj.contains_key("events"));

    let doc = ocel::read_document(&path).expect("read back via public API");

    // Two distinct command objects, one process object (both dispatches
    // happened in this one test-process invocation).
    let command_objects: Vec<&ocel::OcelObject> =
        doc.objects.iter().filter(|o| o.obj_type == "command").collect();
    assert_eq!(command_objects.len(), 2);
    assert!(command_objects.iter().any(|o| o.id == "ocel_it_widgets:list"));
    assert!(command_objects.iter().any(|o| o.id == "ocel_it_widgets:create"));

    let process_objects: Vec<&ocel::OcelObject> =
        doc.objects.iter().filter(|o| o.obj_type == "process").collect();
    assert_eq!(process_objects.len(), 1);

    // Two cli_invocation events, each relationship pointing at a real,
    // registered object id.
    assert_eq!(doc.events.len(), 2);
    let object_ids: std::collections::HashSet<&str> =
        doc.objects.iter().map(|o| o.id.as_str()).collect();
    for event in &doc.events {
        assert_eq!(event.event_type, "cli_invocation");
        assert_eq!(event.relationships.len(), 2);
        for rel in &event.relationships {
            assert!(
                object_ids.contains(rel.object_id.as_str()),
                "relationship {} must reference a real object id",
                rel.object_id
            );
        }
        assert!(
            event
                .attributes
                .iter()
                .any(|a| a.name == "noun" && a.value == serde_json::json!("ocel_it_widgets")),
            "event must carry a noun attribute"
        );
        assert!(
            event.attributes.iter().any(|a| a.name == "verb"),
            "event must carry a verb attribute"
        );
        assert!(
            event
                .attributes
                .iter()
                .any(|a| a.name == "success" && a.value == serde_json::json!(true)),
            "event must carry a success attribute"
        );
        assert!(
            event.attributes.iter().any(|a| a.name == "duration_ms"),
            "event must carry a duration_ms attribute"
        );
    }

    let event_verbs: std::collections::HashSet<String> = doc
        .events
        .iter()
        .filter_map(|e| {
            e.attributes.iter().find(|a| a.name == "verb").and_then(|a| a.value.as_str())
        })
        .map(|s| s.to_string())
        .collect();
    assert!(event_verbs.contains("list"));
    assert!(event_verbs.contains("create"));

    fs::remove_file(&path).ok();
}
