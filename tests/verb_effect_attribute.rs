// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Integration test: `#[verb(..., effect = "...")]` really declares a real
//! [`clap_noun_verb::autonomic::Effect`], carried all the way through to
//! the receipt recorded for a real dispatch -- not merely accepted syntax.

use clap_noun_verb::autonomic::{self, Effect};
use clap_noun_verb::cli::CommandRegistry;
use clap_noun_verb::logic::{HandlerContext, HandlerInput};
use clap_noun_verb_macros::verb;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

static ENV_LOCK: Mutex<()> = Mutex::new(());

fn temp_receipt_path(label: &str) -> PathBuf {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let dir = std::env::temp_dir().join(format!("cnv-verb-effect-test-{label}-{nanos}"));
    std::fs::create_dir_all(&dir).expect("create real temp dir");
    dir.join("receipts.jsonl")
}

/// A real verb declaring a real effect via the macro attribute.
#[verb("status", "verb_effect_it_fleet", effect = "read_only")]
fn handle_verb_effect_status() -> clap_noun_verb::Result<serde_json::Value> {
    Ok(serde_json::json!({"status": "ok"}))
}

#[test]
fn verb_effect_attribute_produces_a_real_receipt_with_the_declared_effect() {
    // Arrange
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let path = temp_receipt_path("declared");
    std::env::set_var("CLAP_NOUN_VERB_RECEIPT_PATH", &path);

    // Act: dispatch the real macro-registered verb through the real registry.
    let registry_lock = CommandRegistry::get();
    let registry = registry_lock.lock().unwrap_or_else(|e| e.into_inner());
    let input = HandlerInput {
        args: HashMap::new(),
        args_multi: HashMap::new(),
        opts: HashMap::new(),
        context: HandlerContext::new("status").with_noun("verb_effect_it_fleet"),
    };
    let result = registry.execute_verb("verb_effect_it_fleet", "status", input);
    drop(registry);
    std::env::remove_var("CLAP_NOUN_VERB_RECEIPT_PATH");

    assert!(result.is_ok(), "the verb must dispatch successfully: {result:?}");

    // Assert: the real receipt recorded for this dispatch carries the
    // REAL declared effect (ReadOnly), not the Effect::Unknown default.
    let receipts = autonomic::read_and_verify_ledger(&path).expect("read real receipt ledger");
    let receipt = receipts
        .iter()
        .find(|r| r.noun == "verb_effect_it_fleet" && r.verb == "status")
        .expect("a receipt for this dispatch must exist");
    assert_eq!(receipt.effect, Effect::ReadOnly);

    std::fs::remove_file(&path).ok();
}

/// A real verb with NO effect declared -- must still default honestly to
/// Unknown, proving the attribute is opt-in and never silently guessed.
#[verb("ping", "verb_effect_it_no_declaration")]
fn handle_verb_effect_no_declaration() -> clap_noun_verb::Result<serde_json::Value> {
    Ok(serde_json::json!({"pong": true}))
}

#[test]
fn a_verb_with_no_declared_effect_still_records_unknown_honestly() {
    // Arrange
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let path = temp_receipt_path("undeclared");
    std::env::set_var("CLAP_NOUN_VERB_RECEIPT_PATH", &path);

    // Act
    let registry_lock = CommandRegistry::get();
    let registry = registry_lock.lock().unwrap_or_else(|e| e.into_inner());
    let input = HandlerInput {
        args: HashMap::new(),
        args_multi: HashMap::new(),
        opts: HashMap::new(),
        context: HandlerContext::new("ping").with_noun("verb_effect_it_no_declaration"),
    };
    let result = registry.execute_verb("verb_effect_it_no_declaration", "ping", input);
    drop(registry);
    std::env::remove_var("CLAP_NOUN_VERB_RECEIPT_PATH");

    assert!(result.is_ok());

    // Assert
    let receipts = autonomic::read_and_verify_ledger(&path).expect("read real receipt ledger");
    let receipt = receipts
        .iter()
        .find(|r| r.noun == "verb_effect_it_no_declaration" && r.verb == "ping")
        .expect("a receipt for this dispatch must exist");
    assert_eq!(receipt.effect, Effect::Unknown);

    std::fs::remove_file(&path).ok();
}
