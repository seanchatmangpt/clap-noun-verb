# Finance Operations Guide

**Billing, invoicing, revenue tracking, and financial forecasting**

---

## Executive Summary

**Goal**: Know your business health in real-time.

**Key Questions Finance Must Answer**:
1. What's my MRR (Monthly Recurring Revenue)?
2. What's my ARR (Annual Recurring Revenue)?
3. What's my burn rate (monthly costs)?
4. Am I cash-flow positive?
5. Where's my revenue coming from?

---

## Part 1: Stripe Billing Setup (30 minutes)

### Creating Your Billing System

```
STRIPE SETUP CHECKLIST:

1. Account Creation (5 min)
   ✓ Go to stripe.com
   ✓ Create account
   ✓ Verify email
   ✓ Complete business info

2. Bank Account Linking (10 min)
   ✓ Add bank account for payouts
   ✓ Verify microdeposits (2-3 days)
   ✓ Set payout schedule (daily? weekly?)

3. Products Creation (10 min)
   ✓ Create product for each tier
   ✓ Product: Support - Startup
     - Price: $99/month
     - Billing interval: Monthly
     - Type: Recurring subscription

   ✓ Product: Support - Team
     - Price: $499/month
     - Billing interval: Monthly

   ✓ Product: Support - Enterprise
     - Price: $1,999/month
     - Billing interval: Monthly
     - OR custom pricing (contact sales)

   ✓ Training Course (Single Product)
     - Price: $49 (one-time)
     - Multiple pricing tiers in name

4. Payment Links (5 min)
   ✓ Create checkout links for each product
   ✓ Share links in your sales/marketing
   ✓ Example: "Buy here: [Stripe link]"

5. Customer Portal (10 min)
   ✓ Enable customer self-service
   ✓ Customers can manage subscriptions
   ✓ Customers can download invoices
   ✓ Reduces support tickets

6. API Keys (2 min)
   ✓ Get publishable key
   ✓ Get secret key
   ✓ Save in secure location (not GitHub!)
```

### Testing Stripe (Use Test Mode)

```
TEST CARDS (Work only in Test Mode):
Successful charge:     4242 4242 4242 4242
Card declined:         4000 0000 0000 0002
Requires 3D secure:    4000 0025 0000 3155

TEST PROCESS:
1. Use card "4242 4242 4242 4242"
2. Any future expiration date
3. Any 3-digit CVC
4. Any name
5. Process transaction
6. Verify in Stripe dashboard

LIVE MODE:
- Switch to Live mode when ready
- Real cards only
- Real money collected
- No test cards work
```

---

## Part 2: Monthly Revenue Dashboard (Google Sheets)

### Create This Spreadsheet

```
REVENUE DASHBOARD - January 2026

┌─────────────────────────────────────────┐
│          REVENUE AT A GLANCE             │
├─────────────────────────────────────────┤
│ MRR (Monthly Recurring):        $12,450  │
│ One-time Revenue (this month):  $2,100   │
│ Total Revenue:                  $14,550  │
│ Growth Rate (vs. last month):   +22%     │
│ Annual Recurring Revenue (ARR): $149,400 │
└─────────────────────────────────────────┘

BY STREAM:
┌──────────────────────────────────────────────────┐
│ Stream           │ Customers │ MRR    │ % of Total
├──────────────────────────────────────────────────┤
│ Support & SLA    │ 5         │ $3,500 │ 28%
│ Training         │ 12        │ $588   │ 5%
│ Consulting       │ 2         │ $5,000 │ 40%
│ ggen SaaS        │ 8         │ $1,200 │ 10%
│ Frontier Pro     │ 2         │ $900   │ 7%
│ Marketplace      │ 50        │ $600   │ 5%
│ Enterprise       │ 1         │ $662   │ 5%
├──────────────────────────────────────────────────┤
│ TOTAL            │ 80        │ $12,450│ 100%
└──────────────────────────────────────────────────┘

MONTHLY REVENUE TREND:
Month 1:  $8,500   (Month 1 baseline)
Month 2:  $11,300  (↑ 33% growth)
Month 3:  $14,550  (↑ 29% growth)
Avg Mo:   $11,450
Trend:    📈 Strong growth

UNIT ECONOMICS:
CAC (Customer Acquisition Cost): $350
LTV (Lifetime Value):            $3,600
Payback Period:                  2.4 months
LTV:CAC Ratio:                   10:1 ✓ (healthy)

CASH METRICS:
├─ Total cash in (YTD):          $34,350
├─ Stripe fees (2.9% + $0.30):   $1,187
├─ Net cash collected:           $33,163
├─ Operating costs:              $4,200
├─ Net profit (YTD):             $28,963
└─ Cash runway:                  12+ months
```

### Spreadsheet Template (Copy This)

```
COLUMNS:
A. Date (when transaction occurred)
B. Customer Name
C. Stream (Support/Training/Consulting/ggen/etc.)
D. Amount ($)
E. Type (Monthly / Annual / One-time)
F. Payment Method (Stripe / Bank transfer / etc.)
G. Status (Paid / Pending / Failed)
H. Invoice Sent? (Yes/No)
I. Contract Length (1mo / 3mo / 6mo / 12mo / one-time)
J. Renewal Date (when payment renews)
K. Notes (customer name, project, etc.)

FORMULAS:
- Monthly Revenue = SUM(Amount where Type="Monthly")
- Annual Revenue = SUM(Amount where Type="Annual") / 12
- Total Revenue = SUM(Amount)
- By Stream Total = SUMIF(Stream, "[Stream]", Amount)
- Growth Rate = (This Month - Last Month) / Last Month * 100
- Revenue by Customer = SUMIF(Customer, "[Name]", Amount)
```

---

## Part 3: Unit Economics Dashboard

### Key Metrics to Track

```
UNIT ECONOMICS:

Customer Acquisition Cost (CAC):
= Total Sales & Marketing spend / Number of customers acquired
= $1,500 / 5 customers = $300 per customer

Customer Lifetime Value (LTV):
= Average customer revenue * Average customer lifetime
= $200/month * 18 months = $3,600 LTV

Payback Period:
= CAC / (ARPU - COGS)
= $300 / ($200/mo - $10/mo) = 1.6 months
(How many months to recover acquisition cost)

LTV:CAC Ratio:
= LTV / CAC
= $3,600 / $300 = 12:1
(Should be >3:1 for healthy business)

Monthly Churn Rate:
= Customers lost this month / Starting customers
= 2 / 50 = 4%
(Every 1% = 12% annual churn)

Net Revenue Retention (NRR):
= (Expansion revenue - Churn revenue) / Starting MRR
= ($500 expansion - $200 churn) / $10,000 = 3%
(Target: >100% means net growth from existing customers)
```

### Unit Economics Dashboard Template

```
UNIT ECONOMICS DASHBOARD - January 2026

COHORT: New customers started in [Month]
Start Date: Jan 1, 2026
Cohort Size: 5 customers
Survival Time: 30 days

Month 1 Metrics:
├─ Churn (customers lost): 0 / 5 (0%)
├─ Revenue: $5,200
├─ Cost to serve: $400
├─ Gross margin: 92%
├─ NPS: +35 (strong)
└─ Health: ✓ Excellent

30-Day Forecast:
├─ Expected month 2 churn: 1 / 5 (20%)
├─ Expected month 2 revenue: $4,160
├─ LTV projection: $2,200
├─ CAC: $300
├─ Payback: 1.8 months
└─ LTV:CAC ratio: 7.3:1 ✓

---

UNIT ECONOMICS BY STREAM:

Support & SLA:
├─ Avg customer value: $700/month
├─ Avg customer lifetime: 24 months
├─ LTV: $16,800
├─ CAC: $200 (easy to reach)
├─ LTV:CAC: 84:1 ✓ (Excellent!)
└─ Churn: 2%/month

Training:
├─ Avg customer value: $49 (one-time)
├─ Avg customer lifetime: 1 course
├─ LTV: $49
├─ CAC: $15 (content marketing)
├─ LTV:CAC: 3.3:1 ✓ (Healthy)
└─ Expansion to consulting: 15%

Consulting:
├─ Avg project value: $25,000
├─ Avg projects/year: 0.5
├─ LTV: $12,500/year
├─ CAC: $500 (direct outreach)
├─ LTV:CAC: 25:1 ✓ (Excellent!)
└─ Repeat customer rate: 40%

ggen SaaS:
├─ Avg customer value: $150/month
├─ Avg customer lifetime: 12 months
├─ LTV: $1,800
├─ CAC: $100 (word of mouth)
├─ LTV:CAC: 18:1 ✓ (Excellent!)
└─ Expansion: 25% upgrade to Frontier Pro
```

---

## Part 4: Monthly Invoice Process

### Automated Invoice System

```
STRIPE SETUP:
1. Enable automatic invoicing
   - Settings → Billing → Invoices
   - Send invoice before payment due
   - Auto-retry failed payments

2. Invoice customization
   - Add company logo
   - Add payment instructions
   - Add support contact
   - Add next billing date

3. Automatic payment collection
   - For recurring subscriptions (automatic)
   - For one-time purchases (collect at checkout)
   - Failed payment retry (3 attempts)

MANUAL INVOICING (If not using Stripe):
1. Create invoice template (Google Docs)
   - Invoice number (e.g., INV-2026-001)
   - Date
   - Due date (e.g., net 15)
   - Customer name and email
   - Service description
   - Price
   - Payment instructions
   - Your contact info

2. Send invoice
   - Send via email
   - Mark when sent in spreadsheet
   - Confirm delivery

3. Payment tracking
   - Note payment received date
   - Send receipt
   - Log in revenue spreadsheet
```

### Invoice Template (Copy This)

```
                    [YOUR COMPANY NAME]

INVOICE

Invoice #:           INV-2026-001
Date:                January 15, 2026
Due Date:            January 29, 2026

BILL TO:
[Customer Name]
[Company Name]
[Email]

DESCRIPTION:
┌──────────────────────────────────────────────┐
│ Support & SLA - Team Tier (1 month)          │
│ Service dates: Jan 1 - Jan 31, 2026          │
│ Includes: 24h response SLA, unlimited qs.    │
└──────────────────────────────────────────────┘

Subtotal:            $499.00
Tax (0% - digital):  $0.00
────────────────────────────
TOTAL:               $499.00

PAYMENT INSTRUCTIONS:
Pay by: [PayPal link / Stripe link / Bank transfer details]
Reference: Invoice INV-2026-001

Next Billing: February 15, 2026
Subscription: Auto-renews monthly

Questions? Email: billing@[company].com

[Your Name]
Clap Creator
```

---

## Part 5: Cash Flow Forecast (3-Month + 12-Month)

### Create Your Cash Forecast

```
3-MONTH CASH FLOW FORECAST

                  January      February     March       Q1 Total
Revenue Forecast:
├─ Recurring       $8,500       $10,300     $12,200     $31,000
├─ New customers   $2,100       $1,800      $2,500      $6,400
├─ Expansion       $500         $800        $1,200      $2,500
└─ TOTAL           $11,100      $12,900     $15,900     $39,900

Operating Costs:
├─ Tools (Stripe) $300         $350        $400        $1,050
├─ Infrastructure $500         $500        $500        $1,500
├─ Marketing      $500         $800        $1,200      $2,500
└─ Other          $200         $200        $200        $600

Total Expenses:    $1,500       $1,850      $2,300      $5,650

─────────────────────────────────────────────────
NET CASH FLOW:     +$9,600      +$11,050    +$13,600    +$34,250

Cumulative Cash:   $9,600       $20,650     $34,250

Runway (months):   52+ months (Excellent!)

---

12-MONTH FORECAST

Month  MRR Target   Total Revenue   Costs      Net        Cumulative
1      $8,500       $11,100         $1,500     $9,600     $9,600
2      $10,300      $12,900         $1,850     $11,050    $20,650
3      $12,200      $15,900         $2,300     $13,600    $34,250
4      $15,000      $18,500         $3,000     $15,500    $49,750
5      $18,000      $21,200         $3,500     $17,700    $67,450
6      $22,000      $25,000         $4,000     $21,000    $88,450
...
12     $50,000      $62,000         $8,000     $54,000    $500,000

KEY INSIGHTS:
- Cash positive month 1 ✓
- Runway: 12+ months from day 1
- Expense growth: 100% (following revenue)
- Profitability: Strong and improving
```

---

## Part 6: Tax & Accounting Essentials

### What You Need to Know

```
VAT / SALES TAX:
- US: 0% (digital goods exempt in many states)
- EU: 20-27% (must charge on B2C sales)
- Other: Varies by country

Solution: Use Stripe or Lemonsqueezy (they handle tax compliance)

EXPENSE TRACKING:
- Keep receipts for all business expenses
- Tools: Stripe, ConvertKit, Calendly = business expenses
- Home office: If working from home, deduct portion
- Internet: Business portion deductible
- Equipment: Laptop, monitor, etc.

TAX DEDUCTIONS (Typical):
- Tools & software: 100%
- Home office: Pro-rata share
- Internet: Pro-rata share
- Phone: Pro-rata share
- Equipment: Depreciate over time
- Education: Conferences, courses
- Travel: Client meetings, conferences

CONTRACTOR PAYMENTS:
- If you pay freelancers: Need their W9 (US) or equivalent
- Must issue 1099 (US) if paid >$600/year
- Keep records for IRS

QUARTERLY ESTIMATES (US):
- If profit expected: Pay quarterly taxes
- Due: April 15, June 15, Sept 15, Jan 15
- Amount: ~25% of estimated profit

BUSINESS STRUCTURE:
Recommend: Single-member LLC
- Limited personal liability
- Pass-through taxation (don't pay corporate tax)
- Easy to set up ($50-300 depending on state)
- Costs: ~$100-300/year state filing
```

### Expense Tracking Template

```
EXPENSE TRACKING - January 2026

Date      Description          Category         Amount    Business%   Deduction
─────────────────────────────────────────────────────────────────────────────
1/1       Stripe setup         Professional     $50       100%        $50
1/3       ConvertKit (monthly) Software tools   $29       100%        $29
1/3       Calendly pro         Software tools   $10       100%        $10
1/5       Domain renewal       Website          $12       100%        $12
1/10      Internet (Jan)       Office           $60       50%         $30
1/15      Laptop (new)         Equipment        $1,200    100%        $1,200*
1/20      YouTube Premium      Marketing       $14       50%         $7

TOTAL EXPENSES:                                 $1,375
TOTAL DEDUCTIONS:              $1,338

*Equipment depreciated over 5 years = $240/year deduction

PROFIT & TAX:
Revenue (Month 1):            $11,100
Deductible Expenses:          -$1,338
Taxable Income:               $9,762

Federal Income Tax (~22%):    -$2,147
Self-Employment Tax (~15%):   -$1,465
─────────────────────────────
NET PROFIT (after tax):       $6,150
```

---

## Part 7: Financial Dashboard (Monthly Review)

### What to Review Every Month

```
FINANCIAL HEALTH CHECK - [Month]

REVENUE METRICS:
├─ MRR: $____ (Target: $____)
├─ Growth vs. last month: ___% (Target: >15%)
├─ By stream:
│  ├─ Support: $____ (vs. last mo: ___%)
│  ├─ Training: $____ (vs. last mo: ___%)
│  ├─ Consulting: $____ (vs. last mo: ___%)
│  └─ Other: $____ (vs. last mo: ___%)
└─ Churn revenue: -$____ (customers lost)

CUSTOMER METRICS:
├─ New customers: __
├─ Churned: __
├─ Net new: __
├─ Total customers: __
└─ Expansion revenue: $____

UNIT ECONOMICS:
├─ CAC: $____ (vs. LTV: $____)
├─ Payback: ___ months
├─ LTV:CAC ratio: __:1
└─ Health: ✓ / ⚠ / ❌

EXPENSE METRICS:
├─ Total monthly costs: $____
├─ As % of revenue: __% (Target: <20%)
├─ Breakdown:
│  ├─ Tools: $____
│  ├─ Marketing: $____
│  ├─ Operations: $____
│  └─ Other: $____
└─ Trend: ↑ / → / ↓

CASH METRICS:
├─ Cash in (revenue): $____
├─ Cash out (expenses): -$____
├─ Net cash flow: +$____ / -$____
├─ Cumulative cash: $____
└─ Runway: ___ months

PROFITABILITY:
├─ Gross profit: $____ (___%)
├─ Operating profit: $____ (___%)
├─ Net profit (before tax): $____ (___%)
└─ Projected annual profit: $____

RED FLAGS TO WATCH:
├─ Churn rate >5%?
├─ CAC increasing?
├─ LTV decreasing?
├─ Growth slowing (<15% MoM)?
├─ Burn rate increasing?
└─ Runway <6 months?

DECISIONS FOR NEXT MONTH:
├─ [ ] Invest in: [What's working?]
├─ [ ] Kill: [What's not working?]
├─ [ ] Experiment: [New channel or product?]
└─ [ ] Optimize: [Biggest leverage opportunity?]
```

---

## Part 8: Annual Financial Summary

### End of Year Review

```
ANNUAL FINANCIAL REPORT - 2026

REVENUE:
├─ Year 1 Total Revenue: $428,000
├─ Monthly Average: $35,667
├─ Growth Trajectory: $11K (Jan) → $42K (Dec)
├─ By Stream:
│  ├─ Support & SLA: $48,000 (11%)
│  ├─ Training: $114,000 (27%)
│  ├─ Consulting: $50,000 (12%)
│  ├─ ggen SaaS: $136,000 (32%)
│  ├─ Frontier Pro: $26,000 (6%)
│  ├─ Marketplace: $30,000 (7%)
│  └─ Enterprise: $24,000 (6%)
└─ Actual vs. Forecast: +12% ahead of target

CUSTOMERS:
├─ Total customers acquired: 189
├─ Annual churn rate: 18% (monthly: 1.5%)
├─ Average customer LTV: $2,250
├─ Repeat customer rate: 25%
└─ Customer concentration: Top 5 = 35% of revenue

PROFITABILITY:
├─ Total expenses: $89,000
├─ Gross profit: $339,000 (79%)
├─ Operating profit: $339,000 (79%)
├─ Net profit (before tax): $339,000
├─ Tax payable (~25%): -$85,000
└─ NET PROFIT (after tax): $254,000

METRICS:
├─ CAC: $471
├─ LTV: $2,250
├─ Payback period: 2.3 months
├─ LTV:CAC: 4.8:1 ✓
├─ NRR: 85% (need to improve to 100%+)
└─ Rule of 40: Revenue growth (312%) + profit (79%) = 391% ✓✓✓

KEY WINS:
✓ Positive cash flow from Month 1
✓ Zero debt
✓ 4-month cash runway
✓ Strong unit economics
✓ Multiple revenue streams diversified
✓ Customer satisfaction high (NPS: +35)

OPPORTUNITIES FOR YEAR 2:
⚠ Improve retention (reduce churn 18% → 8%)
⚠ Improve NRR (add expansion revenue)
⚠ Automate support (scale without hiring)
⚠ Product development (build ggen SaaS further)
⚠ Team scaling (hire CS rep + 1 sales)

YEAR 2 PROJECTION:
├─ Revenue target: $2.9M (6.7x growth)
├─ Customer target: 350+ customers
├─ Profitability: ~$1.2M net profit
├─ Path: Strong sustainable growth
└─ Status: On track for Series A in 2 years
```

---

## Finance Checklist: Month 1

- [ ] Stripe account set up (30 min)
- [ ] Billing products created (30 min)
- [ ] Google Sheets revenue dashboard (1 hour)
- [ ] First invoice sent (15 min)
- [ ] Unit economics calculated (30 min)
- [ ] 3-month forecast created (1 hour)
- [ ] Expense tracking started (ongoing)

**Total**: 4 hours
**Result**: Complete financial visibility

---

## Key Ratios Reference

```
HEALTHY BUSINESS BENCHMARKS:

Revenue Metrics:
- MRR growth: 10-20% month-over-month
- CAC payback: 1-3 months
- LTV:CAC ratio: 3:1 or higher
- Churn rate: <5% monthly
- NRR: >100% (growing from existing customers)

Profitability:
- Gross margin: >70% (SaaS)
- Operating margin: >30% (mature)
- Net margin: >20% (healthy)

Growth:
- Rule of 40: Growth % + Profit % > 40
- YoY growth: >3x in year 1
- Customer acquisition: Doubling every 6-9 months
```

---

**Author**: Claude (AI Finance Advisor)
**Created**: January 2026
**Version**: 1.0 - Initial Finance Operations Guide

Remember: You can't manage what you don't measure. Review financial metrics weekly, analyze monthly.
