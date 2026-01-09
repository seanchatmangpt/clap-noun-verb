# Wizard Generation from TTL: Design Proposal

## Overview

Extend the existing ggen integration to generate type-safe Wizard CLI applications from Turtle RDF specifications. This enables **declarative specification of AI-powered interactive CLI workflows**.

## Architecture

### Current State

```
Turtle Spec → ggen-integration (Parser → AST → Codegen) → Rust CLI Code
```

### Extended State (Wizard Generation)

```
Turtle Spec
    ↓
Parser (RDF + Ontology)
    ├─→ CLI Generation (existing)
    ├─→ Wizard Generation (NEW)
    │   ├─ Session Config
    │   ├─ Prompt Templates
    │   ├─ Fallback Chains
    │   └─ Error Handling
    └─→ Integration
        └─ Generated CLI + Wizards
```

## Ontology Extension

### New TTL Properties for Wizards

```turtle
@prefix cli: <http://clap-noun-verb.io/ontology#> .
@prefix wiz: <http://clap-noun-verb.io/wizard#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

# Define a wizard-enabled verb
cli:CreateProject a cli:Verb ;
    wiz:hasWizard true ;
    wiz:wizardMode wiz:Interactive ;
    wiz:systemPrompt "You are a helpful project generator" ;
    wiz:temperature 0.7 ;
    wiz:defaultModel wiz:Claude3Opus .

# Define wizard prompts for each step
cli:CreateProject wiz:hasPromptSequence (
    [ wiz:step 1 ;
      wiz:prompt "What is your project name?" ;
      wiz:validator wiz:NonEmpty ;
      wiz:stores [ wiz:variable "project_name" ] ;
    ]
    [ wiz:step 2 ;
      wiz:prompt "Describe your project (1-2 sentences)" ;
      wiz:validator wiz:NotEmpty ;
      wiz:stores [ wiz:variable "description" ] ;
    ]
) .

# Define model fallback chain
cli:CreateProject wiz:primaryModel wiz:Claude3Opus ;
    wiz:fallbackModels (
        wiz:Claude3Sonnet
        wiz:Gpt4Turbo
        wiz:GeminiPro
    ) ;
    wiz:fallbackStrategy wiz:CostOptimized .

# Define error handling
cli:CreateProject wiz:onRateLimit wiz:RetryWithBackoff ;
    wiz:maxRetries 3 ;
    wiz:retryBackoff wiz:Exponential ;
    wiz:jitterFactor 0.2 .
```

## Implementation Components

### 1. WizardOntology Parser

**File**: `src/ggen_integration/wizard_parser.rs`

```rust
pub struct WizardOntology {
    pub system_prompt: String,
    pub temperature: f32,
    pub primary_model: ModelConfig,
    pub fallback_models: Vec<ModelConfig>,
    pub prompts: Vec<WizardPromptSpec>,
    pub error_handling: ErrorHandlingSpec,
}

impl WizardOntology {
    pub fn from_turtle(rdf: &RdfGraph) -> Result<Self> {
        // Parse wizard:hasWizard property
        // Extract wiz:systemPrompt, wiz:temperature, etc.
        // Build structured WizardOntology
    }
}
```

### 2. WizardCodeGenerator

**File**: `src/ggen_integration/wizard_codegen.rs`

```rust
pub struct WizardCodeGenerator;

impl WizardCodeGenerator {
    pub fn generate(&self, ontology: &WizardOntology, verb_name: &str) -> Result<String> {
        // Generate:
        // 1. WizardConfig initialization
        // 2. Prompt builders for each step
        // 3. Error handling code
        // 4. Integration with CLI handler

        let template = include_str!("templates/wizard_handler.rs.tera");
        tera.render_str(template, &context)
    }
}
```

### 3. Unified Pipeline

**File**: `src/ggen_integration/unified_generator.rs`

```rust
pub struct UnifiedGgenGenerator;

impl UnifiedGgenGenerator {
    pub async fn generate_from_turtle(
        ttl_path: &Path,
        output_dir: &Path,
    ) -> Result<GenerationReceipt> {
        // 1. Parse TTL
        let rdf = parse_turtle(ttl_path)?;

        // 2. Extract CLI specs
        let cli_specs = extract_cli_specs(&rdf)?;

        // 3. Extract wizard specs
        let wizard_specs = extract_wizard_specs(&rdf)?;

        // 4. Generate CLI code (existing)
        let cli_code = CliCodeGenerator::generate(&cli_specs)?;

        // 5. Generate wizard handlers (NEW)
        let wizard_code = WizardCodeGenerator::generate(&wizard_specs)?;

        // 6. Integrate and write
        self.integrate_and_write(&cli_code, &wizard_code, output_dir)?;

        Ok(receipt)
    }
}
```

## Generated Code Example

### Input TTL

```turtle
cli:SetupProject a cli:Verb ;
    wiz:hasWizard true ;
    wiz:systemPrompt "Guide the user through project setup" ;
    wiz:primaryModel wiz:Claude3Opus ;
    wiz:fallbackModels ( wiz:Claude3Sonnet wiz:Gpt4Turbo ) .

cli:SetupProject wiz:hasPromptSequence (
    [ wiz:step 1 ; wiz:prompt "Project name?" ]
    [ wiz:step 2 ; wiz:prompt "Description?" ]
    [ wiz:step 3 ; wiz:prompt "Main features?" ]
) .
```

### Generated Rust Code

```rust
use clap_noun_verb::wizard::{
    WizardBuilder, WizardConfig, Prompt, FallbackConfig, ModelConfig
};

#[verb(name = "setup", noun = "project")]
async fn setup_project(ctx: AppContext) -> Result<(), Box<dyn std::error::Error>> {
    println!("🧙 Project Setup Wizard\n");

    // Configuration generated from TTL
    let config = WizardConfig::from_env()?
        .with_temperature(0.7);

    let fallback = FallbackConfig::new(ModelConfig::Claude3Opus)
        .with_fallback(ModelConfig::Claude3Sonnet)
        .with_fallback(ModelConfig::Gpt4Turbo)
        .with_strategy(SelectionStrategy::CostOptimized);

    let mut wizard = WizardBuilder::new()
        .with_config(config)
        .with_fallback(fallback)
        .build()
        .await?;

    // Wizard flow generated from TTL
    let mut responses = Vec::new();

    // Step 1
    let prompt1 = Prompt::new("Project name?")
        .with_system("Guide the user through project setup");
    let resp1 = wizard.generate(prompt1).await?;
    responses.push(("project_name", resp1.text.clone()));

    // Step 2
    let prompt2 = Prompt::new("Description?")
        .with_history(vec![Message {
            role: Role::Assistant,
            content: resp1.text,
        }]);
    let resp2 = wizard.generate(prompt2).await?;
    responses.push(("description", resp2.text.clone()));

    // Step 3
    let prompt3 = Prompt::new("Main features?")
        .with_history(/* accumulated history */);
    let resp3 = wizard.generate(prompt3).await?;
    responses.push(("features", resp3.text.clone()));

    // Output results
    println!("\n✨ Project configuration:");
    for (key, value) in responses {
        println!("{}: {}", key, value);
    }

    Ok(())
}
```

## Implementation Phases

### Phase 1: Foundation (Week 1)

- [ ] Design WizardOntology extension to existing ontology
- [ ] Create WizardOntologyParser from RDF graphs
- [ ] Add WizardCodeGenerator with basic templates
- [ ] Create integration with existing ggen pipeline

**Deliverables:**
- Ontology schema documentation
- Parser implementation
- Basic code generation

### Phase 2: Template System (Week 2)

- [ ] Create wizard handler templates (Tera)
- [ ] Support prompt sequences with history
- [ ] Generate error handling code
- [ ] Add model fallback generation

**Deliverables:**
- Complete template library
- Generated handler code examples

### Phase 3: Advanced Features (Week 3)

- [ ] Validator generation (TTL → Rust validators)
- [ ] Dynamic variable storage
- [ ] Configuration serialization
- [ ] Testing framework

**Deliverables:**
- End-to-end generated wizard with validation
- Test generation for wizards

### Phase 4: Polish (Week 4)

- [ ] Performance optimization
- [ ] Documentation and examples
- [ ] Integration tests
- [ ] Release and publication

**Deliverables:**
- Production-ready wizard generator
- Comprehensive documentation

## Code Structure

```
src/ggen_integration/
├── mod.rs (existing)
├── parser.rs (existing)
├── codegen.rs (existing)
├── ast.rs (existing)
├── error.rs (existing)
├── wizard_ontology.rs (NEW)
├── wizard_parser.rs (NEW)
├── wizard_codegen.rs (NEW)
├── wizard_templates/ (NEW)
│   ├── handler.rs.tera
│   ├── prompt_sequence.rs.tera
│   ├── error_handling.rs.tera
│   └─── fallback.rs.tera
└── unified_generator.rs (NEW)
```

## Integration Points

### 1. With Existing ggen

- Reuse TurtleParser infrastructure
- Extend CodeGenerator trait
- Integrate with tera template system
- Leverage existing AST types

### 2. With Wizard Package

- Generate WizardBuilder calls
- Generate WizardConfig initialization
- Create FallbackConfig chains
- Handle error scenarios

### 3. With clap-noun-verb

- Generate `#[verb]` macro handlers
- Support context passing
- Integrate with output formatting
- Maintain noun-verb patterns

## Benefits

✅ **Declarative CLI Wizards** - Define in TTL, get generated Rust
✅ **Type Safety** - Invalid specs caught at generation time
✅ **Deterministic Output** - Same TTL → same code always
✅ **Maintainability** - Change TTL → regenerate handler
✅ **Best Practices** - Generated code follows patterns
✅ **Reduced Boilerplate** - No manual prompt wrangling

## Example Use Cases

### 1. Interactive Project Generator

```bash
cargo ttl-to-wizard project-setup.ttl --output src/handlers/
# Generates: setup_project_wizard.rs with full interactive flow
```

### 2. Configuration Wizard

```turtle
wiz:DatabaseSetupWizard
    wiz:step 1 "What database?" → stores db_type
    wiz:step 2 "Connection string?" → stores connection_string
    wiz:step 3 "Credentials?" → stores credentials
```

### 3. Multi-Provider AI Selection

```turtle
wiz:ChatWizard
    wiz:primaryModel wiz:Claude3Opus
    wiz:fallbackModels (wiz:Claude3Sonnet wiz:Gpt4Turbo)
    wiz:fallbackStrategy wiz:CostOptimized
```

## Research Questions

1. **Turtle Validation** - How to validate TTL specs before code gen?
2. **Template Customization** - How to let users customize generated code?
3. **Iterative Refinement** - How to regenerate only changed handlers?
4. **Type Generation** - Generate Rust types from TTL schemas?
5. **Testing** - Auto-generate tests from TTL specs?

## Related Standards

- [SHACL](https://www.w3.org/TR/shacl/) - Shape Constraint Language for RDF
- [Turtle](https://www.w3.org/TR/turtle/) - Terse RDF Triple Language
- [RDFS](https://www.w3.org/TR/rdf-schema/) - RDF Schema for ontologies
- [OWL](https://www.w3.org/TR/owl2-overview/) - Web Ontology Language

## Success Metrics

- [ ] Can generate a complete wizard from a 50-line TTL spec
- [ ] Generated code passes `cargo build` without errors
- [ ] Generated wizards work with v1 and v2 features
- [ ] <100ms code generation time
- [ ] <2KB generated code per prompt step
- [ ] 95%+ test coverage of generator

## Next Steps

1. **Design Review** - Get feedback on ontology extension
2. **Prototype** - Build Phase 1 foundation
3. **User Testing** - Test with real wizard specs
4. **Documentation** - Write end-to-end tutorial
5. **Release** - Publish with v2.1 or v3.0

---

This proposal builds on the existing ggen infrastructure while extending it specifically for wizard generation. The TTL ontology provides a declarative, maintainable way to specify complex wizard flows that would otherwise require substantial boilerplate Rust code.
