//! RevOps Revenue Dashboard
//!
//! Real-time revenue tracking and forecasting
//! Build: cargo build --example revops_revenue_dashboard --features examples
//! Run: ./target/debug/examples/revops_revenue_dashboard

fn main() {
    let revenue = RevenueTracker::sample_data();

    println!("╔════════════════════════════════════════════════════════╗");
    println!("║         REVENUE OPERATIONS DASHBOARD                   ║");
    println!("╚════════════════════════════════════════════════════════╝\n");

    // MRR Summary
    println!("💰 MONTHLY RECURRING REVENUE (MRR)");
    println!("─────────────────────────────────────────────────────────");
    println!("Current MRR:              ${:.0}", revenue.current_mrr());
    println!("New this month:           ${:.0}", revenue.new_mrr());
    println!("Expansion revenue:        ${:.0}", revenue.expansion_revenue());
    println!("Churn:                    -${:.0}", revenue.churn());
    println!("Growth:                   {:.1}% MoM\n", revenue.growth_rate());

    // Annual Projections
    println!("📈 ANNUAL PROJECTIONS");
    println!("─────────────────────────────────────────────────────────");
    println!("Current ARR:              ${:.0}", revenue.current_mrr() * 12.0);
    println!("Projected Year-End ARR:   ${:.0}", revenue.projected_year_end_arr());
    println!("Year 1 Total Revenue:     ${:.0}\n", revenue.projected_year1_total());

    // By Stream
    println!("🎯 REVENUE BY STREAM");
    println!("─────────────────────────────────────────────────────────");
    let streams = vec![
        ("Support & SLA", revenue.stream_mrr("support")),
        ("Training", revenue.stream_mrr("training")),
        ("Consulting", revenue.stream_mrr("consulting")),
        ("ggen SaaS", revenue.stream_mrr("ggen")),
        ("Frontier Pro", revenue.stream_mrr("frontier")),
        ("Marketplace", revenue.stream_mrr("marketplace")),
        ("Enterprise", revenue.stream_mrr("enterprise")),
    ];

    for (stream, mrr) in streams {
        let pct = (mrr / revenue.current_mrr()) * 100.0;
        println!("{:<18} ${:.0}  ({:>5.1}%)", stream, mrr, pct);
    }

    println!("\n👥 CUSTOMER METRICS");
    println!("─────────────────────────────────────────────────────────");
    println!("Total customers:          {}", revenue.total_customers());
    println!("New customers (month):    {}", revenue.new_customers);
    println!("Churn (month):            {}", revenue.churned_customers);
    println!("Net new:                  {}", revenue.new_customers - revenue.churned_customers);
    println!("Churn rate:               {:.1}%", revenue.churn_rate());

    println!("\n💹 UNIT ECONOMICS");
    println!("─────────────────────────────────────────────────────────");
    println!("CAC (acquisition cost):   ${:.0}", revenue.cac());
    println!("LTV (lifetime value):     ${:.0}", revenue.ltv());
    println!("Payback period:           {:.1} months", revenue.payback_period());
    println!(
        "LTV:CAC ratio:            {:.1}:1 {}",
        revenue.ltv_cac_ratio(),
        if revenue.ltv_cac_ratio() > 3.0 { "✓" } else { "⚠" }
    );

    println!("\n🎯 FORECAST (Next 6 Months)");
    println!("─────────────────────────────────────────────────────────");
    for month in 1..=6 {
        let projected = revenue.forecast_month(month);
        println!(
            "Month {}: ${:.0}  ({:>+6.1}% growth)",
            month,
            projected,
            revenue.forecast_growth(month)
        );
    }

    println!("\n🚀 KEY METRICS");
    println!("─────────────────────────────────────────────────────────");
    println!("NRR (Net Revenue Retention):  {:.0}%", revenue.nrr());
    println!("Expansion rate:                {:.1}%", revenue.expansion_rate());
    println!("Rule of 40 Score:              {:.0} (optimal: 40+)", revenue.rule_of_40());
    println!("Runway (months):               {}", revenue.runway_months());
}

struct RevenueTracker {
    support_mrr: f64,
    training_mrr: f64,
    consulting_mrr: f64,
    ggen_mrr: f64,
    frontier_mrr: f64,
    marketplace_mrr: f64,
    enterprise_mrr: f64,
    total_customers: u32,
    new_customers: u32,
    churned_customers: u32,
    monthly_costs: f64,
    months_operating: u32,
}

impl RevenueTracker {
    fn sample_data() -> Self {
        RevenueTracker {
            support_mrr: 3500.0,
            training_mrr: 1200.0,
            consulting_mrr: 5000.0,
            ggen_mrr: 2000.0,
            frontier_mrr: 800.0,
            marketplace_mrr: 400.0,
            enterprise_mrr: 550.0,
            total_customers: 52,
            new_customers: 8,
            churned_customers: 1,
            monthly_costs: 3000.0,
            months_operating: 3,
        }
    }

    fn current_mrr(&self) -> f64 {
        self.support_mrr
            + self.training_mrr
            + self.consulting_mrr
            + self.ggen_mrr
            + self.frontier_mrr
            + self.marketplace_mrr
            + self.enterprise_mrr
    }

    fn new_mrr(&self) -> f64 {
        (self.new_customers as f64) * 1000.0 // avg new customer value
    }

    fn expansion_revenue(&self) -> f64 {
        (self.total_customers as f64) * 50.0 // avg expansion per customer
    }

    fn churn(&self) -> f64 {
        (self.churned_customers as f64) * 500.0 // avg churned customer value
    }

    fn growth_rate(&self) -> f64 {
        ((self.new_mrr() - self.churn()) / self.current_mrr()) * 100.0
    }

    fn projected_year_end_arr(&self) -> f64 {
        let base = self.current_mrr() * 12.0;
        let growth = self.growth_rate() / 100.0;
        base * (1.0 + (growth * 9.0)) // 9 months remaining growth
    }

    fn projected_year1_total(&self) -> f64 {
        self.current_mrr() * 12.0 * 1.2 // Conservative 20% avg growth
    }

    fn stream_mrr(&self, stream: &str) -> f64 {
        match stream {
            "support" => self.support_mrr,
            "training" => self.training_mrr,
            "consulting" => self.consulting_mrr,
            "ggen" => self.ggen_mrr,
            "frontier" => self.frontier_mrr,
            "marketplace" => self.marketplace_mrr,
            "enterprise" => self.enterprise_mrr,
            _ => 0.0,
        }
    }

    fn total_customers(&self) -> u32 {
        self.total_customers
    }

    fn churn_rate(&self) -> f64 {
        (self.churned_customers as f64 / self.total_customers as f64) * 100.0
    }

    fn cac(&self) -> f64 {
        5000.0 / (self.new_customers as f64).max(1.0) // assume $5K marketing spend
    }

    fn ltv(&self) -> f64 {
        (self.current_mrr() / self.total_customers as f64) * 24.0 // 24-month avg lifetime
    }

    fn payback_period(&self) -> f64 {
        let avg_customer_value = self.current_mrr() / self.total_customers as f64;
        self.cac() / avg_customer_value
    }

    fn ltv_cac_ratio(&self) -> f64 {
        self.ltv() / self.cac()
    }

    fn nrr(&self) -> f64 {
        ((self.expansion_revenue() - self.churn()) / self.current_mrr()) * 100.0 + 100.0
    }

    fn expansion_rate(&self) -> f64 {
        (self.expansion_revenue() / self.current_mrr()) * 100.0
    }

    fn rule_of_40(&self) -> f64 {
        let growth = self.growth_rate();
        let profit_margin =
            ((self.current_mrr() - self.monthly_costs) / self.current_mrr()) * 100.0;
        growth + profit_margin.max(0.0)
    }

    fn runway_months(&self) -> u32 {
        let monthly_profit = (self.current_mrr() - self.monthly_costs).max(0.0);
        if monthly_profit > 0.0 {
            999 // Positive cash flow = infinite runway
        } else {
            30 // Default runway estimate
        }
    }

    fn forecast_month(&self, month: u32) -> f64 {
        let growth = self.growth_rate() / 100.0;
        self.current_mrr() * (1.0 + growth * month as f64).min(2.0)
    }

    fn forecast_growth(&self, month: u32) -> f64 {
        ((self.forecast_month(month) - self.current_mrr()) / self.current_mrr()) * 100.0
    }
}
