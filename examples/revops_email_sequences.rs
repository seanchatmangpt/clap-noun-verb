//! RevOps Email Sequence Generator
//!
//! Ready-to-send email templates for all revenue streams
//! Build: cargo build --example revops_email_sequences --features examples
//! Run: ./target/debug/examples/revops_email_sequences

fn main() {
    let email = EmailSequences::new();

    println!("╔════════════════════════════════════════════════════════╗");
    println!("║       EMAIL SEQUENCE TEMPLATES (Copy & Send)           ║");
    println!("╚════════════════════════════════════════════════════════╝\n");

    // Cold Email Examples
    println!("❄️  COLD EMAIL SEQUENCES\n");

    println!("═══ SUPPORT & SLA COLD SEQUENCE ═══");
    for (i, email) in email.support_cold_sequence().iter().enumerate() {
        println!("\n📧 EMAIL {} (Send {}):", i + 1, ["Immediately", "Day 3", "Day 5"][i]);
        println!("{}\n", email);
        println!("─────────────────────────────────────────────────────────");
    }

    println!("\n❄️  CONSULTING COLD SEQUENCE\n");
    println!("═══ CONSULTING COLD SEQUENCE ═══");
    for (i, email) in email.consulting_cold_sequence().iter().enumerate() {
        println!("\n📧 EMAIL {} (Send {}):", i + 1, ["Immediately", "Day 4", "Day 6"][i]);
        println!("{}\n", email);
        println!("─────────────────────────────────────────────────────────");
    }

    // Warm Email Examples
    println!("\n🔥 WARM EMAIL SEQUENCES\n");
    println!("═══ INTERESTED → PROPOSAL SEQUENCE ═══");
    for (i, email) in email.interested_to_proposal().iter().enumerate() {
        println!("\n📧 EMAIL {} (After discovery call):", i + 1);
        println!("{}\n", email);
        println!("─────────────────────────────────────────────────────────");
    }

    // Nurture Email Examples
    println!("\n💚 NURTURE SEQUENCES\n");
    println!("═══ WEEK 1 NURTURE SEQUENCE ═══");
    for (i, email) in email.nurture_week1().iter().enumerate() {
        println!("\n📧 NURTURE {} (Day {}):", i + 1, i * 2);
        println!("{}\n", email);
        println!("─────────────────────────────────────────────────────────");
    }

    // Statistics
    println!("\n📊 EMAIL STATISTICS");
    println!("─────────────────────────────────────────────────────────");
    println!("Expected open rate:       30% (developer audience)");
    println!("Expected click rate:      8% (high-intent audience)");
    println!("Expected response rate:   3-5% (cold email baseline)");
    println!("Expected conversion:      0.5-1% (to demo/call)");
    println!("\nWith 100 cold emails:");
    println!("  ├─ Opens:               30");
    println!("  ├─ Clicks:              8");
    println!("  ├─ Responses:           3-5");
    println!("  └─ Calls/Demos:         0-1");
}

struct EmailSequences;

impl EmailSequences {
    fn new() -> Self {
        EmailSequences
    }

    fn support_cold_sequence(&self) -> Vec<String> {
        vec![
            // Email 1
            r#"Subject: Quick question about [Company]'s CLI stack?

Hi [Name],

I noticed [Company] builds a lot of CLI tools. Quick question:

Are you still using clap for most of them, or have you moved to
something else?

Asking because I maintain clap and help teams like yours ship
production CLIs 2-3x faster.

If that interests you, I offer:
- SLA support (24h response time for Startup, 4h for Enterprise)
- Architecture consulting (free 30-min review)
- Team training (bring my expertise in-house)

No pressure either way. Just wanted to check in.

Best,
[Your Name]
Clap Maintainer & Consulting"#
                .to_string(),
            // Email 2
            r#"Subject: RE: Quick question about [Company]'s CLI stack?

Hey [Name],

I haven't heard back - no worries if I caught you at a busy time.

Just wanted to share one quick win: Most teams save about 10 hours/week
on CLI debugging and architecture decisions with our SLA support.

That's usually the difference between 4 sprints and 5 sprints on a project.

If you're interested in a quick chat to see if it applies to [Company],
I've got 30 min open this Thursday or Friday.

[Calendar Link]

No obligation - just want to help if I can.

Cheers,
[Your Name]"#
                .to_string(),
            // Email 3
            r#"Subject: RE: Quick question about [Company]'s CLI stack?

Hi [Name],

Last check-in - if now's not the right time, totally understand.

I'm launching SLA support this month and offering 50% off Year 1 for
early adopters (before Feb 1).

If that's interesting, let me know. Otherwise, no worries and good luck
with the CLI work!

[Calendar Link]

Best,
[Your Name]"#
                .to_string(),
        ]
    }

    fn consulting_cold_sequence(&self) -> Vec<String> {
        vec![
            // Email 1
            r#"Subject: Free architecture review for [Company]

Hi [Name],

I noticed [Company] is working on [specific CLI project].

I specialize in CLI architecture for large teams. Most of my consulting
is helping teams like yours scale from 100 engineers to 500+ without
their CLI tooling becoming a bottleneck.

I'd love to offer a free 30-minute architecture review with no strings attached.

I could give you quick wins that might save 10-20 engineering hours.

Interested?

[Calendar Link]

[Your Name]
Clap Creator & Principal Architect"#
                .to_string(),
            // Email 2
            r#"Subject: RE: Free architecture review for [Company]

Hi [Name],

No worries if last email got buried. I've got a couple of 30-min slots
this Thursday if you want to chat about CLI architecture.

[Calendar Link]

[Your Name]"#
                .to_string(),
            // Email 3
            r#"Subject: RE: Free architecture review for [Company]

Hi [Name],

Last offer: Free 30-min architecture review this month.

[Calendar Link]

If this isn't a priority right now, I get it. Feel free to reach out
if it becomes one.

[Your Name]"#
                .to_string(),
        ]
    }

    fn interested_to_proposal(&self) -> Vec<String> {
        vec![
            // After call, warm them up
            r#"Subject: Great chat today - proposal attached

Hi [Name],

Thanks for taking the time to chat about [topic].

Based on our conversation, I've put together a proposal that addresses:
- [Pain point 1 they mentioned]
- [Pain point 2 they mentioned]
- [Pain point 3 they mentioned]

See attached: [Proposal PDF]

Let me know if you have any questions or want to adjust anything.

Looking forward to working together!

[Your Name]"#
                .to_string(),
            // Follow-up if no response
            r#"Subject: RE: proposal for [Company] - quick question

Hi [Name],

Just checking in on the proposal I sent on [date].

Do you have any questions about:
- The scope?
- The timeline?
- The pricing?

Happy to adjust if needed.

[Your Name]"#
                .to_string(),
            // Closing/negotiation
            r#"Subject: RE: Let's make this happen

Hi [Name],

Great to hear you're interested! A few options:

OPTION 1: Accept as-is ($X amount, Y timeline)
OPTION 2: Negotiate timeline (let's discuss)
OPTION 3: Pilot project (start smaller, expand later)

What works best for you?

[Your Name]"#
                .to_string(),
        ]
    }

    fn nurture_week1(&self) -> Vec<String> {
        vec![
            // Day 0
            r#"Subject: Free preview: [Resource]

Hi [Name],

Following up on our conversation.

I promised you [resource] - here it is:
[Link to tutorial/guide/template]

This should help with [specific problem they mentioned].

Let me know if you try it!

[Your Name]"#
                .to_string(),
            // Day 2
            r#"Subject: Most teams see [benefit] with [solution]

Hi [Name],

Quick insight: Most teams like [Company] see about [10 hours/week saved]
by implementing [solution].

Here's how [Competitor/Similar company] did it:
[Case study link]

Similar to your situation?

[Your Name]"#
                .to_string(),
            // Day 4
            r#"Subject: [Social proof] - thought of you

Hi [Name],

Saw this on [Twitter/LinkedIn/HackerNews]:

[Quote or link]

Made me think of our conversation about [topic].

This might be relevant: [Resource link]

[Your Name]"#
                .to_string(),
            // Day 6
            r#"Subject: [Limited time] special for [Company]

Hi [Name],

Real quick: I'm offering [early adopters/limited beta/50% off] to
[X] companies before [date].

Since we've been talking, wanted to make sure you knew.

Interested? [Calendar link] or reply here.

[Your Name]"#
                .to_string(),
        ]
    }
}
