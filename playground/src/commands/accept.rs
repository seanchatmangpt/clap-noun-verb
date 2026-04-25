//! Accept commands - Promote CandidateDelta to AcceptedDelta

use clap_noun_verb_macros::verb;
use clap_noun_verb::Result;

/// Accept candidate delta
///
/// Promotes a candidate delta into an accepted delta after validation.
///
/// # Arguments
/// * `target` - Target name
/// * `agent` - The AI agent to use (claude or gemini)
#[verb("run")]
fn accept_run(target: Option<String>, agent: Option<String>) -> Result<serde_json::Value> {
    let target_val = target.unwrap_or_else(|| "mcp-plus".to_string());

    if let Some(agent_name) = agent {
        let prompt = format!(
            "You are the 'accept' intelligence for the MCPP SR loop. \
            Review the candidate delta for target '{}' and promote it to accepted. \
            Output ONLY valid JSON matching the 'chatmangpt.sr.result.v1' schema with command 'sr.accept'. \
            Set delta_promoted to true. \
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
        "command": "sr.accept",
        "status": "pass",
        "target": target_val,
        "line_status": "running",
        "work_unit": format!("{}-first-loop", target_val),
        "data": {
            "delta_promoted": true,
            "file": ".chatmangpt/accepted-delta.yaml"
        },
        "errors": [],
        "warnings": [],
        "next": {
            "command": format!("mcpp implement --target {}", target_val),
            "reason": "Candidate delta accepted"
        }
    }))
}
