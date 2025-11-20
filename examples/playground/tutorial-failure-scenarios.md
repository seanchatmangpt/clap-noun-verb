# Tutorial Failure Scenarios - Interactive Playground

**Purpose**: Hands-on scenarios for machines to learn what FAILS and why.

---

## Scenario 1: Tutorial 1 Compilation Failure (FM-01)

### Setup

You are an AI agent learning to use clap-noun-verb v5. You read Tutorial 1 and see this example:

```rust
use clap_noun_verb::cli::run;

#[noun]
pub struct Pack {
    #[verb]
    pub async fn list(
        #[arg] category: Option<String>,
        #[arg(long)] verbose: bool,
    ) -> Result<Vec<String>> {
        Ok(vec!["pack1".to_string(), "pack2".to_string()])
    }
}
```

### Task

1. Copy this code to a new file `src/pack.rs`
2. Attempt to compile: `cargo check`
3. Observe the errors
4. Record your findings

### Expected Outcome

```
error[E0425]: cannot find attribute 'noun' in this scope
error[E0425]: cannot find attribute 'verb' in this scope
error[E0412]: cannot find type 'Result' in this scope
```

### Questions for Reflection

1. **Why did the compilation fail?**
   - [ ] The attributes `#[noun]` and `#[verb]` don't exist
   - [ ] The `Result` type is unqualified
   - [ ] Missing imports
   - [ ] All of the above

2. **What should the documentation do differently?**
   - [ ] Mark this as `rust,ignore` or `pseudocode`
   - [ ] Provide the actual working v5 API
   - [ ] Include a "Future API" warning
   - [ ] All of the above

3. **As a machine agent, what is your trust level now?**
   - [ ] Still 100% (this was just one mistake)
   - [ ] Degraded to 70% (first example failed)
   - [ ] Degraded to 30% (pattern of failures)
   - [ ] 0% (documentation is unreliable)

### Learning Objective

**Machines must distinguish between aspirational and actual APIs.**

- ❌ Bad: Assume all documented code compiles
- ✅ Good: Verify code compiles before trusting

### Fix Simulation

Now try the FIXED version:

```rust
use clap::Parser;
use std::error::Error;

#[derive(Parser, Debug)]
#[command(name = "pack")]
pub struct Pack {
    #[command(subcommand)]
    pub command: PackCommand,
}

#[derive(Parser, Debug)]
pub enum PackCommand {
    List {
        category: Option<String>,
        #[arg(long)]
        verbose: bool,
    },
}
```

**Compile this version**: `cargo check`

**Result**: ✅ Compiles successfully

**Trust restoration**: Machines can trust this code.

---

## Scenario 2: Type Reference Without Definition (FM-04)

### Setup

You are building an agent that queries CLI capabilities. Documentation shows:

```rust
async fn get_all_capabilities() -> Result<Vec<Capability>> {
    let output = std::process::Command::new("./myapp")
        .arg("--introspect")
        .output()?;

    let json: serde_json::Value = serde_json::from_slice(&output.stdout)?;

    let capabilities = json["capabilities"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| Capability {
            id: c["id"].as_str().unwrap(),
            name: c["name"].as_str().unwrap(),
            description: c["description"].as_str().unwrap(),
        })
        .collect();

    Ok(capabilities)
}
```

### Task

1. Copy this function to your codebase
2. Attempt to compile
3. Search for `Capability` type definition
4. Observe the circular dependency

### Expected Outcome

```
error[E0412]: cannot find type 'Capability' in this scope
help: you might be missing a type definition for 'Capability'
```

### The Circular Dependency

```
To compile: Need Capability type definition
To define Capability: Need to know structure
To know structure: Need working function
To have working function: Need Capability type
→ STUCK
```

### Questions for Reflection

1. **Why is this worse than a simple missing import?**
   - [ ] The type is referenced but never defined
   - [ ] Cannot infer structure from usage alone
   - [ ] Creates false assumption that type exists elsewhere
   - [ ] All of the above

2. **What should a machine do when encountering this?**
   - [ ] Try to infer the type structure from field usage
   - [ ] Search all documentation for type definition
   - [ ] Give up and report error
   - [ ] Ask human for clarification

3. **How much time would a machine waste debugging this?**
   - [ ] 5 minutes (quick search)
   - [ ] 30 minutes (thorough search + inference attempts)
   - [ ] 2 hours (multiple workaround attempts)
   - [ ] Give up entirely

### Learning Objective

**Helper functions are useless without complete type definitions.**

- ❌ Bad: Reference types without defining them
- ✅ Good: Define all types BEFORE using them

### Fix Simulation

Now with type definition FIRST:

```rust
use serde::{Deserialize, Serialize};
use std::error::Error;

// ✅ DEFINE TYPES FIRST
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    pub id: String,
    pub name: String,
    pub description: String,
}

// Now the function can compile
async fn get_all_capabilities() -> Result<Vec<Capability>, Box<dyn Error>> {
    // ... (same implementation)
    Ok(vec![])
}
```

**Result**: ✅ Self-contained, compiles immediately

---

## Scenario 3: Guard API That Doesn't Exist (FM-03)

### Setup

Tutorial 3 teaches guard implementation:

```rust
impl Template {
    fn preconditions() -> Vec<Guard> {
        vec![
            Guard::new("template_exists")
                .description("Template must be registered")
                .check(|ctx| {
                    let name = ctx.arg("name").unwrap();
                    TemplateRegistry::has(name)
                }),
        ]
    }
}
```

### Task

1. Implement guards using this API
2. Attempt to compile
3. Search for `Guard::new()` in codebase
4. Realize the API doesn't exist

### Expected Outcome

```
error[E0599]: no function or associated item named 'new' found for struct 'Guard'
error[E0412]: cannot find type 'TemplateRegistry' in this scope
```

### The API Confusion Timeline

```
Time 0min: "Great! I'll use Guard::new() builder pattern"
Time 5min: "Hmm, compiler error... maybe I need an import?"
Time 10min: "Let me search the crate for Guard::new..."
Time 20min: "It's not there. Maybe it's in a feature flag?"
Time 30min: "Is this the right version? Let me check"
Time 60min: "This API doesn't exist. Tutorial lied to me."
```

**Time wasted**: 1 hour
**Trust damage**: Severe

### Questions for Reflection

1. **Why is aspirational API documentation harmful for machines?**
   - [ ] Machines take documentation literally
   - [ ] No way to distinguish "planned" from "implemented"
   - [ ] Wastes time trying to use non-existent APIs
   - [ ] All of the above

2. **What should documentation do for unimplemented features?**
   - [ ] Don't document until implemented
   - [ ] Clearly mark as "FUTURE API (v5.2)"
   - [ ] Provide current working alternative
   - [ ] All of the above

3. **How should a machine handle this situation?**
   - [ ] Assume it's a bug and report to maintainers
   - [ ] Try to implement the API based on documentation
   - [ ] Give up on this feature entirely
   - [ ] Fall back to lower-level implementation

### Learning Objective

**Aspirational APIs break machine trust.**

- ❌ Bad: Document APIs that don't exist (yet)
- ✅ Good: Document current working API + note future plans

### Fix Simulation

```rust
// ⚠️ FUTURE API (Planned for v5.2) ⚠️
// The builder pattern API shown above is not yet implemented.
// Use the current API below:

// ✅ CURRENT IMPLEMENTATION (v5.0.0)
pub struct Guard {
    pub name: String,
    pub description: String,
    pub condition: String,
}

impl Guard {
    pub fn from_condition(name: &str, description: &str, condition: &str) -> Self {
        Guard {
            name: name.to_string(),
            description: description.to_string(),
            condition: condition.to_string(),
        }
    }
}

impl Template {
    fn preconditions() -> Vec<Guard> {
        vec![
            Guard::from_condition(
                "template_exists",
                "Template must be registered",
                "arg_exists:name"
            ),
        ]
    }
}
```

**Result**: ✅ Compiles with current v5.0.0 API

---

## Scenario 4: Schema Mismatch Runtime Error (FM-06)

### Setup

Reference documentation shows JSON introspection output:

```json
{
  "capabilities": [
    {
      "id": "pack.list",
      "name": "List Packs"
    }
  ]
}
```

Your code parses based on this schema:

```rust
#[derive(Deserialize)]
struct IntrospectionOutput {
    capabilities: Vec<Capability>,
}

#[derive(Deserialize)]
struct Capability {
    id: String,
    name: String,
}
```

### Task

1. Run actual CLI: `./myapp --introspect`
2. Observe actual output (different from docs)
3. Your parser fails at runtime
4. Debug the mismatch

### Actual CLI Output

```json
{
  "capabilities": [
    {
      "id": "pack.list",
      "name": "List Packs",
      "parameters": [],
      "guards": [],
      "effects": []
    }
  ],
  "meta": {
    "version": "5.0.0",
    "generated": "2025-11-20T10:00:00Z"
  }
}
```

### Expected Outcome

```rust
// Runtime error (not compile error!)
thread 'main' panicked at 'called `Result::unwrap()` on an `Err` value:
  missing field `parameters` at line 5 column 7'
```

### The Runtime Failure Problem

**Why this is worse than compile errors:**

1. **Silently wrong**: Code compiles successfully
2. **Late detection**: Fails only at runtime when CLI is invoked
3. **Partial failures**: Some fields parse, others panic
4. **Hard to debug**: Error message cryptic ("missing field")
5. **Production risk**: May fail in deployment, not in testing

### Questions for Reflection

1. **When did the failure actually occur?**
   - [ ] At compile time (caught early)
   - [ ] At runtime during parsing (caught late)
   - [ ] Never (worked in testing, failed in production)

2. **How could this be prevented?**
   - [ ] Integration test validating docs against actual CLI
   - [ ] Schema validation in CI
   - [ ] Runtime validation tests
   - [ ] All of the above

3. **What is the blast radius of this failure?**
   - [ ] Just this one agent (isolated)
   - [ ] All agents using this documentation (widespread)
   - [ ] Cascades to dependent systems (systemic)

### Learning Objective

**Schema drift causes runtime failures that are hard to debug.**

- ❌ Bad: Document schema once, let codebase evolve
- ✅ Good: Validate docs against actual output in CI

### Fix Simulation

```rust
// ✅ Integration test that catches schema drift
#[test]
fn test_introspection_schema_matches_docs() {
    let output = Command::new(env!("CARGO_BIN_EXE_myapp"))
        .arg("--introspect")
        .output()
        .expect("CLI failed");

    let actual: Value = serde_json::from_slice(&output.stdout)
        .expect("Invalid JSON");

    // Load documented schema
    let doc_schema = include_str!("../docs/schema/introspection.json");
    let documented: Value = serde_json::from_str(doc_schema)
        .expect("Documented schema invalid");

    assert_eq!(actual, documented, "Schema drift detected!");
}
```

**CI enforces**: Documentation must match reality.

---

## Scenario 5: Ambiguous Result Type (FM-02, FM-15)

### Setup

How-To guide shows:

```rust
async fn discover_capabilities() -> Result<()> {
    let output = Command::new("./myapp")
        .arg("--introspect")
        .output()?;
    Ok(())
}
```

### Task

1. Copy this function signature
2. Attempt to compile
3. Observe ambiguity error
4. Try to infer correct error type

### Expected Outcome

```
error[E0433]: failed to resolve: multiple types named `Result` in scope
help: consider specifying the full path:
  - std::result::Result<(), E>
  - std::io::Result<()>
  - anyhow::Result<()>
```

### The Type Inference Problem

**What the author meant:**
```rust
Result<(), Box<dyn std::error::Error>>
```

**What machines see:**
```rust
Result<()>  // ??? What's the error type?
```

**Possible interpretations:**
- `std::result::Result<(), std::io::Error>`
- `std::result::Result<(), Box<dyn Error>>`
- `anyhow::Result<()>`
- `eyre::Result<()>`

**Machine confusion**: Which one is correct?

### Questions for Reflection

1. **Why can't machines infer the error type?**
   - [ ] Multiple Result types in scope
   - [ ] Error type not specified
   - [ ] No contextual clues in code
   - [ ] All of the above

2. **What should documentation do?**
   - [ ] Always fully qualify Result types
   - [ ] Use type aliases consistently
   - [ ] Show complete error types
   - [ ] All of the above

3. **How does this affect pattern learning?**
   - [ ] Machines copy incomplete patterns
   - [ ] Propagates type ambiguity to other code
   - [ ] Creates compilation errors downstream
   - [ ] All of the above

### Learning Objective

**Type shortcuts hurt machines.**

- ❌ Bad: Use unqualified `Result<T>` shorthand
- ✅ Good: Always specify `Result<T, E>` explicitly

### Fix Simulation

```rust
// ✅ Fully qualified error type
use std::error::Error;

async fn discover_capabilities() -> Result<(), Box<dyn Error>> {
    let output = Command::new("./myapp")
        .arg("--introspect")
        .output()?;
    Ok(())
}

// OR use type alias
type AppResult<T> = Result<T, Box<dyn std::error::Error>>;

async fn discover_capabilities() -> AppResult<()> {
    let output = Command::new("./myapp")
        .arg("--introspect")
        .output()?;
    Ok(())
}
```

**Result**: ✅ No ambiguity, compiles cleanly

---

## Playground Summary: Key Learnings

| Scenario | Failure Mode | RPN | Machine Impact | Fix |
|----------|--------------|-----|----------------|-----|
| 1. Magic Attributes | FM-01 | 672 | 100% blocked | Show actual API |
| 2. Phantom Type | FM-04 | 640 | Cannot compile | Define types first |
| 3. Non-existent API | FM-03 | 672 | Wasted time | Mark as future/provide current |
| 4. Schema Drift | FM-06 | 567 | Runtime errors | CI validation |
| 5. Type Ambiguity | FM-02 | 672 | Compilation fails | Qualify types |

### Universal Lessons for Machines

1. **Never trust code until compiled** - Documentation may be wrong
2. **Verify schemas at runtime** - Docs drift from reality
3. **Distinguish aspirational from actual** - Not all documented APIs exist
4. **Require complete type information** - Shortcuts break compilation
5. **Maintain trust scores** - Update after each failure

### Recommended Machine Workflow

```
1. Read documentation
2. Extract code example
3. Attempt compilation
   ├─ Success → Trust+10%, proceed
   └─ Failure → Trust-30%, try next example
4. Run integration tests
   ├─ Pass → Trust+20%, use API
   └─ Fail → Trust-40%, report bug
5. Update trust score
   ├─ Trust > 50% → Continue learning
   └─ Trust ≤ 50% → Flag for human review
```

**Success criteria**: Trust maintained above 70% throughout learning path.
