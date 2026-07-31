// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Deterministic sales-pipeline inspection with no implicit CSV actuation.

use clap_noun_verb::{noun, run_cli_with_args, verb, NounVerbError, Result, VerbArgs};
use serde::Serialize;

#[derive(Debug, Clone)]
struct Deal {
    company: &'static str,
    amount: u64,
    stage: &'static str,
    probability: u32,
    days_inactive: u32,
}

#[derive(Debug, Serialize)]
struct PipelineSummary {
    deals: usize,
    open_weighted_value: u64,
    expected_close_30d: u64,
    at_risk: Vec<&'static str>,
    top_deal: &'static str,
}

fn deals() -> Vec<Deal> {
    vec![
        Deal { company: "Acme Corp", amount: 5_000, stage: "Proposal", probability: 75, days_inactive: 2 },
        Deal { company: "TechStart", amount: 25_000, stage: "Interested", probability: 60, days_inactive: 4 },
        Deal { company: "StartupXYZ", amount: 2_000, stage: "Proposal", probability: 85, days_inactive: 1 },
        Deal { company: "BigTech Inc", amount: 15_000, stage: "Interested", probability: 50, days_inactive: 8 },
    ]
}

fn weighted_value(deal: &Deal) -> u64 {
    deal.amount * u64::from(deal.probability) / 100
}

fn summary() -> PipelineSummary {
    let mut deals = deals();
    deals.sort_by(|left, right| {
        right.amount.cmp(&left.amount).then_with(|| left.company.cmp(right.company))
    });
    let open_weighted_value = deals.iter().map(weighted_value).sum();
    let expected_close_30d = deals
        .iter()
        .filter(|deal| deal.stage == "Proposal" && deal.probability > 60)
        .map(weighted_value)
        .sum();
    let at_risk = deals
        .iter()
        .filter(|deal| deal.days_inactive > 7)
        .map(|deal| deal.company)
        .collect();
    PipelineSummary {
        deals: deals.len(),
        open_weighted_value,
        expected_close_30d,
        at_risk,
        top_deal: deals.first().map_or("none", |deal| deal.company),
    }
}

fn emit() -> Result<()> {
    println!(
        "{}",
        serde_json::to_string(&summary())
            .map_err(|error| NounVerbError::execution_error(error.to_string()))?
    );
    Ok(())
}

fn build() -> impl FnOnce(clap_noun_verb::CliBuilder) -> clap_noun_verb::CliBuilder {
    |builder| {
        builder.name("revops").version("26.7.62").noun(noun!(
            "pipeline",
            "Inspect the admitted sales pipeline",
            [verb!("summary", "Render weighted pipeline standing", |_args: &VerbArgs| {
                emit()
            })]
        ))
    }
}

fn main() -> Result<()> {
    let observed = summary();
    assert_eq!(observed.deals, 4);
    assert_eq!(observed.top_deal, "TechStart");
    assert_eq!(observed.at_risk, vec!["BigTech Inc"]);
    run_cli_with_args(
        vec!["revops".into(), "pipeline".into(), "summary".into()],
        build(),
    )?;
    println!("Pipeline summary dispatched without filesystem mutation");
    Ok(())
}
