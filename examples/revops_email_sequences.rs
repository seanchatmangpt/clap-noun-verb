// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Local-only email sequence rendering. No message is sent or drafted remotely.

use clap_noun_verb::{noun, run_cli_with_args, verb, NounVerbError, Result, VerbArgs};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
struct SequenceStep {
    day: u32,
    subject: &'static str,
    body: &'static str,
}

#[derive(Debug, Serialize)]
struct SequenceReceipt {
    sequence: &'static str,
    steps: Vec<SequenceStep>,
    delivery_performed: bool,
}

fn support_sequence() -> SequenceReceipt {
    SequenceReceipt {
        sequence: "support-evaluation",
        steps: vec![
            SequenceStep {
                day: 0,
                subject: "CLI support evaluation",
                body: "Describe the current CLI constraint and request a bounded architecture review.",
            },
            SequenceStep {
                day: 3,
                subject: "CLI support evaluation follow-up",
                body: "Provide one concrete diagnostic or benchmark relevant to the stated constraint.",
            },
            SequenceStep {
                day: 7,
                subject: "Close the evaluation loop",
                body: "Ask whether the evaluation should proceed, pause, or be closed.",
            },
        ],
        delivery_performed: false,
    }
}

fn emit() -> Result<()> {
    println!(
        "{}",
        serde_json::to_string(&support_sequence())
            .map_err(|error| NounVerbError::execution_error(error.to_string()))?
    );
    Ok(())
}

fn build() -> impl FnOnce(clap_noun_verb::CliBuilder) -> clap_noun_verb::CliBuilder {
    |builder| {
        builder.name("revops").version("26.7.62").noun(noun!(
            "sequence",
            "Render local communication sequences",
            [verb!("render", "Render a bounded support sequence", |_args: &VerbArgs| {
                emit()
            })]
        ))
    }
}

fn main() -> Result<()> {
    let sequence = support_sequence();
    assert_eq!(sequence.steps.len(), 3);
    assert!(!sequence.delivery_performed);
    assert!(sequence.steps.windows(2).all(|pair| pair[0].day < pair[1].day));
    run_cli_with_args(
        vec!["revops".into(), "sequence".into(), "render".into()],
        build(),
    )?;
    println!("Sequence rendered locally; delivery_performed=false");
    Ok(())
}
