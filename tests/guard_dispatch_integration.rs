// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Integration test: a real `Guard` registered via
//! `CommandRegistry::add_guard` actually runs before a real verb
//! dispatch and refuses it -- the Autonomic Layer's `GuardSet` wired into
//! `execute_verb`/`execute_root_verb`, not merely available as a
//! standalone type.
//!
//! `CommandRegistry::add_guard` registers process-wide (there is no
//! per-test reset), so every guard here is scoped to check only its own
//! uniquely-named test verb and admit everything else unconditionally --
//! it must never interfere with any other test in this binary that
//! dispatches a different verb.

use clap_noun_verb::autonomic::{Guard, GuardContext, GuardDenial};
use clap_noun_verb::cli::CommandRegistry;
use clap_noun_verb::logic::{HandlerContext, HandlerInput, HandlerOutput};
use std::collections::HashMap;

/// Refuses only `noun == GUARDED_NOUN`; admits every other invocation
/// unconditionally, so registering this guard never affects unrelated
/// tests sharing the same process-wide `CommandRegistry`.
struct RefusesOneNoun {
    noun: &'static str,
}

impl Guard for RefusesOneNoun {
    fn name(&self) -> &'static str {
        "refuses_one_noun"
    }
    fn check(&self, ctx: &GuardContext<'_>) -> std::result::Result<(), GuardDenial> {
        if ctx.noun == self.noun {
            Err(GuardDenial::new("NOUN_REFUSED", "this noun is refused by a real test guard"))
        } else {
            Ok(())
        }
    }
}

#[test]
fn a_real_guard_refuses_a_real_dispatch_before_the_handler_ever_runs() {
    // Arrange: a handler that would prove itself invoked via a side effect,
    // so we can assert it was NEVER called.
    static HANDLER_WAS_CALLED: std::sync::atomic::AtomicBool =
        std::sync::atomic::AtomicBool::new(false);

    CommandRegistry::register_verb(
        "guard_it_refused",
        "action",
        "A verb a real guard refuses before dispatch",
        |_input: HandlerInput| -> clap_noun_verb::Result<HandlerOutput> {
            HANDLER_WAS_CALLED.store(true, std::sync::atomic::Ordering::SeqCst);
            Ok(HandlerOutput::from_data(serde_json::json!({"ran": true}))?)
        },
    );
    CommandRegistry::add_guard(Box::new(RefusesOneNoun { noun: "guard_it_refused" }));

    // Act
    let registry_lock = CommandRegistry::get();
    let registry = registry_lock.lock().unwrap_or_else(|e| e.into_inner());
    let input = HandlerInput {
        args: HashMap::new(),
        opts: HashMap::new(),
        context: HandlerContext::new("action").with_noun("guard_it_refused"),
    };
    let result = registry.execute_verb("guard_it_refused", "action", input);
    drop(registry);

    // Assert: refused, with the real guard's reason surfaced, and the
    // handler never ran.
    let error = result.expect_err("a denying guard must refuse the dispatch");
    let message = error.to_string();
    assert!(message.contains("NOUN_REFUSED"), "error was: {message}");
    assert!(message.contains("this noun is refused by a real test guard"), "error was: {message}");
    assert!(
        !HANDLER_WAS_CALLED.load(std::sync::atomic::Ordering::SeqCst),
        "the handler must never run once a guard has refused the invocation"
    );
}

#[test]
fn a_registered_guard_does_not_affect_dispatch_of_an_unrelated_noun() {
    // Arrange: register the SAME kind of guard (refusing only its own
    // noun), then dispatch a completely different, real verb -- it must
    // succeed normally, proving guard scoping is real, not accidental.
    CommandRegistry::register_verb(
        "guard_it_unaffected",
        "ping",
        "A verb no guard refuses",
        |_input: HandlerInput| -> clap_noun_verb::Result<HandlerOutput> {
            Ok(HandlerOutput::from_data(serde_json::json!({"pong": true}))?)
        },
    );
    CommandRegistry::add_guard(Box::new(RefusesOneNoun { noun: "guard_it_someone_elses_noun" }));

    // Act
    let registry_lock = CommandRegistry::get();
    let registry = registry_lock.lock().unwrap_or_else(|e| e.into_inner());
    let input = HandlerInput {
        args: HashMap::new(),
        opts: HashMap::new(),
        context: HandlerContext::new("ping").with_noun("guard_it_unaffected"),
    };
    let result = registry.execute_verb("guard_it_unaffected", "ping", input);
    drop(registry);

    // Assert
    assert!(result.is_ok(), "an unrelated noun must not be refused: {result:?}");
}
