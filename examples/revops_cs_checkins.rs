// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! RevOps Customer Success Check-in System
//!
//! Automated customer check-in system with metrics and alerts
//! Build: cargo build --example revops_cs_checkins --features examples
//! Run: ./target/debug/examples/revops_cs_checkins

fn main() {
    let cs = CSCheckInSystem::new();

    println!("╔════════════════════════════════════════════════════════╗");
    println!("║      CUSTOMER SUCCESS CHECK-IN SYSTEM                  ║");
    println!("╚════════════════════════════════════════════════════════╝\n");

    // Customer Examples
    let customers = vec![
        ("Acme Corp", "support", 15, 100, 8),
        ("TechStart", "consulting", 42, 95, 2),
        ("StartupXYZ", "training", 8, 75, 5),
        ("BigTech", "ggen", 30, 120, 1),
        ("MidCorp", "support", 25, 50, 9),
    ];

    println!("📋 CUSTOMER STATUS DASHBOARD\n");
    println!(
        "{:<20} {:<15} {:<8} {:<8} {:<8} {:<12}",
        "Company", "Stream", "Days In", "Usage %", "Days Idle", "Status"
    );
    println!("{}", "─".repeat(75));

    for (company, stream, days_in, usage, days_idle) in &customers {
        let status = if *days_idle > 7 {
            "⚠️  AT RISK"
        } else if *usage < 50 {
            "🟡 LOW"
        } else {
            "🟢 HEALTHY"
        };

        println!(
            "{:<20} {:<15} {:<8} {:<8}% {:<8} {:<12}",
            company, stream, days_in, usage, days_idle, status
        );
    }

    // Day 7 Check-ins
    println!("\n\n📧 DAY 7 CHECK-IN EMAILS\n");
    for (company, stream, _, _, _) in customers.iter().take(2) {
        println!("═══ {} ({}) ═══", company, stream);
        let email = cs.day_7_checkin(company, stream);
        println!("{}\n", email);
    }

    // 30-Day Review
    println!("\n\n✅ 30-DAY SUCCESS REVIEW\n");
    let review = cs.thirty_day_review("Acme Corp", "support");
    for line in review {
        println!("{}", line);
    }

    // Monthly CS Metrics
    println!("\n\n📊 MONTHLY CS METRICS\n");
    println!("Total Active Customers:            52");
    println!("New Customers (this month):        8");
    println!("Churned Customers:                 1");
    println!("Net New:                           +7");
    println!("Net Revenue Retention:             85%");
    println!("\nActivation Rate (D7):              80% (target: >75%)");
    println!("Onboarding Success (D30):          75% (target: >70%)");
    println!("Retention Rate (D90):              95% (target: >90%)");
    println!("Expansion Rate:                    20% (target: >15%)");
    println!("\nHealth Metrics:");
    println!("├─ At Risk (idle 7+ days):         3 customers");
    println!("├─ Low Usage (50-70%):             5 customers");
    println!("├─ Expansion Ready (90%+ usage):   8 customers");
    println!("└─ Churn Prevention Alerts:        2 active");

    // CS Actions
    println!("\n\n🎯 CS ACTIONS THIS WEEK\n");
    let actions = cs.weekly_actions();
    for (day, action) in actions {
        println!("{:<12} {}", day, action);
    }

    // Churn Prevention
    println!("\n\n⚠️  CHURN PREVENTION SYSTEM\n");
    let churn_triggers = cs.churn_triggers();
    for (trigger, action, priority) in churn_triggers {
        println!("{:<35} → {:<25} [{}]", trigger, action, priority);
    }

    // Upsell Opportunities
    println!("\n\n🚀 EXPANSION OPPORTUNITIES\n");
    let upsells = cs.upsell_opportunities();
    for (customer, current, upsell, potential_arr) in upsells {
        println!(
            "{:<20} {} → {}  (${:.0} ARR potential)",
            customer, current, upsell, potential_arr
        );
    }

    // Success Plan Template
    println!("\n\n📋 SUCCESS PLAN TEMPLATE\n");
    let template = cs.success_plan_template();
    for line in template {
        println!("{}", line);
    }
}

struct CSCheckInSystem;

impl CSCheckInSystem {
    fn new() -> Self {
        CSCheckInSystem
    }

    fn day_7_checkin(&self, company: &str, stream: &str) -> String {
        format!(
            r#"Subject: Let's define success for {} 🎯

Hi [Contact],

It's been one week! Quick check-in to make sure everything's working well.

QUESTIONS:
1. Were you able to access everything? (Yes / No)
2. Any confusion or blockers? (Optional)
3. What does success look like for you?
   → For {}: [Examples based on stream]

NEXT STEPS:
Once I hear from you, I'll create a success plan so we're aligned on goals
and how to measure results.

Questions? Reply here or schedule a call:
[Calendar Link]

Looking forward to your feedback!

[Your Name]"#,
            company, stream
        )
    }

    fn thirty_day_review(&self, company: &str, stream: &str) -> Vec<String> {
        vec![
            format!("MONTH 1 REVIEW: {} - {}", company, stream),
            "═".repeat(50).to_string(),
            "".to_string(),
            "USAGE METRICS:".to_string(),
            "├─ Platform logins:     15 (target: 20, on track)".to_string(),
            "├─ Support questions:   8 (avg 2/week, healthy)".to_string(),
            "├─ Feature adoption:    75% (strong engagement)".to_string(),
            "└─ Time saved:          ~40 hours (basis for ROI)".to_string(),
            "".to_string(),
            "SATISFACTION:".to_string(),
            "├─ Overall sentiment:   Positive ✓".to_string(),
            "├─ NPS score:           +25 (good)".to_string(),
            "├─ Issues encountered:  1 (resolved same day)".to_string(),
            "└─ Support satisfaction: Excellent".to_string(),
            "".to_string(),
            "NEXT MONTH FOCUS:".to_string(),
            "├─ Goal 1: Hit 100% adoption of key feature".to_string(),
            "├─ Goal 2: Implement feedback from month 1".to_string(),
            "├─ Goal 3: Discuss expansion to [Stream 2]".to_string(),
            "└─ Timeline: Scheduled check-in for Feb 15".to_string(),
        ]
    }

    fn weekly_actions(&self) -> Vec<(String, String)> {
        vec![
            ("Monday 9 AM".to_string(), "Review pipeline: identify at-risk customers".to_string()),
            ("Monday 2 PM".to_string(), "Send day-7 check-ins to new customers".to_string()),
            ("Tuesday 10 AM".to_string(), "CS calls: 3 customers (30 min each)".to_string()),
            ("Wednesday 2 PM".to_string(), "Usage review: identify low-usage patterns".to_string()),
            ("Thursday 10 AM".to_string(), "Send 30-day value confirmations".to_string()),
            ("Thursday 2 PM".to_string(), "Churn prevention: outreach to at-risk".to_string()),
            ("Friday 9 AM".to_string(), "Weekly CS metrics review & planning".to_string()),
        ]
    }

    fn churn_triggers(&self) -> Vec<(String, String, String)> {
        vec![
            (
                "No activity for 7 days".to_string(),
                "Day 7: Send 'we miss you' email".to_string(),
                "HIGH".to_string(),
            ),
            (
                "Support complaint received".to_string(),
                "Within 30 min: Call customer, own problem".to_string(),
                "URGENT".to_string(),
            ),
            (
                "Usage drop 30%+ for 14 days".to_string(),
                "Personal check-in call scheduled".to_string(),
                "HIGH".to_string(),
            ),
            (
                "Didn't complete onboarding (D7)".to_string(),
                "Call to remove blockers".to_string(),
                "HIGH".to_string(),
            ),
            (
                "Budget concerns mentioned".to_string(),
                "Offer downgrade or payment plan".to_string(),
                "MEDIUM".to_string(),
            ),
            (
                "Missed 2 check-in calls".to_string(),
                "Reach out: everything OK?".to_string(),
                "MEDIUM".to_string(),
            ),
        ]
    }

    fn upsell_opportunities(&self) -> Vec<(String, String, String, f64)> {
        vec![
            (
                "Acme Corp".to_string(),
                "Support Startup".to_string(),
                "Support Team (3-4x value)".to_string(),
                150000.0,
            ),
            (
                "TechStart".to_string(),
                "Consulting Project".to_string(),
                "Training + Frontier Pro".to_string(),
                80000.0,
            ),
            (
                "StartupXYZ".to_string(),
                "Training".to_string(),
                "Team Cert Program (5 seats)".to_string(),
                50000.0,
            ),
            (
                "BigTech".to_string(),
                "ggen Free".to_string(),
                "ggen Pro + Frontier Premium".to_string(),
                200000.0,
            ),
        ]
    }

    fn success_plan_template(&self) -> Vec<String> {
        vec![
            "SUCCESS PLAN TEMPLATE".to_string(),
            "═".repeat(50),
            "".to_string(),
            "CUSTOMER: [Name] | STREAM: [Support/Training/Consulting/ggen]".to_string(),
            "PERIOD: [90 days]".to_string(),
            "".to_string(),
            "GOALS (What success looks like):".to_string(),
            "1. [Quantifiable goal, e.g., 'Reduce CLI bugs by 40%']".to_string(),
            "2. [Quantifiable goal, e.g., 'Deploy on new tech stack']".to_string(),
            "".to_string(),
            "KEY METRICS:".to_string(),
            "├─ Metric 1: [Current: X, Target: Y, Timeline: when]".to_string(),
            "├─ Metric 2: [Current: X, Target: Y, Timeline: when]".to_string(),
            "└─ Metric 3: [Current: X, Target: Y, Timeline: when]".to_string(),
            "".to_string(),
            "PROGRESS CHECKPOINTS:".to_string(),
            "├─ Day 30: [Milestone or review]".to_string(),
            "├─ Day 60: [Milestone or review]".to_string(),
            "└─ Day 90: [Success evaluation + renewal]".to_string(),
            "".to_string(),
            "CUSTOMER COMMITMENT:".to_string(),
            "├─ [Action customer must take]".to_string(),
            "├─ [Frequency: daily/weekly/monthly]".to_string(),
            "└─ Time investment: [X hours/week]".to_string(),
            "".to_string(),
            "OUR COMMITMENT:".to_string(),
            "├─ [Deliverable or service]".to_string(),
            "├─ Response time: [SLA or commitment]".to_string(),
            "└─ Support: [How we'll help]".to_string(),
        ]
    }
}
