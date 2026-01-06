//! RevOps Customer Onboarding Automation
//!
//! Automated onboarding sequences and tracking
//! Build: cargo build --example revops_customer_onboarding --features examples
//! Run: ./target/debug/examples/revops_customer_onboarding

use std::collections::HashMap;

fn main() {
    let onboarding = CustomerOnboarding::new();

    println!("╔════════════════════════════════════════════════════════╗");
    println!("║      CUSTOMER ONBOARDING AUTOMATION                    ║");
    println!("╚════════════════════════════════════════════════════════╝\n");

    // Customer Examples
    let customers = vec![
        ("Acme Corp", "support", 0),
        ("TechStart", "consulting", 1),
        ("StartupXYZ", "training", 2),
        ("BigTech", "ggen", 3),
    ];

    for (company, stream, day) in customers {
        println!("📧 {} - {} (Day {})", company, stream, day);
        let emails = onboarding.get_day_emails(stream, day);
        for email in emails {
            println!("   → {}", email);
        }
        println!();
    }

    // Success Plan Template
    println!("📋 SUCCESS PLAN TEMPLATE");
    println!("─────────────────────────────────────────────────────────");
    let plan = onboarding.generate_success_plan("Acme Corp", "support");
    for line in plan {
        println!("{}", line);
    }

    println!("\n✅ ONBOARDING CHECKLIST");
    println!("─────────────────────────────────────────────────────────");
    let checklist = onboarding.checklist("support");
    for item in checklist {
        println!("☐ {}", item);
    }

    // Metrics
    println!("\n📊 ONBOARDING METRICS");
    println!("─────────────────────────────────────────────────────────");
    println!("Expected activation rate:  80% (day 3 first action)");
    println!("Expected success rate:     75% (day 30 achieving goals)");
    println!("Expected retention:        95% (after 90 days)");
    println!("Expected expansion:        20% (upsell opportunity)");

    // Churn Prevention
    println!("\n⚠️  CHURN PREVENTION TRIGGERS");
    println!("─────────────────────────────────────────────────────────");
    let triggers = onboarding.churn_prevention();
    for (trigger, action) in triggers {
        println!("{:<30} → {}", trigger, action);
    }
}

struct CustomerOnboarding;

impl CustomerOnboarding {
    fn new() -> Self {
        CustomerOnboarding
    }

    fn get_day_emails(&self, stream: &str, day: u32) -> Vec<String> {
        match (stream, day) {
            ("support", 0) => vec![
                "Welcome to [Company] Support! Your SLA: 24h response time".to_string(),
                "Submit your first question: [link]".to_string(),
                "Support hours: Mon-Fri 9 AM - 5 PM EST".to_string(),
            ],
            ("support", 1) => vec![
                "How's your first 24 hours going?".to_string(),
                "Quick setup guide: [link]".to_string(),
            ],
            ("support", 3) => vec![
                "30 days in: Are we delivering value?".to_string(),
                "Let's define success: What does winning look like?".to_string(),
            ],
            ("support", 7) => vec![
                "Success plan created - tracking your goals".to_string(),
            ],
            ("training", 0) => vec![
                "Welcome to Clap Certification! 🎓".to_string(),
                "Start Lesson 1: Getting Started".to_string(),
                "Expected time: 2-3 hours".to_string(),
            ],
            ("training", 1) => vec![
                "How's Lesson 1 going?".to_string(),
                "Remember: 1 lesson per week = certified in 4 weeks".to_string(),
            ],
            ("training", 7) => vec![
                "Completing Lesson 2?".to_string(),
                "You're on track for certification!".to_string(),
            ],
            ("consulting", 0) => vec![
                "Your consulting engagement starts Monday!".to_string(),
                "Pre-engagement checklist: [link]".to_string(),
                "Send us your biggest challenge".to_string(),
            ],
            ("consulting", 1) => vec![
                "Kick-off meeting scheduled for Wed 2 PM".to_string(),
                "Prep agenda: [link]".to_string(),
            ],
            ("consulting", 3) => vec![
                "Month 1 check-in: How's the project going?".to_string(),
                "Tracking progress toward milestones".to_string(),
            ],
            ("ggen", 0) => vec![
                "Welcome to ggen! Start generating CLIs".to_string(),
                "First 10 generations free".to_string(),
                "Tutorial: [link]".to_string(),
            ],
            ("ggen", 1) => vec![
                "Generated any CLIs yet?".to_string(),
                "Try this template: [link]".to_string(),
            ],
            ("ggen", 7) => vec![
                "Week 1 summary: X CLIs generated, X hours saved".to_string(),
                "Ready to upgrade to pro plan?".to_string(),
            ],
            _ => vec!["Generic welcome email".to_string()],
        }
    }

    fn generate_success_plan(&self, company: &str, stream: &str) -> Vec<String> {
        let mut plan = vec![
            format!("SUCCESS PLAN: {} - {}", company, stream),
            "═".repeat(50).to_string(),
            "".to_string(),
        ];

        match stream {
            "support" => {
                plan.extend(vec![
                    "GOAL 1: Reduce support tickets by 30%".to_string(),
                    "GOAL 2: Improve CLI stability".to_string(),
                    "KEY METRIC: Response time < 24h (SLA: 100%)".to_string(),
                    "TIMELINE: 90 days".to_string(),
                    "".to_string(),
                    "DAY 30 CHECK: Usage trends + customer feedback".to_string(),
                    "DAY 60 CHECK: Baseline metrics + early wins".to_string(),
                    "DAY 90 CHECK: Full assessment + renewal decision".to_string(),
                ]);
            },
            "consulting" => {
                plan.extend(vec![
                    "PROJECT: CLI Architecture Review & Redesign".to_string(),
                    "GOAL: Scale from 100 to 500 engineers".to_string(),
                    "DELIVERABLE: Architecture doc + training".to_string(),
                    "TIMELINE: 8 weeks".to_string(),
                    "".to_string(),
                    "MILESTONE 1 (Week 2): Discovery complete".to_string(),
                    "MILESTONE 2 (Week 4): Draft architecture".to_string(),
                    "MILESTONE 3 (Week 8): Final delivery + training".to_string(),
                ]);
            },
            "training" => {
                plan.extend(vec![
                    "GOAL 1: Get team certified".to_string(),
                    "GOAL 2: Deploy production CLI using patterns".to_string(),
                    "COMPLETION RATE TARGET: 80% of team".to_string(),
                    "TIMELINE: 4 weeks".to_string(),
                    "".to_string(),
                    "WEEK 1: Lessons 1-2 (basics)".to_string(),
                    "WEEK 2: Lessons 3-4 (advanced)".to_string(),
                    "WEEK 3: Project work".to_string(),
                    "WEEK 4: Exam + certification".to_string(),
                ]);
            },
            _ => {
                plan.push(format!("Standard success plan for {}", stream));
            }
        }

        plan
    }

    fn checklist(&self, stream: &str) -> Vec<String> {
        match stream {
            "support" => vec![
                "Send welcome email with SLA details".to_string(),
                "Add customer to support Slack channel".to_string(),
                "Schedule optional kick-off call".to_string(),
                "Day 3 check-in: Everything working?".to_string(),
                "Day 7 success plan definition".to_string(),
                "Day 30 value confirmation".to_string(),
                "Day 60 expansion discussion".to_string(),
                "Day 90 renewal review".to_string(),
            ],
            "consulting" => vec![
                "Send project kick-off email".to_string(),
                "Schedule discovery call (within 3 days)".to_string(),
                "Send pre-call discovery questionnaire".to_string(),
                "Create project timeline + milestones".to_string(),
                "Kick-off call: Align on scope".to_string(),
                "Week 2: Initial findings".to_string(),
                "Week 4: Draft presentation".to_string(),
                "Week 8: Final delivery + training".to_string(),
            ],
            "training" => vec![
                "Send course welcome + first lesson".to_string(),
                "Add to student cohort (optional group)".to_string(),
                "Day 3: Usage check-in".to_string(),
                "Day 7: Lesson 1 completion check".to_string(),
                "Week 2: Lesson 2 encouragement".to_string(),
                "Week 3: Mid-course support".to_string(),
                "Week 4: Exam scheduling".to_string(),
                "Certification issuance upon completion".to_string(),
            ],
            _ => vec!["Generic checklist".to_string()],
        }
    }

    fn churn_prevention(&self) -> Vec<(String, String)> {
        vec![
            ("No usage for 7 days".to_string(), "Day 7: 'We miss you' email".to_string()),
            ("Feature misunderstanding".to_string(), "Immediate: Send video tutorial".to_string()),
            ("Expectation mismatch".to_string(), "Day 1: Schedule discovery call".to_string()),
            ("Usage drop 30%+ for 2 weeks".to_string(), "Personal check-in call".to_string()),
            ("Support complaint".to_string(), "Within 30 min: Respond + own problem".to_string()),
            ("Budget constraints mentioned".to_string(), "Offer downgrade or discount".to_string()),
        ]
    }
}
