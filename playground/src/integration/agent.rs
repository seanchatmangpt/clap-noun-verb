use clap_noun_verb::error::{NounVerbError, Result};
use std::process::Command;

pub enum AgentType {
    Claude,
    Gemini,
}

pub fn run_headless(agent: AgentType, prompt: &str) -> Result<String> {
    let (cmd, args) = match agent {
        AgentType::Claude => ("claude", vec!["-p", prompt]),
        AgentType::Gemini => ("gemini", vec!["ask", prompt]),
    };

    let output = Command::new(cmd)
        .args(&args)
        .output()
        .map_err(|e| NounVerbError::execution_error(format!("Failed to execute agent {}: {}", cmd, e)))?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(NounVerbError::execution_error(format!("Agent failed: {}", err)));
    }

    let result = String::from_utf8_lossy(&output.stdout).to_string();
    
    // Robustly extract the JSON object from the response (first { to last })
    let start_idx = result.find('{');
    let end_idx = result.rfind('}');
    
    if let (Some(start), Some(end)) = (start_idx, end_idx) {
        if start <= end {
            return Ok(result[start..=end].to_string());
        }
    }
    
    // If we didn't find braces, just clean up markdown blocks and trim
    let mut cleaned = result.trim();
    if cleaned.starts_with("```json") {
        cleaned = cleaned.strip_prefix("```json").unwrap_or(cleaned);
    } else if cleaned.starts_with("```") {
        cleaned = cleaned.strip_prefix("```").unwrap_or(cleaned);
    }
    
    if cleaned.ends_with("```") {
        cleaned = cleaned.strip_suffix("```").unwrap_or(cleaned);
    }

    Ok(cleaned.trim().to_string())
}
