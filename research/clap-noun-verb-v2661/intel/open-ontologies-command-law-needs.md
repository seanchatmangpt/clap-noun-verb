# Open Ontologies Command Law Integration — clap-noun-verb v26.6.1

**Generated:** 2026-06-01  
**Scope:** What Open Ontologies should store about clap-noun-verb v26.6.1  
**Analysis Basis:** Static API audit (no assumptions about old names)

---

## Executive Summary

clap-noun-verb v26.6.1 provides **9 of 11 command law concepts**. The framework is:

- ✅ **Production-ready** for CommandSurface, ActionSurface, OutputContract, DispatchRoute
- ⚠️ **Partial** for ArgumentSurface (scalar types only), RefusalCondition (no guards), HelpSurface (no semantic docs)
- ❌ **Missing** ReceiptContract (critical for process mining; needs design + implementation)

### Critical Gaps for Process Mining Chicago Doctrine

| Issue | Impact | Solution |
|-------|--------|----------|
| **No Receipt type** | Cannot prove execution lawfulness via event logs | Add Receipt<T> struct with proof chain |
| **No refusal guards** | Cannot model preconditions; verbs always execute if args parse | Add #[requires(...)] syntax + SHACL validation |
| **Sync-only actions** | Async handlers require workaround (tokio::block_on) | Extend VerbCommand trait for async (Rust 1.75+) |
| **No causal chain** | Cannot link parent→child verb calls | Add execution_id, parent_id to Receipt |

---

## Detailed Concept Inventory

### 1. CommandSurface — FULLY SUPPORTED ✅

**What it is:** How commands are declared and discovered at compile-time.

**Current API:**
- **VerbCommand trait** (sync, dyn-compatible)
  - `name(&self) -> &'static str` — verb identifier
  - `about(&self) -> &'static str` — human description
  - `run(&self, args: &VerbArgs) -> Result<()>` — execution
- **NounCommand trait** (hierarchical)
  - `name(&self) -> &'static str` — noun identifier
  - `about(&self) -> &'static str` — description
  - `verbs(&self) -> Vec<Box<dyn VerbCommand>>` — verb list
  - `sub_nouns(&self) -> Vec<Box<dyn NounCommand>>` — nested nouns
- **#[verb(...)] macro** — declarative registration via linkme distributed_slice

**What Open Ontologies should store:**
```yaml
Command:
  has_verb: VerbDefinition
  has_noun: NounDefinition
  source_module: String  # filename of #[verb] macro application
  line_number: usize     # compile-time metadata
  registration_order: u32  # linkme slice order

VerbDefinition:
  name: string (static)
  about: string (static extracted from doc comments)
  noun_context: Option<string>
  is_async: boolean  # future: async support

NounDefinition:
  name: string (static)
  about: string (static)
  parent_noun: Option<string>  # for nesting
  verb_count: usize
  sub_noun_count: usize
```

**ggen Rendering:** YES — Trait names and about strings are static; hierarchical tree extractable.

**Claude Execution:** YES — VerbCommand.run() called with full VerbArgs context.

**Receipt Need:** YES — Each verb.run() is a discrete execution event; receipt must capture:
- verb_name, noun_path (for nested nouns)
- timestamp, execution_id, parent_execution_id (for causal chains)
- exit_status (Ok / Err with error_type)

**Extension Need:** YES
- [ ] Async variant (spawn ActionSurface trait, or async fn in trait)
- [ ] Execution_id generation and propagation
- [ ] Result<T> typed return (currently returns ())

---

### 2. CommandGroup — FULLY SUPPORTED ✅

**What it is:** Hierarchical grouping of commands (nouns → verbs; nouns → sub_nouns).

**Current API:**
- **NounCommand::sub_nouns()** — Nested command groups
- **CommandRouter::route_recursive()** — Hierarchical dispatch
- **CliBuilder** fluent API — Compose nouns at any depth

**Routing Example:**
```
myapp
  ├─ services (noun)
  │   ├─ status (verb)
  │   └─ start (verb)
  └─ collector (noun)
      └─ deploy (verb)
```

**What Open Ontologies should store:**
```yaml
CommandGroup:
  name: string
  parent_group: Optional[string]  # null for root
  direct_children: List[CommandGroup | VerbDefinition]
  depth: usize  # 0 = root nouns; 1+ = sub_nouns
  dispatch_path: string  # e.g., "services -> status"
```

**ggen Rendering:** YES — Complete hierarchical tree via NounCommand::sub_nouns().

**Claude Execution:** YES — Routing dispatcher selects correct group then verb.

**Receipt Need:** YES — Receipt must include noun_path for nested groups:
- e.g., parent_noun="collector", child_noun=null, verb="deploy"

**Extension Need:** YES
- [ ] Lazy group initialization (currently all groups loaded at startup)
- [ ] Dynamic group composition (current: static tree)
- [ ] Alias support for multi-path commands

---

### 3. ActionSurface — MOSTLY SUPPORTED ⚠️

**What it is:** Enumeration of all possible actions (verbs) and their signatures.

**Current API:**
- **VerbCommand trait** (global interface for all verbs)
- **#[verb(...)] macro** — Extracts verb name, about, parameter types from function signature
- **ArgMetadata** — Compile-time parameter extraction (type, required, defaults, env)

**Limitations:**
- ✅ Scalar types fully supported (String, i64, u16, bool, etc.)
- ⚠️ Complex types (Vec<T>, nested structs) need manual serialization
- ❌ Async actions require workaround (no async fn in trait until Rust 1.75+)
- ❌ No function return type capture (always returns ())

**What Open Ontologies should store:**
```yaml
ActionDefinition:
  verb: string
  noun: string
  parameters: List[Parameter]
  return_type: string  # e.g., "Result<()>"
  is_async: boolean
  execution_semantics:
    idempotent: boolean  # declarable
    side_effect_free: boolean
    transactional: boolean

Parameter:
  name: string
  type: string  # Rust type string
  required: boolean
  default_value: Optional[string]
  env_name: Optional[string]
  value_parser: Optional[string]  # e.g., "i64", "PathBuf"
  validation: Optional[ValidationRule]
  serialization_strategy: Optional[string]  # future
```

**ggen Rendering:** YES — Macro extracts all metadata; callable at compile-time.

**Claude Execution:** YES — VerbArgs provides parsed arguments; can execute via verb.run().

**Receipt Need:** YES — Receipt must capture:
- action_name (verb), action_context (noun)
- input_parameters (name → parsed value)
- output_serialization_format (JSON, YAML, etc.)
- execution_time_ms

**Extension Need:** YES (CRITICAL)
```rust
// v5.0 design: Async ActionSurface
#[async_verb("status")]
async fn services_status(name: String) -> Result<StatusOutput> {
    // Can await async operations
}

// Future: Typed output
impl VerbCommand for status {
    type Output = StatusOutput;  // Serializable
    fn about(&self) -> &'static str { ... }
    async fn run(&self, args: &VerbArgs) -> Result<Self::Output> { ... }
}
```

---

### 4. ArgumentSurface — PARTIALLY SUPPORTED ⚠️

**What it is:** Specification of command arguments and their validation.

**Current API:**
- **ArgMetadata** (compile-time extraction)
  - name, type (Rust type string), required, default_value
  - short/long flags, environment variable mapping
  - value_parser (e.g., "i64", "PathBuf", "url::Url")
  - min/max bounds (numeric), min/max length (string)
- **#[arg(...)]** attributes on parameters (doc comment style or explicit)
- **validators module** (standalone functions)
  - validate_email, validate_url, validate_port, validate_not_empty, etc.
- **OutputValidationHook** (pluggable validation at format stage)

**Limitations:**
- ✅ Scalar types (String, i64, bool, PathBuf) fully supported
- ⚠️ Vec<T>, complex structs require manual #[arg(...)] annotations
- ❌ No custom validator registration (only static validators module)
- ❌ Validation fires only at argument parse stage; no runtime preconditions

**What Open Ontologies should store:**
```yaml
ArgumentSurface:
  arguments: List[ArgumentSpec]
  validation_rules: List[ValidationRule]
  serialization_contracts: List[SerializationContract]

ArgumentSpec:
  name: string
  type: string  # Rust type
  required: boolean
  default_value: Optional[string]
  short_flag: Optional<char>
  long_flag: Optional<string>
  env_variable: Optional<string>
  doc: string  # extracted from /// comments
  value_parser: Optional<string>
  bounds: Optional<(i64, i64)>  # min, max
  length_bounds: Optional<(usize, usize)>  # min_length, max_length
  
ValidationRule:
  applies_to: string  # argument name
  type: enum  # "required", "range", "pattern", "custom"
  parameters: Dict  # e.g., {min: 0, max: 100}
  error_message: string
  applies_at_stage: enum  # "parse", "pre_execution", "output"

SerializationContract:
  type_name: string  # e.g., "Vec<String>", "User"
  strategy: string  # "json", "csv", "yaml", "custom"
  schema: Optional<string>  # JSON Schema or equivalent
```

**ggen Rendering:** PARTIAL
- ✅ Scalar args: Full metadata extractable
- ⚠️ Complex types: Need type schema (JSON Schema or RDF)
- ❌ clio::Input/Output: Auto-detected but needs special handling

**Claude Execution:** YES — VerbArgs.matches() provides parsed arguments.

**Receipt Need:** YES — Receipt must capture:
- argument_name → provided_value
- validation_status (passed / failed)
- error_message (if validation failed)
- serialization_format_used

**Extension Need:** YES (CRITICAL)
```rust
// v5.0: Type-aware argument contracts
#[verb("create")]
fn create_user(
    #[arg(validate = "user_email")]
    #[arg(json_schema = r#"{"type": "string", "pattern": "^[a-z]+@[a-z]+\.[a-z]+$"}"#)]
    email: String,
) -> Result<()> { }

// Register custom validators at startup
register_validator("user_email", |value: &str| {
    if value.contains('@') { Ok(()) }
    else { Err("Invalid email") }
});
```

---

### 5. OutputContract — FULLY SUPPORTED ✅

**What it is:** How verb results are formatted and validated for output.

**Current API:**
- **OutputFormat enum**
  - Json (compact), JsonPretty (default), Yaml, Table, Plain, Tsv
- **format_output<T: Serialize>(data: T, format: OutputFormat) -> Result<String>**
- **OutputValidationHook** (register global validation)
  - `Fn(&serde_json::Value) -> Result<(), Box<dyn Error>>`
- **HandlerOutput** (future handler-based verbs)
  - data: serde_json::Value
  - message: Option<String>

**Validation Flow:**
```
verb.run() → serialize to JSON → apply hooks → format to OutputFormat → output
```

**What Open Ontologies should store:**
```yaml
OutputContract:
  selected_format: enum  # Json, JsonPretty, Yaml, Table, Plain, Tsv
  serialization_type: string  # e.g., "serde_json::Value"
  schema: Optional<string>  # JSON Schema describing output structure
  validation_hooks:
    - hook_name: string
    - applies_to_format: Optional[string]  # null = all formats
    - rule_description: string
  rendered_output: string  # final formatted output

SerializationEvent:
  input_value: any  # Original Rust value
  json_representation: string  # After serde_json::to_value()
  format_selected: OutputFormat
  validation_passed: boolean
  output_string: string
  bytes_length: usize
```

**ggen Rendering:** YES
- ✅ OutputFormat enum is static
- ✅ Validation hooks are callable
- ✅ Format selection logic is deterministic

**Claude Execution:** YES — format_output() can be called post-execution.

**Receipt Need:** YES — Receipt must capture:
- selected_format (enum)
- output_bytes (or hash thereof)
- validation_status (passed / failed + error_message)
- serialization_time_ms

**Extension Need:** YES
```rust
// v5.0: Output contracts with schema binding
#[verb("list-users")]
fn list_users() -> Result<UsersOutput> {
    // Macro generates: struct UsersOutput { ... }
    // Macro validates: UsersOutput implements Serialize
    // At output stage: Apply JSON Schema from derive(JsonSchema)
}

// Register output validation schema
register_output_validation_hook(|value| {
    let schema = serde_json::json!({
        "type": "object",
        "properties": {
            "users": { "type": "array", "items": { "type": "object" } }
        },
        "required": ["users"]
    });
    jsonschema::validate(value, &schema)
});
```

---

### 6. ErrorContract — FULLY SUPPORTED ✅

**What it is:** Error reporting and recovery strategy.

**Current API:**
- **NounVerbError enum** (thiserror::Error)
  - CommandNotFound { noun, suggestion }
  - VerbNotFound { noun, verb, suggestion }
  - InvalidStructure { message }
  - ExecutionError { message }
  - ArgumentError { message }
  - PluginError(String), ValidationFailed(String), etc.
- **Result<T>** type alias (std::result::Result<T, NounVerbError>)
- **find_best_matches(input, candidates) -> Vec<&'a str>**
  - Levenshtein distance-based suggestions
- **with_recovery_suggestions()** — Placeholder for v5.1 RDF-based recovery

**Error Flow:**
```
Routing error → NounVerbError variant → Display (with suggestions) → stderr
Execution error → Result::Err → ErrorContract receipt → stderr
```

**What Open Ontologies should store:**
```yaml
ErrorContract:
  error_type: enum  # CommandNotFound, VerbNotFound, ExecutionError, etc.
  error_message: string
  error_context:
    noun: Optional[string]
    verb: Optional[string]
    argument: Optional[string]
  suggestions: List<string>  # Levenshtein-matched candidates
  recovery_attempted: boolean
  recovery_suggestion: Optional<string]  # v5.1: RDF-generated
  causality:
    caused_by: Optional<ErrorContract>  # error chain
    recoverable: boolean  # can verb be retried?

SuggestionAlgorithm:
  algorithm: "levenshtein_distance"
  max_distance: 3  # only suggest within 3 edits
  threshold: input.len()  # distance must be < input length
```

**ggen Rendering:** YES — NounVerbError variants are static enum.

**Claude Execution:** YES — All Err paths propagate; context carries error source.

**Receipt Need:** YES — Receipt must capture:
- error_type, error_message
- suggestions_offered (count + list)
- recovery_attempted (boolean)
- exception_stack (future)

**Extension Need:** YES
```rust
// v5.1: RDF-driven recovery suggestions
with_recovery_suggestions() {
    // Query RDF: "What commands are alternatives to this failed command?"
    // Apply SPARQL: SELECT ?alternative WHERE { ?failed rdfs:seeAlso ?alternative }
    // Return suggestions from RDF ontology
}

// v5.0: Error recovery proof
error_recovery_proof: {
    error_type: "VerbNotFound",
    suggestions: ["status", "start"],
    user_selected: "status",
    recovery_success: true
}
```

---

### 7. RefusalCondition — PARTIALLY SUPPORTED ⚠️

**What it is:** Conditions under which a verb should refuse execution (preconditions, guards).

**Current Implementation:**
- ✅ Validators module (validate_not_empty, validate_path_exists, etc.)
- ✅ ArgMatches validation (clap parser rejects invalid args)
- ⚠️ No declarative refusal syntax (no @requires, @guard, @precondition)
- ❌ No SHACL shape enforcement at runtime
- ❌ Validators fire only at parse stage; no pre-execution checks

**What Open Ontologies should store:**
```yaml
RefusalCondition:
  condition_type: enum  # "precondition", "guard", "capability_check"
  applies_to_verb: string
  condition_expression: string  # SPARQL ASK query or guard predicate
  refusal_reason: string
  refusal_remediation: Optional<string>

# Example: Precondition for "deploy" verb
RefusalCondition:
  condition_type: "precondition"
  applies_to_verb: "deploy"
  condition_expression: "path_exists('/config/deployment.toml')"
  refusal_reason: "Deployment config not found"
  refusal_remediation: "Run 'myapp config init' first"

# Example: Guard for "delete" verb (destructive operation)
RefusalCondition:
  condition_type: "guard"
  applies_to_verb: "delete"
  condition_expression: "--confirm flag provided AND user_is_admin"
  refusal_reason: "Destructive operation requires confirmation and admin privilege"
```

**Current Gaps:**
```rust
// MISSING: Declarative precondition syntax
#[verb("deploy", requires = "config_exists AND is_authenticated")]
fn deploy(app: String) -> Result<()> { }

// MISSING: Guard expressions
#[verb("delete", guard = "has_flag(--confirm) && is_admin()")]
fn delete_app(app: String) -> Result<()> { }

// MISSING: SHACL shape validation at runtime
// Current: ArgMetadata.value_parser only does type parsing
// Needed: Query RDF shape; validate against SHACL shape
```

**ggen Rendering:** PARTIAL
- ✅ Validators are static (extractable)
- ❌ No declarative condition syntax yet

**Claude Execution:** PARTIAL
- ✅ Can manually validate in verb.run() logic
- ❌ No automatic pre-execution guard check

**Receipt Need:** YES — Receipt must capture:
- precondition_evaluated: boolean
- precondition_result: "passed" | "failed"
- guard_evaluated: boolean
- guard_result: "passed" | "refused"
- refusal_message (if any)

**Extension Need:** YES (CRITICAL)
```rust
// v5.0 design: Declarative refusal conditions
#[requires("output_path_writable")]
#[guard("user_confirms || not_destructive")]
#[verb("export")]
fn export_data(format: String, #[arg(env = "OUTPUT")] path: PathBuf) -> Result<()> { }

// Precondition checker (runs before verb.run())
fn check_preconditions(verb_name: &str) -> Result<()> {
    // Query RDF: SELECT ?precondition WHERE { verb_name rdfs:requires ?precondition }
    // For each precondition, evaluate and return Err if any fail
}

// Guard checker (runs after arg validation, before business logic)
fn evaluate_guards(verb_name: &str, args: &VerbArgs) -> Result<()> {
    // Query RDF: SELECT ?guard WHERE { verb_name cnv:guard ?guard }
    // Evaluate guard expressions (e.g., parse --confirm flag)
}
```

---

### 8. HelpSurface — MOSTLY SUPPORTED ✅

**What it is:** Help generation and documentation for commands.

**Current API:**
- **VerbCommand::about()** — Human-readable verb description
- **NounCommand::about()** — Noun description
- **#[verb(...)]** macro extracts doc comments (/// ...) as about text
- **ArgMetadata.about** — Argument documentation
- **clap::Command::about()** — Integrated into clap help system
- **clap::Arg::help()** — Per-argument help text

**Help Generation:**
```
$ myapp --help
→ clap renders from Command tree + about strings

$ myapp services --help
→ clap renders noun help + verb list

$ myapp services status --help
→ clap renders verb help + arguments
```

**What Open Ontologies should store:**
```yaml
HelpSurface:
  for_verb: string
  for_noun: string
  help_text: string
  help_source: enum  # "derived_from_doc", "explicit_about", "auto_generated"
  
  arguments_help: List[ArgumentHelp]
  
  examples: Optional[List[string]]  # Future: extractable examples
  semantic_description: Optional[string]  # Future: type descriptions

ArgumentHelp:
  argument_name: string
  help_text: string
  type_description: string
  example_value: Optional<string>
  constraints: Optional<string>  # e.g., "Must be > 0 and < 100"
```

**ggen Rendering:** YES
- ✅ All about() strings are static (extractable from macro metadata)
- ✅ Doc comments are compile-time metadata
- ⚠️ Semantic help (type semantics, constraint descriptions) not yet modeled

**Claude Execution:** YES — Help rendered by clap; can be captured via --help or introspected.

**Receipt Need:** PARTIAL
- Could track help_requested, help_section_viewed for UX analytics
- Not critical for command law, but useful for agent introspection

**Extension Need:** YES
```rust
// v5.0: Semantic help (type descriptions, constraint semantics)
#[verb("create")]
/// Create a new user account.
///
/// [semantic] This verb creates a new User object in the system.
/// [precondition] No user with this email must already exist.
/// [example] myapp users create --email alice@example.com --admin
fn create_user(
    #[arg(semantic = "User email address (unique, must be valid)")]
    #[arg(example = "alice@example.com")]
    email: String,
) -> Result<()> { }

// v5.0: Auto-generated examples from test cases
// Extract examples from #[test] marked with #[example]
// Render in help output
```

---

### 9. DispatchRoute — FULLY SUPPORTED ✅

**What it is:** How commands are routed from CLI arguments to verb execution.

**Current API:**
- **CommandRouter::route()** — Main routing entrypoint
  - Extracts noun from root subcommand
  - Looks up noun in registry
  - Calls route_recursive()
- **CommandRouter::route_recursive()** — Hierarchical routing
  - Checks for verb in noun.verbs()
  - Falls back to sub-noun check via sub_nouns()
  - Builds VerbArgs and calls verb.run()
- **CommandRegistry::run()** — Entry point (links to routers)
  - Parses args with clap::Command
  - Delegates to router

**Routing Logic:**
```
myapp services status --verbose
├─ Parse with clap → ArgMatches
├─ route() extracts "services" noun
├─ route_recursive() finds "status" verb in services.verbs()
├─ Build VerbArgs { matches, context }
└─ call status.run(args)
```

**What Open Ontologies should store:**
```yaml
DispatchRoute:
  input_args: List[string]
  parsed_matches: ArgMatches  # clap's parse result
  
  routing_path:
    - step: enum  # "noun_lookup", "verb_lookup", "subcommand_check"
      target: string  # noun name or verb name
      found: boolean
      candidates_checked: List<string>  # for error suggestions
  
  selected_verb: string
  selected_noun: string
  selected_noun_path: List<string]  # for nested: ["collector", "deploy"]
  
  verb_args: VerbArgs  # input to verb.run()
  routing_time_ms: usize

RoutingError:
  error_type: enum  # "noun_not_found", "verb_not_found"
  input_noun: string
  input_verb: Optional<string>
  candidates: List<string>  # from command registry
  suggestion: Optional<string>  # Levenshtein match
```

**ggen Rendering:** YES
- ✅ Routing logic is deterministic
- ✅ CommandRegistry tree structure is static

**Claude Execution:** YES
- ✅ Dispatcher validates args, then calls verb.run()
- ✅ Complete execution path: parse → route → execute

**Receipt Need:** YES — Receipt must capture:
- dispatch_path (noun_path + verb_name)
- args_provided (all CLI args)
- routing_time_ms (for SLO tracking)
- routing_errors (if any)

**Extension Need:** YES
```rust
// v5.0: Middleware chain during dispatch
dispatch_route()
  .with_middleware(|ctx| {
      // Pre-dispatch: validate auth, check rate limits
      check_permission(ctx.verb_name)?;
      check_rate_limit(ctx.user_id)?;
      Ok(())
  })
  .execute_verb()
  .with_middleware(|ctx, result| {
      // Post-dispatch: audit log, metrics
      audit_log(ctx.verb_name, result.is_ok());
      record_latency(ctx.dispatch_time_ms);
      Ok(result)
  })

// v5.0: Transaction semantics
// Wrap verb.run() in a transaction:
// BEGIN → route → execute → (COMMIT or ROLLBACK)
```

---

### 10. EmittedCommandModule — FULLY SUPPORTED ✅

**What it is:** Generated code artifacts from #[verb] macro.

**Current Implementation:**
- **#[verb(...)] macro** generates:
  - VerbCommand impl for the decorated function
  - linkme::distributed_slice registration in VERBS
  - Static metadata (name, about, parameter metadata)
- **Compile-time validation**
  - Return type must implement Serialize (checked)
  - No duplicate verb names (checked)
  - Parameter complexity bounds (max 10 params)
- **I/O detection** (io_detection.rs)
  - Auto-detects clio::Input / clio::Output parameters
  - Generates appropriate clap configuration

**What Open Ontologies should store:**
```yaml
EmittedCommandModule:
  source_function: string  # e.g., "services_status"
  source_file: string  # e.g., "src/commands/services.rs"
  source_line: usize
  
  generated_artifacts:
    - artifact_type: "VerbCommandImpl"
      impl_struct: string  # name of impl block
    - artifact_type: "LinkmeEntry"
      slice_name: "VERBS"
      entry_position: usize  # order in distributed slice
    
  metadata_extracted:
    verb_name: string  # from macro argument or function name
    about: string  # from /// doc comments
    parameters: List<ArgMetadata>
    return_type: string  # validated Serialize
    io_detection_result: enum  # "input_detected", "output_detected", "both", "none"
  
  validation_results:
    return_type_serializable: boolean  # Serialize impl
    no_duplicate_names: boolean
    parameter_count_valid: boolean  # <= 10
```

**ggen Rendering:** YES
- ✅ Macro output is introspectable
- ✅ All generated code is deterministic

**Claude Execution:** YES
- ✅ Generated VerbCommand collected at startup via linkme
- ✅ Executed via verb.run()

**Receipt Need:** YES — Receipt should capture:
- module_name (source function)
- generated_type_count (artifacts)
- registration_order (linkme position)

**Extension Need:** YES
```rust
// v5.0: Async verb macro
#[async_verb("fetch-data")]
async fn fetch_data(url: String) -> Result<Data> {
    let data = reqwest::get(&url).await?.json().await?;
    Ok(data)
}

// v5.0: Macro-generated result type
#[verb("list-users")]
fn list_users() -> Result<UsersOutput> {  // Macro generates: struct UsersOutput { ... }
    // Compile error if UsersOutput doesn't implement Serialize
}

// v5.0: Middleware-aware macro
#[verb("sensitive-op")]
#[requires_middleware("authentication")]  // Automatically injected
fn sensitive_op() -> Result<()> { }
```

---

### 11. ReceiptContract — NOT YET SUPPORTED ❌ (CRITICAL)

**What it is:** Evidence artifact for command execution (proof of lawful process per Process Mining Chicago doctrine).

**Current State:**
- ❌ No Receipt type defined
- ❌ No proof generation (hash chains, timestamps, signatures)
- ❌ No OCEL event log emission
- ❌ No causality tracking (parent execution_id → child verb links)
- ⚠️ Telemetry module is placeholder only

**Why This is Critical:**
> "If the code says it worked but the event log cannot prove a lawful process happened, then it did not work." — Van der Aalst Constitution

Without receipts, Process Mining Chicago cannot:
- ✅ Derive actual runtime process from execution logs
- ✅ Compare actual vs. declared process model
- ✅ Detect skipped/repeated stages, hidden loops, retries
- ✅ Prove object lifecycle conformance
- ✅ Validate temporal ordering of stages

**What Open Ontologies Should Require:**

```yaml
Receipt:
  metadata:
    receipt_id: string  # UUID or deterministic hash
    execution_id: string  # Unique per verb invocation
    parent_execution_id: Optional<string>  # For nested/sub-commands
    timestamp_start: ISO8601  # When verb.run() started
    timestamp_end: ISO8601  # When verb.run() completed
    duration_ms: usize
  
  command:
    verb_name: string
    noun_path: List<string>  # e.g., ["collector", "deploy"]
    full_command: string  # e.g., "myapp collector deploy --force"
  
  input:
    arguments: Dict<string, serde_json::Value>  # all CLI args
    environment_variables: Optional<Dict<string, string>>  # captured at execution
  
  execution:
    execution_status: enum  # "started", "completed", "failed", "refused"
    exit_code: i32
    error_type: Optional<string>  # NounVerbError variant
    error_message: Optional<string>
  
  output:
    output_format: enum  # Json, Yaml, etc.
    output_bytes: string  # or hash thereof
    serialization_time_ms: usize
  
  proof_chain:
    hash_chain: List<string>  # cryptographic proof of lineage
      # hash[i] = H(hash[i-1] || receipt[i])
    signature: Optional<string>  # GPG or similar
    timestamp_authority: Optional<string>  # RFC 3161 TSA

  lifecycle:
    stage: enum  # "invoked", "validated", "executing", "output", "completed", "failed", "refused"
    object_id: Optional<string>  # If verb operates on specific object (user, app, etc.)
    object_type: Optional<string>  # Type of object ("User", "Deployment", etc.)
    object_lifecycle_before: Optional<string>  # JSON snapshot of object state
    object_lifecycle_after: Optional<string]  # JSON snapshot after execution

# OCEL Mapping (for Process Mining Chicago)
OCEL_Event:
  event_id: string  # = Receipt.receipt_id
  timestamp: ISO8601  # = Receipt.timestamp_end
  activity: string  # = Receipt.verb_name
  object_type: string  # = Receipt.object_type
  object_id: string  # = Receipt.object_id
  attributes:
    noun_path: List<string>
    exit_code: i32
    duration_ms: usize
    error_message: Optional<string>
```

**Implementation Plan for v5.0:**

```rust
// Step 1: Define Receipt type
pub struct Receipt<T: Serialize> {
    pub metadata: ReceiptMetadata,
    pub execution: ExecutionRecord,
    pub output: OutputRecord<T>,
    pub proof: ProofChain,
}

impl<T: Serialize> Receipt<T> {
    pub fn new(execution_id: String) -> Self { ... }
    pub fn with_command(self, verb: String, noun_path: Vec<String>) -> Self { ... }
    pub fn with_input(self, args: ArgMatches) -> Self { ... }
    pub fn with_output(self, data: T, format: OutputFormat) -> Self { ... }
    pub fn finalize(self, status: ExecutionStatus) -> Self { ... }
    pub fn hash_chain(&self) -> Vec<String> { ... }
}

// Step 2: Emit receipt in verb dispatch
fn route_verb(verb: &dyn VerbCommand, args: &VerbArgs) -> Result<()> {
    let receipt = Receipt::new(uuid::Uuid::new_v4().to_string());
    let receipt = receipt.with_command(verb.name(), args.context.noun.clone());
    let receipt = receipt.with_input(args.matches.clone());
    
    let start = std::time::Instant::now();
    match verb.run(args) {
        Ok(()) => {
            let receipt = receipt.finalize(ExecutionStatus::Completed);
            emit_receipt(&receipt);  // Send to event log
            Ok(())
        }
        Err(e) => {
            let receipt = receipt.finalize(ExecutionStatus::Failed);
            emit_receipt(&receipt);  // Always emit, even on failure
            Err(e)
        }
    }
}

// Step 3: Register receipt handler (MCP endpoint)
pub fn register_receipt_handler(tx: Sender<Receipt<serde_json::Value>>) {
    // Forward receipts to event log, OCEL converter, process miner
}
```

**ggen Rendering:** NO (no Receipt type yet to render)

**Claude Execution:** YES (once Receipt type is implemented)

**Receipt Need:** CRITICAL
- Without receipts, no proof of lawful execution
- Without OCEL, Process Mining Chicago cannot validate

**Extension Need:** YES (CRITICAL)
- [ ] Receipt<T> struct definition
- [ ] Receipt emission in verb dispatch
- [ ] Hash chain generation
- [ ] OCEL event emission
- [ ] Process miner integration (pm4py)

---

## Open Ontologies Storage Recommendations

### 1. Command Law Vocabulary (RDF Ontology)

Store in Open Ontologies as Turtle RDF:

```turtle
@prefix cnv: <http://clap-noun-verb.org/command-law/> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix sh: <http://www.w3.org/ns/shacl#> .
@prefix prov: <http://www.w3.org/ns/prov#> .
@prefix ocel: <https://ocel.readthedocs.io/1.0/> .

# Command surface
cnv:Verb a rdfs:Class ;
    rdfs:label "Verb (Action)" ;
    rdfs:comment "A verb is an action that can be executed within a noun context" .

cnv:Noun a rdfs:Class ;
    rdfs:label "Noun (Command Group)" ;
    rdfs:comment "A noun groups related verbs; nouns can nest hierarchically" .

cnv:hasVerb rdfs:domain cnv:Noun ;
    rdfs:range cnv:Verb ;
    rdfs:label "has verb" .

cnv:parentNoun rdfs:domain cnv:Noun ;
    rdfs:range cnv:Noun ;
    rdfs:label "parent noun" ;
    rdfs:comment "For nested nouns" .

# Argument and output contracts
cnv:Argument a rdfs:Class .
cnv:OutputContract a rdfs:Class .
cnv:ErrorContract a rdfs:Class .

cnv:hasArgument rdfs:domain cnv:Verb ;
    rdfs:range cnv:Argument .

# Execution proof (Process Mining Chicago)
cnv:Receipt a rdfs:Class ;
    rdfs:label "Execution Receipt" ;
    rdfs:comment "Proof artifact for command execution; used for process mining" .

cnv:receiptId rdfs:domain cnv:Receipt ;
    rdfs:range xsd:string ;
    rdfs:label "Receipt ID" .

cnv:executionId rdfs:domain cnv:Receipt ;
    rdfs:range xsd:string ;
    rdfs:comment "Unique per execution; enables causal chains via parentExecutionId" .

cnv:parentExecutionId rdfs:domain cnv:Receipt ;
    rdfs:range xsd:string ;
    rdfs:comment "Links to parent verb execution (for nested commands)" .

cnv:refusalCondition a rdfs:Class ;
    rdfs:label "Refusal Condition" ;
    rdfs:comment "Precondition or guard that must be satisfied for verb execution" .

cnv:requiresPrecondition rdfs:domain cnv:Verb ;
    rdfs:range cnv:RefusalCondition ;
    rdfs:label "requires precondition" .

cnv:guard rdfs:domain cnv:Verb ;
    rdfs:range cnv:RefusalCondition ;
    rdfs:label "guard condition" .
```

### 2. Metadata Files (YAML or JSON)

Store compile-time metadata alongside source:

```yaml
# In Open Ontologies: clap-noun-verb-v2661-metadata.yaml

commands:
  - verb: status
    noun: services
    file: src/commands/services.rs
    line: 42
    about: "Show service status"
    parameters:
      - name: service_name
        type: String
        required: true
        about: "Name of service to check"
    output_contract:
      format: Json
      schema: "ServiceStatus"
    refusal_conditions: []

  - verb: deploy
    noun: collector
    file: src/commands/collector.rs
    line: 87
    about: "Deploy collector to cloud"
    parameters:
      - name: environment
        type: String
        required: true
        about: "Target environment (dev, staging, prod)"
        validation: "enum: [dev, staging, prod]"
      - name: confirm
        type: bool
        required: false
        about: "Confirm destructive operation"
    output_contract:
      format: Json
      schema: "DeploymentResult"
    refusal_conditions:
      - type: precondition
        expression: "config_file_exists('/config/deploy.toml')"
        reason: "Deployment config missing"
      - type: guard
        expression: "confirm_flag_provided || not_prod_environment"
        reason: "Production deployments require --confirm flag"
```

### 3. Receipt Emission (OCEL Format)

For process mining compliance:

```json
{
  "ocel:events": [
    {
      "ocel:event_id": "exec-12345-status",
      "ocel:timestamp": "2026-06-01T14:23:45.123Z",
      "ocel:activity": "status",
      "ocel:object_type": "Service",
      "ocel:object_id": "service:redis",
      "attributes": {
        "noun_path": ["services"],
        "exit_code": 0,
        "duration_ms": 245,
        "output_format": "json"
      }
    }
  ]
}
```

---

## Summary: What Open Ontologies Must Store

| Concept | Storage Type | Priority | Rationale |
|---------|--------------|----------|-----------|
| CommandSurface | RDF + Metadata YAML | CRITICAL | Core API; enables introspection |
| ActionSurface | RDF + ArgMetadata | CRITICAL | Verb enumeration; arg specifications |
| OutputContract | RDF + Schema Registry | HIGH | Serialization contracts; validation rules |
| ErrorContract | RDF + Recovery Rules | HIGH | Error types; suggestions |
| DispatchRoute | Metadata YAML | MEDIUM | Routing logic; optimization |
| Receipt | OCEL + RDF | CRITICAL | Process Mining Chicago compliance; MUST implement |
| RefusalCondition | RDF + SPARQL Queries | CRITICAL | Preconditions, guards; MUST design |

**Immediate Actions:**
1. **Define Receipt<T> struct** in clap-noun-verb (v5.0)
2. **Design refusal condition syntax** (#[requires], #[guard])
3. **Export metadata to RDF** at compile-time via macro
4. **Emit OCEL events** to process miner at runtime
5. **Integrate with Open Ontologies** (API, queries, validation)

---

## References

- **Process Mining Chicago:** Van der Aalst, "Process Mining" (2nd ed.), Springer 2022
- **OCEL:** https://ocel.readthedocs.io/1.0/
- **SHACL:** https://www.w3.org/TR/shacl/
- **RDF/OWL:** https://www.w3.org/RDF/
- **clap-noun-verb:** https://docs.rs/clap-noun-verb
