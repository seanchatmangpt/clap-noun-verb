# v4 → v5 File Integration: Complete Reference

**Purpose**: Strategic integration of machine-centric v5 layer alongside human-centric v4

**Approach**: SIDE-BY-SIDE - Keep both paths active in same binary

---

## EXECUTIVE REFERENCE

**Total Files**: 300+ (in src/ examples/ tests/ docs/)

**Changes**:
- ✅ **KEEP**: 35+ files (human-only components - v4 path)
- ✅ **KEEP**: 45+ files (core logic unchanged)
- ✅ **KEEP**: 180+ files (existing infrastructure)
- ✨ **EXPAND**: 27 files in `src/autonomic/` (v5 machine layer - ALREADY IMPLEMENTED)
- ✨ **NEW**: 2-3 files (v5 dispatcher/router to call autonomic layer)

---

## PART 1: SIDE-BY-SIDE ARCHITECTURE (Keep Everything, Add v5 Path)

### 1.1 Human Path (v4) - KEEP EVERYTHING

| File | Purpose | Status |
|------|---------|--------|
| `src/cli/help.rs` | Prose help system for humans | ✅ KEEP - v4 path |
| `src/cli/interactive.rs` | Interactive prompts for humans | ✅ KEEP - v4 path |
| `src/cli/examples.rs` | Human learning examples | ✅ KEEP - v4 path |
| `src/cli/discovery.rs` | Human command search | ✅ KEEP - v4 path |

**Behavior**: These files are used when humans call the CLI directly. No changes needed.

### 1.2 Machine Path (v5) - ALREADY IMPLEMENTED IN src/autonomic/

| Component | Files | Status |
|-----------|-------|--------|
| Introspection | `src/autonomic/introspection.rs` | ✅ READY |
| Schema | `src/autonomic/schema.rs` | ✅ READY |
| Guards | `src/autonomic/guards.rs` | ✅ READY |
| Effects | `src/autonomic/effects.rs` | ✅ READY |
| Receipts | `src/autonomic/receipts.rs` | ✅ READY |
| Delegation | `src/autonomic/delegation.rs` | ✅ READY |
| Contracts | `src/autonomic/contracts.rs` | ✅ READY |
| Governance | `src/autonomic/governance.rs` | ✅ READY |
| Protocol | `src/autonomic/protocol.rs` | ✅ READY |
| And 15+ more modules | All present | ✅ COMPLETE |

**No action needed** - v5 machine layer already built and tested!

### 1.3 Documentation Strategy (SIDE-BY-SIDE)

Keep existing docs for v4 human path:
- ✅ `docs/QUICKSTART.md` - For humans learning CLI
- ✅ `docs/TUTORIALS/` - Remain for human onboarding
- ✅ `docs/CLI_REFERENCE.md` - Human command reference
- ✅ `docs/CLI_COOKBOOK.md` - How-to guides for humans

Add NEW docs for v5 machine path:
- ✨ `docs/V5_MACHINE_API.md` - Machine API specification
- ✨ `docs/CAPABILITY_REGISTRY.md` - Capability discovery guide
- ✨ `docs/INTROSPECTION_API.md` - Introspection API reference
- ✨ `docs/AGENT_INTEGRATION.md` - How agents call the system
- ✨ `docs/DELEGATION_GUIDE.md` - Agent-to-agent authorization
- ✨ `docs/AUDIT_LEDGER.md` - Audit trail documentation

Update existing docs (don't delete):
- `docs/ARCHITECTURE.md` - Add dual-mode explanation
- `docs/ARCHITECTURE_REVIEW_V4.md` - Rename to reflect dual-mode

### 1.4 Examples Strategy (SIDE-BY-SIDE)

Keep v4 examples (human learning):
- ✅ `examples/basic.rs` - Basic CLI usage
- ✅ `examples/services.rs` - Service management example
- ✅ `examples/async_example.rs` - Async patterns
- ✅ `examples/context_example.rs` - App context
- ✅ `examples/validation.rs` - Validation patterns

Add NEW v5 examples (machine integration):
- ✨ `examples/machine_integration.rs` - Call as machine/agent
- ✨ `examples/agent_example.rs` - Agent calling system
- ✨ `examples/mcp_server.rs` - MCP protocol server
- ✨ `examples/capability_discovery.rs` - Introspection usage
- ✨ `examples/delegation_example.rs` - Agent delegation
- ✨ `examples/audit_ledger_example.rs` - Audit trail

---

## PART 2: INTEGRATION WORK (Add v5 Dispatcher Layer)

### 2.1 NEW: Create v5 Dispatcher/Router

**File**: `src/v5/dispatcher.rs` (NEW FILE)

**Purpose**: Route machine requests to autonomic layer, while preserving v4 human CLI path

**Logic**:
```rust
pub struct V5Dispatcher;

impl V5Dispatcher {
    /// Detect if request is from machine or human
    pub fn detect_caller(args: &[String]) -> CallerType {
        if args.contains(&"--machine".to_string()) {
            CallerType::Machine
        } else if args.contains(&"--introspect".to_string()) {
            CallerType::Machine
        } else if is_json_input(args) {
            CallerType::Machine
        } else {
            CallerType::Human
        }
    }

    /// Route to appropriate handler
    pub fn dispatch(args: &[String]) -> Result<()> {
        match Self::detect_caller(args) {
            CallerType::Human => {
                // Use existing v4 CLI path (help.rs, interactive.rs, etc.)
                v4_cli_run(args)
            }
            CallerType::Machine => {
                // Use autonomic layer for machine requests
                autonomic_dispatcher(args)
            }
        }
    }
}
```

### 2.2 NEW: Machine Handler Bridge

**File**: `src/v5/machine_handler.rs` (NEW FILE)

**Purpose**: Bridge between dispatcher and autonomic layer

**Calls**:
- `autonomic::introspection::get_capabilities()` - Capability discovery
- `autonomic::guards::evaluate_guards()` - Pre-execution checks
- `autonomic::schema::validate_input()` - Schema validation
- `autonomic::effects::declare_effects()` - Effect declaration
- `autonomic::receipts::create_receipt()` - Execution proof

### 2.3 EXISTING: No Changes Needed to v4 Path

| File | Change | Reason |
|------|--------|--------|
| `src/cli/help.rs` | ✅ NO CHANGE | Used by v4 human path |
| `src/cli/interactive.rs` | ✅ NO CHANGE | Used by v4 human path |
| `src/cli/examples.rs` | ✅ NO CHANGE | Used by v4 documentation |
| `src/cli/discovery.rs` | ✅ NO CHANGE | Used by v4 human path |
| `src/cli/router.rs` | ✅ NO CHANGE | Handles v4 routing |
| `src/cli/mod.rs` | ✅ MINOR: Add v5 exports | Export new dispatcher |

**v4 router logic stays unchanged**:
```rust
// Existing v4 CLI handling - KEEP AS-IS
match args.len() {
    0 => show_help(),           // ← KEEP
    1 if args[0] == "--help" => help_system.main_help(),  // ← KEEP
    1 if args[0] == "help" => interactive_help(),  // ← KEEP
    _ => execute_noun_verb()    // ← KEEP
}
```

Only wrapper checks for v5 machine request first:
```rust
pub fn run(args: &[String]) -> Result<()> {
    // NEW: Check if machine request first
    if let Some(result) = try_v5_dispatch(args) {
        return result;
    }

    // EXISTING: v4 human CLI path
    existing_v4_cli_logic(args)
}
```
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
