# Explanation: Agent Architecture Patterns

**Purpose**: Understand how MCP agents generate CLIs through RDF-driven architecture

## Multi-Agent Workflow

### The Agent Pipeline

```
┌─────────────────────────────────────────────────────────────┐
│ Agent 1: Discovery Agent                                     │
│ Role: Discover what commands are needed                      │
│ Tool: QueryCapabilities (list all verbs)                    │
│ Output: CommandList                                          │
└─────────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────────┐
│ Agent 2: Validation Agent                                    │
│ Role: Ensure ontology is correct                             │
│ Tools: QueryCapabilities (validation queries)                │
│ Output: ValidationReport                                     │
└─────────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────────┐
│ Agent 3: Code Generation Agent                               │
│ Role: Generate production Rust code                          │
│ Tool: GenerateCliFromTurtle                                  │
│ Output: GeneratedCli (code + diagnostics)                    │
└─────────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────────┐
│ Agent 4: Deployment Agent                                    │
│ Role: Compile and deploy                                     │
│ Tools: Compile (cargo), Deploy (systemd/k8s)                │
│ Output: DeploymentResult (URL, endpoint)                     │
└─────────────────────────────────────────────────────────────┘
```

## Agent Specialization

### Discovery Agent

**Purpose**: Understand what CLI operations are needed

**Workflow**:
```rust
pub struct DiscoveryAgent;

impl DiscoveryAgent {
    pub async fn discover(&self, ontology: &ParsedTurtle) -> Result<DiscoveryResult> {
        let executor = SparqlExecutor::new(ontology)?;

        // Query 1: How many nouns (command categories)?
        let nouns = executor.execute_query(
            "SELECT DISTINCT ?noun WHERE { ?noun a cnv:Noun }"
        )?;

        // Query 2: How many verbs (total commands)?
        let verbs = executor.execute_query(
            "SELECT DISTINCT ?verb WHERE { ?verb a cnv:Verb }"
        )?;

        // Query 3: Commands per noun distribution?
        let distribution = executor.execute_query(
            "SELECT ?noun (COUNT(?verb) as ?count) WHERE {
                ?noun a cnv:Noun .
                ?verb a cnv:Verb ; cnv:hasNoun ?noun .
            } GROUP BY ?noun"
        )?;

        Ok(DiscoveryResult {
            noun_count: nouns.len(),
            verb_count: verbs.len(),
            distribution,
        })
    }
}
```

**Decisions made**:
- Is ontology suitable for code generation?
- What's the complexity (1000s of verbs)?
- Which nouns are most important?

### Validation Agent

**Purpose**: Ensure ontology is correct before generation

**Validation queries**:
```sparql
# Check 1: All verbs have nouns
SELECT ?verb WHERE {
    ?verb a cnv:Verb .
    FILTER NOT EXISTS { ?verb cnv:hasNoun ?n }
}

# Check 2: All noun references exist
SELECT ?verb WHERE {
    ?verb cnv:hasNoun ?noun .
    FILTER NOT EXISTS { ?noun a cnv:Noun }
}

# Check 3: Names are valid Rust identifiers
SELECT ?name WHERE {
    ?r cnv:name ?name .
    FILTER (CONTAINS(?name, " ") || CONTAINS(?name, "-"))
}

# Check 4: No duplicate verbs per noun
SELECT ?noun ?name (COUNT(?verb) as ?count) WHERE {
    ?noun a cnv:Noun .
    ?verb cnv:hasNoun ?noun ; cnv:name ?name .
} GROUP BY ?noun ?name HAVING (?count > 1)
```

**Output**: ValidationReport with errors/warnings

### Code Generation Agent

**Purpose**: Transform RDF into production Rust code

**Process**:
```rust
pub struct CodeGenerationAgent {
    generator: CliCodeGenerator,
}

impl CodeGenerationAgent {
    pub async fn generate(&self, ontology: &ParsedTurtle) -> Result<GenerationResult> {
        // Step 1: Generate code
        let generated = self.generator.generate_from_ontology(ontology)?;

        // Step 2: Verify syntax
        syn::parse_file(generated.rust_code())?;

        // Step 3: Check diagnostics
        if generated.diagnostics().iter().any(|d| d.level == "error") {
            return Err("Generation had errors".into());
        }

        // Step 4: Estimate compilation time
        let estimate = self.estimate_compile_time(generated.verb_count())?;

        Ok(GenerationResult {
            code: generated.rust_code().to_string(),
            verb_count: generated.verb_count(),
            estimated_compile_seconds: estimate,
        })
    }
}
```

**Output**: GeneratedCli (code, line count, diagnostics)

### Deployment Agent

**Purpose**: Get code from generated to production

**Deployment process**:
```rust
pub struct DeploymentAgent;

impl DeploymentAgent {
    pub async fn deploy(&self, code: &str, target: &str) -> Result<DeploymentResult> {
        // Step 1: Write code to file
        std::fs::write("src/generated.rs", code)?;

        // Step 2: Compile
        let compile_output = std::process::Command::new("cargo")
            .args(&["build", "--release"])
            .output()?;

        if !compile_output.status.success() {
            return Err(format!("Compilation failed: {}", String::from_utf8_lossy(&compile_output.stderr)));
        }

        // Step 3: Test
        let test_output = std::process::Command::new("cargo")
            .args(&["test", "--release"])
            .output()?;

        // Step 4: Deploy based on target
        match target {
            "systemd" => self.deploy_systemd()?,
            "kubernetes" => self.deploy_kubernetes()?,
            "docker" => self.deploy_docker()?,
            _ => return Err(format!("Unknown target: {}", target)),
        }

        Ok(DeploymentResult {
            success: true,
            endpoint: "https://api.example.com/cli".to_string(),
            version: get_version()?,
        })
    }
}
```

## Communication Patterns

### Agent-to-Agent Communication

Agents communicate through standardized interfaces:

```rust
// Discovery Agent output → Validation Agent input
pub struct DiscoveryResult {
    pub noun_count: usize,
    pub verb_count: usize,
    pub distribution: Vec<(String, usize)>,
}

// Validation Agent output → Code Generation Agent input
pub struct ValidationReport {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

// Code Generation Agent output → Deployment Agent input
pub struct GenerationResult {
    pub code: String,
    pub verb_count: usize,
    pub estimated_compile_seconds: u64,
}
```

### MCP Tool Communication

Agents invoke tools through MCP protocol:

```
Agent → MCP Server → Tool → Result → Agent

Example workflow:
1. Agent calls: GenerateCliFromTurtle("@prefix cnv: ...")
2. Tool executes: Parse → Validate → Generate
3. Result returned: GeneratedCli { code: "...", diagnostics: [] }
4. Agent processes: Writes file, compiles, deploys
```

## Orchestration Patterns

### Sequential Orchestration

Simple left-to-right pipeline:

```
Discovery → Validation → Generation → Deployment
```

**Pros**: Simple, easy to debug
**Cons**: If one step fails, all downstream blocked

### Parallel Orchestration

Run independent checks in parallel:

```
    ┌─ Discovery ─┐
    │             ├─→ Generation
    └─ Validation ┘
        ↓
    Deployment
```

**Pros**: Faster overall time
**Cons**: More complex coordination

### Conditional Orchestration

Skip steps based on results:

```
Discovery → Check size
    ├─ Large (1000+ verbs) → Warn, ask agent
    ├─ Medium (100-999 verbs) → Proceed
    └─ Small (<100 verbs) → Proceed fast
```

## Error Handling

### Cascade Errors

Failure in one agent triggers recovery:

```rust
match discovery_agent.discover().await {
    Ok(result) if result.verb_count > 1000 => {
        // Ask for confirmation
        agent.ask_user("Generate 1000+ commands? (slow)").await?;
    }
    Ok(result) => {
        // Continue to validation
        validation_agent.validate().await?;
    }
    Err(e) => {
        // Stop, report error
        eprintln!("Discovery failed: {}", e);
        return;
    }
}
```

### Graceful Degradation

Continue with warnings:

```rust
match code_gen_agent.generate().await {
    Ok(generated) if !generated.diagnostics().is_empty() => {
        eprintln!("⚠️ Warnings during generation:");
        for diag in generated.diagnostics() {
            eprintln!("  {}: {}", diag.level, diag.message);
        }
        // Continue anyway
        deployment_agent.deploy(generated.code()).await?;
    }
    // ...
}
```

## State Management

### Immutable Pipeline State

```rust
pub struct PipelineState {
    pub ontology: Arc<ParsedTurtle>,
    pub discovery_result: DiscoveryResult,
    pub validation_result: ValidationReport,
    pub generated_code: GeneratedCli,
}

// Each agent receives and returns state
impl DiscoveryAgent {
    pub async fn run(
        state: PipelineState,
    ) -> Result<PipelineState> {
        let mut state = state;
        state.discovery_result = self.discover(&state.ontology).await?;
        Ok(state)
    }
}
```

### Checkpoint Strategy

Save state between agents:

```rust
pub async fn run_pipeline_with_checkpoints(
    ontology_path: &str,
) -> Result<()> {
    // Checkpoint 1: Discovery
    let discovery = discovery_agent.discover(ontology_path).await?;
    save_checkpoint("discovery", &discovery)?;

    // Checkpoint 2: Validation
    let validation = validation_agent.validate(&discovery).await?;
    save_checkpoint("validation", &validation)?;

    // Can resume from checkpoint if needed
    if validation.valid {
        // Checkpoint 3: Generation
        let generated = code_gen_agent.generate().await?;
        save_checkpoint("generation", &generated)?;
    }

    Ok(())
}
```

## Monitoring and Observability

### Agent Health Checks

```rust
pub struct AgentHealthCheck {
    pub name: String,
    pub status: HealthStatus,
    pub last_execution_ms: u64,
    pub error_count: usize,
    pub success_rate: f64,
}

// Monitor agent performance
impl AgentMonitor {
    pub async fn check_health(&self) -> Vec<AgentHealthCheck> {
        vec![
            AgentHealthCheck {
                name: "DiscoveryAgent".to_string(),
                status: HealthStatus::Healthy,
                last_execution_ms: 42,
                error_count: 0,
                success_rate: 1.0,
            },
            // ... other agents
        ]
    }
}
```

### Tracing and Logging

```rust
pub async fn run_with_tracing(
    ontology_path: &str,
) -> Result<()> {
    // Use OpenTelemetry for distributed tracing
    let tracer = global::tracer("cli-generator");

    let span = tracer.start("pipeline");
    debug!("Starting pipeline with {}", ontology_path);

    let discovery = discovery_agent.discover().await?;
    debug!("Discovery complete: {} verbs", discovery.verb_count);

    // ... continue
    span.end();
    Ok(())
}
```

## Real-World Example

Complete agent workflow:

```rust
pub async fn agent_driven_cli_generation(
    ontology_path: &str,
) -> Result<DeploymentResult> {
    // 1. Discovery Agent
    let discovery = DiscoveryAgent::new()
        .discover_from_file(ontology_path)
        .await?;

    println!("📊 Found {} verbs in {} nouns",
        discovery.verb_count, discovery.noun_count);

    // 2. Validation Agent
    let validation = ValidationAgent::new()
        .validate(&discovery)
        .await?;

    if !validation.valid {
        eprintln!("❌ Ontology validation failed:");
        for error in validation.errors {
            eprintln!("  {}", error);
        }
        return Err("Validation failed".into());
    }

    println!("✅ Ontology validated");

    // 3. Code Generation Agent
    let generated = CodeGenerationAgent::new()
        .generate_from_file(ontology_path)
        .await?;

    println!("✅ Generated {} lines of code",
        generated.code.lines().count());

    // 4. Deployment Agent
    let deployed = DeploymentAgent::new()
        .deploy(&generated.code, "kubernetes")
        .await?;

    println!("🚀 Deployed to {}", deployed.endpoint);

    Ok(deployed)
}
```

---

**Related**:
- [How-to: Integrate with Agents](../howto/agent-integration.md)
- [Tutorial 5: Deploy Production CLIs](../tutorials/tutorial-5-deployment.md)
- [Explanation: Design Patterns for CLIs](design-patterns.md)
