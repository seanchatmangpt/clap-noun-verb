// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Integration test: real verbs dispatched through the real
//! `CommandRegistry::execute_verb`/`execute_root_verb` seams must produce a
//! real, verifiable receipt-chain automatically, with no per-verb opt-in --
//! the same "always-on" guarantee `tests/ocel_dispatch_integration.rs`
//! already proves for the OCEL event log, for `src/autonomic.rs`'s receipt
//! ledger instead.

use clap_noun_verb::autonomic::{self, Effect};
use clap_noun_verb::cli::CommandRegistry;
use clap_noun_verb::logic::{HandlerContext, HandlerInput, HandlerOutput};
use std::collections::HashMap;
use std::fs;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

// Serializes tests in this binary that set the shared RECEIPT_PATH env var
// so they don't race each other (Chicago style: real env, real files, no
// mocks) -- mirroring tests/ocel_dispatch_integration.rs's own lock.
static ENV_LOCK: Mutex<()> = Mutex::new(());

fn temp_receipt_path(label: &str) -> std::path::PathBuf {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let dir = std::env::temp_dir().join(format!("cnv-autonomic-dispatch-{label}-{nanos}"));
    fs::create_dir_all(&dir).expect("create temp test dir");
    dir.join("receipts.jsonl")
}

#[test]
fn two_real_dispatched_verbs_produce_a_real_verifiable_receipt_chain() {
    // Arrange: point the receipt ledger at an isolated temp path, then
    // register two distinct real verbs (one succeeding, one failing) on the
    // real, globally-shared CommandRegistry.
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let path = temp_receipt_path("two-verbs");
    std::env::set_var("CLAP_NOUN_VERB_RECEIPT_PATH", &path);

    CommandRegistry::register_verb(
        "autonomic_it_widgets",
        "list",
        "List widgets",
        |_input: HandlerInput| -> clap_noun_verb::Result<HandlerOutput> {
            Ok(HandlerOutput::from_data(serde_json::json!({"widgets": []}))?)
        },
    );
    CommandRegistry::register_verb(
        "autonomic_it_widgets",
        "explode",
        "Deliberately fail",
        |_input: HandlerInput| -> clap_noun_verb::Result<HandlerOutput> {
            Err(clap_noun_verb::error::NounVerbError::execution_error("boom"))
        },
    );

    // Act: dispatch both verbs through the real registry seam, exactly as
    // the CLI entry point does -- no receipt-specific code required from
    // the caller.
    let registry_lock = CommandRegistry::get();
    let registry = registry_lock.lock().unwrap_or_else(|e| e.into_inner());
    let input = HandlerInput {
        args: HashMap::new(),
        opts: HashMap::new(),
        context: HandlerContext::new("list").with_noun("autonomic_it_widgets"),
    };
    let list_result = registry.execute_verb("autonomic_it_widgets", "list", input);
    assert!(list_result.is_ok(), "list verb should dispatch successfully");

    let input = HandlerInput {
        args: HashMap::new(),
        opts: HashMap::new(),
        context: HandlerContext::new("explode").with_noun("autonomic_it_widgets"),
    };
    let explode_result = registry.execute_verb("autonomic_it_widgets", "explode", input);
    assert!(explode_result.is_err(), "explode verb should fail, and still be receipted");
    drop(registry);

    std::env::remove_var("CLAP_NOUN_VERB_RECEIPT_PATH");

    // Assert: a real, verifiable receipt chain was written automatically.
    let receipts = autonomic::read_and_verify_ledger(&path)
        .expect("receipt ledger must exist and verify after dispatch");
    assert_eq!(receipts.len(), 2);

    let list_receipt =
        receipts.iter().find(|r| r.verb == "list").expect("a receipt for the list verb must exist");
    assert_eq!(list_receipt.noun, "autonomic_it_widgets");
    assert!(list_receipt.success);
    assert_eq!(list_receipt.effect, Effect::Unknown);

    let explode_receipt = receipts
        .iter()
        .find(|r| r.verb == "explode")
        .expect("a receipt for the explode verb must exist, even though it failed");
    assert_eq!(explode_receipt.noun, "autonomic_it_widgets");
    assert!(!explode_receipt.success, "a failed dispatch must still be receipted, as a failure");

    fs::remove_file(&path).ok();
}

#[test]
fn a_root_verb_dispatch_produces_a_real_receipt_under_the_pseudo_noun() {
    // Arrange
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let path = temp_receipt_path("root-verb");
    std::env::set_var("CLAP_NOUN_VERB_RECEIPT_PATH", &path);

    CommandRegistry::register_verb(
        "",
        "autonomic_it_ping",
        "Root-level ping",
        |_input: HandlerInput| -> clap_noun_verb::Result<HandlerOutput> {
            Ok(HandlerOutput::from_data(serde_json::json!({"pong": true}))?)
        },
    );

    // Act
    let registry_lock = CommandRegistry::get();
    let registry = registry_lock.lock().unwrap_or_else(|e| e.into_inner());
    let input = HandlerInput {
        args: HashMap::new(),
        opts: HashMap::new(),
        context: HandlerContext::new("autonomic_it_ping"),
    };
    let result = registry.execute_root_verb("autonomic_it_ping", input);
    assert!(result.is_ok());
    drop(registry);

    std::env::remove_var("CLAP_NOUN_VERB_RECEIPT_PATH");

    // Assert
    let receipts =
        autonomic::read_and_verify_ledger(&path).expect("read and verify real receipt ledger");
    assert_eq!(receipts.len(), 1);
    assert_eq!(receipts[0].noun, "_root");
    assert_eq!(receipts[0].verb, "autonomic_it_ping");
    assert!(receipts[0].success);

    fs::remove_file(&path).ok();
}
