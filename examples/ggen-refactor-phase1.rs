//! ggen 80/20 Refactoring Plan - Phase 1: CLI Migration
//!
//! This example demonstrates the Phase 1 refactoring for ggen's CLI:
//! - Foundation setup with utils doctor (proof-of-concept)
//! - Simple commands: utils help-me, project new, hook create
//! - Core commands: ai project, ai generate, marketplace search, template generate
//! - Remaining commands: ai graph, ai sparql, marketplace install/list/publish
//!
//! This example shows the complete Phase 1 migration from regular clap to clap-noun-verb v3.0.0.

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde::Serialize;

// ============================================================================
// PHASE 1.1: Foundation - utils doctor (proof-of-concept)
// ============================================================================

//! Utility commands for diagnostics and help
//!
//! This module demonstrates Phase 1.1: Foundation setup with utils doctor
//! as a proof-of-concept for migrating to clap-noun-verb v3.0.0.

// Business Logic Layer (Pure Functions - Reusable)
fn run_diagnostics() -> DoctorOutput {
    DoctorOutput {
        checks: vec![
            CheckResult {
                name: "Template Engine".to_string(),
                status: "OK".to_string(),
                message: Some("Tera engine initialized successfully".to_string()),
            },
            CheckResult {
                name: "RDF Processor".to_string(),
                status: "OK".to_string(),
                message: Some("SPARQL engine ready".to_string()),
            },
            CheckResult {
                name: "AI Integration".to_string(),
                status: "WARNING".to_string(),
                message: Some("OpenAI API key not set".to_string()),
            },
        ],
        overall: "WARNING".to_string(),
    }
}

fn get_help_me() -> HelpOutput {
    HelpOutput {
        suggestions: vec![
            "Try 'ggen ai generate -d \"your description\"' to generate templates".to_string(),
            "Use 'ggen marketplace search \"rust\"' to find packages".to_string(),
            "Run 'ggen utils doctor' to check system status".to_string(),
        ],
    }
}

#[derive(Serialize, Debug)]
struct CheckResult {
    name: String,
    status: String,
    message: Option<String>,
}

#[derive(Serialize, Debug)]
struct DoctorOutput {
    checks: Vec<CheckResult>,
    overall: String,
}

#[derive(Serialize, Debug)]
struct HelpOutput {
    suggestions: Vec<String>,
}

// CLI Layer (Input Validation + Output Shaping Only)
/// Run diagnostics to check system status
///
/// This command performs various health checks and reports the status.
#[verb("doctor", "utils")] // Verb "doctor" auto-inferred, noun "utils" explicitly specified
fn utils_doctor() -> Result<DoctorOutput> {
    Ok(run_diagnostics())
}

/// Get personalized help and suggestions
#[verb("help-me", "utils")] // Verb "help-me" auto-inferred, noun "utils" explicitly specified
fn utils_help_me() -> Result<HelpOutput> {
    Ok(get_help_me())
}

// ============================================================================
// PHASE 1.2: Simple Commands - project new, hook create
// ============================================================================

// Business Logic Layer
fn create_project(name: String, description: Option<String>, rust: bool) -> ProjectOutput {
    ProjectOutput {
        name,
        description,
        rust,
        success: true,
        message: Some("Project created successfully".to_string()),
    }
}

fn create_hook(name: String, trigger: String) -> HookOutput {
    HookOutput {
        name,
        trigger,
        success: true,
        message: Some("Knowledge hook created successfully".to_string()),
    }
}

#[derive(Serialize, Debug)]
struct ProjectOutput {
    name: String,
    description: Option<String>,
    rust: bool,
    success: bool,
    message: Option<String>,
}

#[derive(Serialize, Debug)]
struct HookOutput {
    name: String,
    trigger: String,
    success: bool,
    message: Option<String>,
}

// CLI Layer
/// Create a new project
///
/// # Arguments
/// * `name` - Project name
/// * `description` - Optional project description
/// * `rust` - Generate Rust project structure (flag)
#[verb("new", "project")] // Verb "new" auto-inferred, noun "project" explicitly specified
fn project_new(
    name: String,
    description: Option<String>,
    rust: bool,
) -> Result<ProjectOutput> {
    Ok(create_project(name, description, rust))
}

/// Create a knowledge hook
///
/// # Arguments
/// * `name` - Hook name
/// * `trigger` - Trigger condition
#[verb("create", "hook")] // Verb "create" auto-inferred, noun "hook" explicitly specified
fn hook_create(name: String, trigger: String) -> Result<HookOutput> {
    Ok(create_hook(name, trigger))
}

// ============================================================================
// PHASE 1.3: Core Commands - ai project, ai generate, marketplace search, template generate
// ============================================================================

// Business Logic Layer
fn generate_project(name: String, description: Option<String>, rust: bool) -> AiProjectOutput {
    AiProjectOutput {
        name,
        description,
        rust,
        success: true,
        files_created: vec![
            "src/main.rs".to_string(),
            "Cargo.toml".to_string(),
            "README.md".to_string(),
        ],
    }
}

fn generate_template(description: String, output: Option<String>) -> AiGenerateOutput {
    AiGenerateOutput {
        description: description.clone(),
        output: output.clone().unwrap_or_else(|| "template.tmpl".to_string()),
        success: true,
        template: format!("// Generated template for: {}\n", description),
    }
}

fn search_marketplace(query: String) -> SearchOutput {
    SearchOutput {
        query,
        results: vec![
            PackageInfo {
                name: "io.ggen.rust.axum".to_string(),
                description: "Axum web framework template".to_string(),
                version: "1.0.0".to_string(),
            },
            PackageInfo {
                name: "io.ggen.rust.clap".to_string(),
                description: "CLI application template".to_string(),
                version: "1.0.0".to_string(),
            },
        ],
    }
}

fn generate_from_template(template: String, vars: Vec<String>) -> TemplateGenerateOutput {
    TemplateGenerateOutput {
        template,
        vars: vars.iter().map(|v| v.split('=').collect()).collect(),
        success: true,
        output: "generated_code.rs".to_string(),
    }
}

#[derive(Serialize, Debug)]
struct AiProjectOutput {
    name: String,
    description: Option<String>,
    rust: bool,
    success: bool,
    files_created: Vec<String>,
}

#[derive(Serialize, Debug)]
struct AiGenerateOutput {
    description: String,
    output: String,
    success: bool,
    template: String,
}

#[derive(Serialize, Debug)]
struct PackageInfo {
    name: String,
    description: String,
    version: String,
}

#[derive(Serialize, Debug)]
struct SearchOutput {
    query: String,
    results: Vec<PackageInfo>,
}

#[derive(Serialize, Debug)]
struct TemplateGenerateOutput {
    template: String,
    vars: Vec<Vec<String>>,
    success: bool,
    output: String,
}

// CLI Layer
/// Generate a complete project using AI
///
/// # Arguments
/// * `name` - Project name
/// * `description` - Optional project description
/// * `rust` - Generate Rust project structure (flag)
#[verb("project", "ai")] // Verb "project" auto-inferred, noun "ai" explicitly specified
fn ai_project(
    name: String,
    description: Option<String>,
    rust: bool,
) -> Result<AiProjectOutput> {
    Ok(generate_project(name, description, rust))
}

/// Generate a template from a description using AI
///
/// # Arguments
/// * `description` - Template description
/// * `output` - Optional output file path
#[verb("generate", "ai")] // Verb "generate" auto-inferred, noun "ai" explicitly specified
fn ai_generate(
    description: String,
    output: Option<String>,
) -> Result<AiGenerateOutput> {
    Ok(generate_template(description, output))
}

/// Search the marketplace for packages
///
/// # Arguments
/// * `query` - Search query
#[verb("search", "marketplace")] // Verb "search" auto-inferred, noun "marketplace" explicitly specified
fn marketplace_search(query: String) -> Result<SearchOutput> {
    Ok(search_marketplace(query))
}

/// Generate code from a template
///
/// # Arguments
/// * `template` - Template file path
/// * `vars` - Template variables in key=value format
#[verb("generate", "template")] // Verb "generate" auto-inferred, noun "template" explicitly specified
fn template_generate(template: String, vars: Vec<String>) -> Result<TemplateGenerateOutput> {
    Ok(generate_from_template(template, vars))
}

// ============================================================================
// PHASE 1.4: Remaining Commands - ai graph, ai sparql, marketplace install/list/publish
// ============================================================================

// Business Logic Layer
fn generate_rdf_graph(description: String, output: Option<String>) -> AiGraphOutput {
    AiGraphOutput {
        description,
        output: output.unwrap_or_else(|| "ontology.ttl".to_string()),
        success: true,
        triples_count: 42,
    }
}

fn generate_sparql_query(description: String, graph: Option<String>) -> AiSparqlOutput {
    AiSparqlOutput {
        description,
        graph,
        success: true,
        query: "SELECT ?s ?p ?o WHERE { ?s ?p ?o }".to_string(),
    }
}

fn install_package(package: String) -> InstallOutput {
    InstallOutput {
        package,
        success: true,
        message: Some("Package installed successfully".to_string()),
    }
}

fn list_packages() -> ListOutput {
    ListOutput {
        packages: vec![
            PackageInfo {
                name: "io.ggen.rust.axum".to_string(),
                description: "Axum web framework template".to_string(),
                version: "1.0.0".to_string(),
            },
        ],
    }
}

fn publish_package(package: String) -> PublishOutput {
    PublishOutput {
        package,
        success: true,
        message: Some("Package published successfully".to_string()),
    }
}

#[derive(Serialize, Debug)]
struct AiGraphOutput {
    description: String,
    output: String,
    success: bool,
    triples_count: usize,
}

#[derive(Serialize, Debug)]
struct AiSparqlOutput {
    description: String,
    graph: Option<String>,
    success: bool,
    query: String,
}

#[derive(Serialize, Debug)]
struct InstallOutput {
    package: String,
    success: bool,
    message: Option<String>,
}

#[derive(Serialize, Debug)]
struct ListOutput {
    packages: Vec<PackageInfo>,
}

#[derive(Serialize, Debug)]
struct PublishOutput {
    package: String,
    success: bool,
    message: Option<String>,
}

// CLI Layer
/// Generate an RDF ontology from a description using AI
///
/// # Arguments
/// * `description` - Ontology description
/// * `output` - Optional output file path
#[verb("graph", "ai")] // Verb "graph" auto-inferred, noun "ai" explicitly specified
fn ai_graph(
    description: String,
    output: Option<String>,
) -> Result<AiGraphOutput> {
    Ok(generate_rdf_graph(description, output))
}

/// Generate a SPARQL query from a description using AI
///
/// # Arguments
/// * `description` - Query description
/// * `graph` - Optional graph file path
#[verb("sparql", "ai")] // Verb "sparql" auto-inferred, noun "ai" explicitly specified
fn ai_sparql(
    description: String,
    graph: Option<String>,
) -> Result<AiSparqlOutput> {
    Ok(generate_sparql_query(description, graph))
}

/// Install a package from the marketplace
///
/// # Arguments
/// * `package` - Package identifier
#[verb("install", "marketplace")] // Verb "install" auto-inferred, noun "marketplace" explicitly specified
fn marketplace_install(package: String) -> Result<InstallOutput> {
    Ok(install_package(package))
}

/// List installed packages
#[verb("list", "marketplace")] // Verb "list" auto-inferred, noun "marketplace" explicitly specified
fn marketplace_list() -> Result<ListOutput> {
    Ok(list_packages())
}

/// Publish a package to the marketplace
///
/// # Arguments
/// * `package` - Package identifier
#[verb("publish", "marketplace")] // Verb "publish" auto-inferred, noun "marketplace" explicitly specified
fn marketplace_publish(package: String) -> Result<PublishOutput> {
    Ok(publish_package(package))
}

// ============================================================================
// Main Entry Point - Auto-Discovery
// ============================================================================

fn main() -> Result<()> {
    // Auto-discover all registered commands and run
    // The framework automatically:
    // - Discovers all #[verb] functions in this file
    // - Infers noun names from module/file context
    // - Infers verb names from function names
    // - Builds the CLI structure
    // - Handles argument parsing and validation
    // - Executes the appropriate handler
    // - Serializes output to JSON
    clap_noun_verb::run()
}

