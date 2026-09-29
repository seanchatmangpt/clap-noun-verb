// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Read-only customer-health diagnosis through a noun-verb route.

use clap_noun_verb::{noun, run_cli_with_args, verb, NounVerbError, Result, VerbArgs};
use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum HealthStanding {
    Healthy,
    LowUsage,
    AtRisk,
}

#[derive(Debug, Serialize)]
struct CustomerHealth {
    customer: &'static str,
    usage_percent: u32,
    days_inactive: u32,
    standing: HealthStanding,
    recommended_action: &'static str,
    outreach_performed: bool,
}

fn diagnose(customer: &'static str, usage_percent: u32, days_inactive: u32) -> CustomerHealth {
    let (standing, recommended_action) = if days_inactive > 7 {
        (HealthStanding::AtRisk, "review inactivity and request a check-in")
    } else if usage_percent < 50 {
        (HealthStanding::LowUsage, "review onboarding and remove blockers")
    } else {
        (HealthStanding::Healthy, "continue the scheduled success review")
    };
    CustomerHealth {
        customer,
        usage_percent,
        days_inactive,
        standing,
        recommended_action,
        outreach_performed: false,
    }
}

fn receipt() -> Vec<CustomerHealth> {
    vec![diagnose("Acme Corp", 100, 8), diagnose("TechStart", 95, 2), diagnose("StartupXYZ", 40, 5)]
}

fn emit() -> Result<()> {
    println!(
        "{}",
        serde_json::to_string(&receipt())
            .map_err(|error| NounVerbError::execution_error(error.to_string()))?
    );
    Ok(())
}

fn build() -> impl FnOnce(clap_noun_verb::CliBuilder) -> clap_noun_verb::CliBuilder {
    |builder| {
        builder.name("revops").version("26.7.62").noun(noun!(
            "customer",
            "Diagnose customer-success observations",
            [verb!("health", "Render customer health standing", |_args: &VerbArgs| { emit() })]
        ))
    }
}

fn main() -> Result<()> {
    let observed = receipt();
    assert_eq!(observed[0].standing, HealthStanding::AtRisk);
    assert_eq!(observed[1].standing, HealthStanding::Healthy);
    assert_eq!(observed[2].standing, HealthStanding::LowUsage);
    assert!(observed.iter().all(|item| !item.outreach_performed));
    run_cli_with_args(vec!["revops".into(), "customer".into(), "health".into()], build())?;
    println!("Customer health diagnosed; outreach_performed=false");
    Ok(())
}
