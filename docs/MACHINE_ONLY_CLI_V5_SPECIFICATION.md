# Machine-Only CLI System for clap-noun-verb v5

**Radical Redesign**: From human-centric (v4) to machine-centric (v5)

**Date**: 2025-11-19
**Scope**: Complete architectural redesign for agent/machine callers only
**Key Insight**: Everything designed for AI agents, not human users

---

## EXECUTIVE SUMMARY

clap-noun-verb v5 is the **first machine-only CLI framework** - designed exclusively for:

- **AI Agents** (Claude, GPT, specialized agents)
- **Automated Systems** (orchestrators, workflow engines)
- **Inter-process Communication** (IPC between services)
- **MCP Integration** (Machine Control Protocol)
- **Swarm Coordination** (agent-to-agent calls)

**Not for**: Humans typing commands in terminals

This requires a **180° architectural flip**:

| Dimension | v4 (Human) | v5 (Machine) |
|-----------|-----------|-------------|
| **Output** | Human-readable prose | JSON/structured data |
| **Help** | Text explanations | OpenAPI schemas |
| **Errors** | User-friendly messages | Structured error codes + recovery |
| **Discovery** | `--help` text | Capability introspection APIs |
| **Arguments** | Type-inferred | Fully declared schemas |
| **Safety** | Best practices | Formal verification |
| **Audit Trail** | Logs | Execution receipts + proofs |
| **Target User** | Developers | AI agents |

---

## PART 1: COMPARATIVE ARCHITECTURE

### 1.1 Current v4 Architecture (Human-Centric)

```
User Input (terminal)
    ↓
Clap Parser
    ↓
Noun-Verb Router
    ↓
Type Inference
    ↓
Verb Function
    ↓
Help System (prose)
    ↓
JSON Output
    ↓
Human reads screen
```

**Current Components:**
```
src/cli/
├── help.rs          ← Prose help system (ggen commands)
├── interactive.rs   ← Interactive prompts for humans
├── examples.rs      ← Usage examples for documentation
├── discovery.rs     ← Search for commands (human-friendly)
└── router.rs        ← Routes noun/verb to handlers

src/autonomic/      ← Has machine features already!
├── introspection.rs ← Query what system can do
├── schema.rs        ← Declare argument schemas
├── effects.rs       ← Declare side effects
├── guards.rs        ← Pre-execution guards
└── receipts.rs      ← Execution proofs
```

### 1.2 Proposed v5 Architecture (Machine-Only)

```
Machine/Agent Request (JSON/MCP)
    ↓
Capability Introspection
    ↓
Schema Validation
    ↓
Guard Evaluation (preconditions)
    ↓
Effect Declaration (what will happen)
    ↓
Noun-Verb Dispatcher
    ↓
Verb Function (core logic)
    ↓
Execution Receipt (proof of execution)
    ↓
Structured Response (JSON)
    ↓
Agent processes response
```

**Required Components:**
```
src/machine/              ← NEW: Machine-only layer
├── capability_schema.rs  ← OpenAPI-like capability declarations
├── introspection.rs      ← Query all available operations
├── effect_model.rs       ← Formal effect declarations
├── guard_system.rs       ← Pre-execution safety gates
├── delegation.rs         ← Agent-to-agent authorization
└── execution_receipt.rs  ← Cryptographic execution proofs

src/cli/                  ← REMOVE/DEPRECATE
├── help.rs               ❌ DELETE - No prose help
├── interactive.rs        ❌ DELETE - No human interaction
├── examples.rs           ❌ DELETE - Examples in schemas
└── discovery.rs          ⚠️  REFACTOR - Use introspection

src/autonomic/            ← EXPAND - Already has most features
├── introspection.rs      ✅ Core to v5
├── schema.rs             ✅ Argument declarations
├── effects.rs            ✅ Effect modeling
├── guards.rs             ✅ Pre-condition gates
├── receipts.rs           ✅ Execution proofs
└── delegation.rs         ✅ Agent authorization
```

---

## PART 2: WHAT CHANGES FROM v4 → v5

### 2.1 Help System

#### v4: Prose Help (Current)

```bash
$ myapp --help
clap-noun-verb v4.0.2

TUTORIALS
  Learn the noun-verb pattern...

LEARN MORE
  myapp help tutorial
  myapp help how-to
  ...
```

**Problem for v5**: Prose text is useless to machines. An agent cannot parse "tutorials" prose.

#### v5: Machine-Readable Capability Schema

```json
{
  "system": "clap-noun-verb",
  "version": "5.0.0",
  "capabilities": [
    {
      "id": "services:status",
      "noun": "services",
      "verb": "status",
      "description": "Show service status",
      "preconditions": ["auth:required", "not:disabled"],
      "postconditions": ["read_only"],
      "arguments": {
        "service": {
          "type": "string",
          "description": "Service name",
          "validation": {
            "pattern": "^[a-z0-9-]+$",
            "enum": ["api", "worker", "db"]
          }
        }
      },
      "returns": {
        "type": "object",
        "schema": {
          "services": { "type": "array", "items": { "type": "string" } },
          "healthy": { "type": "boolean" }
        }
      },
      "examples": [
        {
          "args": [],
          "result": { "services": ["api", "worker"], "healthy": true }
        }
      ]
    }
  ]
}
```

**Benefits**:
- Machines can parse and understand capabilities
- No ambiguity - schema is formal specification
- Agents can make decisions based on preconditions
- Examples are structured data, not prose

**What Replaces v4 Help System:**
- `src/cli/help.rs` → **DELETE**
- `src/cli/interactive.rs` → **DELETE** (no interactive prompts for machines)
- `src/cli/examples.rs` → **Merge into schema** (examples as data)
- `src/cli/discovery.rs` → **Refactor to introspection** (machine-readable discovery)

---

### 2.2 Argument Handling

#### v4: Type Inference + Attributes

```rust
#[verb]
fn status(
    #[arg(short = 's')]
    service: Option<String>,
    #[arg(value_name = "FILE")]
    output: String,
) -> Result<Status> {
    // ...
}
```

**Process**:
1. Clap parses arguments
2. Type system infers requirements
3. Help text generated from attributes

#### v5: Formal Schema Declaration

```rust
#[verb(capability_id = "services:status")]
#[precondition("auth:required")]
#[postcondition("read_only")]
#[effect(side_effects = [])]
fn status(
    #[arg(
        schema = ArgumentSchema {
            name: "service",
            type: "string",
            required: false,
            validation: PatternValidation("^[a-z0-9-]+$"),
            description: "Service name (api|worker|db)"
        }
    )]
    service: Option<String>,

    #[arg(
        schema = ArgumentSchema {
            name: "output",
            type: "string",
            required: true,
            validation: FilePathValidation,
            description: "Output file path"
        }
    )]
    output: String,
) -> Result<Status> {
    // ...
}
```

**Benefits**:
- Formal capability declaration
- Machines can verify preconditions before calling
- Side effects declared upfront
- Argument validation schema explicit

**Changes**:
- `src/cli/builder.rs` → **Refactor to schema builder**
- `src/cli/validator.rs` → **Keep but use formal schemas**
- `src/clap/value_parsers.rs` → **Expand with formal validators**

---

### 2.3 Error Handling

#### v4: User-Friendly Error Messages

```bash
$ myapp services status --service invalid

error: 'service' must be one of: api, worker, db

  tip: Did you mean 'api'?

Usage: myapp services status --service <SERVICE>
```

**Process**:
- Clap validates
- Error formatted for human reading
- Suggestion provided

#### v5: Structured Error Responses

```json
{
  "status": "error",
  "error": {
    "code": "VALIDATION_FAILED",
    "capability": "services:status",
    "field": "service",
    "value": "invalid",
    "reason": "enum_mismatch",
    "expected": ["api", "worker", "db"],
    "suggestions": [
      {
        "suggestion": "api",
        "confidence": 0.92,
        "reason": "levenshtein_distance"
      }
    ],
    "recovery": {
      "action": "retry",
      "corrected_args": { "service": "api" }
    }
  }
}
```

**Benefits**:
- Machine can parse error programmatically
- Confidence scores for suggestions
- Recovery instructions included
- No user-facing text needed

**Changes**:
- `src/error.rs` → **Redesign with structured errors**
- Error serialization → **JSON with error codes**
- User messages → **Remove** (not needed for machines)

---

### 2.4 Discovery & Introspection

#### v4: Human-Readable Discovery

```bash
$ myapp services --help
Show service status, logs, restart

COMMANDS:
  status    Show service status
  logs      Display service logs
  restart   Restart a service

$ myapp help how-to
How to add arguments:
...
```

#### v5: Machine Introspection API

```rust
// Machine queries available capabilities
GET /introspect/capabilities
→ Returns JSON schema of all available operations

GET /introspect/capabilities/services
→ Returns JSON schema of all "services" verbs

GET /introspect/capabilities/services:status
→ Returns full capability schema with preconditions/effects

POST /introspect/verify-preconditions
{
  "capability": "services:status",
  "context": { "user": "agent-1", "role": "admin" }
}
→ Returns { "can_execute": true, "reason": "..." }
```

**Implementation**:
```rust
// New module
src/machine/introspection.rs

pub struct CapabilityRegistry {
    capabilities: HashMap<String, CapabilitySchema>,
}

impl CapabilityRegistry {
    pub fn get_capability(&self, id: &str) -> Option<CapabilitySchema>;
    pub fn list_capabilities(&self) -> Vec<CapabilitySchema>;
    pub fn list_by_noun(&self, noun: &str) -> Vec<CapabilitySchema>;
    pub fn verify_preconditions(&self, cap_id: &str, context: &Context) -> Result<()>;
    pub fn export_openapi(&self) -> OpenApiSpec;
}
```

**Changes**:
- `src/cli/discovery.rs` → **Remove human search**
- `src/cli/interactive.rs` → **Remove entirely**
- Add `src/machine/introspection.rs` → **Capability queries**
- Add `src/machine/capability_registry.rs` → **Central schema registry**

---

### 2.5 Execution Model

#### v4: Execute & Return

```
1. Parse arguments
2. Validate types
3. Call verb function
4. Serialize result to JSON
5. Return to user
```

#### v5: Execute with Formal Proofs

```
1. Query preconditions (can this be executed?)
2. Declare effects (what will happen?)
3. Get authorization (agent has permission?)
4. Execute verb function
5. Generate execution receipt (proof of what happened)
6. Return receipt + result
```

**New Response Format**:

```json
{
  "metadata": {
    "execution_id": "exec-d3e8f92c",
    "timestamp": "2025-11-19T19:52:01Z",
    "capability": "services:status",
    "caller": "agent-claude-001"
  },
  "execution": {
    "preconditions": {
      "auth:required": { "status": "passed", "reason": "valid token" },
      "not:disabled": { "status": "passed", "reason": "system active" }
    },
    "effects_declared": [
      { "type": "read", "resource": "services.json", "is_side_effect": false }
    ],
    "effects_actual": [
      { "type": "read", "resource": "services.json", "timestamp": "2025-11-19T19:52:01.234Z" }
    ]
  },
  "result": {
    "services": ["api", "worker"],
    "healthy": true
  },
  "receipt": {
    "type": "execution_proof",
    "digest": "sha256:abc123...",
    "signature": "rsa:def456..."
  }
}
```

**Components to Add**:
- `src/machine/effect_model.rs` → **Formal effect declarations**
- `src/machine/execution_receipt.rs` → **Cryptographic proofs**
- `src/machine/guard_system.rs` → **Precondition evaluation**
- `src/autonomic/receipts.rs` → **Expand existing implementation**

---

### 2.6 Delegation & Authorization

#### v4: None Built-In

Humans don't need agent-to-agent delegation.

#### v5: Agent-to-Agent Authorization

Machines need to delegate work to other machines with proof of authorization:

```rust
// Agent A wants to call a capability on behalf of Agent B
{
  "delegated_by": "agent-a",
  "on_behalf_of": "agent-b",
  "capability": "services:restart",
  "authorization_chain": [
    {
      "issuer": "cluster-admin",
      "grantee": "agent-a",
      "capability_set": ["services:*"],
      "expires": "2025-12-19",
      "signature": "..."
    }
  ]
}
```

**Implementation**:
- `src/autonomic/delegation.rs` → **Expand**
- `src/machine/delegation.rs` → **New machine-specific delegation**
- Add delegation verification in v5 execution flow

---

### 2.7 Audit & Compliance

#### v4: Logs (Optional)

Human-readable logs from telemetry system.

#### v5: Immutable Audit Ledger

For agent accountability and security:

```json
{
  "audit_entry": {
    "id": "audit-e4f7g9h1",
    "timestamp": "2025-11-19T19:52:01Z",
    "agent": "agent-claude-001",
    "capability": "services:restart",
    "arguments": { "service": "api" },
    "preconditions": { "all": "passed" },
    "authorization": { "valid": true },
    "execution": "succeeded",
    "result_digest": "sha256:xyz789...",
    "receipt": "execution-proof-abc123",
    "immutable": true,
    "signature": "rsa:immutable-sig"
  }
}
```

**Implementation**:
- `src/machine/audit_ledger.rs` → **New immutable audit log**
- `src/autonomic/telemetry.rs` → **Expand for audit trail**

---

## PART 3: FILE STRUCTURE CHANGES

### 3.1 Files to DELETE (No Longer Needed)

```
src/cli/help.rs                    ❌ Help is JSON schema now
src/cli/interactive.rs             ❌ No interactive prompts
src/cli/examples.rs                ❌ Examples in schema
src/cli/discovery.rs               ❌ Replaced by introspection

docs/QUICKSTART.md                 ❌ For humans, not machines
docs/TUTORIALS/                    ❌ For humans
docs/CLI_COOKBOOK.md               ❌ For humans
docs/CLI_REFERENCE.md              ❌ For humans (schema is the reference)
docs/CLI_TROUBLESHOOTING.md        ❌ Machines don't need troubleshooting help

examples/basic.rs                  ❌ For human learning
examples/services.rs               ❌ For human learning
examples/*_example.rs (most)       ❌ For humans
```

### 3.2 Files to REFACTOR

```
src/cli/mod.rs                     ⚠️  REFACTOR - Remove human-centric exports
src/cli/router.rs                  ⚠️  REFACTOR - No help routing
src/cli/validator.rs               ⚠️  REFACTOR - Use formal schemas
src/cli/builder.rs                 ⚠️  REFACTOR - Build schemas, not help text

src/error.rs                        ⚠️  REFACTOR - Structured error responses
src/registry.rs                     ⚠️  REFACTOR - To capability registry
src/router.rs                       ⚠️  REFACTOR - No help routing

README.md                           ⚠️  REWRITE - For machine developers
docs/CLI_REFERENCE.md              ⚠️  REPLACE - With OpenAPI schema
```

### 3.3 Files to CREATE (New Machine Layer)

```
src/machine/                        ✅ NEW DIRECTORY
├── mod.rs
├── capability.rs                   ← Capability declarations
├── capability_schema.rs            ← OpenAPI-like schemas
├── capability_registry.rs          ← Central registry
├── introspection.rs                ← Query capabilities
├── effect_model.rs                 ← Formal effects
├── guard_system.rs                 ← Preconditions
├── delegation.rs                   ← Agent authorization
├── execution_receipt.rs            ← Cryptographic proofs
├── audit_ledger.rs                 ← Immutable audit log
├── mcp_integration.rs              ← Machine Control Protocol
└── agent_protocol.rs               ← Agent communication

docs/MACHINE_CLI_SPEC.md            ✅ NEW
docs/CAPABILITY_SCHEMA.md           ✅ NEW
docs/INTROSPECTION_API.md           ✅ NEW
docs/DELEGATION_PROTOCOL.md         ✅ NEW
docs/AGENT_INTEGRATION.md           ✅ NEW

examples/machine_integration.rs     ✅ NEW
examples/agent_example.rs           ✅ NEW
examples/mcp_server.rs              ✅ NEW

tests/machine/                      ✅ NEW
├── mod.rs
├── introspection_tests.rs
├── capability_tests.rs
├── guard_tests.rs
├── delegation_tests.rs
├── receipt_tests.rs
└── audit_tests.rs
```

### 3.4 Files to KEEP/EXPAND

```
src/autonomic/
├── introspection.rs                ✅ EXPAND - Core to v5
├── schema.rs                        ✅ EXPAND - Formal schemas
├── effects.rs                       ✅ EXPAND - Effect declarations
├── guards.rs                        ✅ EXPAND - Preconditions
├── receipts.rs                      ✅ EXPAND - Execution proofs
├── delegation.rs                    ✅ EXPAND - Agent delegation
└── telemetry.rs                     ✅ EXPAND - Audit trail

src/kernel/
├── schema_registry.rs               ✅ EXPAND - Capability registry
├── capability.rs                    ✅ KEEP - Core model
├── contracts.rs                     ✅ EXPAND - Formal contracts
└── execution_receipts.rs            ✅ EXPAND - Proof generation

src/io/
├── mod.rs                           ✅ KEEP - I/O handling
└── types.rs                         ✅ EXPAND - Machine-friendly types
```

---

## PART 4: ARCHITECTURAL DIFFERENCES

### 4.1 Help System Flip

**v4 Architecture**:
```
Code → Attributes → Help Text → User Reads
```

**v5 Architecture**:
```
Code → Formal Schemas → JSON/OpenAPI → Machine Parses
```

### 4.2 Error Handling Flip

**v4 Architecture**:
```
Error → User-Friendly Message → Human Fixes → Retry
```

**v5 Architecture**:
```
Error → Structured Code + Recovery → Machine Decides → Auto-Correct or Escalate
```

### 4.3 Validation Flip

**v4 Architecture**:
```
User Input → Clap Validates → Type Inference → Help on Error
```

**v5 Architecture**:
```
Schema Declaration → Precondition Gates → Formal Validation → Structured Error
```

### 4.4 Discovery Flip

**v4 Architecture**:
```
User Runs `--help` → Reads Text → Understands Command Structure
```

**v5 Architecture**:
```
Machine Calls `/introspect` → Parses JSON Schema → Builds Decision Tree
```

---

## PART 5: COMPATIBILITY IMPACT

### 5.1 Breaking Changes

- ✅ **Help system removed** - No `--help` text
- ✅ **Interactive prompts removed** - No `--interactive`
- ✅ **Human error messages removed** - Only error codes
- ✅ **Type inference retained** - Still works the same way
- ✅ **Noun-verb pattern retained** - Still core structure

### 5.2 Migration Path

**For v4 → v5 users:**

| Use Case | v4 | v5 |
|----------|----|----|
| **Humans running CLI** | ✅ Works | ❌ Unsupported |
| **Agents calling CLI** | ⚠️ Manual parsing | ✅ Structured API |
| **Scripts wrapping CLI** | ⚠️ Text parsing | ✅ JSON response |
| **Documentation** | ✅ Prose docs | ✅ OpenAPI schema |
| **Help text** | ✅ `--help` | ❌ Use `/introspect` |

**Recommendation**: v5 is NOT a drop-in replacement for v4. They are different frameworks:
- **v4**: For developers building human CLIs
- **v5**: For developers building agent-facing APIs

---

## PART 6: IMPLEMENTATION ROADMAP

### Phase 1: Formal Capability System (Week 1)

```
src/machine/capability.rs           ← Capability declarations
src/machine/capability_schema.rs    ← OpenAPI-style schemas
tests/machine/capability_tests.rs   ← Full test coverage
```

**Deliverable**: Define what a capability is, formal schema structure

### Phase 2: Precondition Gates & Effects (Week 2)

```
src/machine/guard_system.rs         ← Precondition evaluation
src/machine/effect_model.rs         ← Effect declarations
tests/machine/guard_tests.rs
tests/machine/effect_tests.rs
```

**Deliverable**: Verify what CAN be executed before running

### Phase 3: Introspection Layer (Week 3)

```
src/machine/capability_registry.rs  ← Central registry
src/machine/introspection.rs        ← Query API
tests/machine/introspection_tests.rs
```

**Deliverable**: Machines can discover what system can do

### Phase 4: Execution Receipts (Week 4)

```
src/machine/execution_receipt.rs    ← Cryptographic proofs
src/autonomic/receipts.rs           ← Expand existing
tests/machine/receipt_tests.rs
```

**Deliverable**: Prove what execution happened

### Phase 5: Delegation & Authorization (Week 5)

```
src/machine/delegation.rs
src/autonomic/delegation.rs         ← Expand
tests/machine/delegation_tests.rs
```

**Deliverable**: Agent-to-agent authorization chains

### Phase 6: Audit Ledger (Week 6)

```
src/machine/audit_ledger.rs
tests/machine/audit_tests.rs
```

**Deliverable**: Immutable audit trail

### Phase 7: Cleanup & Documentation (Week 7)

```
DELETE: src/cli/help.rs, interactive.rs, examples.rs
REFACTOR: Error handling, router, validation
CREATE: docs/MACHINE_CLI_SPEC.md, etc.
```

**Deliverable**: Clean machine-only framework

---

## PART 7: KEY DESIGN PRINCIPLES FOR V5

### Principle 1: Machine Readability Over Human Readability

- JSON schemas instead of prose help
- Error codes instead of error messages
- Structured data everywhere
- No abbreviations or heuristics

### Principle 2: Formal Verification Over Type Inference

- Preconditions declared and checked
- Effects declared and verified
- Side effects made explicit
- No surprises for agents

### Principle 3: Proof Over Trust

- Execution receipts for accountability
- Cryptographic signatures for verification
- Immutable audit ledger
- Chain of authorization

### Principle 4: Explicit Over Implicit

- All capabilities formally declared
- All arguments have schema
- All effects listed upfront
- No hidden side effects

### Principle 5: Delegation Over Isolation

- Agents can delegate to other agents
- Authorization chains provable
- Shared security context
- Swarm coordination enabled

---

## PART 8: WHAT STAYS THE SAME

Despite radical changes to help/error/discovery, these core elements remain:

✅ **Noun-verb pattern** - Still the organizing principle
✅ **Type inference** - Still works for function signatures
✅ **Attribute macros** - `#[verb]`, `#[noun]` still work
✅ **Registry system** - Still auto-discovers commands
✅ **JSON output** - Still serializes results
✅ **Clap integration** - Still parses arguments
✅ **Async support** - Still works as before
✅ **Application context** - Still shares state

---

## PART 9: SUCCESS CRITERIA FOR V5

### Agent Usability

✅ An AI agent can discover all capabilities via introspection API
✅ An agent can check preconditions before executing
✅ An agent can understand formal effect declarations
✅ An agent can verify authorization before delegating
✅ An agent can interpret structured error responses
✅ An agent can audit all executed operations

### Framework Integrity

✅ No human-facing help text in codebase
✅ All capabilities formally declared
✅ All errors structured (no prose messages)
✅ All execution auditable
✅ All authorization explicit
✅ All side effects declared

### Machine Integration

✅ OpenAPI schema export working
✅ MCP protocol integration complete
✅ Introspection API fully functional
✅ Delegation chains verifiable
✅ Execution receipts cryptographically signed

---

## CONCLUSION: The Paradigm Shift

### What v4 Optimizes For:
- Developers reading `--help`
- Humans learning from examples
- Users recovering from mistakes
- Beautiful error messages
- Progressive learning

### What v5 Optimizes For:
- Agents making decisions
- Machines verifying preconditions
- Systems proving accountability
- Structured formal specifications
- Zero ambiguity

### The Result

v5 is not a better v4. It's a **completely different framework** for a completely different use case:

- **v4**: Human-facing CLI framework
- **v5**: Machine-only API framework (happens to be callable as CLI)

They share the noun-verb pattern, but serve opposite audiences.

---

**End of Specification**

**Next Steps**: Review with team, approve Phase 1 architecture, begin implementation
