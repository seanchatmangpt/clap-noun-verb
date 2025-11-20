//! Domain logic for report generation - ZERO CLI dependencies
//!
//! This module contains pure business logic with no knowledge of:
//! - Command-line arguments
//! - File paths
//! - User interaction

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Domain error types
#[derive(Debug, Error, PartialEq)]
pub enum ReportError {
    #[error("No data available for report")]
    NoData,

    #[error("Invalid aggregation: {0}")]
    InvalidAggregation(String),

    #[error("Formatting failed: {0}")]
    FormattingFailed(String),
}

/// Sales record - domain model
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SalesRecord {
    pub id: String,
    pub product: String,
    pub category: String,
    pub amount: f64,
    pub quantity: u32,
    pub date: DateTime<Utc>,
}

/// Aggregated statistics - domain model
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SalesStats {
    pub total_revenue: f64,
    pub total_quantity: u32,
    pub average_amount: f64,
    pub by_category: HashMap<String, CategoryStats>,
    pub by_product: HashMap<String, ProductStats>,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CategoryStats {
    pub revenue: f64,
    pub quantity: u32,
    pub product_count: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProductStats {
    pub revenue: f64,
    pub quantity: u32,
    pub average_price: f64,
}

/// Report format - domain model
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ReportFormat {
    Json,
    Csv,
    Markdown,
}

/// Core domain logic - aggregate sales data
///
/// Pure function: takes data, returns statistics
pub fn aggregate_sales(records: Vec<SalesRecord>) -> Result<SalesStats, ReportError> {
    if records.is_empty() {
        return Err(ReportError::NoData);
    }

    let mut total_revenue = 0.0;
    let mut total_quantity = 0u32;
    let mut by_category: HashMap<String, CategoryStats> = HashMap::new();
    let mut by_product: HashMap<String, ProductStats> = HashMap::new();

    let period_start = records.iter()
        .map(|r| r.date)
        .min()
        .unwrap();

    let period_end = records.iter()
        .map(|r| r.date)
        .max()
        .unwrap();

    for record in records {
        total_revenue += record.amount;
        total_quantity += record.quantity;

        // Aggregate by category
        let cat_stats = by_category.entry(record.category.clone()).or_insert(CategoryStats {
            revenue: 0.0,
            quantity: 0,
            product_count: 0,
        });
        cat_stats.revenue += record.amount;
        cat_stats.quantity += record.quantity;

        // Aggregate by product
        let prod_stats = by_product.entry(record.product.clone()).or_insert(ProductStats {
            revenue: 0.0,
            quantity: 0,
            average_price: 0.0,
        });
        prod_stats.revenue += record.amount;
        prod_stats.quantity += record.quantity;
    }

    // Compute product counts per category
    for (product_name, _stats) in &by_product {
        let category = product_name.split('-').next().unwrap_or("Unknown").to_string();
        if let Some(cat) = by_category.get_mut(&category) {
            cat.product_count += 1;
        }
    }

    // Compute average prices
    for stats in by_product.values_mut() {
        if stats.quantity > 0 {
            stats.average_price = stats.revenue / stats.quantity as f64;
        }
    }

    let average_amount = if total_quantity > 0 {
        total_revenue / total_quantity as f64
    } else {
        0.0
    };

    Ok(SalesStats {
        total_revenue,
        total_quantity,
        average_amount,
        by_category,
        by_product,
        period_start,
        period_end,
    })
}

/// Format report as JSON - pure function
pub fn format_json(stats: &SalesStats) -> Result<String, ReportError> {
    serde_json::to_string_pretty(stats)
        .map_err(|e| ReportError::FormattingFailed(e.to_string()))
}

/// Format report as CSV - pure function
pub fn format_csv(stats: &SalesStats) -> Result<String, ReportError> {
    let mut output = String::new();

    // Summary section
    output.push_str("Summary\n");
    output.push_str(&format!("Total Revenue,{:.2}\n", stats.total_revenue));
    output.push_str(&format!("Total Quantity,{}\n", stats.total_quantity));
    output.push_str(&format!("Average Amount,{:.2}\n", stats.average_amount));
    output.push_str(&format!("Period,{} to {}\n\n", stats.period_start, stats.period_end));

    // Category breakdown
    output.push_str("Category,Revenue,Quantity,Product Count\n");
    let mut categories: Vec<_> = stats.by_category.iter().collect();
    categories.sort_by(|a, b| b.1.revenue.partial_cmp(&a.1.revenue).unwrap());

    for (category, cat_stats) in categories {
        output.push_str(&format!("{},{:.2},{},{}\n",
            category, cat_stats.revenue, cat_stats.quantity, cat_stats.product_count));
    }

    output.push_str("\n");

    // Product breakdown
    output.push_str("Product,Revenue,Quantity,Average Price\n");
    let mut products: Vec<_> = stats.by_product.iter().collect();
    products.sort_by(|a, b| b.1.revenue.partial_cmp(&a.1.revenue).unwrap());

    for (product, prod_stats) in products {
        output.push_str(&format!("{},{:.2},{},{:.2}\n",
            product, prod_stats.revenue, prod_stats.quantity, prod_stats.average_price));
    }

    Ok(output)
}

/// Format report as Markdown - pure function
pub fn format_markdown(stats: &SalesStats) -> Result<String, ReportError> {
    let mut output = String::new();

    output.push_str("# Sales Report\n\n");
    output.push_str(&format!("**Period:** {} to {}\n\n", stats.period_start, stats.period_end));

    // Summary
    output.push_str("## Summary\n\n");
    output.push_str(&format!("- **Total Revenue:** ${:.2}\n", stats.total_revenue));
    output.push_str(&format!("- **Total Quantity:** {}\n", stats.total_quantity));
    output.push_str(&format!("- **Average Amount:** ${:.2}\n\n", stats.average_amount));

    // Category breakdown
    output.push_str("## By Category\n\n");
    output.push_str("| Category | Revenue | Quantity | Products |\n");
    output.push_str("|----------|---------|----------|----------|\n");

    let mut categories: Vec<_> = stats.by_category.iter().collect();
    categories.sort_by(|a, b| b.1.revenue.partial_cmp(&a.1.revenue).unwrap());

    for (category, cat_stats) in categories {
        output.push_str(&format!("| {} | ${:.2} | {} | {} |\n",
            category, cat_stats.revenue, cat_stats.quantity, cat_stats.product_count));
    }

    output.push_str("\n");

    // Product breakdown
    output.push_str("## Top Products\n\n");
    output.push_str("| Product | Revenue | Quantity | Avg Price |\n");
    output.push_str("|---------|---------|----------|----------|\n");

    let mut products: Vec<_> = stats.by_product.iter().collect();
    products.sort_by(|a, b| b.1.revenue.partial_cmp(&a.1.revenue).unwrap());

    for (product, prod_stats) in products.iter().take(10) {
        output.push_str(&format!("| {} | ${:.2} | {} | ${:.2} |\n",
            product, prod_stats.revenue, prod_stats.quantity, prod_stats.average_price));
    }

    Ok(output)
}

/// Format report based on requested format - dispatcher function
pub fn format_report(stats: &SalesStats, format: ReportFormat) -> Result<String, ReportError> {
    match format {
        ReportFormat::Json => format_json(stats),
        ReportFormat::Csv => format_csv(stats),
        ReportFormat::Markdown => format_markdown(stats),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_records() -> Vec<SalesRecord> {
        vec![
            SalesRecord {
                id: "1".to_string(),
                product: "Electronics-Laptop".to_string(),
                category: "Electronics".to_string(),
                amount: 1200.0,
                quantity: 1,
                date: Utc::now(),
            },
            SalesRecord {
                id: "2".to_string(),
                product: "Electronics-Mouse".to_string(),
                category: "Electronics".to_string(),
                amount: 25.0,
                quantity: 2,
                date: Utc::now(),
            },
            SalesRecord {
                id: "3".to_string(),
                product: "Books-Novel".to_string(),
                category: "Books".to_string(),
                amount: 15.0,
                quantity: 3,
                date: Utc::now(),
            },
        ]
    }

    #[test]
    fn test_aggregate_sales_success() {
        // Arrange
        let records = sample_records();

        // Act
        let stats = aggregate_sales(records).unwrap();

        // Assert
        assert_eq!(stats.total_revenue, 1240.0);
        assert_eq!(stats.total_quantity, 6);
        assert_eq!(stats.by_category.len(), 2);
        assert_eq!(stats.by_product.len(), 3);
    }

    #[test]
    fn test_aggregate_sales_empty_fails() {
        // Arrange
        let records = vec![];

        // Act
        let result = aggregate_sales(records);

        // Assert
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ReportError::NoData);
    }

    #[test]
    fn test_format_json_produces_valid_json() {
        // Arrange
        let stats = aggregate_sales(sample_records()).unwrap();

        // Act
        let json = format_json(&stats).unwrap();

        // Assert
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed["total_revenue"].is_f64());
        assert!(parsed["by_category"].is_object());
    }

    #[test]
    fn test_format_csv_contains_headers() {
        // Arrange
        let stats = aggregate_sales(sample_records()).unwrap();

        // Act
        let csv = format_csv(&stats).unwrap();

        // Assert
        assert!(csv.contains("Summary"));
        assert!(csv.contains("Category,Revenue,Quantity"));
        assert!(csv.contains("Product,Revenue,Quantity"));
    }

    #[test]
    fn test_format_markdown_contains_tables() {
        // Arrange
        let stats = aggregate_sales(sample_records()).unwrap();

        // Act
        let md = format_markdown(&stats).unwrap();

        // Assert
        assert!(md.contains("# Sales Report"));
        assert!(md.contains("## Summary"));
        assert!(md.contains("| Category | Revenue"));
        assert!(md.contains("| Product | Revenue"));
    }
}
