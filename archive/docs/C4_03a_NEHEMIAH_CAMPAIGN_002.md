# C4-03a: Nehemiah 52 Campaign Components

## Mission

Show the campaign loop: how work is requested, routed, executed, verified, and witnessed. Refuse false gates.

## Campaign Loop

```
                    ┌─────────────┐
                    │   PRAYER    │
                    │  (covenant) │
                    └──────┬──────┘
                           │ "God, let us rebuild"
                           │
                    ┌──────▼──────┐
                    │ COURIER     │
                    │ INTAKE      │
                    │             │
                    │ • Record    │
                    │   petition  │
                    │ • Validate  │
                    │   authority │
                    │ • Log event │
                    └──────┬──────┘
                           │
                    ┌──────▼──────┐
                    │   GATE      │
                    │   ROUTER    │
                    │             │
                    │ • Classify  │
                    │   work type │
                    │ • Assign    │
                    │   builder   │
                    │ • Choose    │
                    │   gate path │
                    └──────┬──────┘
                           │
                    ┌──────▼──────┐
                    │  MUSTER     │
                    │  LEDGER     │
                    │             │
                    │ • Record    │
                    │   capacity  │
                    │ • Schedule  │
                    │   assignment│
                    │ • Plan gate │
                    │   sequence  │
                    └──────┬──────┘
                           │
                    ┌──────▼──────┐
                    │   WALL      │
                    │  SECTION    │
                    │  BUILDER    │
                    │             │
                    │ • Builders  │
                    │   execute   │
                    │   work      │
                    │ • Emit      │
                    │   receipts  │
                    │ • Log       │
                    │   events    │
                    └──────┬──────┘
                           │
                    ┌──────▼──────────────┐
                    │     MOCKERS TEST    │
                    │                     │
                    │ • Adversarial probe │
                    │ • Conformance check │
                    │ • Security audit    │
                    │ • Process mining    │
                    └──────┬──────────────┘
                           │
                    ┌──────▼──────┐
                    │   REPAIR    │
                    │   GATE      │
                    │             │
                    │ • If failed: │
                    │   rework    │
                    │ • Log retry │
                    │ • Re-emit   │
                    │   receipt   │
                    └──────┬──────┘
                           │
                    ┌──────▼──────────────┐
                    │  USURY LEDGER       │
                    │                     │
                    │ • Track cost        │
                    │ • Capacity consumed │
                    │ • Interest accrued  │
                    │ • Debt owed         │
                    └──────┬──────────────┘
                           │
                    ┌──────▼──────────────┐
                    │ INSPECTION GATE     │
                    │                     │
                    │ • Final verdict     │
                    │ • Conformance proof │
                    │ • Receipt signed    │
                    │ • Work approved     │
                    └──────┬──────────────┘
                           │
                    ┌──────▼──────────────┐
                    │  NATIONS LEDGER     │
                    │                     │
                    │ • Public record     │
                    │ • What was claimed  │
                    │ • What was proved   │
                    │ • Alignment gap     │
                    └──────┬──────────────┘
                           │
                    ┌──────▼──────────────┐
                    │  REMEMBER PRAYER    │
                    │  (next cycle)       │
                    │                     │
                    │ • God kept covenant │
                    │ • Record results    │
                    │ • Thank offering    │
                    │ • Plan next work    │
                    └─────────────────────┘
```

## Component Specifications

| Component | Type | Role | Inputs | Outputs |
|-----------|------|------|--------|---------|
| **PRAYER** | Initiator | Covenant request to God | Faith, need | Petition (event) |
| **COURIER INTAKE** | Gate | Receive and validate petitions | Petition | Validated request (event) |
| **GATE ROUTER** | Gate | Classify work, assign builder | Validated request | Assigned work (event) |
| **MUSTER LEDGER** | Record | Track capacity and schedule | Work assignment | Muster plan (event) |
| **WALL SECTION BUILDER** | Executor | Execute work; emit receipts | Muster plan | Receipts, work events |
| **MOCKERS TEST** | Gate | Adversarial verification | Receipts, work events | Conformance report (event) |
| **REPAIR GATE** | Gate | Handle failures; rework if needed | Conformance report | Repair plan or approval (event) |
| **USURY LEDGER** | Record | Track cost, interest, debt | Work executed | Cost report (event) |
| **INSPECTION GATE** | Gate | Final verdict; sign receipt | Repair result, cost report | Inspection verdict (event) |
| **NATIONS LEDGER** | Record | Public proof; alignment check | Inspection verdict | Public witness (event) |
| **REMEMBER PRAYER** | Closure | Covenant renewal; plan next cycle | Nations ledger | Prayer for next cycle |

## Critical Rules — False Gates Refused

### ❌ PRAYER is NOT an Agent
- Prayer is a covenant request, not a worker or decision-maker.
- Agents execute work *after* prayer, not during.
- Prayer appears in event logs as an initiating event only.

### ❌ PROPHETS are NOT a Gate
- Prophets interpret God's will; they do not execute work or route it.
- Prophets appear in the wall only if they speak to God (outside the system).
- No "prophet gate" in the campaign flow.

### ❌ NATIONS are NOT a Gate
- Nations witness publicly; they do not route, admit, or approve work.
- Nations appear as external readers of the Nations Ledger.
- Nations cannot change the work; they can only read the record.

### ❌ INTEREST/USURY is NOT a Gate
- Interest is a recorded quantity in the Usury Ledger, not a decision point.
- The Usury Ledger records cost, not approves or rejects work.
- Interest accrues as a passive side-effect; it is not gated.

### ❌ PEOPLE are NOT Gates
- People (Sean, agents, builders) work *through* gates, not *as* gates.
- A gate is a decision point with admission rules. A person implements the decision.
- A person assigned to a gate executes the gate's logic; they are not the gate itself.

## Event Log Structure

Every component transition creates an OCEL event:

```
{
  "event_id": "uuid",
  "timestamp": "ISO8601",
  "component": "COURIER_INTAKE",
  "event_type": "petition_received",
  "objects": {
    "petition": "uuid",
    "requester": "person_or_agent",
    "scope": "work_type"
  },
  "attributes": {
    "priority": "high",
    "authority_validated": true,
    "claimed_duration": "7_days"
  }
}
```

## Conformance Rules

The declared loop must match the event log:

1. Every petition must have a courier intake event.
2. Every intake must have a router assignment.
3. Every assignment must have a muster ledger entry.
4. Every builder action must emit a receipt event.
5. Every receipt must be tested by mockers.
6. If mockers fail, repair gate must emit a rework event.
7. All repairs must be re-tested.
8. Only repairs that pass mockers reach inspection gate.
9. Inspection gate produces a final verdict event.
10. Final verdict is recorded in Nations Ledger.
11. Alignment between claimed scope and proved receipts must be auditable.

## Architecturally Forbidden

- ❌ Prayer → direct execution (must go through all gates)
- ❌ Petition → muster (must go through router)
- ❌ Work → nations ledger (must be approved by inspection gate)
- ❌ Failed work → next cycle (must be repaired or documented as abandoned)
- ❌ Silent failures (every event must be logged)
