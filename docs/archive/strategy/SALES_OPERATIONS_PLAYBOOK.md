# Sales Operations Playbook

**Daily, weekly, and monthly routines to execute sales strategy across 7 revenue streams**

---

## Table of Contents

1. **Sales Pipeline System Setup**
2. **Lead Scoring Framework**
3. **Daily Sales Ritual**
4. **Outreach Templates (By Stream)**
5. **Deal Progression Playbook**
6. **Weekly Sales Review**
7. **Monthly Revenue Forecast**
8. **Pipeline Dashboard**

---

## Part 1: Sales Pipeline System Setup

### Option A: Google Sheets (Free, 30 minutes)

```
Create spreadsheet with columns:

PROSPECT INFO:
├─ Lead Name (Text)
├─ Company Name (Text)
├─ Email (Text)
├─ Phone (Text)
├─ LinkedIn URL (URL)
├─ Source (Dropdown: Cold Email, LinkedIn, Referral, Community, Conference)
└─ Industry (Dropdown: Dev Tools, SaaS, Enterprise, Startup, Freelancer)

ENGAGEMENT:
├─ Date Added (Date)
├─ First Contact Date (Date)
├─ Last Contact Date (Date)
├─ Contact Method (Dropdown: Email, LinkedIn, Twitter, Phone, Meeting)
└─ Response Received? (Checkbox)

OPPORTUNITY:
├─ Revenue Stream (Dropdown: Support, Training, Consulting, ggen, Frontier Pro, Marketplace, Enterprise)
├─ Estimated Deal Size (Currency)
├─ Pain Point Identified (Text)
├─ Product Interest (Dropdown: High, Medium, Low, None)
└─ Buying Timeline (Dropdown: Immediate, 1-3 mo, 3-6 mo, 6-12 mo, Unaware)

PIPELINE:
├─ Stage (Dropdown: Prospect → Contacted → Interested → Proposal → Negotiating → Closed-Won → Closed-Lost)
├─ Stage Entry Date (Date)
├─ Probability % (Number: 0-100)
├─ Notes (Text)
└─ Next Action (Text)

CLOSED:
├─ Deal Closed Date (Date)
├─ Final Amount (Currency)
├─ Contract Length (Dropdown: 1 mo, 3 mo, 6 mo, 12 mo, Annual, One-time)
└─ Customer Success Owner (Dropdown: You, Team Member 1, Team Member 2)
```

**Formulas to Add**:
```
PIPELINE VALUE:
=SUMIF(Stage, "Proposal", Estimated Deal Size) +
 SUMIF(Stage, "Negotiating", Estimated Deal Size)
→ Shows what's likely to close this month

CLOSED REVENUE:
=SUMIF(Stage, "Closed-Won", Final Amount)
→ Total revenue from closed deals

CONVERSION RATE:
=COUNTIF(Stage, "Closed-Won") / COUNTIF(Stage, "Prospect")
→ What % of prospects become customers

PIPELINE BY STREAM:
=SUMIF(Revenue Stream, "Support", Estimated Deal Size)
→ Repeat for each stream
```

**Share Access**: Keep private (or share with co-founder/VP sales)

---

### Option B: Pipedrive CRM ($15/month, 2 hours)

**Setup Steps**:

1. Go to pipedrive.com, create account
2. Create deal stages:
   ```
   Prospect → Contacted → Interested → Proposal →
   Negotiating → Closed-Won / Closed-Lost
   ```

3. Create custom fields:
   - Revenue Stream (dropdown)
   - Pain Point (text)
   - First Contact Method (dropdown)
   - Source (dropdown)

4. Create people fields:
   - Company Name
   - Industry
   - LinkedIn URL

5. Set up automations:
   - New deal created → Send reminder to reach out within 24h
   - Deal moved to "Proposal" → Calendar alert
   - Deal moved to "Closed-Won" → Create follow-up task in 30 days

6. Create views:
   - "This Week's Calls" (deals that need action this week)
   - "Pipeline Value" (all open deals sorted by probability)
   - "Revenue by Stream" (deals grouped by stream)

**Benefit over Sheets**: Automation, mobile app, built-in analytics, email integration

---

## Part 2: Lead Scoring Framework

**Why**: Not all leads are equal. Score them to focus effort on likely buyers.

### Scoring System (0-100)

```
PROFILE FIT (0-40 points):
├─ 10 pts: Company size 5-500 employees (sweet spot for SaaS)
├─ 10 pts: Industry = tech/developer tools
├─ 10 pts: Located in US/EU (easier to close)
└─ 10 pts: Budget authority (decision maker, not gatekeeper)

ENGAGEMENT (0-30 points):
├─ 5 pts: Responded to first outreach
├─ 10 pts: Opened email + clicked link
├─ 10 pts: Attended demo or meeting
└─ 5 pts: Asked detailed questions

BUYING SIGNALS (0-30 points):
├─ 10 pts: Current customer of competitor
├─ 10 pts: Posted about pain point on Twitter/Reddit/HN
├─ 10 pts: Mentioned budget/timeline in conversation
└─ 0 pts: No buying signals (ice cold)
```

**Scoring Rules**:
- **80-100**: READY TO CLOSE - Spend 80% effort here
- **60-79**: WARM - Nurture with emails, schedule calls
- **40-59**: COLD - Mass sequences, low effort
- **0-39**: NOT READY - Add to mailing list, revisit in 6 months

**Action**:
- Score new prospects weekly
- Move high-scorers to "Interested" stage
- Create separate nurture flow for 40-59 range

---

## Part 3: Daily Sales Ritual (15-30 minutes)

### Morning Ritual (8 AM, 10 minutes)

```
1. REVIEW PIPELINE (3 min)
   - Open sales pipeline dashboard
   - Check "This Week's Action Items"
   - Identify deals at risk (no contact in 7+ days)

2. QUICK WINS (4 min)
   - Find 1-2 deals ready for next action
   - Make decision: Email? Call? Proposal?
   - Add to today's "To-Do"

3. ENERGY CHECK (3 min)
   - Pick the time slot when you'll do sales work
   - Morning 8-10 AM? Afternoon 2-4 PM?
   - Protect this time (no meetings)
```

### Active Selling Block (90 minutes, Daily or 4x/week)

```
HOUR 1: OUTREACH (45 minutes)
├─ 0-5 min: Open prospect list
├─ 5-45 min: Send 5 cold emails OR call 3 prospects
│            (Or do 5 LinkedIn messages)
│            (Or do 1 strategy call)
└─ 45-50 min: Log all activity in pipeline

HOUR 2: FOLLOW-UP (45 minutes)
├─ 0-25 min: Review prospects who responded
├─ 25-45 min: Send follow-up emails to warm leads
└─ 45-50 min: Schedule next actions + update pipeline
```

### End of Day Ritual (5 PM, 5 minutes)

```
1. PIPELINE UPDATE
   - Update all today's contact outcomes
   - Move deals through pipeline if applicable
   - Log next action date for each lead

2. TOMORROW'S PREP
   - Which prospects should you prioritize tomorrow?
   - Who are you expecting a response from?
   - Any calls scheduled?

3. METRICS SNAPSHOT
   - Emails sent today: __
   - Responses received: __
   - New deals added: __
   - Deals moved to next stage: __
```

---

## Part 4: Outreach Templates (By Stream)

### Stream 1: Support & SLA Services

**Cold Email Template**:
```
Subject: Quick question about [Company]'s CLI stack?

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
Clap Maintainer & Consulting
```

**Follow-up Email 1** (3 days later):
```
Subject: RE: Quick question about [Company]'s CLI stack?

Hey [Name],

I haven't heard back - no worries if I caught you at a busy time.

Just wanted to share one quick win: Most teams save about 10 hours/week
on CLI debugging and architecture decisions with our SLA support.

That's usually the difference between 4 sprints and 5 sprints on a project.

If you're interested in a quick chat to see if it applies to [Company],
I've got 30 min open this Thursday or Friday.

No obligation - just want to help if I can.

[Calendar Link]

Cheers,
[Your Name]
```

**Follow-up Email 2** (5 days later):
```
Subject: RE: Quick question about [Company]'s CLI stack?

Hi [Name],

Last check-in - if now's not the right time, totally understand.

I'm launching SLA support this month and offering 50% off Year 1 for
early adopters (before Feb 1).

If that's interesting, let me know. Otherwise, no worries and good luck
with the CLI work!

[Calendar Link]

Best,
[Your Name]
```

**Response Rate Expected**: 2-5% (typical cold email)

---

### Stream 2: Training & Courses

**Cold Email Template**:
```
Subject: Free clap course preview for [Company]

Hi [Name],

I'm creating a certification program for clap, and I'm looking for
beta testers from companies like [Company].

Interested in a free course preview (usually $49)?

The course covers:
- Type-safe CLI design (2 hours)
- Advanced argument parsing (3 hours)
- Testing & deployment (2 hours)
- Certification exam (1 hour)

Most teams see 20-30% fewer CLI bugs after completing it.

Interested?

[Course Link]

[Your Name]
```

**Follow-up Email 1** (3 days):
```
Subject: RE: Free clap course preview for [Company]

Hi [Name],

Course starts next Monday. Last chance for free preview access.

Regular price: $49
Early bird (this week only): Free for beta testers

[Course Link]

[Your Name]
```

**Follow-up Email 2** (2 days):
```
Subject: RE: Free clap course preview for [Company]

Hi [Name],

Course is starting in 24 hours. Dropping this to $49 if you're interested.

[Course Link]

[Your Name]
```

**Conversion Expected**: 5-10% (educational product, high value)

---

### Stream 3: Consulting Services

**Cold Email Template**:
```
Subject: Free architecture review for [Company]

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
Clap Creator & Principal Architect
```

**Follow-up Email 1** (4 days):
```
Subject: RE: Free architecture review for [Company]

Hi [Name],

No worries if last email got buried. I've got a couple of 30-min slots
this Thursday if you want to chat about CLI architecture.

[Calendar Link]

[Your Name]
```

**Follow-up Email 2** (5 days):
```
Subject: RE: Free architecture review for [Company]

Hi [Name],

Last offer: Free 30-min architecture review this month.

[Calendar Link]

If this isn't a priority right now, I get it. Feel free to reach out
if it becomes one.

[Your Name]
```

**Conversion Expected**: 15-25% (high-value offer, decision maker likely)

---

### Stream 4: ggen SaaS Platform

**Cold Email Template**:
```
Subject: Save 40 hours on CLI code generation

Hi [Name],

Quick question: How much time does [Company] spend on CLI boilerplate
code?

We built ggen because most teams spend 40-60 hours per year on repetitive
CLI scaffolding:
- Argument parsing setup
- Error handling patterns
- Help text generation
- Testing boilerplate

ggen cuts that down to 30 minutes.

We're in closed beta. Interested in free access?

[Sign-up Link]

[Your Name]
```

**Lead Magnet**: Free ggen trial (30 days, unlimited generations)

**Conversion Expected**: 2-5% (SaaS products, slower decision cycle)

---

## Part 5: Deal Progression Playbook

### PROSPECT → CONTACTED (First Touch)

```
CRITERIA:
- Cold email sent OR cold call made OR LinkedIn message sent
- Evidence in pipeline notes

ACTION:
- Email sent with clear value prop
- Add follow-up reminder (3 days)
- Add to email sequence

MEASUREMENT:
- "First Contact Date" logged
- Email open rate tracked
- Response rate tracked
```

### CONTACTED → INTERESTED (First Conversation)

```
CRITERIA:
- Prospect responded with question
- Attended demo
- Clicked link and asked follow-up
- Answered discovery questions

ACTION:
- Schedule exploratory call (30-60 min)
- Prepare discovery questions
- Send meeting agenda 24h before call

DISCOVERY CALL STRUCTURE (30 min):
1. Introduction (5 min)
   - "Thanks for taking the time"
   - "Here's what I'm hoping we talk about today..."
   - Get permission: "Is 30 min still okay?"

2. Their Situation (10 min)
   - "Tell me about your current CLI setup"
   - "What's working well?"
   - "What's causing the most headache?"
   - Listen for pain points

3. Our Solution (10 min)
   - "Based on what you described, here's what could help..."
   - Show 1-2 relevant examples
   - Ask: "Does this resonate?"

4. Next Steps (5 min)
   - "If this interests you, what would the next step look like?"
   - "Would a proposal make sense?"
   - Get explicit yes/no

MEASUREMENT:
- Call happened (checkbox)
- Pain points documented (text field)
- Interest level rated (1-5)
- Next step defined
```

### INTERESTED → PROPOSAL (Decision Coming)

```
CRITERIA:
- Prospect expressed clear interest
- Agreed to next step of "proposal"
- Budget window identified (month, quarter)
- Decision-making process explained

ACTION:
- Send proposal within 24 hours (while momentum hot)
- Proposal format varies by stream:

  SUPPORT SLA PROPOSAL:
  - Tier recommendation (Startup/Team/Enterprise)
  - Pricing (monthly + annual discount)
  - What's included (response times, queries/month)
  - Support channels (email, Slack, phone)
  - Onboarding process
  - 1-page, leave signature field for agreement

  CONSULTING PROPOSAL:
  - Scope of work (deliverables)
  - Timeline (weeks/months)
  - Daily/hourly rates
  - Payment schedule (upfront? 50% deposit?)
  - References + case study
  - "Accept this proposal" button (make it easy)

  TRAINING PROPOSAL:
  - Course list with topics
  - Time commitment (hours/week)
  - Certificate upon completion
  - Group or individual
  - Price with bulk discount if multiple employees
  - Start date options

MEASUREMENT:
- Proposal sent date logged
- Proposal amount recorded
- Probability updated (60-70%)
- Follow-up date set (5 days)
```

### PROPOSAL → NEGOTIATING (They're Asking Questions)

```
CRITERIA:
- Prospect has read proposal
- Asked clarifying questions
- Not yet agreed to terms
- May be negotiating price or scope

ACTION:
- Answer questions within 24 hours (show responsiveness)
- Be willing to flex:
  * Price: Consider 10-20% discount for longer commitment
  * Scope: Consider removing lower-priority items
  * Timeline: Could accelerate if needed
  * Payment: Could do 50% deposit + 50% on completion

NEGOTIATION RULES:
- Never go below 70% of asking price
- Always get longer commitments (annual beats monthly)
- Always require upfront deposit (shows serious intent)
- Always timeline explicitly (avoid scope creep)

MEASUREMENT:
- Negotiation notes logged
- Counter-proposal sent date
- Probability updated based on progress
```

### NEGOTIATING → CLOSED-WON (They Said Yes!)

```
CRITERIA:
- Verbal agreement received
- Signed proposal or contract received
- Payment initiated or scheduled

ACTION:
1. IMMEDIATE (same day):
   - Send welcome email
   - Create customer record in CS system
   - Schedule kick-off call
   - Send first invoice (or checkout link)

2. WITHIN 24 HOURS:
   - Onboarding package sent
   - Add to support ticketing system
   - Create success plan (what success looks like)
   - Schedule 30-day check-in

3. WITHIN 1 WEEK:
   - Kick-off call completed
   - Customer using product/service
   - Success metrics defined
   - Check-in scheduled for 7 days

MEASUREMENT:
- Deal closed date
- Final amount
- Contract length
- Revenue by stream
- Monthly recurring revenue (if applicable)
```

### ANY STAGE → CLOSED-LOST (They Said No)

```
CRITERIA:
- Prospect explicitly declined
- Or you haven't heard from them in 30 days
- Or they unsubscribed

ACTION:
1. LOG IT:
   - Reason: "Not interested", "No budget", "Wrong fit", "Chose competitor"
   - Learnings: What could you have done differently?
   - Add to "Lost Deal Review" monthly

2. STAY WARM:
   - Add to long-term nurture list (email every quarter)
   - Save notes for future attempt (circumstances change)
   - Consider re-reach in 6-12 months

MEASUREMENT:
- Close date
- Lost reason (for learning)
- Plan to re-engage (6, 12 mo)
```

---

## Part 6: Weekly Sales Review (30 minutes, Every Friday)

### Review Template

```
WEEK OF: [Date]

ACTIVITY METRICS:
├─ Emails sent: ___ (Goal: 25)
├─ Cold calls made: ___ (Goal: 5)
├─ Prospects contacted: ___ (Goal: 20)
├─ Responses received: ___ (Goal: 3-5, ~15% response rate)
├─ Demos scheduled: ___ (Goal: 2)
├─ Demos completed: ___ (Goal: 1-2)
└─ Deals closed: ___ (Goal: 1 per month)

PIPELINE HEALTH:
├─ Total prospects in pipeline: ___
├─ Total pipeline value: $___
├─ Expected closes next 30 days: $___
├─ By stream:
│  ├─ Support pipeline: $___
│  ├─ Training pipeline: $___
│  ├─ Consulting pipeline: $___
│  └─ [Other streams]: $___
└─ Bottleneck stage: ___

CONVERSION RATES:
├─ Prospect → Contacted: ___% (Goal: 80%)
├─ Contacted → Interested: ___% (Goal: 20%)
├─ Interested → Proposal: ___% (Goal: 50%)
├─ Proposal → Closed: ___% (Goal: 30%)
└─ Overall Prospect → Closed: ___% (Goal: 2%)

WINS & LOSSES:
├─ WINS:
│  └─ [Customer name] - $[amount] - [stream]
├─ LOSSES:
│  └─ [Company] - Reason: [chose competitor / no budget / wrong fit]
└─ LEARNING: [What did we learn this week?]

WHAT'S WORKING:
├─ Best source: [LinkedIn / Cold Email / Referral]
├─ Best stream: [Support / Training / Consulting]
├─ Best message angle: [Pain point / Use case / Social proof]
└─ Best time to reach out: [Morning / Afternoon / Day]

WHAT'S NOT WORKING:
├─ Poor response rate to: [Specific email template]
├─ Low conversion from: [Specific source]
└─ Action to take: [A/B test new message / focus on better channel]

NEXT WEEK PRIORITIES:
├─ [ ] Follow up on [X] warm leads
├─ [ ] Close [X] deals in pipeline
├─ [ ] Test new outreach angle: [___]
├─ [ ] Reach out to [specific segment]
└─ [ ] Improve [specific metric]
```

---

## Part 7: Monthly Revenue Forecast

### Create Monthly Dashboard

```
MONTH: [January 2026]

PIPELINE FORECAST:
├─ Closed already: $___
├─ Likely to close (>75% prob): $___
├─ Expected to close (50-75% prob): $___
├─ Could close (25-50% prob): $___
├─ Long shot (0-25% prob): $___
└─ TOTAL PIPELINE: $___

REVENUE BY STREAM:
├─ Support & SLA: $___
├─ Training: $___
├─ Consulting: $___
├─ ggen: $___
├─ Frontier Pro: $___
├─ Marketplace: $___
└─ Enterprise: $___

METRICS:
├─ MRR (monthly recurring): $___
├─ ARR (annual recurring): $___
├─ One-time revenue: $___
├─ Total revenue: $___
├─ Month-over-month growth: ___%
├─ Projected Year-End ARR: $___

CUSTOMER COUNT:
├─ New customers this month: __
├─ Total customers active: __
├─ Churn (customers lost): __
├─ Net new: __
├─ Customers by stream: [Support: __, Training: __, etc.]

KEY METRICS:
├─ CAC (Customer Acquisition Cost): $___
├─ LTV (Lifetime Value): $___
├─ Payback Period: __ months
├─ NPS (Net Promoter Score): __
└─ Churn Rate: ___%

NEXT MONTH PLAN:
├─ Outreach target: __ new prospects
├─ Close target: $__ revenue
├─ Focus area: [Support / Training / Consulting / etc.]
├─ New initiative: [New email template / New channel / etc.]
└─ Metrics to improve: [Response rate / Conversion rate / Deal size]
```

---

## Part 8: Pipeline Dashboard (At a Glance)

### The Sales Dashboard (Update Weekly)

```
                    SALES PIPELINE DASHBOARD
                          [Today's Date]

┌────────────────────────────────────────────────────────┐
│                    PIPELINE VALUE                      │
├────────────────────────────────────────────────────────┤
│                    $47,500 (Open Deals)                │
│  Prospect      Interested      Proposal      Closing   │
│    $18K          $15K           $10K           $4.5K   │
└────────────────────────────────────────────────────────┘

┌────────────────────────────────────────────────────────┐
│              REVENUE BY STREAM (Open Deals)             │
├────────────────────────────────────────────────────────┤
│  Support SLA   Training  Consulting   ggen    Frontier │
│    $12K         $5K       $18K        $8K      $4.5K   │
└────────────────────────────────────────────────────────┘

┌────────────────────────────────────────────────────────┐
│           THIS MONTH'S CLOSE PROBABILITY                │
├────────────────────────────────────────────────────────┤
│  High (>75%)        Medium (50-75%)      Low (<50%)     │
│   $15,000           $18,000               $14,500       │
│   2 deals           3 deals               4 deals       │
│   Close chance: 75%  Close chance: 60%   Close chance: 20%
│                                                         │
│  ➜ Realistic Close Target: $24,000 (75% + 60% of med)  │
└────────────────────────────────────────────────────────┘

┌────────────────────────────────────────────────────────┐
│              THIS WEEK'S ACTION ITEMS                   │
├────────────────────────────────────────────────────────┤
│  [ ] Follow up - Acme Corp proposal (sent 3 days ago)  │
│  [ ] Call - TechStart CEO (scheduled Wednesday 2 PM)   │
│  [ ] Proposal - BigCorp Consulting (due Friday)        │
│  [ ] Nurture - 5 warm prospects (send email Tue)       │
│  [ ] Outreach - 25 cold emails (do Monday-Wednesday)   │
│  [ ] Demo - StartupXYZ (scheduled Thursday 10 AM)      │
└────────────────────────────────────────────────────────┘

┌────────────────────────────────────────────────────────┐
│              CONVERSION FUNNEL (This Quarter)           │
├────────────────────────────────────────────────────────┤
│  Prospects        →  Interested    →  Proposal   →  Won
│  Contacted: 75       20 (27%)       10 (50%)      3 (30%)
│                                                    │
│  Conversion rate: 3/75 = 4% (TARGET: 2%)        ✓     │
└────────────────────────────────────────────────────────┘
```

---

## Sales Playbook: Daily Checklist

### MORNING (8 AM)
- [ ] Review pipeline dashboard (2 min)
- [ ] Check for responses from yesterday (3 min)
- [ ] Identify today's priority actions (3 min)
- [ ] Block 90-minute selling session (2 min)

### SELLING HOUR (90 min, your choice of time)
- [ ] Send 5 cold emails to prospects (25 min)
  OR call 3 prospects (45 min)
  OR schedule 1-2 discovery calls (60 min)
  OR send 3 proposals (30 min)
- [ ] Follow up with warm leads (25 min)
- [ ] Log all activity in pipeline (10 min)

### EVENING (5 PM)
- [ ] Update pipeline with day's activity (3 min)
- [ ] Set tomorrow's priorities (2 min)

### WEEKLY (Friday 5 PM, 30 min)
- [ ] Complete weekly sales review (30 min)
- [ ] Adjust next week's plan based on results

### MONTHLY (Last Friday of month, 60 min)
- [ ] Analyze monthly metrics (20 min)
- [ ] Forecast next month (20 min)
- [ ] Plan next month's initiatives (20 min)

---

## Success Metrics: What "Good" Looks Like

| Metric | Target | Why It Matters |
|--------|--------|----------------|
| **Weekly Activity** | | |
| Emails sent | 25+ | Enough volume for results |
| Response rate | 15%+ | Shows message resonates |
| Demos scheduled | 2+ | Converts to interested |
| **Monthly Metrics** | | |
| New prospects added | 20+ | Building pipeline |
| Deals closed | 1-3 | Revenue generation |
| Pipeline value | 3-5x monthly close | 3-5 months of revenue |
| MRR growth | +15-20% | Business scaling |
| **Conversion Rates** | | |
| Prospect → Interested | 20%+ | Good email + targeting |
| Interested → Proposal | 50%+ | Good discovery calls |
| Proposal → Closed | 30%+ | Good proposals |
| Overall Prospect → Closed | 2%+ | Efficient funnel |
| **Deal Health** | | |
| Avg deal size | $1K-10K | Mix of sizes is good |
| Sales cycle | 14-45 days | Time to close |
| CAC | <30% of LTV | Unit economics healthy |

---

## Troubleshooting: When Sales Aren't Working

### Low Response Rate (<10%)?
```
DIAGNOSIS:
- Wrong prospect? (Targeting poor fit)
- Wrong message? (Value prop unclear)
- Wrong channel? (Email not right for audience)
- Wrong timing? (Reaching out at wrong time)

FIX (Try these in order):
1. Change message angle (test new pain point)
2. Change target audience (try different company size)
3. Change channel (LinkedIn vs email vs Twitter)
4. Change timing (send at different time of day/week)
5. Improve subject line (A/B test 3 new ones)

TRACK:
- Log A/B test results
- Double down on what works
- Kill what doesn't
```

### Low Demo Attendance?
```
ISSUE: People agreed to demo but no-showed

FIX:
1. Send calendar invite 24h before (not just email)
2. Send reminder 30 min before (with Zoom link)
3. Pre-demo check-in day before (confirm still interested)
4. Make it easy (Zoom link in email, not buried in text)
5. Ask: "What time works best for you?" (their timezone)
```

### Low Proposal Acceptance?
```
ISSUE: You're sending proposals but they're saying no

FIX:
1. Check proposal quality:
   - Clear problem statement (does it resonate?)
   - Clear solution (does it address their pain?)
   - Clear price (is it reasonable for value?)
   - Clear next steps (make it easy to say yes)

2. Check timing:
   - Did discovery call happen <48h before proposal?
   - Is proposal sent <24h after call (while hot)?
   - Is there momentum or did it die?

3. Negotiate:
   - Are they asking for lower price? (Try 3-month discount)
   - Are they unsure about scope? (Offer pilot project)
   - Are they stalling? (Set deadline: "Offer ends Friday")
```

---

## Templates You Can Copy

### Email Subject Line Formulas (High Open Rates)

```
CURIOSITY:
"Quick question about [Company]'s CLI stack?"
"Did you see this?" (with link)

RELEVANCE:
"[Person]'s CLI infrastructure is worth $1.5M/year"
"How [Competitor] saves 40 hours/year with ggen"

SOCIAL PROOF:
"[X] companies like [Company] already use this"
"Why [Well-Known Company] switched to clap"

TIME-SENSITIVE:
"This expires Friday"
"1 slot left for this week"
"Last early-bird discount"

SPECIFICITY:
"Free architecture review for [Company]'s CLI stack"
"How [Company] could save 200 engineering hours"
"The CLI bottleneck at [Company]"

TESTED HIGHEST OPENERS (40%+):
"Quick question about [Company]?"
"Idea for [Name]?"
"[Name] - [specific reference]?"
```

---

## Remember: The Pareto Principle (80/20)

**80% of your revenue will come from:**
- 20% of your effort (focus on high-efficiency activities)
- 20% of your customers (serve them best)
- 20% of your time (your selling hours = prime time)

**Spending all your time on low-leverage activities:**
- Creating perfect proposals (diminishing returns)
- Perfecting email templates (testing is good, perfecting is waste)
- Organizing CRM (should take 30 min/week max)
- Tracking metrics (30 min/week is enough)

**Spend most time on high-leverage:**
- Getting in front of prospects (80% of time)
- Asking good discovery questions (converts better)
- Following up persistently (3-7 touches before close)
- Closing deals (ask for the sale!)

---

## Launch Checklist: Getting Started This Week

### DAY 1 (Monday, 1 hour)
- [ ] Create sales pipeline in Google Sheets (copy template above)
- [ ] Add first 20 prospects from your list
- [ ] Identify your best sales time (morning, afternoon, evening)

### DAY 2-4 (Tue-Thu, 90 min each day)
- [ ] Do first selling hour:
  - [ ] Send 5 cold emails to prospects
  - [ ] Log all activity in pipeline
- [ ] Do it 3 more days this week (90 min each)

### DAY 5 (Friday, 30 min)
- [ ] Review pipeline
- [ ] Check activity metrics
- [ ] Plan next week

### RESULT:
- 20+ outreach activities this week
- 3-4 responses likely
- 1-2 interested prospects
- Foundation for ongoing sales

**First closed deal expected**: Week 3-4 (if you execute consistently)

---

**Author**: Claude (AI Sales Advisor)
**Created**: January 2026
**Version**: 1.0 - Initial Sales Operations Playbook

For questions: Refer to REVOPS_GAP_ANALYSIS.md for context and priorities.
