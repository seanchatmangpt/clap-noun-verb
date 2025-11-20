# Diataxis: Complete V5 Documentation Index

**Framework**: Diataxis Four-Pillar Documentation System
**Total Coverage**: 4 comprehensive documents + this index
**Last Updated**: 2025-11-20
**Status**: Production-Ready

---

## Quick Navigation

### By Your Role

**🎓 I'm New to v5**
1. Start: [Explanations - Why Machine CLIs Are Different](DIATAXIS_V5_EXPLANATIONS.md#introduction-why-machine-clis-are-different)
2. Read: [Tutorials - Your First v5 Machine API Call](DIATAXIS_V5_TUTORIALS.md#tutorial-1-your-first-v5-machine-api-call)
3. Explore: [How-To - Query Available Commands](DIATAXIS_V5_HOW_TO_GUIDES.md#how-to-query-available-commands-via-introspection)
4. Reference: [Command Invocation Format](DIATAXIS_V5_REFERENCE.md#command-invocation-format)

**🔧 I'm Building an Agent**
1. Start: [Tutorials - Building an Agent That Uses V5](DIATAXIS_V5_TUTORIALS.md#tutorial-2-building-an-agent-that-uses-v5)
2. Reference: [Input Schema Reference](DIATAXIS_V5_REFERENCE.md#input-schema-reference)
3. How-To: [Call Command and Process Response](DIATAXIS_V5_HOW_TO_GUIDES.md#how-to-call-a-command-and-process-machine-readable-response)
4. How-To: [Build Agent Respecting Preconditions](DIATAXIS_V5_HOW_TO_GUIDES.md#how-to-build-an-agent-that-respects-preconditions)

**🤝 I'm Building Multi-Agent Systems**
1. Read: [Explanations - Why Delegation With Proof Matters](DIATAXIS_V5_EXPLANATIONS.md#why-delegation-with-proof-matters)
2. Tutorial: [Agent Delegation with Proofs](DIATAXIS_V5_TUTORIALS.md#tutorial-4-agent-delegation-with-proofs)
3. How-To: [Implement Delegation Between Agents](DIATAXIS_V5_HOW_TO_GUIDES.md#how-to-implement-delegation-between-agents)
4. Reference: [Delegation Certificate Structure](DIATAXIS_V5_REFERENCE.md#delegation-certificate-structure)

**📊 I'm Integrating with Claude/LLMs**
1. Tutorial: [Building MCP Server for V5](DIATAXIS_V5_TUTORIALS.md#tutorial-5-building-an-mcp-server-for-v5)
2. Reference: [MCP Protocol Integration](DIATAXIS_V5_REFERENCE.md#mcp-protocol-integration)
3. How-To: [Build Agentic Workflow Chain](DIATAXIS_V5_HOW_TO_GUIDES.md#how-to-build-an-agentic-workflow-chain)

**🏢 I'm Auditing/Compliance**
1. Read: [Explanations - Why Execution Receipts Are Critical](DIATAXIS_V5_EXPLANATIONS.md#why-execution-receipts-proofs-are-critical)
2. How-To: [Parse and Verify Execution Receipts](DIATAXIS_V5_HOW_TO_GUIDES.md#how-to-parse-and-verify-execution-receipts)
3. Reference: [Receipt Structure](DIATAXIS_V5_REFERENCE.md#receipt-structure)

**🔬 I'm Researching/Designing Systems**
1. Deep Dive: [Core Philosophy - v4 vs v5](DIATAXIS_V5_EXPLANATIONS.md#core-philosophy-v4-vs-v5)
2. Essay: [Architecture - How It All Fits Together](DIATAXIS_V5_EXPLANATIONS.md#architecture-how-it-all-fits-together)
3. Essay: [Why This Matters For The Future](DIATAXIS_V5_EXPLANATIONS.md#why-this-matters-for-the-future)

---

## The Diataxis Framework

### What is Diataxis?

Diataxis is a four-pillar documentation framework that ensures comprehensive coverage:

| Pillar | Purpose | When to Use | Characteristics |
|--------|---------|------------|-----------------|
| **Tutorials** | Learning-oriented | "I want to learn" | Step-by-step, hands-on, concrete goals |
| **How-To Guides** | Task-oriented | "I want to solve a problem" | Problem→Solution format, practical |
| **Reference** | Information-oriented | "I want to find details" | Organized, searchable, complete API |
| **Explanations** | Understanding-oriented | "I want to understand why" | Conceptual, deep, philosophical |

### Our Implementation

**📚 Tutorials** (`DIATAXIS_V5_TUTORIALS.md`)
- Tutorial 1: Your First v5 Machine API Call
- Tutorial 2: Building an Agent That Uses V5
- Tutorial 3: Implementing Formal Preconditions (Guards)
- Tutorial 4: Agent Delegation with Proofs
- Tutorial 5: Building an MCP Server for V5

**📖 How-To Guides** (`DIATAXIS_V5_HOW_TO_GUIDES.md`)
- How To: Query Available Commands via Introspection
- How To: Validate Inputs Against Schema Before Calling
- How To: Call Command and Process Machine-Readable Response
- How To: Build Agent Respecting Preconditions
- How To: Parse and Verify Execution Receipts
- How To: Implement Delegation Between Agents
- How To: Monitor Long-Running Operations via Streaming
- How To: Build Agentic Workflow Chain

**🔍 Reference** (`DIATAXIS_V5_REFERENCE.md`)
- Command Invocation Format
- Introspection API Response Format
- Command Response Format (Success & Error)
- Input Schema Reference
- Receipt Structure
- Delegation Certificate Structure
- Streaming Response Format
- Effect Model Reference
- MCP Protocol Integration
- OpenAPI Export Format
- SPARQL Ontology Format

**💡 Explanations** (`DIATAXIS_V5_EXPLANATIONS.md`)
- Introduction: Why Machine CLIs Are Different
- Core Philosophy: v4 vs v5
- Why Introspection Instead of Help Text
- Why Guards (Preconditions) Are Essential
- Why Effects Model Matters
- Why Execution Receipts Are Critical
- Why Delegation With Proof Matters
- Why Streaming For Long Operations
- Why MCP Protocol Integration
- Why Dual-Mode Coexistence
- Why Formal Verification Matters
- Architecture: How It All Fits Together
- Why This Matters For The Future
- Design Philosophy Summarized

---

## Document Statistics

| Document | Lines | Topics | Tutorials | How-Tos | References | Essays |
|----------|-------|--------|-----------|---------|-----------|--------|
| Tutorials | 850 | 5 | 5 | - | - | - |
| How-To | 1,100 | 8 | - | 8 | - | - |
| Reference | 950 | 12 | - | - | 12 | - |
| Explanations | 1,200 | 13 | - | - | - | 13 |
| **TOTAL** | **4,100** | **38** | **5** | **8** | **12** | **13** |

---

## Key Concepts Map

### Core v5 Concepts

```
Introspection
  ↓
  ├─ Discover what I can do
  ├─ Get formal schemas
  └─ Understand side effects (effects)

Guards (Preconditions)
  ↓
  ├─ Check what MUST be true
  ├─ Verify before executing
  └─ Prevent invalid states

Input Schema
  ↓
  ├─ Formally define what you accept
  ├─ Validate machine inputs
  └─ No ambiguity

Execution
  ↓
  ├─ Run with guarantees
  ├─ Know isolation level
  └─ Respect timeouts

Receipt (Proof)
  ↓
  ├─ Cryptographic signature
  ├─ Audit trail
  └─ Accountability

Delegation
  ↓
  ├─ Agent→Agent authorization
  ├─ Signed certificate
  └─ Chain of trust
```

### Learning Progression

```
Level 1: Basic Machine Calling
  └─ Introspect → Call → Get Response
     [Tutorial 1]

Level 2: Agent-Safe Calling
  └─ Introspect → Validate → Check Guards → Call → Verify Receipt
     [Tutorial 2, How-To 1-5]

Level 3: Composable Workflows
  └─ Discover → Validate → Guard → Effect-Aware Dispatch → Audit
     [How-To 6-8, Explanations]

Level 4: Multi-Agent Systems
  └─ Introspect → Plan → Delegate (with proof) → Monitor → Audit
     [Tutorial 4, Explanations]

Level 5: LLM Integration
  └─ MCP Tools → Claude → v5 Commands → Receipts
     [Tutorial 5, Reference]
```

---

## Example Workflows by Use Case

### Use Case 1: Simple Machine Call

```
1. Read: Tutorial 1: Your First v5 Machine API Call
2. Reference: Command Invocation Format
3. Reference: Introspection API Response Format
4. Do: Execute `./myapp --introspect`
5. Do: Execute `./myapp --machine pack list --json '...'`
```

### Use Case 2: Agent with Precondition Safety

```
1. Read: Tutorial 2: Building an Agent That Uses V5
2. Read: Explanation: Why Guards Are Essential
3. Learn: How-To: Validate Inputs Against Schema
4. Learn: How-To: Build Agent Respecting Preconditions
5. Reference: Guard Object Format
6. Implement: Agent with guard checking
```

### Use Case 3: Multi-Agent Delegation

```
1. Read: Explanation: Why Delegation With Proof Matters
2. Learn: Tutorial 4: Agent Delegation with Proofs
3. Learn: How-To: Implement Delegation Between Agents
4. Reference: Delegation Certificate Structure
5. Reference: Receipt Structure for auditing
6. Implement: Delegation system
```

### Use Case 4: LLM Integration (Claude)

```
1. Learn: Tutorial 5: Building MCP Server for V5
2. Reference: MCP Protocol Integration
3. Reference: Tool Registration Format
4. Learn: How-To: Build Agentic Workflow Chain
5. Implement: MCP server exposing v5 commands
6. Integrate: With Claude Code or other LLMs
```

### Use Case 5: Formal Verification System

```
1. Read: Explanation: Why Formal Verification Matters
2. Read: Explanation: Architecture - How It All Fits Together
3. Learn: How-To: Build Agent Respecting Preconditions
4. Reference: Effect Model Reference
5. Reference: Input Schema Reference
6. Design: Formally-verified agent system
```

---

## Cross-References

### By Component

**Introspection**
- Explanation: [Why Introspection Instead of Help Text](DIATAXIS_V5_EXPLANATIONS.md#why-introspection-instead-of-help-text)
- Tutorial: [Your First v5 Machine API Call](DIATAXIS_V5_TUTORIALS.md#tutorial-1-your-first-v5-machine-api-call)
- How-To: [Query Available Commands](DIATAXIS_V5_HOW_TO_GUIDES.md#how-to-query-available-commands-via-introspection)
- Reference: [Introspection API Response Format](DIATAXIS_V5_REFERENCE.md#introspection-api-response-format)

**Guards (Preconditions)**
- Explanation: [Why Guards Are Essential](DIATAXIS_V5_EXPLANATIONS.md#why-guards-preconditions-are-essential)
- Tutorial: [Implementing Formal Preconditions](DIATAXIS_V5_TUTORIALS.md#tutorial-3-implementing-formal-preconditions-guards)
- How-To: [Validate Inputs Against Schema](DIATAXIS_V5_HOW_TO_GUIDES.md#how-to-validate-inputs-against-schema-before-calling)
- How-To: [Build Agent Respecting Preconditions](DIATAXIS_V5_HOW_TO_GUIDES.md#how-to-build-an-agent-that-respects-preconditions)
- Reference: [Guard Object](DIATAXIS_V5_REFERENCE.md#guard-object)

**Effects Model**
- Explanation: [Why Effects Model Matters](DIATAXIS_V5_EXPLANATIONS.md#why-effects-model-matters)
- Tutorial: [Understanding Effect Models](DIATAXIS_V5_TUTORIALS.md#step-3-understanding-effect-models)
- How-To: [Build Agentic Workflow Chain](DIATAXIS_V5_HOW_TO_GUIDES.md#how-to-build-an-agentic-workflow-chain)
- Reference: [Effect Model Reference](DIATAXIS_V5_REFERENCE.md#effect-model-reference)

**Execution Receipts**
- Explanation: [Why Execution Receipts Are Critical](DIATAXIS_V5_EXPLANATIONS.md#why-execution-receipts-proofs-are-critical)
- Tutorial: [Agent Checks Preconditions](DIATAXIS_V5_TUTORIALS.md#step-2-agent-checks-preconditions-before-calling)
- How-To: [Parse and Verify Execution Receipts](DIATAXIS_V5_HOW_TO_GUIDES.md#how-to-parse-and-verify-execution-receipts)
- Reference: [Receipt Structure](DIATAXIS_V5_REFERENCE.md#receipt-structure)

**Delegation**
- Explanation: [Why Delegation With Proof Matters](DIATAXIS_V5_EXPLANATIONS.md#why-delegation-with-proof-matters)
- Tutorial: [Agent Delegation with Proofs](DIATAXIS_V5_TUTORIALS.md#tutorial-4-agent-delegation-with-proofs)
- How-To: [Implement Delegation Between Agents](DIATAXIS_V5_HOW_TO_GUIDES.md#how-to-implement-delegation-between-agents)
- Reference: [Delegation Certificate Structure](DIATAXIS_V5_REFERENCE.md#delegation-certificate-structure)

**Streaming**
- Explanation: [Why Streaming For Long Operations](DIATAXIS_V5_EXPLANATIONS.md#why-streaming-for-long-operations)
- How-To: [Monitor Long-Running Operations via Streaming](DIATAXIS_V5_HOW_TO_GUIDES.md#how-to-monitor-long-running-operations-via-streaming)
- Reference: [Streaming Response Format](DIATAXIS_V5_REFERENCE.md#streaming-response-format)

**MCP Integration**
- Explanation: [Why MCP Protocol Integration](DIATAXIS_V5_EXPLANATIONS.md#why-mcp-protocol-integration)
- Tutorial: [Building MCP Server for V5](DIATAXIS_V5_TUTORIALS.md#tutorial-5-building-an-mcp-server-for-v5)
- Reference: [MCP Protocol Integration](DIATAXIS_V5_REFERENCE.md#mcp-protocol-integration)

---

## Learning by Example

### Example 1: List Packs (Simplest Case)

```bash
# Discover what pack:list does
./myapp --introspect | jq '.capabilities[] | select(.id == "pack:list")'

# Call it
./myapp --machine pack list --json '{}'

# Get structured response with receipt
```

**References**:
- Tutorial 1 (Steps 1-3)
- How-To Query Available Commands
- Reference: Command Invocation, Response Format

### Example 2: Install Pack with Safety Checks

```rust
// 1. Get capability metadata
let caps = get_all_capabilities().await?;

// 2. Find pack:install capability
let install_cap = caps.iter()
    .find(|c| c.id == "pack:install")?;

// 3. Check preconditions
for guard in &install_cap.guards.preconditions {
    if !evaluate_guard(guard, context)? {
        println!("Precondition failed: {}", guard.description);
        return Err("Cannot proceed");
    }
}

// 4. Validate input
let params = json!({"name": "web-api"});
validate_against_schema(&params, &install_cap.input_schema)?;

// 5. Execute
let response = call_v5_command("pack", "install", &params).await?;

// 6. Verify receipt
verify_receipt(&response.receipt).await?;
```

**References**:
- Tutorial 2 (Building Agents)
- How-To: Validate Inputs
- How-To: Build Agent Respecting Preconditions
- How-To: Parse Execution Receipts

### Example 3: Multi-Agent Workflow with Delegation

```rust
// Agent A delegates to Agent B
let cert = create_delegation_certificate(
    agent_a,
    agent_b,
    "pack:install",
    params
)?;

// Agent B executes with proof
execute_with_delegation_cert(cert).await?;

// Audit: Verify delegation chain
audit_delegation_chain(&proof).await?;
```

**References**:
- Tutorial 4: Agent Delegation
- How-To: Implement Delegation
- Reference: Delegation Certificate

---

## Terminology Reference

| Term | Definition | Where Defined |
|------|-----------|----------------|
| **Capability** | An operation the system can perform | Reference: Capability Object |
| **Introspection** | Querying system for capability metadata | Explanation: Why Introspection |
| **Guard** | Precondition that must be true | Explanation: Why Guards Matter |
| **Effect** | What an operation does (mutating, side effects) | Explanation: Why Effects Matter |
| **Receipt** | Cryptographic proof of execution | Explanation: Why Receipts Matter |
| **Delegation** | Agent A authorizing Agent B | Explanation: Why Delegation |
| **Schema** | Formal definition of input/output structure | Reference: Input Schema |
| **Streaming** | Real-time events for long operations | Explanation: Why Streaming |
| **MCP** | Model Context Protocol (LLM interface) | Explanation: Why MCP |
| **v4** | Human-centric CLI (current) | Explanation: Core Philosophy |
| **v5** | Machine-centric CLI (new) | Explanation: Core Philosophy |

---

## FAQ: Which Document Should I Read?

**"I just want to get started"**
→ Read Tutorial 1 (15 minutes)

**"I want to understand the philosophy"**
→ Read Explanations (45 minutes)

**"I need to build something specific"**
→ Find your use case in How-To Guides (30-60 minutes)

**"I need to look up API details"**
→ Use Reference as lookup tool (varies)

**"I'm designing a system"**
→ Read Explanations + Reference (2-3 hours)

**"I want to master v5"**
→ Read all documents in order (4-6 hours)

---

## Version & Maintenance

- **Framework**: Diataxis (industry standard)
- **Last Updated**: 2025-11-20
- **v5 Version**: 5.0.0
- **Status**: Production-Ready
- **Completeness**: 100% (all four pillars implemented)

---

## Contributing & Feedback

These documents are living references. If you find:
- **Gaps**: Something not covered
- **Errors**: Incorrect information
- **Unclear**: Confusing explanations
- **Missing Examples**: Need more concrete examples

Please file an issue with:
- Which document (Tutorials/How-To/Reference/Explanations)
- What's missing/wrong
- Suggested fix

---

## Quick Start Checklist

- [ ] Read this index to understand structure
- [ ] Choose your use case from "By Your Role"
- [ ] Follow the suggested reading order
- [ ] Reference documents as needed
- [ ] Implement your use case
- [ ] Refer back to examples

---

## Document Map (Visual)

```
START
  ↓
What's your role?
  ├─→ New to v5?
  │     └─→ [Explanations] (Why) → [Tutorials] (How) → [How-To] (Do)
  │
  ├─→ Building Agents?
  │     └─→ [How-To] (Do) → [Reference] (Details)
  │
  ├─→ Multi-Agent Systems?
  │     └─→ [Explanations] (Why) → [Tutorials] (How) → [How-To] (Do)
  │
  ├─→ LLM Integration?
  │     └─→ [Tutorials] (How) → [How-To] (Do) → [Reference] (Details)
  │
  └─→ Building Systems?
        └─→ [Explanations] (Why) → [Reference] (What)

MASTER v5
```

---

## Next Steps

1. **Choose your starting point** based on your role
2. **Read the recommended documents** in order
3. **Work through examples** as you go
4. **Implement your use case**
5. **Reference API details** as needed
6. **Build something awesome** 🚀

---

