# v4 → v5 File Migration: Complete Reference

**Purpose**: Line-by-line breakdown of what changes in the codebase for machine-only transformation

---

## EXECUTIVE REFERENCE

**Total Files**: 300+ (in src/ examples/ tests/ docs/)

**Changes**:
- ❌ **DELETE**: 35 files (human-only components)
- ⚠️  **REFACTOR**: 45 files (redesign for machines)
- ✅ **KEEP**: 180 files (core logic unchanged)
- ✨ **NEW**: 25+ files (machine layer)

---

## PART 1: DELETION MATRIX (Remove Human-Only Components)

### 1.1 Help System Files - DELETE

| File | Reason | Impact |
|------|--------|--------|
| `src/cli/help.rs` | Prose help useless to machines | HIGH - Remove 450 lines |
| `src/cli/interactive.rs` | Interactive prompts for humans only | MEDIUM - Remove 200 lines |
| `src/cli/examples.rs` | Human learning examples | LOW - Merge into schema |
| `src/cli/discovery.rs` | Human search feature | MEDIUM - Replace with introspection |

### 1.2 Documentation Files - DELETE

| File | Reason | Replacement |
|------|--------|-------------|
| `docs/QUICKSTART.md` | For humans learning CLI | `docs/MACHINE_CLI_SPEC.md` |
| `docs/TUTORIALS/` | Directory of tutorial docs | API documentation only |
| `docs/CLI_REFERENCE.md` | Human command reference | OpenAPI schema export |
| `docs/CLI_COOKBOOK.md` | "How to do X" for humans | Agent integration guide |
| `docs/CLI_TROUBLESHOOTING.md` | Troubleshooting guide | Error code reference |
| `docs/HELP_SYSTEM_REDESIGN.md` | Help system design (for v4) | `docs/CAPABILITY_SCHEMA.md` |

**Files to delete from docs/**:
```
docs/QUICKSTART.md
docs/CLI_REFERENCE.md
docs/CLI_COOKBOOK.md
docs/CLI_TROUBLESHOOTING.md
docs/HELP_SYSTEM_REDESIGN.md
docs/COMMON_MISTAKES.md
docs/ERROR_MESSAGE_IMPROVEMENTS.md
docs/CLAP_TYPER_ANALYSIS_FOR_V5.md  ← Not applicable to machine-only system
```

### 1.3 Example Files - DELETE

**Reason**: Examples are for human learning. Machines query OpenAPI schema.

| File | Delete? | Alternative |
|------|---------|-------------|
| `examples/basic.rs` | ✅ DELETE | Use `examples/machine_integration.rs` |
| `examples/services.rs` | ✅ DELETE | Use agent example |
| `examples/attribute_macro.rs` | ✅ DELETE | Schema-driven development |
| `examples/async_example.rs` | ⚠️ KEEP | Shows async patterns still valid |
| `examples/context_example.rs` | ⚠️ KEEP | App context still used |
| `examples/format_example.rs` | ✅ DELETE | Output format is always JSON in v5 |
| `examples/interactive.rs` | ✅ DELETE | No interactivity |
| `examples/ggen/` | ✅ DELETE | Specific to ggen project |
| `examples/nested.rs` | ✅ DELETE | For human understanding |
| `examples/positional.rs` | ✅ DELETE | For human understanding |

**Examples to KEEP** (still relevant):
```
examples/async_example.rs           ← Async patterns
examples/context_example.rs         ← App context
examples/validation.rs              ← Validation patterns
examples/env_vars.rs                ← Environment handling
```

**New examples to CREATE**:
```
examples/machine_integration.rs      ← Call as machine
examples/agent_example.rs           ← Agent calling system
examples/mcp_server.rs              ← MCP protocol server
examples/capability_discovery.rs    ← Introspection usage
examples/delegation_example.rs      ← Agent delegation
examples/audit_ledger_example.rs    ← Audit trail
```

---

## PART 2: REFACTORING MATRIX (Redesign for Machines)

### 2.1 CLI Layer - REFACTOR

#### `src/cli/mod.rs`

**Current** (v4):
```rust
pub use help::{CommandCategory, CommandInfo, HelpSystem};
pub use examples::{Example, ExamplesRegistry};
pub use interactive::{InteractiveHelp, InteractiveOutput};
pub use discovery::{CommandDiscovery, SearchResult};

pub fn run() -> Result<()> {
    // Auto-run CLI with help system
}
```

**Refactored** (v5):
```rust
// REMOVE: help, examples, interactive, discovery exports
// KEEP: Registry, Router, Validator, Builder

pub use registry::CommandRegistry;
pub use router::CommandRouter;
pub use validator::ArgValidator;
pub use builder::CliBuilder;

// NEW: Machine layer exports
pub use crate::machine::{
    CapabilityRegistry,
    Introspection,
    ExecutionReceipt
};

pub fn run() -> Result<()> {
    // Parse args, execute, return JSON/receipt
    // NO help system
}
```

**Changes**:
- Remove 4 pub use statements
- Remove help routing
- Add machine layer exports
- Keep core routing logic

#### `src/cli/router.rs`

**Current** (v4):
```rust
match args.len() {
    0 => show_help(),           // ← REMOVE
    1 if args[0] == "--help" => help_system.main_help(),  // ← REMOVE
    1 if args[0] == "help" => interactive_help(),  // ← REMOVE
    _ => execute_noun_verb()
}
```

**Refactored** (v5):
```rust
match args.len() {
    0 => return_introspection_endpoint(),  // ← NEW: Return capability schema
    1 if args[0] == "--version" => return_version(),
    _ => execute_noun_verb()  // ← Simplified
}
```

**Changes**:
- Remove all `--help` handling
- Remove interactive help routing
- Add introspection endpoint
- Simplify error messages

#### `src/cli/builder.rs`

**Current** (v4):
```rust
pub struct CliBuilder {
    help_text: String,
    examples: Vec<String>,
    categories: Vec<CommandCategory>,
    // ...
}

impl CliBuilder {
    pub fn with_help(&mut self, help: String);
    pub fn with_examples(&mut self, examples: Vec<String>);
    pub fn build_help_output(&self) -> HelpOutput;
}
```

**Refactored** (v5):
```rust
pub struct CliBuilder {
    schema: CapabilitySchema,      // ← NEW
    preconditions: Vec<Guard>,     // ← NEW
    postconditions: Vec<Effect>,   // ← NEW
    // Remove: help_text, examples, categories
}

impl CliBuilder {
    pub fn with_schema(&mut self, schema: CapabilitySchema);
    pub fn with_guards(&mut self, guards: Vec<Guard>);
    pub fn with_effects(&mut self, effects: Vec<Effect>);
    pub fn build_capability(&self) -> Capability;  // ← NEW
}
```

**Changes**:
- Replace help/examples with schema/guards/effects
- Add formal capability building
- Remove human-oriented methods

#### `src/cli/validator.rs`

**Current** (v4):
```rust
pub fn validate_argument(&self, arg: &str) -> Result<String> {
    // Type-based validation
    // Returns user-friendly error on failure
}
```

**Refactored** (v5):
```rust
pub fn validate_argument(&self, arg: &str, schema: &ArgumentSchema) -> Result<String> {
    // Schema-based validation
    // Returns structured error code
    // Provides recovery suggestions
}

pub fn validate_preconditions(&self, cap_id: &str, context: &Context) -> Result<()> {
    // NEW: Verify capability can be executed
}
```

**Changes**:
- Add schema-based validation
- Add precondition checking
- Return structured errors instead of messages

### 2.2 Error Handling - REFACTOR

#### `src/error.rs`

**Current** (v4) - 200+ lines:
```rust
#[derive(Debug)]
pub enum NounVerbError {
    CommandNotFound(String),
    InvalidArgument { field: String, message: String },
    ValidationFailed(String),
    ExecutionError(String),
}

impl Display for NounVerbError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        // User-friendly error messages
        match self {
            Self::CommandNotFound(cmd) =>
                write!(f, "Command not found: '{}'", cmd),
            Self::InvalidArgument { field, message } =>
                write!(f, "error: {} - {}", field, message),
        }
    }
}
```

**Refactored** (v5) - Structured:
```rust
#[derive(Debug, Serialize)]
pub struct StructuredError {
    pub code: ErrorCode,                    // enum: VALIDATION_FAILED, etc.
    pub capability: String,
    pub field: Option<String>,
    pub value: Option<String>,
    pub expected: Option<Vec<String>>,
    pub reason: ErrorReason,                // enum: enum_mismatch, pattern_mismatch
    pub suggestions: Vec<ErrorSuggestion>,
    pub recovery: Option<RecoveryInfo>,
}

#[derive(Debug, Serialize)]
pub enum ErrorCode {
    VALIDATION_FAILED = 400,
    PRECONDITION_FAILED = 403,
    NOT_FOUND = 404,
    CAPABILITY_DISABLED = 423,
    INTERNAL_ERROR = 500,
}

#[derive(Debug, Serialize)]
pub struct ErrorSuggestion {
    pub suggestion: String,
    pub confidence: f32,
    pub reason: String,
}

#[derive(Debug, Serialize)]
pub struct RecoveryInfo {
    pub action: String,  // "retry", "correct_field", "check_precondition"
    pub corrected_args: Option<Map<String, Value>>,
}
```

**Response Format**:
```json
{
  "status": "error",
  "error": {
    "code": 400,
    "capability": "services:status",
    "field": "service",
    "value": "invalid-svc",
    "expected": ["api", "worker", "db"],
    "reason": "enum_mismatch",
    "suggestions": [
      {"suggestion": "api", "confidence": 0.95, "reason": "levenshtein_distance"},
      {"suggestion": "worker", "confidence": 0.85, "reason": "levenshtein_distance"}
    ],
    "recovery": {
      "action": "retry",
      "corrected_args": {"service": "api"}
    }
  }
}
```

**Changes**:
- Eliminate Display implementation
- Add Serialize derive
- Make all errors serializable
- Add structured suggestion system
- Add recovery recommendations
- Return error codes instead of messages

### 2.3 Registry - REFACTOR

#### `src/registry.rs`

**Current** (v4):
```rust
pub struct CommandRegistry {
    commands: HashMap<String, CommandMeta>,
}

impl CommandRegistry {
    pub fn find(&self, noun: &str, verb: &str) -> Option<&CommandMeta>;
    pub fn list_all(&self) -> Vec<&CommandMeta>;
}
```

**Refactored** (v5):
```rust
pub struct CommandRegistry {
    commands: HashMap<String, CommandMeta>,
    capabilities: HashMap<String, CapabilitySchema>,  // ← NEW
    guards: HashMap<String, Vec<Guard>>,               // ← NEW
}

impl CommandRegistry {
    pub fn find(&self, noun: &str, verb: &str) -> Option<&CommandMeta>;
    pub fn get_capability(&self, cap_id: &str) -> Option<&CapabilitySchema>;  // ← NEW
    pub fn get_preconditions(&self, cap_id: &str) -> Option<Vec<Guard>>;      // ← NEW
    pub fn export_openapi(&self) -> OpenApiSpec;                               // ← NEW
}
```

**Changes**:
- Add capability schema storage
- Add guard/precondition storage
- Add OpenAPI export
- Keep command routing

---

## PART 3: KEEP MATRIX (Unchanged Core Logic)

These files work the same in v4 and v5:

### 3.1 Type System - KEEP AS-IS

```
src/io/types.rs               ← Type definitions still valid
src/clap/value_parsers.rs    ← Parsing logic unchanged
src/validators.rs             ← Validation logic still used
```

### 3.2 Async Runtime - KEEP AS-IS

```
src/async_verb.rs             ← Async patterns unchanged
src/autonomic/async_runtime.rs ← Still needed for machines
```

### 3.3 Context & State - KEEP AS-IS

```
src/context.rs                ← Application context
src/config.rs                 ← Configuration loading
```

### 3.4 Core Macro System - KEEP/EXPAND

```
src/macros.rs                 ← Attribute macros still work
src/noun.rs                    ← #[noun] still valid
src/verb.rs                    ← #[verb] still valid
clap-noun-verb-macros/         ← Expand with schema generation
```

### 3.5 Autonomic System - EXPAND (Already Machine-Like)

```
src/autonomic/introspection.rs  ← Expand for v5
src/autonomic/schema.rs         ← Core to v5
src/autonomic/effects.rs        ← Expand for effects model
src/autonomic/guards.rs         ← Expand for preconditions
src/autonomic/receipts.rs       ← Expand for proofs
src/autonomic/delegation.rs     ← Expand for agent delegation
src/autonomic/telemetry.rs      ← Expand for audit
```

### 3.6 Kernel System - EXPAND (Strong Foundation)

```
src/kernel/capability.rs        ← Core capability model
src/kernel/schema_registry.rs   ← Use for machines
src/kernel/contracts.rs         ← Formal contracts
src/kernel/execution_receipts.rs ← Expand for proofs
```

---

## PART 4: NEW FILES TO CREATE

### 4.1 Machine Layer - NEW DIRECTORY

```
src/machine/
├── mod.rs                       ← Module exports
├── capability.rs                ← Capability declarations
├── capability_schema.rs         ← OpenAPI schema format
├── capability_registry.rs       ← Central schema registry
├── introspection.rs             ← Query APIs
├── effect_model.rs              ← Formal effect declarations
├── guard_system.rs              ← Precondition evaluation
├── delegation.rs                ← Agent-to-agent authorization
├── execution_receipt.rs         ← Cryptographic proofs
├── audit_ledger.rs              ← Immutable audit log
├── mcp_integration.rs           ← Machine Control Protocol
└── agent_protocol.rs            ← Agent communication format
```

**Total new lines**: ~3,000 lines of new machine-specific code

### 4.2 Documentation - NEW FILES

```
docs/
├── MACHINE_CLI_SPEC.md          ← Complete v5 specification
├── CAPABILITY_SCHEMA.md         ← How to declare capabilities
├── INTROSPECTION_API.md         ← Query API documentation
├── EFFECT_MODELING.md           ← Effect declaration guide
├── GUARD_SYSTEM.md              ← Precondition declaration
├── DELEGATION_PROTOCOL.md       ← Agent delegation guide
├── EXECUTION_RECEIPTS.md        ← Receipt verification
├── AUDIT_LEDGER.md              ← Audit trail format
├── AGENT_INTEGRATION.md         ← For agent developers
├── MCP_INTEGRATION.md           ← MCP protocol implementation
└── MIGRATION_V4_TO_V5.md        ← For v4 users
```

### 4.3 Examples - NEW FILES

```
examples/
├── machine_integration.rs       ← Basic machine integration
├── agent_example.rs             ← AI agent example
├── mcp_server.rs                ← MCP protocol server
├── capability_discovery.rs      ← Introspection API usage
├── delegation_example.rs        ← Agent-to-agent delegation
├── audit_ledger_example.rs      ← Audit trail usage
└── formal_verification.rs       ← Guard verification
```

### 4.4 Tests - NEW FILES

```
tests/machine/
├── mod.rs
├── capability_tests.rs          ← Capability schema tests
├── introspection_tests.rs       ← Query API tests
├── guard_tests.rs               ← Precondition tests
├── delegation_tests.rs          ← Authorization tests
├── receipt_tests.rs             ← Receipt verification tests
├── audit_tests.rs               ← Audit ledger tests
├── error_structured_tests.rs    ← Structured error tests
└── integration_tests.rs         ← End-to-end machine tests
```

---

## PART 5: DETAILED FILE CHANGES

### 5.1 Completely Rewritten Files

#### `README.md`

**v4 README** (~500 lines):
```markdown
# clap-noun-verb

A framework for building composable CLI patterns...

## Quick Start
## Key Features
## How to configure arguments
## How to use async operations
...
```

**v5 README** (~200 lines):
```markdown
# clap-noun-verb v5 - Machine-Only Framework

A formal capability system for AI agents and automated systems...

## For AI Agents
- Introspection API for capability discovery
- Formal preconditions and effects
- Execution receipts with cryptographic proofs
- Agent-to-agent delegation

## For Framework Developers
- OpenAPI schema export
- Automatic capability registry
- Guard system for preconditions
- Audit ledger for accountability

## Quick Integration
```

**Key Changes**:
- Remove human-centric language
- Focus on machine integration
- OpenAPI as primary reference
- Agent-first examples

#### `Cargo.toml`

**v4**:
```toml
[features]
default = ["macros", "help", "interactivity"]
help = []
interactivity = []
```

**v5**:
```toml
[features]
default = ["macros", "introspection", "audit"]
introspection = []     ← NEW
audit = []             ← NEW
mcp-integration = []   ← NEW
formal-verification = [] ← NEW
```

---

## PART 6: IMPACT SUMMARY BY DIRECTORY

### `src/` Changes

```
src/
├── cli/                    ⚠️  MAJOR REFACTOR
│   ├── help.rs              ❌ DELETE (450 lines)
│   ├── interactive.rs       ❌ DELETE (200 lines)
│   ├── examples.rs          ❌ DELETE (150 lines)
│   ├── discovery.rs         ❌ DELETE (300 lines)
│   ├── mod.rs               ⚠️  REFACTOR (50 changes)
│   ├── router.rs            ⚠️  REFACTOR (100 changes)
│   ├── builder.rs           ⚠️  REFACTOR (80 changes)
│   └── validator.rs         ⚠️  REFACTOR (60 changes)
│
├── machine/                ✨ NEW (1,500 lines)
│   ├── capability.rs         ✨ NEW (200 lines)
│   ├── capability_schema.rs  ✨ NEW (300 lines)
│   ├── introspection.rs      ✨ NEW (250 lines)
│   ├── guard_system.rs       ✨ NEW (200 lines)
│   ├── effect_model.rs       ✨ NEW (200 lines)
│   ├── delegation.rs         ✨ NEW (250 lines)
│   └── ...
│
├── error.rs                 ⚠️  MAJOR REFACTOR (200 changes)
├── registry.rs              ⚠️  REFACTOR (80 changes)
├── router.rs                ⚠️  REFACTOR (60 changes)
│
├── autonomic/              ✅ EXPAND (existing code enhanced)
├── kernel/                 ✅ EXPAND (existing code enhanced)
├── io/                     ✅ KEEP (mostly unchanged)
└── ...
```

**Line Changes**:
- DELETE: ~1,100 lines
- REFACTOR: ~700 lines affected
- NEW: ~3,500 lines
- Net change: +2,700 lines

### `docs/` Changes

```
docs/
├── ❌ DELETE (8 files, 3,000 lines)
│   ├── QUICKSTART.md
│   ├── CLI_REFERENCE.md
│   ├── CLI_COOKBOOK.md
│   ├── CLI_TROUBLESHOOTING.md
│   ├── TUTORIALS/
│   └── ...
│
└── ✨ NEW (10 files, 4,000 lines)
    ├── MACHINE_CLI_SPEC.md
    ├── CAPABILITY_SCHEMA.md
    ├── INTROSPECTION_API.md
    ├── DELEGATION_PROTOCOL.md
    └── ...
```

### `examples/` Changes

```
examples/
├── ❌ DELETE (12 files)
│   ├── basic.rs
│   ├── services.rs
│   ├── nested.rs
│   ├── interactive.rs
│   ├── ggen/
│   └── ...
│
├── ✅ KEEP (6 files - async, context, validation patterns)
│
└── ✨ NEW (6 files)
    ├── machine_integration.rs
    ├── agent_example.rs
    ├── mcp_server.rs
    └── ...
```

### `tests/` Changes

```
tests/
├── ❌ DELETE (files testing help/interactive)
│   ├── help_system_tests.rs
│   ├── interactive_tests.rs
│   └── ...
│
├── ✅ KEEP (core logic tests, ~200 test files)
│
└── ✨ NEW (tests/machine/ directory with 8 files)
    ├── capability_tests.rs
    ├── introspection_tests.rs
    ├── guard_tests.rs
    └── ...
```

---

## PART 7: MIGRATION CHECKLIST

### Phase 1: Analysis & Planning
- [ ] Review current help.rs (450 lines)
- [ ] Review current error.rs (200 lines)
- [ ] Understand autonomic system existing features
- [ ] Design CapabilitySchema structure
- [ ] Create RFC for machine layer

### Phase 2: Core Machine Layer (Week 1)
- [ ] Create src/machine/ directory
- [ ] Implement CapabilitySchema struct
- [ ] Implement CapabilityRegistry
- [ ] Write capability_tests.rs
- [ ] Verify backwards compatibility

### Phase 3: Refactor Error Handling (Week 2)
- [ ] Create StructuredError type
- [ ] Implement Serialize for errors
- [ ] Update error.rs completely
- [ ] Update router.rs to use structured errors
- [ ] Update tests/

### Phase 4: Preconditions & Effects (Week 3)
- [ ] Implement guard_system.rs
- [ ] Implement effect_model.rs
- [ ] Integrate with router
- [ ] Write guard_tests.rs

### Phase 5: Introspection API (Week 4)
- [ ] Implement introspection.rs
- [ ] Create /introspect endpoint
- [ ] Export OpenAPI schema
- [ ] Write introspection_tests.rs

### Phase 6: Delegation & Authorization (Week 5)
- [ ] Implement delegation.rs
- [ ] Integration with autonomic/delegation.rs
- [ ] Authorization chain verification
- [ ] Write delegation_tests.rs

### Phase 7: Audit & Cleanup (Week 6)
- [ ] Implement audit_ledger.rs
- [ ] Delete help.rs, interactive.rs, etc.
- [ ] Refactor cli/mod.rs, router.rs
- [ ] Update README.md
- [ ] Create MIGRATION guide

### Phase 8: Documentation (Week 7)
- [ ] Create MACHINE_CLI_SPEC.md
- [ ] Create CAPABILITY_SCHEMA.md
- [ ] Create INTROSPECTION_API.md
- [ ] Create DELEGATION_PROTOCOL.md
- [ ] Create examples/

### Phase 9: Testing & Validation (Week 8)
- [ ] Full integration tests
- [ ] Performance benchmarks
- [ ] Security audit
- [ ] Documentation review
- [ ] Release v5.0.0

---

## SUMMARY TABLE

| Category | v4 | v5 | Change |
|----------|----|----|--------|
| **Help system files** | 4 | 0 | -4 |
| **Error handling approach** | Display trait | Serialize struct | Complete redesign |
| **Machine layer** | 0 | 12 | +12 |
| **Documentation files** | 30 | 15 | -15 (replace) |
| **Example files** | 20 | 14 | -6 |
| **Test files** | 40 | 48 | +8 |
| **Core logic unchanged** | 100% | 100% | ✓ |
| **Breaking changes** | - | YES | Help/errors/discovery |

---

**Created**: 2025-11-19
**For**: clap-noun-verb v5 machine-only redesign
**Next**: Approve file migration plan, begin Phase 1
