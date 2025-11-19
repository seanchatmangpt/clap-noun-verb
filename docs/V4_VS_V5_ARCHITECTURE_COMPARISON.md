# v4 vs v5: Architecture Comparison

## Visual Architecture Flows

### v4 Architecture: Human-Centric

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        HUMAN USER (Terminal)                               │
└─────────────────────────┬───────────────────────────────────────────────────┘
                          │
                          ▼ stdin: "myapp services status"
                  ┌───────────────────┐
                  │   Clap Parser     │
                  │                   │
                  │ Parse arguments   │
                  └────────┬──────────┘
                           │
                           ▼
        ┌──────────────────────────────────────┐
        │   Noun-Verb Router                  │
        │                                      │
        │ Route: services → status verb        │
        └────────┬─────────────────────────────┘
                 │
                 ▼
     ┌──────────────────────────────────┐
     │   Type Inference                 │
     │                                  │
     │ Option<String> = optional arg    │
     └────────┬─────────────────────────┘
              │
              ▼
        ┌─────────────────────────────┐
        │   Help System               │
        │                             │
        │ - Prose help text           │
        │ - Examples documentation    │
        │ - Interactive discovery     │
        └─────────────────────────────┘
              │
              ├─► stdout: Help text (if --help)
              │
              └─► Verb Function
                  │
                  │ (if no help requested)
                  │
                  ▼
          ┌─────────────────────────────┐
          │   Verb Function             │
          │   (Business Logic)          │
          │                             │
          │ Services::status()          │
          └────────┬────────────────────┘
                   │
                   ▼
          ┌─────────────────────────────┐
          │   JSON Serialization        │
          │                             │
          │ Serialize Status struct     │
          └────────┬────────────────────┘
                   │
                   ▼
          ┌─────────────────────────────┐
          │   Output Formatter          │
          │                             │
          │ - JSON (default)            │
          │ - YAML, TOML, CSV, etc.     │
          └────────┬────────────────────┘
                   │
                   ▼
    Human reads: {"services": [...], "healthy": true}
```

**Key Path**: User → Help System → Function → JSON → User Reads

**Help System Prominence**: CENTRAL (blocks all other paths)

---

### v5 Architecture: Machine-Only

```
┌──────────────────────────────────────────────────────────────────────────────┐
│                  AGENT/MACHINE (JSON Request)                               │
└──────────────────────────────────┬──────────────────────────────────────────┘
                                   │
                    ▼ {"capability": "services:status", "args": {...}}
                  ┌──────────────────────────┐
                  │  Introspection Query     │
                  │                          │
                  │  Get Capability Schema   │
                  │  Verify preconditions    │
                  └────────┬─────────────────┘
                           │
                           ├─► Return Capability Schema (if introspect request)
                           │
                           └─► Proceed to Validation
                                   │
                                   ▼
              ┌────────────────────────────────────────┐
              │  Guard System (Preconditions)         │
              │                                        │
              │  ✓ auth:required                      │
              │  ✓ not:disabled                       │
              │  ✓ resource:available                 │
              │                                        │
              │  All must pass → proceed             │
              │  Any fail → return structured error   │
              └────────┬───────────────────────────────┘
                       │
                       ▼ (if all guards pass)
              ┌────────────────────────────────────────┐
              │  Effect Model Declaration              │
              │                                        │
              │  Declare what WILL happen:            │
              │  - read(services.json)                │
              │  - read(status.db)                    │
              │  - NO writes (read_only)              │
              └────────┬───────────────────────────────┘
                       │
                       ▼
              ┌────────────────────────────────────────┐
              │  Formal Validation                     │
              │                                        │
              │  Schema-based argument validation      │
              │  (Not type inference)                  │
              │                                        │
              │  If validation fails:                  │
              │  → Structured error with code          │
              │  → Suggestions with confidence         │
              │  → Recovery instructions               │
              └────────┬───────────────────────────────┘
                       │
                       ▼ (if validation passes)
              ┌────────────────────────────────────────┐
              │  Authorization Check                   │
              │                                        │
              │  Verify delegation chain:              │
              │  agent-a → (delegates to) → agent-b   │
              │  with explicit capability grant        │
              └────────┬───────────────────────────────┘
                       │
                       ▼ (if authorized)
              ┌────────────────────────────────────────┐
              │  Noun-Verb Dispatcher                  │
              │                                        │
              │  Route: services → status verb         │
              └────────┬───────────────────────────────┘
                       │
                       ▼
              ┌────────────────────────────────────────┐
              │  Verb Function                         │
              │  (Business Logic)                      │
              │                                        │
              │  Services::status()                    │
              └────────┬───────────────────────────────┘
                       │
                       ▼
              ┌────────────────────────────────────────┐
              │  Execution Receipt Generation          │
              │                                        │
              │  - Digest of execution                │
              │  - Cryptographic signature            │
              │  - Timestamp & machine ID             │
              │  - Audit trail entry                  │
              └────────┬───────────────────────────────┘
                       │
                       ▼
              ┌────────────────────────────────────────┐
              │  JSON Response with Receipt            │
              │                                        │
              │  {                                     │
              │    "metadata": {...},                  │
              │    "execution": {...},                │
              │    "result": {...},                   │
              │    "receipt": {...}                   │
              │  }                                     │
              └────────┬───────────────────────────────┘
                       │
                       ▼
    Agent processes response, verifies receipt, audits
```

**Key Path**: Schema → Guards → Effects → Validation → Auth → Function → Receipt → Machine Processes

**Help System Prominence**: REMOVED (replaced with formal schemas)

---

## Layer-by-Layer Comparison

### Layer 1: Input

| Aspect | v4 | v5 |
|--------|----|----|
| **Format** | Command-line arguments | JSON request |
| **Discovery** | `--help` text | Schema query |
| **Example** | `myapp services status` | `{"capability":"services:status"}` |
| **Parser** | Clap | Clap (unchanged) |

### Layer 2: Pre-Execution

| Aspect | v4 | v5 |
|--------|----|----|
| **Guards** | None | Formal preconditions |
| **Effects** | Implicit | Formally declared |
| **Authorization** | None | Delegation chain verification |
| **Validation** | Type inference | Schema-based validation |

### Layer 3: Execution

| Aspect | v4 | v5 |
|--------|----|----|
| **Routing** | Noun-verb → function | Same (unchanged) |
| **Logging** | Optional telemetry | Mandatory audit |
| **Proof** | No | Execution receipt |
| **Side effects** | Not tracked | Formally verified |

### Layer 4: Output

| Aspect | v4 | v5 |
|--------|----|----|
| **Format** | JSON (serialized) | JSON with metadata |
| **Result** | Just data | Data + receipt + audit |
| **Errors** | User messages | Error codes + recovery |
| **Verification** | None | Cryptographic proof |

---

## Request/Response Comparison

### v4: Simple Round-Trip

```bash
$ myapp services status

# Returns:
{"services": ["api", "worker"], "healthy": true}
```

**Human reviews result on screen**

### v5: Verified Exchange

```json
// Agent REQUEST
{
  "request_id": "req-abc123",
  "capability": "services:status",
  "arguments": {}
}

// v5 RESPONSE
{
  "metadata": {
    "request_id": "req-abc123",
    "execution_id": "exec-def456",
    "timestamp": "2025-11-19T19:52:01Z",
    "capability": "services:status",
    "caller": "agent-claude-001"
  },
  "guards": {
    "auth:required": {"status": "passed"},
    "not:disabled": {"status": "passed"}
  },
  "effects": {
    "declared": ["read(services.json)", "read(status.db)"],
    "actual": ["read(services.json)", "read(status.db)"]
  },
  "result": {
    "services": ["api", "worker"],
    "healthy": true
  },
  "receipt": {
    "type": "execution_proof",
    "digest": "sha256:abc123...",
    "timestamp": "2025-11-19T19:52:01.234Z",
    "signature": "rsa:sig123..."
  },
  "audit_entry": {
    "id": "audit-ghi789",
    "action": "executed",
    "status": "success"
  }
}
```

**Agent verifies receipt, audits action, makes next decision**

---

## Error Flow Comparison

### v4: User-Friendly Error Messages

```bash
$ myapp services status --service invalid

error: 'service' must be one of: api, worker, db

  tip: Did you mean 'api'?

Usage: myapp services status --service <SERVICE>
```

**Human reads message, corrects, retries**

### v5: Structured Error with Recovery

```json
{
  "status": "error",
  "error": {
    "code": 400,
    "error_type": "VALIDATION_FAILED",
    "capability": "services:status",
    "field": "service",
    "value": "invalid",
    "expected": ["api", "worker", "db"],
    "reason": "enum_mismatch",
    "suggestions": [
      {
        "suggestion": "api",
        "confidence": 0.95,
        "reason": "levenshtein_distance"
      }
    ],
    "recovery": {
      "action": "retry",
      "corrected_args": {"service": "api"},
      "command": "myapp services status --service api"
    }
  },
  "metadata": {
    "request_id": "req-xyz",
    "timestamp": "2025-11-19T19:52:01Z"
  }
}
```

**Machine parses error code, reads recovery action, auto-corrects and retries**

---

## Component Dependency Graph

### v4 Components

```
User
  ↓
Clap Parser
  ├→ Type Inference
  ├→ Help System ← CENTRAL TO DESIGN
  │    ├→ Prose Help
  │    ├→ Examples
  │    └→ Interactive Discovery
  └→ Router
      └→ Verb Function
```

**Help system is GATEKEEPER for all user interactions**

### v5 Components

```
Agent Request
  ↓
Introspection API (if requested)
  ├→ Capability Registry
  └→ OpenAPI Export

Agent Request (with args)
  ↓
Clap Parser
  ↓
Guard System (preconditions)
  ├→ Auth checks
  ├→ Resource checks
  └→ State checks
  ↓
Effect Model (declarations)
  ├→ Read effects
  ├→ Write effects
  └→ Side effect verification
  ↓
Formal Validation (schema-based)
  ├→ Type checking
  ├→ Pattern matching
  └→ Constraint verification
  ↓
Authorization Chain
  ├→ Delegation verification
  └→ Capability grant checking
  ↓
Router → Verb Function
  ├→ Execute
  ├→ Generate Receipt
  ├→ Audit Entry
  └→ Return Response
```

**Multiple safety gates BEFORE execution, formal receipt AFTER**

---

## Design Philosophy Contrast

### v4 Philosophy

> Make CLIs that humans love to use

- Intuitive command structure
- Helpful error messages
- Progressive disclosure (short help → long help)
- Learn by reading examples
- Friendly suggestions

**Optimization**: Human comprehension & ease of use

### v5 Philosophy

> Make CLIs that machines trust to execute

- Formal capability declarations
- Structured error codes
- Complete precondition verification
- Learn by reading schemas
- Automated recovery procedures

**Optimization**: Machine verification & accountability

---

## Core Differences Summary

| Dimension | v4 | v5 |
|-----------|----|----|
| **Target User** | Human developers | AI agents & machines |
| **Help System** | Prose text with examples | JSON schemas with formal specs |
| **Error Model** | User-friendly messages | Structured codes + recovery |
| **Validation** | Type inference | Formal schema validation |
| **Execution Model** | Parse → Execute → Return | Verify → Guard → Execute → Receipt |
| **Authorization** | None | Formal delegation chains |
| **Audit Trail** | Optional logging | Mandatory immutable ledger |
| **Side Effects** | Implicit | Formally declared & verified |
| **Preconditions** | Best practices | Formal guards |
| **Verification** | None | Cryptographic receipts |

---

## Compatibility Matrix

### What Developers Can Reuse (v4 → v5)

✅ **Noun-verb pattern** - Still core structure
✅ **Attribute macros** - `#[verb]`, `#[noun]` still work
✅ **Type system** - Still infers from signatures
✅ **Async support** - Still works
✅ **Registry system** - Still auto-discovers
✅ **Core routing** - Still noun→verb dispatch
✅ **Verb function logic** - Completely unchanged

### What Must be Rewritten (v4 → v5)

❌ **Help system code** - Completely removed
❌ **Error handling** - Redesigned as structured
❌ **Validation** - From inference to schema
❌ **Discovery** - From text help to API introspection
❌ **Documentation** - From prose to specifications

---

## Why This Matters for v5

The shift from human-centric to machine-only unlocks:

1. **Formal Verification**: Guards before execution prevent errors
2. **Accountability**: Receipts prove what happened
3. **Automation**: Machines can auto-recover from structured errors
4. **Delegation**: Agents can delegate to other agents with proof
5. **Scale**: Thousands of agent calls can be audited
6. **Trust**: Cryptographic proofs establish confidence

This is not just a redesign—it's a **fundamental category change** from "helpful CLI tool" to "verifiable capability system."

---

**Created**: 2025-11-19
**Purpose**: Understand v4 vs v5 architectural differences
**Next**: Begin Phase 1 implementation of machine layer
