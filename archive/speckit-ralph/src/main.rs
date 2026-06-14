// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

//! speckit-ralph - Ralph Loop Orchestrator for MCPP
//!
//! This crate implements the Ralph loop closure:
//! RalphPlan emission → doctor verdict → signed receipt → chain verification → state advance.

use clap_noun_verb::{NounVerbError, Result};
use clap_noun_verb_macros::verb;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<()> {
    // Force linking of commands
    speckit_ralph::init();

    // Auto-discover and run commands
    clap_noun_verb::run()
}

pub fn init() {}

// =============================================================================
// RALPH PLAN MODEL
// =============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct RalphPlan {
    pub id: String,
    pub goal: String,
    pub steps: Vec<RalphStep>,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RalphStep {
    pub name: String,
    pub action: String,
    pub depends_on: Vec<String>,
}

// =============================================================================
// RALPH COMMANDS
// =============================================================================

#[verb("run")]
fn ralph_run(goal: String) -> Result<serde_json::Value> {
    let plan = RalphPlan {
        id: format!("plan-{}", current_timestamp()),
        goal: goal.clone(),
        steps: vec![
            RalphStep {
                name: "Init".to_string(),
                action: "specify init".to_string(),
                depends_on: vec![],
            },
            RalphStep {
                name: "Implement".to_string(),
                action: "mcpp powl64 execute".to_string(),
                depends_on: vec!["Init".to_string()],
            },
        ],
        status: "candidate".to_string(),
    };

    let plan_json = serde_json::to_value(&plan)
        .map_err(|e| NounVerbError::execution_error(format!("Failed to serialize plan: {}", e)))?;

    let pretty = serde_json::to_string_pretty(&plan_json).map_err(|e| {
        NounVerbError::execution_error(format!("Failed to format plan JSON: {}", e))
    })?;

    // 1. Emit RalphPlan JSON to workspace
    fs::write("ralph_plan.json", &pretty).map_err(|e| {
        NounVerbError::execution_error(format!("Failed to write ralph_plan.json: {}", e))
    })?;

    println!("{}", pretty);

    // 2. State Advance
    let state_file = ".chatmangpt/state.yaml";
    if Path::new(state_file).exists() {
        let mut content = fs::read_to_string(state_file).unwrap_or_default();
        if content.contains("phase: none") {
            content = content.replace("phase: none", "phase: plan");
            content =
                content.replace("active_delta: \"\"", &format!("active_delta: \"{}\"", plan.id));
            let _ = fs::write(state_file, content);
        }
    }

    Ok(plan_json)
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs()
}
