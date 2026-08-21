> Archived 2026-08-20: superseded/stale as of v26.8.20.

# GGen Quick Start

**For:** Developers extending cargo-cicd or creating new domain CLIs

## 5-Minute Setup

### Step 1: Extend Domain Ontology

Edit `ontology/cargo-cicd.ttl` to add your command:

```turtle
@prefix cnv: <http://clap-noun-verb.io/ontology#> .
@prefix cicd: <http://cargo-cicd.io/ontology#> .

# Add a new noun
cicd:MyDomainNoun a cnv:Noun ;
    cnv:hasNounName "my-domain"@en ;
    cnv:nounAbout "Brief description"@en ;
    cnv:hasVerbs cicd:MyVerbOne, cicd:MyVerbTwo .

# Add verbs under the noun
cicd:MyVerbOne a cnv:Verb ;
    cnv:hasVerbName "action-one"@en ;
    cnv:verbAbout "What this action does"@en ;
    cnv:returnType "String"@en ;
    cnv:outputFormat "json" .

cicd:MyVerbTwo a cnv:Verb ;
    cnv:hasVerbName "action-two"@en ;
    cnv:verbAbout "Another operation"@en ;
    cnv:returnType "Vec<Record>"@en ;
    cnv:outputFormat "table" ;
    cnv:hasArguments cicd:MyArg .

# Add arguments if needed
cicd:MyArg a cnv:Argument ;
    cnv:argName "input-file"@en ;
    cnv:argDescription "Path to input file"@en ;
    cnv:argType "String" ;
    cnv:isRequired true .
```

### Step 2: Validate Ontology

Check that it compiles and parses correctly:

```bash
# Verify Turtle syntax
rapper -i turtle ontology/cargo-cicd.ttl > /dev/null

# Or with Jena
jena.sh --version  # Must be >=4.0
```

### Step 3: Run Manufacturing Pipeline

```bash
# When ggen CLI is implemented:
ggen sync

# Or manually via SPARQL + templates:
# 1. Execute queries/cargo-cicd-commands.rq against ontology/
# 2. Feed results to Jinja2 template processor
# 3. Write artifacts to src/generated/, docs/generated/, tests/generated/
```

### Step 4: Integrate Generated Code

Update your main CLI builder:

```rust
// src/lib.rs or src/main.rs
use clap_noun_verb::CliBuilder;

fn build_cli() -> Result<clap::Command> {
    CliBuilder::new("cargo-cicd")
        // Include generated nouns
        .add_generated_nouns()  // From src/generated/nouns/
        // Manually register if not auto-discovered
        .register_noun("my-domain", MyDomainModule::describe)
        .build()
}
```

### Step 5: Verify Proof Gates

```bash
# Compile generated code
cargo check

# Run generated tests
cargo test --test 'target_*'

# Check documentation completeness
grep -r "///" tests/generated/ | wc -l

# Verify ontology consistency
ggen proof-gate check
```

## Common Workflows

### Add New Verb to Existing Noun

**File:** `ontology/cargo-cicd.ttl`

```turtle
cicd:TargetNoun a cnv:Noun ;
    cnv:hasVerbs ..., cicd:TargetNewVerbNoun .  # Add here

cicd:TargetNewVerbNoun a cnv:Verb ;
    cnv:hasVerbName "new-verb"@en ;
    cnv:verbAbout "Does something new"@en ;
    cnv:returnType "Result"@en ;
    cnv:outputFormat "json" .
```

**Then run:**
```bash
ggen stage trait_generation  # Regenerate verbs
ggen stage test_generation   # Regenerate tests
```

### Add CLI Argument to Verb

**File:** `ontology/cargo-cicd.ttl`

```turtle
cicd:MyVerb a cnv:Verb ;
    cnv:hasArguments cicd:NewArg1, cicd:NewArg2 .

cicd:NewArg1 a cnv:Argument ;
    cnv:argName "my-param"@en ;
    cnv:argType "String" ;
    cnv:isRequired true .

cicd:NewArg2 a cnv:Argument ;
    cnv:argName "optional-param"@en ;
    cnv:argType "Option<u32>" ;
    cnv:isRequired false ;
    cnv:shortForm "o"@en .  # Short flag
```

**Template variables available:**
```jinja
{% for arg in arguments %}
  - {{ arg.name }}      # "my-param"
  - {{ arg.type }}      # "String"
  - {{ arg.description }}
  - {{ arg.is_required }} # true/false
{% endfor %}
```

### Add Proof Gate

**File:** `ggen.toml`

```toml
[ggen.proof_gates]
my_custom_validation = {
    enabled = true,
    severity = "warning",  # "error" or "warning"
    check = "Custom rule: ..."
}
```

## Testing Generated Code

All generated tests follow **AAA (Arrange-Act-Assert)** pattern:

```rust
#[test]
fn test_target_show_success() {
    // ARRANGE: Set up inputs
    let input = create_test_input(/* args */);
    
    // ACT: Execute handler
    let result = handler_function(input).unwrap();
    
    // ASSERT: Verify behavior
    assert_eq!(result.success, true);
    assert!(result.message.contains("expected text"));
}
```

Generated tests include:
- ✅ Happy path (all valid arguments)
- ✅ Error cases (missing required args, invalid types)
- ✅ Output format compliance (JSON serialization)
- ✅ Performance (latency <100ms)
- ✅ Determinism (same input = same output)
- ✅ Integration (CLI builder compatibility)

Run them:
```bash
cargo test --test 'target_show_test'
cargo test --lib generated
cargo test generated -- --include-ignored  # All tests including slow ones
```

## Troubleshooting

### "Ontology parse error: Unknown namespace"

**Fix:** Check prefix declarations at top of `ontology/cargo-cicd.ttl`:

```turtle
@prefix cnv: <http://clap-noun-verb.io/ontology#> .
@prefix cicd: <http://cargo-cicd.io/ontology#> .
```

All used prefixes must be declared.

### "SPARQL query returned no results"

**Check:**
1. Verify ontology was loaded: `ggen status`
2. Check query syntax: `rapper -e queries/cargo-cicd-commands.rq`
3. Verify noun/verb definitions exist in ontology

### "Generated code doesn't compile"

**Check:**
1. Template variables match ontology: `{{ noun_name }}`
2. Rust types are valid: `{{ arg.type }}`
3. Proof gate "code_compilation" should catch this

### "Test fails: 'handler not implemented'"

**Expected!** Generated tests have `// TODO` stubs. Implement domain logic:

```rust
// File: src/generated/verbs/target/show.rs
pub fn execute(&self) -> Result<MyResult> {
    // TODO: Replace with real implementation
    Ok(MyResult::ok("Done"))
}
```

## Next Steps

- Read [`ggen-manufacturing-system.md`](ggen-manufacturing-system.md) for deep dive
- Explore [`queries/cargo-cicd-commands.rq`](../queries/cargo-cicd-commands.rq) to customize query
- Extend templates in [`templates/`](../templates/) for domain-specific output
- Check proof gates in [`ggen.toml`](../ggen.toml)

## Reference: Template Variables

Available in all Jinja2 templates:

| Variable | Type | Example | Used In |
|----------|------|---------|---------|
| `noun_name` | string | `"target"` | All |
| `noun_description` | string | `"Manage build targets..."` | All |
| `noun_struct_name` | PascalCase | `"TargetModule"` | Rust |
| `noun_name_title` | Title Case | `"Target"` | Rust doc comments |
| `verb_name` | kebab-case | `"show"` | All |
| `verb_description` | string | `"Display all..."` | All |
| `verb_name_title` | Title Case | `"Show"` | Rust doc comments |
| `verb_domain_struct` | PascalCase | `"TargetShowDomain"` | Rust |
| `return_type` | Rust type | `"String"` | Rust, Markdown |
| `return_type_name` | PascalCase | `"TargetShowResult"` | Rust |
| `output_format` | lowercase | `"json"` | All |
| `brief_description` | string | `"..."` | Rust doc comment |
| `arguments` | list | `[{...}, {...}]` | Rust, Tests |
| `full_command_name` | kebab-case | `"target show"` | Markdown, Tests |
| `generated_date` | ISO8601 | `"2026-06-02T14:30:00Z"` | Markdown |
| `ontology_version` | semver | `"26.6.1"` | Markdown |

## FAQ

**Q: Can I manually edit generated files?**

A: Yes, but they'll be overwritten on next `ggen sync`. For manual code, use a separate module and import from generated code.

**Q: How do I add custom proof gates?**

A: Edit `ggen.toml` and implement gate logic in your ggen runner (CLI or build script).

**Q: Can templates use custom filters?**

A: Jinja2 built-in filters are available (replace, upper, lower, title, etc.). Custom filters require ggen implementation.

**Q: How do I version the ontology?**

A: Update `owl:versionInfo` in `ontology/cargo-cicd.ttl`. Receipts will include the version.
