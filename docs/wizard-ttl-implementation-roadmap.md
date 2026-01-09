# Wizard from TTL: Implementation Roadmap

## Vision

Transform Turtle RDF specifications into production-ready AI-powered CLI wizards with zero boilerplate.

```
my-wizard.ttl → (ggen) → wizard_handlers.rs → cargo build → CLI with AI
```

## Executive Summary

| Phase | Focus | Duration | Status |
|-------|-------|----------|--------|
| **Phase 1** | Foundation & Ontology | 1 week | 📋 Planned |
| **Phase 2** | Parser & Codegen | 1 week | 📋 Planned |
| **Phase 3** | Templates & Examples | 1 week | 📋 Planned |
| **Phase 4** | Testing & Polish | 1 week | 📋 Planned |

## Phase 1: Foundation & Ontology (Week 1)

### Deliverables

- **WizardOntology Extension** (YAML/RDF)
  - Properties for system prompts, temperature, timeout
  - Model configuration (primary + fallback)
  - Prompt sequence definition
  - Error handling strategies
  - Validator specifications

- **WizardOntologyParser** (Rust)
  - Parse RDF graph into WizardOntology struct
  - Validate ontology constraints
  - Convert RDF properties to structured data

### Implementation

```rust
// File: src/ggen_integration/wizard_ontology.rs

pub struct WizardOntology {
    pub verb_name: String,
    pub noun_name: String,
    pub description: String,
    pub system_prompt: String,
    pub temperature: f32,
    pub timeout: Duration,
    pub primary_model: ModelSpec,
    pub fallback_models: Vec<ModelSpec>,
    pub fallback_strategy: FallbackStrategy,
    pub prompts: Vec<PromptStep>,
    pub error_handling: ErrorHandling,
    pub features: WizardFeatures,
}

pub struct PromptStep {
    pub step_number: usize,
    pub prompt: String,
    pub description: String,
    pub validator: Option<ValidatorSpec>,
    pub stores_variable: String,
    pub use_history: bool,
    pub is_final: bool,
}

impl WizardOntology {
    pub fn from_rdf(graph: &RdfGraph, verb_iri: &str) -> Result<Self> {
        // Parse RDF statements
        // Extract wizard properties
        // Build structured ontology
        // Validate required fields
    }
}
```

### Tasks

- [ ] Design WizardOntology struct
- [ ] Implement RDF property extraction
- [ ] Add validation rules
- [ ] Write unit tests
- [ ] Document ontology schema

### Success Criteria

- ✅ Can parse example TTL files
- ✅ All required properties present
- ✅ Ontology validates correctly
- ✅ Error messages are clear

---

## Phase 2: Parser & Code Generator (Week 2)

### Deliverables

- **WizardCodeGenerator** (Rust)
  - Convert WizardOntology → Rust code
  - Generate handler functions
  - Integrate with template system

- **Templates** (Tera)
  - Main wizard handler template
  - Validation function templates
  - Fallback strategy templates

### Implementation

```rust
// File: src/ggen_integration/wizard_codegen.rs

pub struct WizardCodeGenerator {
    tera: tera::Tera,
}

impl WizardCodeGenerator {
    pub fn new() -> Result<Self> {
        let tera = tera::Tera::new("src/ggen_integration/templates/**/*.tera")?;
        Ok(Self { tera })
    }

    pub fn generate(&self, ontology: &WizardOntology) -> Result<String> {
        let mut context = tera::Context::new();
        context.insert("verb_name", &ontology.verb_name);
        context.insert("noun_name", &ontology.noun_name);
        context.insert("system_prompt", &ontology.system_prompt);
        context.insert("temperature", &ontology.temperature);
        context.insert("prompts", &ontology.prompts);
        // ... more properties

        self.tera.render("wizard_handler.rs.tera", &context)
    }
}

pub struct UnifiedGenerator;

impl UnifiedGenerator {
    pub async fn generate_from_turtle(
        ttl_path: &Path,
        output_dir: &Path,
    ) -> Result<GenerationReceipt> {
        // 1. Parse TTL file
        let rdf = parse_turtle(ttl_path)?;

        // 2. Extract all verbs from RDF
        let verbs = extract_verbs(&rdf)?;

        // 3. For each verb, determine if it's a wizard or CLI
        let mut code_outputs = Vec::new();

        for verb_iri in verbs {
            if is_wizard_verb(&rdf, &verb_iri)? {
                // Generate wizard code
                let ontology = WizardOntology::from_rdf(&rdf, &verb_iri)?;
                let code = WizardCodeGenerator::new()?.generate(&ontology)?;
                code_outputs.push((format!("{}_wizard.rs", ontology.verb_name), code));
            } else {
                // Generate CLI code (existing)
                let cli_spec = extract_cli_spec(&rdf, &verb_iri)?;
                let code = CliCodeGenerator::new()?.generate(&cli_spec)?;
                code_outputs.push((format!("{}_cli.rs", cli_spec.name), code));
            }
        }

        // 4. Write all outputs
        for (filename, code) in code_outputs {
            let path = output_dir.join(&filename);
            std::fs::write(path, code)?;
        }

        Ok(GenerationReceipt { /* ... */ })
    }
}
```

### Tasks

- [ ] Implement WizardCodeGenerator struct
- [ ] Create template context builder
- [ ] Write Tera templates
- [ ] Implement template filters (lowercase, join, etc.)
- [ ] Test code generation with examples

### Success Criteria

- ✅ Generated code compiles
- ✅ Generated code matches expected format
- ✅ All template variables render correctly
- ✅ <100ms generation time

---

## Phase 3: Templates & Examples (Week 3)

### Deliverables

- **Handler Template** (`wizard_handler.rs.tera`)
  - Wizard initialization
  - Prompt loop
  - History management
  - Error handling

- **Validation Templates** (`validators.rs.tera`)
  - NonEmpty validator
  - MinLength validator
  - Choice validator
  - Boolean validator
  - Custom validators

- **Examples** (`examples/`)
  - Project setup wizard
  - Database configuration wizard
  - Multi-turn conversation
  - Advanced features demo

### Example Output

**Input**: `project-setup.ttl`

**Generated**: `handlers/create_project_wizard.rs`

```rust
#[verb(name = "create", noun = "project")]
async fn create_project(ctx: AppContext) -> Result<(), Box<dyn std::error::Error>> {
    println!("🧙 Create a new project\n");

    let config = WizardConfig::from_env()?
        .with_temperature(0.7)
        .with_timeout(Duration::from_secs(30));

    let fallback = FallbackConfig::new(ModelConfig::Claude3Opus)
        .with_fallback(ModelConfig::Claude3Sonnet)
        .with_fallback(ModelConfig::Gpt4Turbo)
        .with_strategy(SelectionStrategy::CostOptimized);

    let mut wizard = WizardBuilder::new()
        .with_config(config)
        .with_fallback(fallback)
        .build()
        .await?;

    // ... step 1: project name
    // ... step 2: description
    // ... step 3: features
    // ... step 4: language
    // ... step 5: confirmation

    println!("\n✨ Project created!");
    Ok(())
}
```

### Tasks

- [ ] Complete wizard_handler.rs.tera
- [ ] Create validators.rs.tera
- [ ] Create fallback.rs.tera
- [ ] Create error_handling.rs.tera
- [ ] Create 3+ working examples
- [ ] Test examples end-to-end

### Success Criteria

- ✅ Generated code compiles without errors
- ✅ Examples work with real TTL specs
- ✅ Output matches expected wizard flow
- ✅ Streaming works (if enabled)
- ✅ Fallback works (if enabled)

---

## Phase 4: Testing & Polish (Week 4)

### Deliverables

- **Unit Tests**
  - Ontology parsing tests
  - Code generation tests
  - Template rendering tests

- **Integration Tests**
  - Full TTL → code → binary pipeline
  - Example execution tests

- **Documentation**
  - Ontology schema reference
  - Generation quickstart
  - Template customization guide
  - Troubleshooting guide

- **CLI Tool** (optional)
  ```bash
  cargo install clap-noun-verb-ggen

  # Use it
  clap-noun-verb-ggen \
    --input wizard-specs.ttl \
    --output src/handlers \
    --format rust
  ```

### Tasks

- [ ] Write comprehensive unit tests
- [ ] Create integration test suite
- [ ] Generate documentation
- [ ] Create CLI wrapper
- [ ] Performance profiling
- [ ] Security audit
- [ ] Create tutorial

### Success Criteria

- ✅ 90%+ test coverage
- ✅ All tests pass
- ✅ Generation <200ms
- ✅ No security issues
- ✅ Clear documentation
- ✅ Ready for public release

---

## Technical Architecture

### Integration Points

```
                      Turtle Files (.ttl)
                             ↓
                    ┌─────────────────┐
                    │  RDF Parser     │ (oxigraph)
                    └─────────────────┘
                             ↓
                    ┌─────────────────┐
                    │  Extractor      │
                    │  (CLI/Wizard)   │
                    └─────────────────┘
                          /    \
                         /      \
            ┌────────────────┐  ┌────────────────┐
            │ CLI Spec       │  │ Wizard Ontology│
            └────────────────┘  └────────────────┘
                    ↓                    ↓
            ┌────────────────┐  ┌────────────────┐
            │ CLI Codegen    │  │ Wizard Codegen │
            │ (existing)     │  │ (NEW)          │
            └────────────────┘  └────────────────┘
                    ↓                    ↓
            ┌────────────────┐  ┌────────────────┐
            │ cli.rs         │  │ wizard.rs      │
            └────────────────┘  └────────────────┘
                    └────────────┬──────────────┘
                                 ↓
                         ┌──────────────────┐
                         │ Integrated Rust  │
                         │ Project Output   │
                         └──────────────────┘
```

### File Structure

```
src/ggen_integration/
├── mod.rs (existing)
├── parser.rs (existing)
├── codegen.rs (existing)
├── ast.rs (existing)
├── error.rs (existing)
│
├── wizard_ontology.rs (NEW)
│   └── WizardOntology struct
│   └── PromptStep struct
│   └── ModelSpec struct
│
├── wizard_parser.rs (NEW)
│   └── WizardOntologyParser
│   └── Verb/Prompt extraction
│
├── wizard_codegen.rs (NEW)
│   └── WizardCodeGenerator
│   └── Template context building
│
├── unified_generator.rs (NEW)
│   └── UnifiedGenerator (CLI + Wizard)
│
└── templates/ (NEW)
    ├── wizard_handler.rs.tera
    ├── validators.rs.tera
    ├── fallback.rs.tera
    └── error_handling.rs.tera

examples/
├── wizard-specs.ttl (NEW)
└── Generated output examples
```

---

## Dependencies

All already in `Cargo.toml`:

- `oxigraph` (0.5.1) - RDF parsing
- `oxrdf` (0.2) - RDF types
- `serde` (1.0) - Serialization
- `serde_json` (1.0) - JSON
- `tera` (1.0+) - Template rendering
- `thiserror` (1.0) - Error handling

**No new dependencies required!**

---

## Success Metrics

### Quantitative

- [ ] Parse TTL in <100ms
- [ ] Generate code in <100ms
- [ ] Generated code compiles in <2s
- [ ] 90%+ test coverage
- [ ] Generated code ~100 LOC per prompt step
- [ ] Support 100+ step wizard

### Qualitative

- [ ] Generated code is readable
- [ ] Generated code follows patterns
- [ ] Documentation is clear
- [ ] Examples are comprehensive
- [ ] Errors are helpful
- [ ] Ready for production use

---

## Risk Mitigation

### Risk: Overly Complex TTL Specs

**Mitigation:** Strict ontology schema validation, helpful error messages

### Risk: Generated Code Quality

**Mitigation:** Templates tested thoroughly, code review before publication

### Risk: Performance

**Mitigation:** Benchmark generation performance, optimize hot paths

### Risk: User Adoption

**Mitigation:** Clear documentation, practical examples, easy CLI tool

---

## Next Steps

1. **Approval** - Get stakeholder buy-in
2. **Phase 1 Kickoff** - Start ontology design
3. **Weekly Reviews** - Track progress
4. **Community Feedback** - Get user input during development
5. **Release** - Publish with v2.1 or v3.0

---

## Related Work

- **ggen** (parent project) - Ontology-driven code generation
- **clap-noun-verb** (this project) - Noun-verb CLI framework
- **Turtle/RDF** (W3C standards) - Semantic web
- **SHACL** - Shape validation for RDF
- **OWL** - Web Ontology Language

## Questions?

- How should TTL ontology be extended for new features?
- Should we support templates beyond Tera?
- How to handle custom validators in TTL?
- Should we generate tests too?
- Should we support multiple output formats?

---

**Document Status:** Design Phase
**Last Updated:** 2026-01-09
**Next Review:** 2026-01-16
