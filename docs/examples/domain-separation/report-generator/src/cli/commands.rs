//! CLI layer - thin wrapper around domain logic

use crate::domain::report::{self, ReportFormat, SalesRecord};
use anyhow::{Context, Result};
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::PathBuf;

/// Generate command - CLI entry point
pub fn generate(
    input_path: PathBuf,
    output_path: Option<PathBuf>,
    format: String,
) -> Result<()> {
    // Parse format - CLI responsibility
    let report_format = parse_format(&format)?;

    // Load data - CLI responsibility
    let records = load_records(&input_path)?;

    // Aggregate using domain logic
    let stats = report::aggregate_sales(records)
        .context("Failed to aggregate sales data")?;

    // Format using domain logic
    let report = report::format_report(&stats, report_format)
        .context("Failed to format report")?;

    // Write output - CLI responsibility
    match output_path {
        Some(path) => {
            let mut file = File::create(&path)
                .with_context(|| format!("Failed to create output file: {:?}", path))?;
            file.write_all(report.as_bytes())?;
            println!("✓ Report saved to: {:?}", path);
        }
        None => {
            println!("{}", report);
        }
    }

    // Print summary - CLI responsibility
    println!();
    println!("Report Summary:");
    println!("  Total Revenue: ${:.2}", stats.total_revenue);
    println!("  Total Quantity: {}", stats.total_quantity);
    println!("  Categories: {}", stats.by_category.len());
    println!("  Products: {}", stats.by_product.len());

    Ok(())
}

fn parse_format(format: &str) -> Result<ReportFormat> {
    match format.to_lowercase().as_str() {
        "json" => Ok(ReportFormat::Json),
        "csv" => Ok(ReportFormat::Csv),
        "md" | "markdown" => Ok(ReportFormat::Markdown),
        _ => anyhow::bail!("Unsupported format: {}. Use json, csv, or markdown", format),
    }
}

fn load_records(path: &PathBuf) -> Result<Vec<SalesRecord>> {
    let file = File::open(path)
        .with_context(|| format!("Failed to open input file: {:?}", path))?;
    let reader = BufReader::new(file);
    let mut csv_reader = csv::Reader::from_reader(reader);

    let mut records = Vec::new();
    for result in csv_reader.deserialize() {
        let record: SalesRecord = result
            .context("Failed to parse CSV record")?;
        records.push(record);
    }

    Ok(records)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_format_success() {
        assert_eq!(parse_format("json").unwrap(), ReportFormat::Json);
        assert_eq!(parse_format("CSV").unwrap(), ReportFormat::Csv);
        assert_eq!(parse_format("markdown").unwrap(), ReportFormat::Markdown);
        assert_eq!(parse_format("md").unwrap(), ReportFormat::Markdown);
    }

    #[test]
    fn test_parse_format_invalid_fails() {
        assert!(parse_format("xml").is_err());
    }

    #[test]
    fn test_generate_command_creates_output() {
        // Arrange - create temp input file
        let mut input_file = NamedTempFile::new().unwrap();
        writeln!(input_file, "id,product,category,amount,quantity,date").unwrap();
        writeln!(input_file, "1,Laptop,Electronics,1200.0,1,2024-01-01T00:00:00Z").unwrap();
        writeln!(input_file, "2,Mouse,Electronics,25.0,2,2024-01-01T00:00:00Z").unwrap();

        let output_file = NamedTempFile::new().unwrap();
        let output_path = output_file.path().to_path_buf();

        // Act
        let result = generate(
            input_file.path().to_path_buf(),
            Some(output_path.clone()),
            "json".to_string(),
        );

        // Assert
        assert!(result.is_ok());
        assert!(output_path.exists());

        let content = std::fs::read_to_string(output_path).unwrap();
        assert!(content.contains("total_revenue"));
        assert!(content.contains("Electronics"));
    }
}
