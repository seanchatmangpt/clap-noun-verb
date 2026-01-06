//! RevOps Sales Pipeline Tracker
//!
//! Minimal CLI for tracking leads, deals, and revenue pipeline
//! Build: cargo build --example revops_sales_pipeline --features examples
//! Run: ./target/debug/examples/revops_sales_pipeline

use std::fs;
use std::path::Path;

fn main() {
    let pipeline = Pipeline::load_or_create("pipeline.csv");

    println!("╔════════════════════════════════════════════════════════╗");
    println!("║          SALES PIPELINE DASHBOARD                      ║");
    println!("╚════════════════════════════════════════════════════════╝\n");

    // Pipeline Summary
    println!("📊 PIPELINE SUMMARY");
    println!("─────────────────────────────────────────────────────────");
    println!("Total Prospects:          {}", pipeline.total_prospects());
    println!("In Pipeline:              {}", pipeline.in_pipeline_count());
    println!("Closed Won:               {}", pipeline.closed_won_count());
    println!("Conversion Rate:          {:.1}%\n", pipeline.conversion_rate());

    // Pipeline Value
    println!("💰 PIPELINE VALUE");
    println!("─────────────────────────────────────────────────────────");
    println!("Prospect Stage ($):       ${:.0}", pipeline.value_by_stage("Prospect"));
    println!("Interested Stage ($):     ${:.0}", pipeline.value_by_stage("Interested"));
    println!("Proposal Stage ($):       ${:.0}", pipeline.value_by_stage("Proposal"));
    println!("Total Open Pipeline:      ${:.0}", pipeline.open_value());
    println!("Expected 30-day Close:    ${:.0}\n", pipeline.expected_close_30d());

    // By Stream
    println!("🎯 REVENUE BY STREAM");
    println!("─────────────────────────────────────────────────────────");
    for stream in ["Support", "Training", "Consulting", "ggen", "Frontier", "Enterprise"].iter() {
        let value = pipeline.value_by_stream(stream);
        let count = pipeline.count_by_stream(stream);
        if count > 0 {
            println!("{:<15} ${:>10.0}  ({} deals)", stream, value, count);
        }
    }

    println!("\n🔥 TOP 5 DEALS");
    println!("─────────────────────────────────────────────────────────");
    for deal in pipeline.top_deals(5) {
        println!("{:<20} ${:>10.0}  {}  ({}%)",
                 deal.company, deal.amount, deal.stage, deal.probability);
    }

    println!("\n⚠️  AT RISK (No activity 7+ days)");
    println!("─────────────────────────────────────────────────────────");
    for deal in pipeline.at_risk() {
        println!("{:<20} {} days idle", deal.company, deal.days_inactive);
    }

    println!("\n📈 QUICK ADD COMMAND:");
    println!("   revops add <company> <stream> <amount> <probability>");
    println!("   Example: revops add Acme Support 500 75");
}

#[derive(Clone)]
struct Deal {
    company: String,
    stream: String,
    amount: f64,
    stage: String,
    probability: u32,
    days_inactive: u32,
}

struct Pipeline {
    deals: Vec<Deal>,
}

impl Pipeline {
    fn load_or_create(path: &str) -> Self {
        if Path::new(path).exists() {
            let data = fs::read_to_string(path).unwrap_or_default();
            let mut deals = Vec::new();

            for line in data.lines().skip(1) {
                let parts: Vec<&str> = line.split(',').collect();
                if parts.len() >= 6 {
                    deals.push(Deal {
                        company: parts[0].to_string(),
                        stream: parts[1].to_string(),
                        amount: parts[2].parse().unwrap_or(0.0),
                        stage: parts[3].to_string(),
                        probability: parts[4].parse().unwrap_or(0),
                        days_inactive: parts[5].parse().unwrap_or(0),
                    });
                }
            }
            Pipeline { deals }
        } else {
            // Create sample data
            Pipeline {
                deals: vec![
                    Deal {
                        company: "Acme Corp".to_string(),
                        stream: "Support".to_string(),
                        amount: 5000.0,
                        stage: "Proposal".to_string(),
                        probability: 75,
                        days_inactive: 2,
                    },
                    Deal {
                        company: "TechStart".to_string(),
                        stream: "Consulting".to_string(),
                        amount: 25000.0,
                        stage: "Interested".to_string(),
                        probability: 60,
                        days_inactive: 4,
                    },
                    Deal {
                        company: "StartupXYZ".to_string(),
                        stream: "Training".to_string(),
                        amount: 2000.0,
                        stage: "Proposal".to_string(),
                        probability: 85,
                        days_inactive: 1,
                    },
                    Deal {
                        company: "BigTech Inc".to_string(),
                        stream: "ggen".to_string(),
                        amount: 15000.0,
                        stage: "Interested".to_string(),
                        probability: 50,
                        days_inactive: 8,
                    },
                ],
            }
        }
    }

    fn total_prospects(&self) -> usize {
        self.deals.len()
    }

    fn in_pipeline_count(&self) -> usize {
        self.deals.iter().filter(|d| d.stage != "Closed-Won" && d.stage != "Closed-Lost").count()
    }

    fn closed_won_count(&self) -> usize {
        self.deals.iter().filter(|d| d.stage == "Closed-Won").count()
    }

    fn conversion_rate(&self) -> f64 {
        if self.total_prospects() == 0 { return 0.0; }
        (self.closed_won_count() as f64 / self.total_prospects() as f64) * 100.0
    }

    fn value_by_stage(&self, stage: &str) -> f64 {
        self.deals
            .iter()
            .filter(|d| d.stage == stage)
            .map(|d| d.amount * (d.probability as f64 / 100.0))
            .sum()
    }

    fn open_value(&self) -> f64 {
        self.deals
            .iter()
            .filter(|d| d.stage != "Closed-Won" && d.stage != "Closed-Lost")
            .map(|d| d.amount * (d.probability as f64 / 100.0))
            .sum()
    }

    fn expected_close_30d(&self) -> f64 {
        self.deals
            .iter()
            .filter(|d| (d.stage == "Proposal" || d.stage == "Negotiating") && d.probability > 60)
            .map(|d| d.amount * (d.probability as f64 / 100.0))
            .sum()
    }

    fn value_by_stream(&self, stream: &str) -> f64 {
        self.deals
            .iter()
            .filter(|d| d.stream == stream && d.stage != "Closed-Lost")
            .map(|d| d.amount * (d.probability as f64 / 100.0))
            .sum()
    }

    fn count_by_stream(&self, stream: &str) -> usize {
        self.deals.iter().filter(|d| d.stream == stream && d.stage != "Closed-Lost").count()
    }

    fn top_deals(&self, count: usize) -> Vec<Deal> {
        let mut sorted = self.deals.clone();
        sorted.sort_by(|a, b| b.amount.partial_cmp(&a.amount).unwrap());
        sorted.into_iter().take(count).collect()
    }

    fn at_risk(&self) -> Vec<Deal> {
        self.deals
            .iter()
            .filter(|d| d.days_inactive > 7 && d.stage != "Closed-Won" && d.stage != "Closed-Lost")
            .cloned()
            .collect()
    }
}
