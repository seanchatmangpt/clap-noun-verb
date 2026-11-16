// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Read-only RevOps revenue summary through the noun-verb dispatch path.

use clap_noun_verb::{noun, run_cli_with_args, verb, NounVerbError, Result, VerbArgs};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
struct RevenueSummary {
    current_mrr: u64,
    current_arr: u64,
    weighted_growth_bps: i64,
    customers: u32,
    churned_customers: u32,
    net_revenue_retention_bps: u32,
}

fn summary() -> RevenueSummary {
    let streams = [3_500_u64, 1_200, 5_000, 2_000, 800, 400, 550];
    let current_mrr: u64 = streams.into_iter().sum();
    let new_mrr = 8_000_i64;
    let churn = 500_i64;
    let expansion = 2_600_i64;
    RevenueSummary {
        current_mrr,
        current_arr: current_mrr * 12,
        weighted_growth_bps: ((new_mrr - churn) * 10_000) / current_mrr as i64,
        customers: 52,
        churned_customers: 1,
        net_revenue_retention_bps: (((current_mrr as i64 + expansion - churn) * 10_000)
            / current_mrr as i64) as u32,
    }
}

fn emit() -> Result<()> {
    let rendered = serde_json::to_string(&summary())
        .map_err(|error| NounVerbError::execution_error(error.to_string()))?;
    println!("{rendered}");
    Ok(())
}

fn build() -> impl FnOnce(clap_noun_verb::CliBuilder) -> clap_noun_verb::CliBuilder {
    |builder| {
        builder.name("revops").version("26.7.62").noun(noun!(
            "revenue",
            "Inspect admitted revenue observations",
            [verb!("summary", "Render deterministic revenue metrics", |_args: &VerbArgs| {
                emit()
            })]
        ))
    }
}

fn main() -> Result<()> {
    let expected = summary();
    assert_eq!(expected.current_mrr, 13_450);
    assert_eq!(expected.current_arr, 161_400);
    assert!(expected.net_revenue_retention_bps > 10_000);
    run_cli_with_args(vec!["revops".into(), "revenue".into(), "summary".into()], build())?;
    println!("Revenue summary dispatched without financial actuation");
    Ok(())
}
