# How-to: Integrate with Agents

**Problem**: You need MCP agents to discover, query, and generate CLIs dynamically

**Solution**: Use the three MCP tools to enable full agent automation

## Agent Workflow Overview

```
Agent 1: Discovers what commands are needed
    ↓
[QueryCapabilities tool]
    ↓
Agent 2: Validates the ontology
    ↓
[QueryCapabilities tool with validation queries]
    ↓
Agent 3: Generates production CLI code
    ↓
[GenerateCliFromTurtle tool]
    ↓
Agent 4: Deploys and monitors
    ↓
[Monitor health checks and metrics]
```

## Step 1: Enable Agent Discovery

Agents can discover available commands using SPARQL:

```rust
pub async fn agent_discover_commands(
    ontology: &ParsedTurtle,
) -> Result<Vec<CommandInfo>, Box<dyn std::error::Error>> {
    let executor = SparqlExecutor::new(ontology)?;

    let query = r#"
    SELECT ?nounName ?verbName ?description WHERE {
        ?noun a cnv:Noun ; cnv:name ?nounName .
        ?verb a cnv:Verb ;
              cnv:hasNoun ?noun ;
              cnv:name ?verbName .
        OPTIONAL { ?verb cnv:description ?description }
    }
    ORDER BY ?nounName ?verbName
    "#;

    let results = executor.execute_query(query)?;

    let commands = results.into_iter()
        .filter_map(|r| {
            Some(CommandInfo {
                noun: r.get("nounName")?.clone(),
                verb: r.get("verbName")?.clone(),
                description: r.get("description").cloned(),
            })
        })
        .collect();

    Ok(commands)
}

pub struct CommandInfo {
    pub noun: String,
    pub verb: String,
    pub description: Option<String>,
}
```

## Step 2: Enable Agent Validation

Agents can validate ontologies before generation:

```rust
pub async fn agent_validate_before_generation(
    ontology: &ParsedTurtle,
) -> Result<ValidationResult, Box<dyn std::error::Error>> {
    let executor = SparqlExecutor::new(ontology)?;
    let mut result = ValidationResult {
        valid: true,
        errors: Vec::new(),
        warnings: Vec::new(),
    };

    // Check 1: All verbs have nouns
    let orphaned = executor.execute_query(
        "SELECT ?verb WHERE { ?verb a cnv:Verb . FILTER NOT EXISTS { ?verb cnv:hasNoun ?n } }"
    )?;

    if !orphaned.is_empty() {
        result.valid = false;
        result.errors.push("Found verbs without nouns".to_string());
    }

    // Check 2: All nouns have at least one verb
    let empty_nouns = executor.execute_query(
        "SELECT ?noun WHERE { ?noun a cnv:Noun . FILTER NOT EXISTS { ?v cnv:hasNoun ?noun } }"
    )?;

    if !empty_nouns.is_empty() {
        result.warnings.push(format!("Found {} nouns without verbs", empty_nouns.len()));
    }

    // Check 3: Valid Rust identifiers
    let bad_names = executor.execute_query(
        "SELECT ?name WHERE { ?r cnv:name ?name . FILTER (CONTAINS(?name, \" \")) }"
    )?;

    if !bad_names.is_empty() {
        result.valid = false;
        result.errors.push("Found names with spaces (invalid Rust identifiers)".to_string());
    }

    Ok(result)
}

pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}
```

## Step 3: Implement GenerateCliFromTurtle Tool

```rust
pub struct GenerateCliFromTurtleTool {
    parser: TurtleParser,
    generator: CliCodeGenerator,
}

impl GenerateCliFromTurtleTool {
    pub async fn execute(
        &self,
        turtle_definition: &str,
    ) -> Result<GenerateCliOutput, String> {
        // Step 1: Validate input
        if turtle_definition.is_empty() {
            return Err("Turtle definition cannot be empty".to_string());
        }

        // Step 2: Parse Turtle
        let ontology = self.parser.parse(turtle_definition)
            .map_err(|e| format!("Parse error: {}", e))?;

        // Step 3: Validate ontology
        ontology.validate_ontology()
            .map_err(|e| format!("Validation error: {}", e))?;

        // Step 4: Generate code
        let generated = self.generator.generate_from_ontology(&ontology)
            .map_err(|e| format!("Generation error: {}", e))?;

        // Step 5: Return output
        Ok(GenerateCliOutput {
            rust_code: generated.rust_code().to_string(),
            noun_count: generated.noun_count(),
            verb_count: generated.verb_count(),
            diagnostics: generated.diagnostics()
                .iter()
                .map(|d| DiagnosticOutput {
                    level: d.level.clone(),
                    message: d.message.clone(),
                })
                .collect(),
        })
    }
}

#[derive(Serialize)]
pub struct GenerateCliOutput {
    pub rust_code: String,
    pub noun_count: usize,
    pub verb_count: usize,
    pub diagnostics: Vec<DiagnosticOutput>,
}

#[derive(Serialize)]
pub struct DiagnosticOutput {
    pub level: String,
    pub message: String,
}
```

## Step 4: Implement QueryCapabilities Tool

```rust
pub struct QueryCapabilitiesTool {
    executor: SparqlExecutor,
}

impl QueryCapabilitiesTool {
    pub async fn execute(
        &self,
        operation: &str,
        query: Option<&str>,
    ) -> Result<QueryCapabilitiesOutput, String> {
        let results = match operation {
            "list_commands" => self.list_all_commands().await?,

            "find_verb" => {
                let verb_name = query.ok_or("find_verb requires verb name")?;
                self.find_verb(verb_name).await?
            }

            "describe" => {
                let resource = query.ok_or("describe requires resource name")?;
                self.describe_resource(resource).await?
            }

            "custom" => {
                let sparql = query.ok_or("custom requires SPARQL query")?;
                self.execute_custom_query(sparql).await?
            }

            _ => return Err(format!("Unknown operation: {}", operation)),
        };

        Ok(QueryCapabilitiesOutput {
            results,
            found: !results.is_empty(),
        })
    }

    async fn list_all_commands(&self) -> Result<Vec<String>, String> {
        let query = r#"
        SELECT DISTINCT ?cmd WHERE {
            ?noun a cnv:Noun ; cnv:name ?nounName .
            ?verb a cnv:Verb ; cnv:hasNoun ?noun ; cnv:name ?verbName .
            BIND(CONCAT(?nounName, " ", ?verbName) as ?cmd)
        }
        ORDER BY ?cmd
        "#;

        let results = self.executor.execute_query(query)
            .map_err(|e| e.to_string())?;

        Ok(results.into_iter()
            .filter_map(|r| r.get("cmd").cloned())
            .collect())
    }

    async fn find_verb(&self, verb_name: &str) -> Result<Vec<String>, String> {
        let query = format!(r#"
        SELECT ?nounName WHERE {{
            ?noun a cnv:Noun ; cnv:name ?nounName .
            ?verb a cnv:Verb ; cnv:hasNoun ?noun ; cnv:name "{}" .
        }}
        "#, verb_name);

        let results = self.executor.execute_query(&query)
            .map_err(|e| e.to_string())?;

        Ok(results.into_iter()
            .filter_map(|r| r.get("nounName").cloned())
            .collect())
    }

    async fn describe_resource(&self, resource: &str) -> Result<Vec<String>, String> {
        let query = format!(r#"
        SELECT ?predicate ?object WHERE {{
            {{{{ ?resource cnv:name "{}" }}}}} .
            ?resource ?predicate ?object .
        }}
        "#, resource);

        let results = self.executor.execute_query(&query)
            .map_err(|e| e.to_string())?;

        Ok(results.into_iter()
            .filter_map(|r| {
                let pred = r.get("predicate")?;
                let obj = r.get("object")?;
                Some(format!("{}: {}", pred, obj))
            })
            .collect())
    }

    async fn execute_custom_query(&self, sparql: &str) -> Result<Vec<String>, String> {
        let results = self.executor.execute_query(sparql)
            .map_err(|e| e.to_string())?;

        Ok(results.into_iter()
            .map(|r| format!("{:?}", r.bindings))
            .collect())
    }
}

#[derive(Serialize)]
pub struct QueryCapabilitiesOutput {
    pub results: Vec<String>,
    pub found: bool,
}
```

## Step 5: Agent Workflow Example

```rust
pub struct CliGenerationAgent {
    generator_tool: GenerateCliFromTurtleTool,
    query_tool: QueryCapabilitiesTool,
}

impl CliGenerationAgent {
    pub async fn generate_cli_workflow(
        &self,
        ontology_path: &str,
        target_noun: &str,
    ) -> Result<AgentResult, Box<dyn std::error::Error>> {
        println!("🤖 Agent: Starting CLI generation workflow\n");

        // Step 1: Load ontology
        println!("1️⃣  Loading ontology from {}", ontology_path);
        let ontology_content = std::fs::read_to_string(ontology_path)?;

        // Step 2: Discover commands for target noun
        println!("2️⃣  Discovering commands for noun '{}'", target_noun);
        let commands = self.query_tool.execute(
            "find_verb",
            Some(target_noun),
        ).await?;

        println!("   Found {} commands", commands.results.len());
        for cmd in &commands.results {
            println!("     - {}", cmd);
        }

        // Step 3: Validate before generation
        println!("3️⃣  Validating ontology");
        let parser = TurtleParser::new();
        let ontology = parser.parse(&ontology_content)?;
        ontology.validate_ontology()?;
        println!("   ✅ Validation passed");

        // Step 4: Generate code
        println!("4️⃣  Generating CLI code");
        let generated = self.generator_tool.execute(&ontology_content).await?;
        println!("   ✅ Generated {} nouns, {} verbs",
            generated.noun_count, generated.verb_count);

        // Step 5: Report
        println!("5️⃣  Generation complete");
        println!("   Code size: {} lines", generated.rust_code.lines().count());

        Ok(AgentResult {
            success: true,
            generated_code: generated.rust_code,
            commands_generated: generated.verb_count,
            diagnostics: generated.diagnostics,
        })
    }
}

pub struct AgentResult {
    pub success: bool,
    pub generated_code: String,
    pub commands_generated: usize,
    pub diagnostics: Vec<DiagnosticOutput>,
}
```

## Step 6: Multi-Agent Coordination

```rust
pub struct MultiAgentOrchestrator {
    discovery_agent: CommandDiscoveryAgent,
    validation_agent: ValidationAgent,
    generation_agent: CliGenerationAgent,
    deployment_agent: DeploymentAgent,
}

impl MultiAgentOrchestrator {
    pub async fn orchestrate_full_pipeline(
        &self,
        ontology_path: &str,
    ) -> Result<PipelineResult, Box<dyn std::error::Error>> {
        println!("🎭 Orchestrating multi-agent pipeline\n");

        // Agent 1: Discover
        let discovered = self.discovery_agent.discover(ontology_path).await?;
        println!("Agent 1 discovered {} command categories\n", discovered.nouns.len());

        // Agent 2: Validate
        let validated = self.validation_agent.validate(&discovered).await?;
        if !validated.valid {
            return Err("Validation failed".into());
        }
        println!("Agent 2 validated ontology\n");

        // Agent 3: Generate
        let generated = self.generation_agent.generate(ontology_path).await?;
        println!("Agent 3 generated {} lines of code\n",
            generated.rust_code.lines().count());

        // Agent 4: Deploy
        let deployed = self.deployment_agent.deploy(&generated).await?;
        println!("Agent 4 deployed to {}\n", deployed.endpoint);

        Ok(PipelineResult {
            success: true,
            discovered_nouns: discovered.nouns.len(),
            generated_verbs: generated.verb_count,
            deployment_endpoint: deployed.endpoint,
        })
    }
}

pub struct PipelineResult {
    pub success: bool,
    pub discovered_nouns: usize,
    pub generated_verbs: usize,
    pub deployment_endpoint: String,
}
```

## Agent Integration Checklist

- ✅ Agents can discover commands via QueryCapabilities
- ✅ Agents can validate ontologies before generation
- ✅ Agents can generate code via GenerateCliFromTurtle
- ✅ Agents can handle errors gracefully
- ✅ Agents can coordinate multi-step workflows
- ✅ All agent operations complete within SLOs
- ✅ Agent communication is type-safe via MCP schemas
- ✅ Agents can monitor deployment and health

---

**Related**:
- [How-to: Deploy Production CLIs](../tutorials/tutorial-5-deployment.md)
- [Explanation: Agent Architecture Patterns](../explanation/agent-architecture.md)
- [Reference: MCP Tool Schemas](../reference/api-reference.md#mcp-tool-schemas)
