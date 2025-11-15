//! Example: Output Format Plugins
//!
//! Demonstrates how to use OutputFormat to generate output in different formats
//! (JSON, YAML, TOML, Table, TSV) instead of just JSON.

use clap_noun_verb_macros::verb;
use clap_noun_verb::{Result, OutputFormat};
use serde::Serialize;

#[derive(Serialize, Debug)]
struct Product {
    id: u32,
    name: String,
    price: f64,
    in_stock: bool,
}

#[derive(Serialize, Debug)]
struct Inventory {
    products: Vec<Product>,
    total_value: f64,
}

// Generate sample inventory
fn get_inventory() -> Inventory {
    let products = vec![
        Product {
            id: 1,
            name: "Laptop".to_string(),
            price: 999.99,
            in_stock: true,
        },
        Product {
            id: 2,
            name: "Mouse".to_string(),
            price: 29.99,
            in_stock: true,
        },
        Product {
            id: 3,
            name: "Keyboard".to_string(),
            price: 79.99,
            in_stock: false,
        },
    ];

    let total_value = products.iter().map(|p| p.price).sum();

    Inventory {
        products,
        total_value,
    }
}

/// List inventory in JSON format (default)
#[verb("json")]
fn show_json() -> Result<Inventory> {
    Ok(get_inventory())
}

/// List inventory (demonstrating format flexibility)
/// In a real app, this would accept --format argument
#[verb("all")]
fn show_all_formats() -> Result<String> {
    let inventory = get_inventory();

    let json = OutputFormat::Json
        .format(&inventory)
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(e.to_string()))?;
    let yaml = OutputFormat::Yaml
        .format(&inventory)
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(e.to_string()))?;
    let table = OutputFormat::Table
        .format(&inventory)
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(e.to_string()))?;

    Ok(format!(
        "=== JSON ===\n{}\n\n=== YAML ===\n{}\n\n=== TABLE ===\n{}",
        json, yaml, table
    ))
}

/// List products as table
#[verb("table")]
fn show_table() -> Result<Vec<Product>> {
    Ok(get_inventory().products)
}

/// Export as TSV for spreadsheet
#[verb("tsv")]
fn export_tsv() -> Result<Vec<Product>> {
    Ok(get_inventory().products)
}

/// Show summary statistics
#[verb("summary")]
fn show_summary() -> Result<Inventory> {
    Ok(get_inventory())
}

fn main() -> Result<()> {
    clap_noun_verb::run()
}

// Usage examples:
// $ cargo run --example format_example -- inventory json
// $ cargo run --example format_example -- inventory table
// $ cargo run --example format_example -- inventory tsv
// $ cargo run --example format_example -- inventory all
