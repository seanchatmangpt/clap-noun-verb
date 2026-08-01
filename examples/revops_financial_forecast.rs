// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Deterministic twelve-period forecast through a read-only noun-verb route.

use clap_noun_verb::{noun, run_cli_with_args, verb, NounVerbError, Result, VerbArgs};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
struct PeriodForecast {
    period: u32,
    revenue: u64,
    costs: u64,
    profit: i64,
}

#[derive(Debug, Serialize)]
struct ForecastReceipt {
    periods: Vec<PeriodForecast>,
    total_revenue: u64,
    total_costs: u64,
    total_profit: i64,
    assumption: &'static str,
}

fn grow(value: u64, basis_points: u64) -> u64 {
    value.saturating_mul(10_000 + basis_points) / 10_000
}

fn forecast() -> ForecastReceipt {
    let mut revenue = 8_500_u64;
    let mut costs = 1_500_u64;
    let mut periods = Vec::new();
    for period in 1..=12 {
        periods.push(PeriodForecast {
            period,
            revenue,
            costs,
            profit: revenue as i64 - costs as i64,
        });
        revenue = grow(revenue, 2_500);
        costs = grow(costs, 1_500);
    }
    let total_revenue = periods.iter().map(|item| item.revenue).sum();
    let total_costs = periods.iter().map(|item| item.costs).sum();
    ForecastReceipt {
        periods,
        total_revenue,
        total_costs,
        total_profit: total_revenue as i64 - total_costs as i64,
        assumption: "revenue +2500bps/period; costs +1500bps/period",
    }
}

fn emit() -> Result<()> {
    println!(
        "{}",
        serde_json::to_string(&forecast())
            .map_err(|error| NounVerbError::execution_error(error.to_string()))?
    );
    Ok(())
}

fn build() -> impl FnOnce(clap_noun_verb::CliBuilder) -> clap_noun_verb::CliBuilder {
    |builder| {
        builder.name("revops").version("26.7.62").noun(noun!(
            "forecast",
            "Manufacture a bounded financial scenario",
            [verb!("annual", "Render twelve deterministic periods", |_args: &VerbArgs| {
                emit()
            })]
        ))
    }
}

fn main() -> Result<()> {
    let receipt = forecast();
    assert_eq!(receipt.periods.len(), 12);
    assert!(receipt.total_profit > 0);
    assert!(receipt.periods.windows(2).all(|pair| pair[1].revenue > pair[0].revenue));
    run_cli_with_args(
        vec!["revops".into(), "forecast".into(), "annual".into()],
        build(),
    )?;
    println!("Forecast manufactured from explicit bounded assumptions");
    Ok(())
}
