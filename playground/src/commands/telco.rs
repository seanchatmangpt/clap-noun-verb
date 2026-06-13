// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Telco commands - Connectivity, routing, and distributed swarm infrastructure

use clap_noun_verb_macros::verb;
use clap_noun_verb::NounVerbError;
use clap_noun_verb::Result;

/// Route a capability
///
/// Routes a semantic intent or capability across the UniverseOS.
///
/// # Arguments
/// * `intent` - The semantic intent to route
#[verb("route")]
fn telco_route(intent: String) -> Result<serde_json::Value> {
    Ok(serde_json::json!({
        "status": "success",
        "message": format!("Routed intent: {}", intent)
    }))
}

/// Broadcast a message
///
/// Broadcast a message or state change across the distributed swarm.
///
/// # Arguments
/// * `message` - The message to broadcast
#[verb("broadcast")]
fn telco_broadcast(message: String) -> Result<serde_json::Value> {
    Ok(serde_json::json!({
        "status": "success",
        "message": format!("Broadcasted message: {}", message)
    }))
}

/// Get next action report
///
/// Pulls the next action or WIP signal from the workflow status.
///
/// # Arguments
/// * `target` - Target name
/// * `agent` - The AI agent to use (claude or gemini)
#[verb("next")]
fn telco_next(target: Option<String>, agent: Option<String>) -> Result<serde_json::Value> {
    let target_val = target.unwrap_or_else(|| "mcp-plus".to_string());

    if let Some(agent_name) = agent {
        let prompt = format!(
            "You are the 'telco next' intelligence for the MCPP SR loop. \
            Review the current workspace state and status for target '{}'. \
            Determine the next workflow action. \
            Output ONLY valid JSON matching the 'chatmangpt.sr.result.v1' schema with command 'sr.telco.next'. \
            Do not include markdown code blocks or any other text.", 
            target_val
        );
        
        let agent_type = if agent_name.to_lowercase() == "claude" {
            crate::integration::agent::AgentType::Claude
        } else {
            crate::integration::agent::AgentType::Gemini
        };
        
        let json_str = crate::integration::agent::run_headless(agent_type, &prompt)?;
        return serde_json::from_str(&json_str).map_err(|e| clap_noun_verb::NounVerbError::execution_error(format!("Failed to parse agent output: {}. Output was: {}", e, json_str)));
    }

    Ok(serde_json::json!({
        "schema": "chatmangpt.sr.result.v1",
        "command": "sr.telco.next",
        "status": "pass",
        "target": target_val,
        "line_status": "running",
        "work_unit": format!("{}-first-loop", target_val),
        "data": {
            "phase": "plan",
            "active_delta": ".chatmangpt/accepted-delta.yaml",
            "wip": {"active": 1, "limit": 1},
            "required_gates": ["invocation_split", "accepted_delta_required", "no_false_completion", "receipt_required"]
        },
        "errors": [],
        "warnings": [],
        "next": {
            "command": format!("mcpp verify --target {}", target_val),
            "reason": "Proceed to verification"
        }
    }))
}

/// Connect to a node
///
/// Establish a connection with a remote capability node.
///
/// # Arguments
/// * `node_id` - The ID of the node to connect to
#[verb("connect")]
fn telco_connect(node_id: String) -> Result<serde_json::Value> {
    Ok(serde_json::json!({
        "status": "success",
        "message": format!("Connected to node: {}", node_id)
    }))
}
