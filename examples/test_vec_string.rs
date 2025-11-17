//! Test example: Using Vec<String> in #[verb] functions

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde::Serialize;

#[derive(Serialize, Debug)]
struct ProcessResult {
    count: usize,
    items: Vec<String>,
}

/// Process multiple items
/// 
/// # Arguments
/// * `items` - Items to process (multiple values)
#[verb("process", "batch")]
fn process_items(items: Vec<String>) -> Result<ProcessResult> {
    Ok(ProcessResult {
        count: items.len(),
        items: items.iter().map(|s| format!("Processed: {}", s)).collect(),
    })
}

/// Filter items
#[verb("filter", "batch")]  
fn filter_items(items: Vec<String>, pattern: String) -> Result<ProcessResult> {
    let filtered: Vec<String> = items
        .into_iter()
        .filter(|item| item.contains(&pattern))
        .collect();
    
    Ok(ProcessResult {
        count: filtered.len(),
        items: filtered,
    })
}

fn main() -> Result<()> {
    clap_noun_verb::run()
}
