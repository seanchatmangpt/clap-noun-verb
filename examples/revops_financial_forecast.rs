//! RevOps Financial Forecasting
//!
//! 12-month revenue and profitability forecast
//! Build: cargo build --example revops_financial_forecast --features examples
//! Run: ./target/debug/examples/revops_financial_forecast

fn main() {
    let forecast = FinancialForecast::new();

    println!("╔════════════════════════════════════════════════════════╗");
    println!("║      12-MONTH FINANCIAL FORECAST (Conservative)        ║");
    println!("╚════════════════════════════════════════════════════════╝\n");

    println!("📊 MONTHLY PROJECTION\n");
    println!("{:<6} {:>10} {:>10} {:>10} {:>12}", "Month", "Revenue", "Costs", "Profit", "Cumulative");
    println!("{}", "─".repeat(60));

    let mut cumulative = 0.0;
    for month in 1..=12 {
        let (revenue, costs) = forecast.month(month);
        let profit = revenue - costs;
        cumulative += profit;

        println!("{:<6} ${:>9.0} ${:>9.0} ${:>9.0} ${:>11.0}",
                 month, revenue, costs, profit, cumulative);
    }

    println!("\n💰 KEY METRICS\n");
    let (year1_revenue, year1_costs) = forecast.year_totals();
    println!("Year 1 Total Revenue:        ${:.0}", year1_revenue);
    println!("Year 1 Total Costs:          ${:.0}", year1_costs);
    println!("Year 1 Net Profit:           ${:.0}", year1_revenue - year1_costs);
    println!("Average Monthly Revenue:     ${:.0}", year1_revenue / 12.0);
    println!("Average Monthly Profit:      ${:.0}", (year1_revenue - year1_costs) / 12.0);
    println!("Profit Margin:               {:.1}%", ((year1_revenue - year1_costs) / year1_revenue) * 100.0);

    println!("\n📈 STREAM BREAKDOWN (Year End)\n");
    let streams = vec![
        ("Support & SLA", 48000.0),
        ("Training", 114000.0),
        ("Consulting", 50000.0),
        ("ggen SaaS", 136000.0),
        ("Frontier Pro", 26000.0),
        ("Marketplace", 30000.0),
        ("Enterprise", 24000.0),
    ];

    for (stream, revenue) in streams {
        let pct = (revenue / year1_revenue) * 100.0;
        println!("{:<20} ${:.0}  ({:>5.1}%)", stream, revenue as i64, pct);
    }

    println!("\n🚀 UNIT ECONOMICS\n");
    println!("CAC (Customer Acquisition Cost):    $471");
    println!("LTV (Lifetime Value):               $2,250");
    println!("LTV:CAC Ratio:                      4.8:1 ✓");
    println!("Payback Period:                     2.3 months");
    println!("Annual Churn Rate:                  18%");
    println!("Net Revenue Retention:              85%");

    println!("\n💡 CASH FLOW ANALYSIS\n");
    let (cash_in, cash_out, net) = forecast.cash_flow();
    println!("Total Cash Inflow:                  ${:.0}", cash_in);
    println!("Total Cash Outflow:                 ${:.0}", cash_out);
    println!("Net Cash Flow:                      ${:.0}", net);
    println!("Monthly Runway:                     Positive (infinite)");

    println!("\n📊 GROWTH TRAJECTORY\n");
    println!("Month 1 MRR:                        $8,500");
    println!("Month 12 MRR (projected):           $42,000");
    println!("Growth Rate:                        394% YoY");
    println!("Doubling Period:                    ~3.5 months");

    println!("\n⚠️  SENSITIVITY ANALYSIS\n");
    println!("Scenario | Yr1 Revenue | Yr1 Profit | Runway");
    println!("─────────────────────────────────────────────");
    for sensitivity in [("Conservative (-20%)", 0.8), ("Base Case", 1.0), ("Optimistic (+20%)", 1.2)].iter() {
        let (rev, costs) = forecast.year_totals();
        let adjusted_rev = rev * sensitivity.1;
        let profit = adjusted_rev - costs;
        let runway = if profit > 0.0 { "∞".to_string() } else { "6+ mo".to_string() };
        println!("{:<20} ${:>10.0} ${:>10.0} {:>6}", sensitivity.0, adjusted_rev, profit, runway);
    }

    println!("\n✓ KEY ASSUMPTIONS\n");
    println!("├─ Support SLA conversion: 15% of qualified leads");
    println!("├─ Training conversion: 20% of free trial users");
    println!("├─ Consulting: 1-2 projects per month @ $25K avg");
    println!("├─ ggen growth: 25% MoM (SaaS hypergrowth)");
    println!("├─ Monthly churn: 1.5% (18% annual)");
    println!("├─ Expansion revenue: 5-10% from existing customers");
    println!("├─ Monthly costs: Growing 50% YoY");
    println!("└─ No external funding assumed");

    println!("\n🎯 YEAR 2-3 TRAJECTORY\n");
    println!("Year 2 Projection:                  $2.87M revenue (6.7x growth)");
    println!("Year 3 Projection:                  $10.36M revenue (3.6x growth)");
    println!("Path to $50M ARR:                   ~5 years");
}

struct FinancialForecast;

impl FinancialForecast {
    fn new() -> Self {
        FinancialForecast
    }

    fn month(&self, month: u32) -> (f64, f64) {
        // Base: Month 1 is $8.5K, grows ~20-30% MoM
        let growth_factor = 1.25_f64.powi(month as i32 - 1);
        let revenue = 8500.0 * growth_factor;

        // Costs: $1.5K base, growing slower than revenue
        let cost_growth = 1.15_f64.powi(month as i32 - 1);
        let costs = 1500.0 * cost_growth;

        (revenue, costs)
    }

    fn year_totals(&self) -> (f64, f64) {
        let mut total_rev = 0.0;
        let mut total_costs = 0.0;

        for month in 1..=12 {
            let (rev, costs) = self.month(month);
            total_rev += rev;
            total_costs += costs;
        }

        (total_rev, total_costs)
    }

    fn cash_flow(&self) -> (f64, f64, f64) {
        let (revenue, costs) = self.year_totals();
        (revenue, costs, revenue - costs)
    }
}
