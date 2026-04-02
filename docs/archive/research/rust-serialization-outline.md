# Rust Serialization System for CLAUDE.md JSON-LD Configuration

## Overview

This document outlines the Rust code structure for converting between CLAUDE.md text format, RDF/Turtle, and JSON-LD configurations.

## Architecture

```
┌─────────────────┐
│   CLAUDE.md     │ (Text format)
│   (Markdown)    │
└────────┬────────┘
         │
         ▼ ClaudemdToRdf::parse()
┌─────────────────┐
│   RDF Graph     │ (oxigraph::Graph)
│   (Turtle)      │
└────────┬────────┘
         │
         ▼ RdfToJsonld::serialize()
┌─────────────────┐
│   JSON-LD       │ (serde_json::Value)
│   (JSON)        │
└────────┬────────┘
         │
         ▼ JsonldToConfig::deserialize()
┌─────────────────┐
│ Rust Config     │ (ClaudeConfig struct)
│ (In-memory)     │
└─────────────────┘
```

## Dependencies

```toml
[dependencies]
# RDF/SPARQL
oxigraph = "0.4"
sophia = "0.8"

# JSON-LD
json-ld = "0.14"
iref = "3.2"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Parsing
pulldown-cmark = "0.9"  # Markdown parsing
regex = "1.10"

# Error handling
thiserror = "1.0"
anyhow = "1.0"
```

## Core Structures

### 1. ClaudemdToRdf - Markdown to RDF Converter

```rust
use oxigraph::model::{Graph, NamedNode, Literal};
use pulldown_cmark::{Parser, Event};
use anyhow::Result;

/// Converts CLAUDE.md markdown format to RDF graph
pub struct ClaudemdToRdf {
    graph: Graph,
    namespaces: NamespaceMap,
}

impl ClaudemdToRdf {
    /// Create new converter with standard namespaces
    pub fn new() -> Self {
        let mut namespaces = NamespaceMap::new();
        namespaces.insert("claudemd", "http://example.org/claude-md#");
        namespaces.insert("project", "http://example.org/project#");
        namespaces.insert("agent", "http://example.org/agent#");
        // ... more namespaces

        Self {
            graph: Graph::new(),
            namespaces,
        }
    }

    /// Parse CLAUDE.md file into RDF graph
    pub fn parse(&mut self, markdown_content: &str) -> Result<&Graph> {
        let parser = Parser::new(markdown_content);
        let mut current_section = Section::None;

        for event in parser {
            match event {
                Event::Start(Tag::Heading(level)) => {
                    current_section = self.detect_section(level);
                }
                Event::Text(text) => {
                    self.parse_section_content(current_section, &text)?;
                }
                Event::Code(code) => {
                    self.parse_code_block(current_section, &code)?;
                }
                Event::Table(_) => {
                    self.parse_table(current_section)?;
                }
                _ => {}
            }
        }

        Ok(&self.graph)
    }

    /// Parse project identity section
    fn parse_project_identity(&mut self, content: &str) -> Result<()> {
        // Extract project name, language, architecture, principles
        let project_uri = self.namespaces.get("project:clap-noun-verb");

        self.graph.insert((
            project_uri,
            NamedNode::new("rdf:type")?,
            NamedNode::new("claudemd:Project")?,
        ));

        // Parse key-value pairs from markdown
        self.parse_key_values(project_uri, content)?;

        Ok(())
    }

    /// Parse hyper-advanced agents table
    fn parse_agents_table(&mut self, table_content: &str) -> Result<()> {
        // Extract rows from markdown table
        for row in extract_table_rows(table_content) {
            let agent_type = row.get("Agent")?;
            let use_case = row.get("Use Case")?;
            let when_to_use = row.get("When to Use")?;

            let agent_uri = format!("agent:{}", agent_type);

            self.graph.insert((
                NamedNode::new(&agent_uri)?,
                NamedNode::new("rdf:type")?,
                NamedNode::new("claudemd:HyperAdvancedAgent")?,
            ));

            self.graph.insert((
                NamedNode::new(&agent_uri)?,
                NamedNode::new("agent:agentType")?,
                Literal::new_simple_literal(agent_type),
            ));

            self.graph.insert((
                NamedNode::new(&agent_uri)?,
                NamedNode::new("agent:useCase")?,
                Literal::new_simple_literal(use_case),
            ));

            // ... more properties
        }

        Ok(())
    }

    /// Parse absolute rules section
    fn parse_absolute_rules(&mut self, content: &str) -> Result<()> {
        // Extract numbered rules
        for (index, rule_content) in extract_numbered_rules(content) {
            let rule_uri = format!("rule:absolute-{}", index);

            self.graph.insert((
                NamedNode::new(&rule_uri)?,
                NamedNode::new("rdf:type")?,
                NamedNode::new("claudemd:AbsoluteRule")?,
            ));

            // Parse rule components
            let requirement = extract_requirement(rule_content)?;
            let rationale = extract_rationale(rule_content)?;

            self.graph.insert((
                NamedNode::new(&rule_uri)?,
                NamedNode::new("rule:requirement")?,
                Literal::new_simple_literal(requirement),
            ));

            // ... more properties
        }

        Ok(())
    }

    /// Parse cargo make commands section
    fn parse_cargo_commands(&mut self, content: &str) -> Result<()> {
        // Extract command listings
        for command_line in extract_command_lines(content) {
            let (command, description) = parse_command_line(command_line)?;
            let command_id = command.replace(' ', "-");
            let command_uri = format!("build:cargo-make-{}", command_id);

            self.graph.insert((
                NamedNode::new(&command_uri)?,
                NamedNode::new("rdf:type")?,
                NamedNode::new("claudemd:CargoMakeCommand")?,
            ));

            self.graph.insert((
                NamedNode::new(&command_uri)?,
                NamedNode::new("build:command")?,
                Literal::new_simple_literal(command),
            ));

            // ... more properties
        }

        Ok(())
    }

    /// Parse performance SLOs section
    fn parse_performance_slos(&mut self, content: &str) -> Result<()> {
        // Extract SLO definitions
        for (metric, target) in extract_slo_metrics(content) {
            let slo_id = metric.to_lowercase().replace(' ', "-");
            let slo_uri = format!("perf:{}", slo_id);

            self.graph.insert((
                NamedNode::new(&slo_uri)?,
                NamedNode::new("rdf:type")?,
                NamedNode::new("claudemd:PerformanceSLO")?,
            ));

            self.graph.insert((
                NamedNode::new(&slo_uri)?,
                NamedNode::new("perf:metric")?,
                Literal::new_simple_literal(metric),
            ));

            self.graph.insert((
                NamedNode::new(&slo_uri)?,
                NamedNode::new("perf:target")?,
                Literal::new_simple_literal(target),
            ));

            // ... more properties
        }

        Ok(())
    }
}

/// Section types in CLAUDE.md
enum Section {
    None,
    ProjectIdentity,
    ConcurrentExecution,
    HyperAdvancedAgents,
    BuildCommands,
    TestingStrategy,
    PerformanceSLOs,
    AndonSignals,
    FileOrganization,
    ProhibitedPatterns,
}
```

### 2. RdfToJsonld - RDF to JSON-LD Serializer

```rust
use json_ld::{JsonLdProcessor, RemoteDocument};
use oxigraph::model::Graph;
use serde_json::Value;
use anyhow::Result;

/// Converts RDF graph to JSON-LD format
pub struct RdfToJsonld {
    context_url: String,
}

impl RdfToJsonld {
    /// Create new serializer with context URL
    pub fn new(context_url: impl Into<String>) -> Self {
        Self {
            context_url: context_url.into(),
        }
    }

    /// Serialize RDF graph to JSON-LD
    pub fn serialize(&self, graph: &Graph) -> Result<Value> {
        // Convert oxigraph Graph to JSON-LD processor format
        let rdf_triples = self.graph_to_triples(graph)?;

        // Create JSON-LD document
        let mut jsonld = serde_json::json!({
            "@context": self.context_url,
            "@graph": []
        });

        // Group triples by subject
        let subjects = self.group_by_subject(rdf_triples);

        let mut graph_array = Vec::new();

        for (subject_uri, properties) in subjects {
            let mut node = serde_json::json!({
                "@id": subject_uri
            });

            // Add type
            if let Some(types) = properties.get("rdf:type") {
                node["@type"] = serde_json::json!(types);
            }

            // Add other properties
            for (predicate, values) in properties {
                if predicate != "rdf:type" {
                    let property_name = self.shorten_uri(predicate);
                    node[property_name] = self.values_to_json(values)?;
                }
            }

            graph_array.push(node);
        }

        jsonld["@graph"] = serde_json::json!(graph_array);

        Ok(jsonld)
    }

    /// Convert graph to triple format
    fn graph_to_triples(&self, graph: &Graph) -> Result<Vec<Triple>> {
        let mut triples = Vec::new();

        for triple in graph.iter() {
            triples.push(Triple {
                subject: triple.subject.to_string(),
                predicate: triple.predicate.to_string(),
                object: triple.object.to_string(),
            });
        }

        Ok(triples)
    }

    /// Group triples by subject
    fn group_by_subject(&self, triples: Vec<Triple>) -> SubjectMap {
        let mut subjects: SubjectMap = HashMap::new();

        for triple in triples {
            subjects
                .entry(triple.subject)
                .or_insert_with(HashMap::new)
                .entry(triple.predicate)
                .or_insert_with(Vec::new)
                .push(triple.object);
        }

        subjects
    }

    /// Shorten URI using context
    fn shorten_uri(&self, uri: &str) -> String {
        // Apply context mappings to shorten URIs
        // e.g., "http://example.org/agent#agentType" -> "agentType"
        uri.split('#').last()
            .or_else(|| uri.split('/').last())
            .unwrap_or(uri)
            .to_string()
    }

    /// Convert values to JSON representation
    fn values_to_json(&self, values: &[String]) -> Result<Value> {
        if values.len() == 1 {
            Ok(serde_json::json!(values[0]))
        } else {
            Ok(serde_json::json!(values))
        }
    }
}

type SubjectMap = HashMap<String, HashMap<String, Vec<String>>>;

struct Triple {
    subject: String,
    predicate: String,
    object: String,
}
```

### 3. JsonldToConfig - JSON-LD to Rust Config Deserializer

```rust
use serde::{Deserialize, Serialize};
use serde_json::Value;
use anyhow::Result;

/// Converts JSON-LD to typed Rust configuration
pub struct JsonldToConfig;

impl JsonldToConfig {
    /// Deserialize JSON-LD to ClaudeConfig
    pub fn deserialize(jsonld: &Value) -> Result<ClaudeConfig> {
        // Extract @graph array
        let graph = jsonld
            .get("@graph")
            .and_then(|g| g.as_array())
            .ok_or_else(|| anyhow::anyhow!("Missing @graph"))?;

        let mut config = ClaudeConfig::default();

        // Process each node in graph
        for node in graph {
            let node_type = node
                .get("@type")
                .and_then(|t| t.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing @type"))?;

            match node_type {
                "Project" => {
                    config.project = serde_json::from_value(node.clone())?;
                }
                "HyperAdvancedAgent" => {
                    let agent: HyperAdvancedAgent = serde_json::from_value(node.clone())?;
                    config.hyper_advanced_agents.push(agent);
                }
                "AbsoluteRule" => {
                    let rule: AbsoluteRule = serde_json::from_value(node.clone())?;
                    config.absolute_rules.push(rule);
                }
                "CargoMakeCommand" => {
                    let command: CargoMakeCommand = serde_json::from_value(node.clone())?;
                    config.cargo_commands.push(command);
                }
                "PerformanceSLO" => {
                    let slo: PerformanceSLO = serde_json::from_value(node.clone())?;
                    config.performance_slos.push(slo);
                }
                "AndonSignal" => {
                    let signal: AndonSignal = serde_json::from_value(node.clone())?;
                    config.andon_signals.push(signal);
                }
                _ => {
                    // Ignore unknown types
                }
            }
        }

        Ok(config)
    }

    /// Validate configuration against constraints
    pub fn validate(config: &ClaudeConfig) -> Result<()> {
        // Validate project has required fields
        if config.project.project_name.is_empty() {
            return Err(anyhow::anyhow!("Project name is required"));
        }

        // Validate at least one hyper-advanced agent
        if config.hyper_advanced_agents.is_empty() {
            return Err(anyhow::anyhow!("At least one hyper-advanced agent required"));
        }

        // Validate absolute rules are present
        if config.absolute_rules.is_empty() {
            return Err(anyhow::anyhow!("Absolute rules are required"));
        }

        // Validate cargo commands
        for command in &config.cargo_commands {
            if !command.command.starts_with("cargo make") {
                return Err(anyhow::anyhow!(
                    "Invalid cargo command: {}. Must use 'cargo make'",
                    command.command
                ));
            }
        }

        // Validate SLO thresholds are positive
        for slo in &config.performance_slos {
            if slo.threshold <= 0.0 {
                return Err(anyhow::anyhow!(
                    "SLO threshold must be positive: {}",
                    slo.slo_name
                ));
            }
        }

        Ok(())
    }
}

/// Top-level configuration structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ClaudeConfig {
    pub project: Project,
    pub hyper_advanced_agents: Vec<HyperAdvancedAgent>,
    pub core_agents: Vec<CoreAgent>,
    pub absolute_rules: Vec<AbsoluteRule>,
    pub cargo_commands: Vec<CargoMakeCommand>,
    pub performance_slos: Vec<PerformanceSLO>,
    pub andon_signals: Vec<AndonSignal>,
    pub directories: Vec<Directory>,
    pub prohibited_patterns: Vec<ProhibitedPattern>,
    pub testing_strategy: TestingStrategy,
    pub methodology: Methodology,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Project {
    #[serde(rename = "projectName")]
    pub project_name: String,
    pub description: String,
    pub language: String,
    pub architecture: String,
    pub version: String,
    #[serde(rename = "corePrinciples")]
    pub core_principles: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HyperAdvancedAgent {
    #[serde(rename = "agentType")]
    pub agent_type: String,
    pub name: String,
    #[serde(rename = "useCase")]
    pub use_case: String,
    #[serde(rename = "whenToUse")]
    pub when_to_use: String,
    pub capabilities: Vec<String>,
    pub specialization: String,
    pub priority: u8,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreAgent {
    #[serde(rename = "agentType")]
    pub agent_type: String,
    pub name: String,
    #[serde(rename = "whenToUse")]
    pub when_to_use: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbsoluteRule {
    #[serde(rename = "ruleType")]
    pub rule_type: String,
    pub name: String,
    pub requirement: String,
    pub rationale: String,
    #[serde(rename = "violationConsequence")]
    pub violation_consequence: String,
    pub enforcement: String,
    #[serde(rename = "mandatoryPattern")]
    pub mandatory_pattern: String,
    pub examples: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CargoMakeCommand {
    pub command: String,
    #[serde(rename = "commandType")]
    pub command_type: String,
    pub timeout: String,
    pub purpose: String,
    pub category: String,
    #[serde(rename = "replacesDirectCommand")]
    pub replaces_direct_command: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSLO {
    #[serde(rename = "sloName")]
    pub slo_name: String,
    pub metric: String,
    pub target: String,
    pub unit: String,
    pub threshold: f64,
    #[serde(rename = "verificationCommand")]
    pub verification_command: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AndonSignal {
    #[serde(rename = "signalType")]
    pub signal_type: String,
    pub severity: String,
    pub action: String,
    pub pattern: String,
    #[serde(rename = "detectionCommand")]
    pub detection_command: String,
    pub workflow: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Directory {
    #[serde(rename = "directoryPath")]
    pub directory_path: String,
    #[serde(rename = "directoryPurpose")]
    pub directory_purpose: String,
    #[serde(rename = "allowedFileTypes")]
    pub allowed_file_types: Vec<String>,
    pub prohibited: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProhibitedPattern {
    #[serde(rename = "prohibitedAction")]
    pub prohibited_action: String,
    pub reason: String,
    #[serde(rename = "correctAlternative")]
    pub correct_alternative: String,
    #[serde(rename = "enforcementLevel")]
    pub enforcement_level: String,
    pub examples: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TestingStrategy {
    pub name: String,
    pub description: String,
    #[serde(rename = "testPrinciple")]
    pub test_principle: String,
    #[serde(rename = "testPattern")]
    pub test_pattern: String,
    pub verifies: Vec<String>,
    pub requires: Vec<String>,
    #[serde(rename = "coverageTarget")]
    pub coverage_target: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Methodology {
    #[serde(rename = "methodologyName")]
    pub methodology_name: String,
    #[serde(rename = "fullName")]
    pub full_name: String,
    pub description: String,
    pub phases: Vec<Phase>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Phase {
    #[serde(rename = "phaseName")]
    pub phase_name: String,
    #[serde(rename = "phaseCommand")]
    pub phase_command: String,
    #[serde(rename = "phaseOrder")]
    pub phase_order: u8,
    pub description: String,
}
```

## Error Handling

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClaudeConfigError {
    #[error("Failed to parse CLAUDE.md: {0}")]
    ParseError(String),

    #[error("Invalid RDF graph: {0}")]
    RdfError(String),

    #[error("JSON-LD serialization failed: {0}")]
    JsonLdError(String),

    #[error("Configuration validation failed: {0}")]
    ValidationError(String),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
}
```

## Usage Examples

```rust
use anyhow::Result;

/// Full pipeline: CLAUDE.md -> RDF -> JSON-LD -> Config
pub fn load_claude_config(markdown_path: &str) -> Result<ClaudeConfig> {
    // 1. Parse CLAUDE.md to RDF
    let markdown = std::fs::read_to_string(markdown_path)?;
    let mut parser = ClaudemdToRdf::new();
    let rdf_graph = parser.parse(&markdown)?;

    // 2. Serialize RDF to JSON-LD
    let serializer = RdfToJsonld::new("claude-md-context.jsonld");
    let jsonld = serializer.serialize(rdf_graph)?;

    // 3. Deserialize JSON-LD to Config
    let config = JsonldToConfig::deserialize(&jsonld)?;

    // 4. Validate configuration
    JsonldToConfig::validate(&config)?;

    Ok(config)
}

/// Query configuration with SPARQL
pub fn query_agents(config: &ClaudeConfig, agent_type: &str) -> Vec<&HyperAdvancedAgent> {
    config
        .hyper_advanced_agents
        .iter()
        .filter(|agent| agent.agent_type == agent_type)
        .collect()
}

/// Find rules by enforcement level
pub fn find_critical_rules(config: &ClaudeConfig) -> Vec<&AbsoluteRule> {
    config
        .absolute_rules
        .iter()
        .filter(|rule| rule.enforcement == "CRITICAL" || rule.enforcement == "NON-NEGOTIABLE")
        .collect()
}

/// Get SLO for specific metric
pub fn get_slo(config: &ClaudeConfig, metric: &str) -> Option<&PerformanceSLO> {
    config
        .performance_slos
        .iter()
        .find(|slo| slo.metric == metric)
}

/// Export configuration back to JSON-LD
pub fn export_to_jsonld(config: &ClaudeConfig) -> Result<String> {
    let json = serde_json::to_string_pretty(config)?;
    Ok(json)
}
```

## CLI Integration

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "clap-noun-verb")]
#[command(about = "CLAUDE.md configuration manager")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Load and validate CLAUDE.md configuration
    Load {
        /// Path to CLAUDE.md file
        #[arg(short, long)]
        path: String,
    },

    /// Query configuration
    Query {
        /// SPARQL query
        #[arg(short, long)]
        query: String,
    },

    /// Export to JSON-LD
    Export {
        /// Output path
        #[arg(short, long)]
        output: String,
    },

    /// Validate configuration
    Validate {
        /// Path to CLAUDE.md file
        #[arg(short, long)]
        path: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Load { path } => {
            let config = load_claude_config(&path)?;
            println!("Configuration loaded successfully!");
            println!("Project: {}", config.project.project_name);
            println!("Agents: {}", config.hyper_advanced_agents.len());
            println!("Rules: {}", config.absolute_rules.len());
        }

        Commands::Query { query } => {
            // Execute SPARQL query
            todo!("Implement SPARQL query")
        }

        Commands::Export { output } => {
            // Export to JSON-LD
            todo!("Implement export")
        }

        Commands::Validate { path } => {
            let config = load_claude_config(&path)?;
            JsonldToConfig::validate(&config)?;
            println!("✓ Configuration is valid");
        }
    }

    Ok(())
}
```

## Testing Strategy

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_claudemd_to_rdf() {
        let markdown = r#"
## 📋 Project Identity

**clap-noun-verb** is a Rust CLI tool that parses Clap arguments.

- **Language**: Rust
- **Architecture**: Binary crate
        "#;

        let mut parser = ClaudemdToRdf::new();
        let graph = parser.parse(markdown).unwrap();

        assert!(!graph.is_empty());
        // Verify project node exists
    }

    #[test]
    fn test_rdf_to_jsonld_serialization() {
        let graph = create_test_graph();
        let serializer = RdfToJsonld::new("claude-md-context.jsonld");
        let jsonld = serializer.serialize(&graph).unwrap();

        assert!(jsonld.get("@context").is_some());
        assert!(jsonld.get("@graph").is_some());
    }

    #[test]
    fn test_jsonld_to_config_deserialization() {
        let jsonld = load_test_jsonld();
        let config = JsonldToConfig::deserialize(&jsonld).unwrap();

        assert_eq!(config.project.project_name, "clap-noun-verb");
        assert!(!config.hyper_advanced_agents.is_empty());
    }

    #[test]
    fn test_validate_config() {
        let config = create_test_config();
        assert!(JsonldToConfig::validate(&config).is_ok());
    }

    #[test]
    fn test_cargo_make_command_validation() {
        let mut config = create_test_config();
        config.cargo_commands.push(CargoMakeCommand {
            command: "cargo test".to_string(), // WRONG - should be cargo make
            command_type: "Testing".to_string(),
            timeout: "PT10S".to_string(),
            purpose: "Run tests".to_string(),
            category: "Development".to_string(),
            replaces_direct_command: None,
        });

        assert!(JsonldToConfig::validate(&config).is_err());
    }
}
```

## Future Enhancements

1. **SPARQL Query Support**: Full SPARQL endpoint for querying configuration
2. **JSON Schema Generation**: Derive JSON Schema from SHACL constraints
3. **Configuration Diff**: Compare two CLAUDE.md configurations
4. **Auto-completion**: Generate shell completions from configuration
5. **Web UI**: Interactive configuration editor
6. **Version Control**: Track configuration changes over time
7. **Import/Export**: Support multiple formats (YAML, TOML, JSON)

## Performance Considerations

- Lazy loading of RDF graph for large configurations
- Caching of JSON-LD context to avoid repeated parsing
- Incremental validation for real-time feedback
- Parallel parsing of independent sections
