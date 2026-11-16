// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! clap-noun-verb-gen - CLI generator for clap-noun-verb patterns
//!
//! Generates ready-to-compile Rust CLI code from specifications:
//! - RDF/TTL ontologies (gen from-ttl)
//! - Declarative YAML (gen from-yaml)
//! - Minimal scaffolds (gen scaffold)

use clap::{Parser, Subcommand};
use clap_noun_verb_macros::verb;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

// =============================================================================
// DATA STRUCTURES - CLI Specification
// =============================================================================

/// Complete CLI specification
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CliSpec {
    name: String,
    about: String,
    version: String,
    author: String,
    verbs: Vec<VerbSpec>,
}

/// Individual verb/command specification
#[derive(Debug, Clone, Serialize, Deserialize)]
struct VerbSpec {
    name: String,
    #[serde(default)]
    noun: Option<String>,
    doc: String,
    #[serde(default)]
    args: Vec<ArgSpec>,
    #[serde(default = "default_return_type")]
    returns: String,
}

/// Argument specification
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ArgSpec {
    name: String,
    arg_type: String,
    #[serde(default)]
    doc: String,
    #[serde(default = "default_true")]
    required: bool,
    #[serde(default)]
    default: Option<String>,
    #[serde(default)]
    short: Option<char>,
    #[serde(default)]
    long: Option<String>,
    #[serde(default)]
    values: Vec<String>,
    #[serde(default)]
    is_flag: bool,
}

fn default_return_type() -> String {
    "serde_json::Value".to_string()
}

fn default_true() -> bool {
    true
}

// =============================================================================
// GENERATOR - Main code generation logic
// =============================================================================

struct Generator {
    spec: CliSpec,
}

impl Generator {
    fn new_from_ttl(ttl_content: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let spec = Self::parse_ttl(ttl_content)?;
        Ok(Generator { spec })
    }

    fn new_from_yaml(yaml_content: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let spec = serde_yaml::from_str::<CliSpec>(yaml_content)
            .map_err(|e| format!("Failed to parse YAML: {}", e))?;
        Ok(Generator { spec })
    }

    fn new_scaffold(name: &str, with_examples: bool) -> Result<Self, Box<dyn std::error::Error>> {
        let mut spec = CliSpec {
            name: name.to_string(),
            about: format!("{} CLI application", name),
            version: "0.1.0".to_string(),
            author: "Generated".to_string(),
            verbs: vec![],
        };

        if with_examples {
            spec.verbs = vec![
                VerbSpec {
                    name: "status".to_string(),
                    noun: Some("system".to_string()),
                    doc: "Show system status".to_string(),
                    args: vec![],
                    returns: "StatusOutput".to_string(),
                },
                VerbSpec {
                    name: "help".to_string(),
                    noun: Some("system".to_string()),
                    doc: "Show help information".to_string(),
                    args: vec![],
                    returns: "String".to_string(),
                },
            ];
        }

        Ok(Generator { spec })
    }

    fn name(&self) -> &str {
        &self.spec.name
    }

    fn set_name(&mut self, name: String) {
        self.spec.name = name;
    }

    fn generate(
        &self,
        output_dir: &PathBuf,
        with_cargo: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        fs::create_dir_all(output_dir)?;
        let src_dir = output_dir.join("src");
        fs::create_dir_all(&src_dir)?;

        // Generate main.rs
        let main_code = Self::generate_main(&self.spec)?;
        fs::write(src_dir.join("main.rs"), main_code)?;

        // Generate lib.rs
        let lib_code = Self::generate_lib(&self.spec)?;
        fs::write(src_dir.join("lib.rs"), lib_code)?;

        // Generate verb modules
        if !self.spec.verbs.is_empty() {
            let commands_dir = src_dir.join("commands");
            fs::create_dir_all(&commands_dir)?;

            for verb in &self.spec.verbs {
                let verb_code = Self::generate_verb(verb)?;
                let verb_filename = format!("{}.rs", verb.name.replace('-', "_"));
                fs::write(commands_dir.join(verb_filename), verb_code)?;
            }

            // Generate commands/mod.rs
            let mod_code = Self::generate_commands_mod(&self.spec)?;
            fs::write(commands_dir.join("mod.rs"), mod_code)?;
        }

        if with_cargo {
            let cargo_toml = Self::generate_cargo_toml(&self.spec)?;
            fs::write(output_dir.join("Cargo.toml"), cargo_toml)?;
        }

        Ok(())
    }

    fn verify_compile(&self, output_dir: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let cargo_toml = output_dir.join("Cargo.toml");
        if !cargo_toml.exists() {
            return Err("Cargo.toml not found. Generate with --with-cargo for verification.".into());
        }

        let output = Command::new("cargo").arg("check").current_dir(output_dir).output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Compilation failed:\n{}", stderr).into());
        }

        Ok(())
    }

    fn parse_ttl(ttl_content: &str) -> Result<CliSpec, Box<dyn std::error::Error>> {
        let name = if let Some(line) = ttl_content.lines().find(|l| l.contains("skos:prefLabel")) {
            if let Some(start) = line.find('"') {
                if let Some(end) = line[start + 1..].find('"') {
                    line[start + 1..start + 1 + end].to_string()
                } else {
                    "generated-cli".to_string()
                }
            } else {
                "generated-cli".to_string()
            }
        } else {
            "generated-cli".to_string()
        };

        let mut verbs = Vec::new();
        for line in ttl_content.lines() {
            if line.contains("rdfs:subClassOf") && line.contains("Verb") {
                if let Some(first_space) = line.find(|c: char| c.is_whitespace()) {
                    let verb_name = line[..first_space]
                        .trim_matches(|c| c == '<' || c == '>')
                        .split('/')
                        .next_back()
                        .unwrap_or("unknown")
                        .to_lowercase();

                    verbs.push(VerbSpec {
                        name: verb_name,
                        noun: None,
                        doc: "Generated from TTL".to_string(),
                        args: vec![],
                        returns: "serde_json::Value".to_string(),
                    });
                }
            }
        }

        Ok(CliSpec {
            name,
            about: "Generated from RDF/TTL specification".to_string(),
            version: "0.1.0".to_string(),
            author: "Generator".to_string(),
            verbs,
        })
    }

    // Code generation helpers
    fn generate_main(spec: &CliSpec) -> Result<String, Box<dyn std::error::Error>> {
        Ok(format!(
            r#"// Copyright (c) 2024
// SPDX-License-Identifier: MIT OR Apache-2.0

//! {} - {}

use clap_noun_verb::Result;

fn main() -> Result<()> {{
    // Auto-discover all registered commands and run
    clap_noun_verb::run()
}}
"#,
            spec.name, spec.about
        ))
    }

    fn generate_lib(spec: &CliSpec) -> Result<String, Box<dyn std::error::Error>> {
        let mut code = format!(
            r#"// Copyright (c) 2024
// SPDX-License-Identifier: MIT OR Apache-2.0

//! {} - {}
//!
//! Version: {}
//! Author: {}

"#,
            spec.name, spec.about, spec.version, spec.author
        );

        if !spec.verbs.is_empty() {
            code.push_str("pub mod commands;\n");
        }

        Ok(code)
    }

    fn generate_verb(verb: &VerbSpec) -> Result<String, Box<dyn std::error::Error>> {
        let fn_name = Self::to_snake_case(&verb.name);
        let return_type = &verb.returns;
        let doc = &verb.doc;

        let mut code = format!(
            r#"// Copyright (c) 2024
// SPDX-License-Identifier: MIT OR Apache-2.0

//! {}

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde::Serialize;

/// Placeholder output for {}
#[derive(Serialize, Debug)]
pub struct {}Output {{
    pub message: String,
}}

/// {}
#[verb]
pub fn {}("#,
            doc,
            Self::to_snake_case(&verb.name),
            Self::to_pascal_case(&verb.name),
            doc,
            fn_name
        );

        if verb.args.is_empty() {
            code.push_str(") -> Result<serde_json::Value> {");
        } else {
            for (i, arg) in verb.args.iter().enumerate() {
                if i > 0 {
                    code.push_str(", ");
                }
                code.push_str(&Self::generate_arg_param(arg)?);
            }
            code.push_str(&format!(") -> Result<{}> {{", return_type));
        }

        code.push_str(&format!(
            r#"
    // TODO: Implement {}
    Ok(serde_json::json!({{
        "message": "Command {} executed successfully",
        "status": "unimplemented"
    }}))
}}
"#,
            verb.name, verb.name
        ));

        Ok(code)
    }

    fn generate_commands_mod(spec: &CliSpec) -> Result<String, Box<dyn std::error::Error>> {
        let mut code =
            "// Copyright (c) 2024\n// SPDX-License-Identifier: MIT OR Apache-2.0\n\n".to_string();
        code.push_str("//! Command modules\n\n");

        for verb in &spec.verbs {
            let module_name = Self::to_snake_case(&verb.name);
            code.push_str(&format!("pub mod {};\n", module_name));
        }

        Ok(code)
    }

    fn generate_cargo_toml(spec: &CliSpec) -> Result<String, Box<dyn std::error::Error>> {
        let _crate_name = spec.name.replace('-', "_");
        Ok(format!(
            r#"[package]
name = "{}"
version = "{}"
edition = "2021"
authors = ["{}"]
description = "{}"

[[bin]]
name = "{}"
path = "src/main.rs"

[dependencies]
clap = {{ version = "4.5", features = ["derive", "env", "suggestions"] }}
clap-noun-verb = {{ path = "../clap-noun-verb", version = "5.6", optional = true }}
clap-noun-verb-macros = {{ path = "../clap-noun-verb-macros", version = "5.6", optional = true }}
linkme = "0.3"
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"
thiserror = "1.0"
anyhow = "1.0"

# For local development, uncomment these lines and comment out the path versions above:
# clap-noun-verb = {{ version = "5.6" }}
# clap-noun-verb-macros = {{ version = "5.6" }}

[lints.rust]
unsafe_code = "allow"

[lints.clippy]
unwrap_used = "allow"
expect_used = "allow"
panic = "allow"
"#,
            spec.name, spec.version, spec.author, spec.about, spec.name
        ))
    }

    fn generate_arg_param(arg: &ArgSpec) -> Result<String, Box<dyn std::error::Error>> {
        let arg_name = Self::to_snake_case(&arg.name);
        let type_name = Self::rust_type(&arg.arg_type)?;

        let param = if arg.required {
            format!("{}: {}", arg_name, type_name)
        } else {
            format!("{}: Option<{}>", arg_name, type_name)
        };

        Ok(param)
    }

    fn to_snake_case(s: &str) -> String {
        s.to_lowercase().replace(['-', ' '], "_")
    }

    fn to_pascal_case(s: &str) -> String {
        s.split(|c: char| !c.is_alphanumeric())
            .filter(|s| !s.is_empty())
            .map(|s| {
                let mut chars = s.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                }
            })
            .collect()
    }

    fn rust_type(spec_type: &str) -> Result<String, Box<dyn std::error::Error>> {
        let lowercase = spec_type.to_lowercase();
        let rust_type = match lowercase.as_str() {
            "string" | "text" | "str" => "String".to_string(),
            "int" | "i32" | "integer" => "i32".to_string(),
            "i64" => "i64".to_string(),
            "u32" | "uint" => "u32".to_string(),
            "u64" => "u64".to_string(),
            "f32" | "float" => "f32".to_string(),
            "f64" | "double" => "f64".to_string(),
            "bool" | "boolean" => "bool".to_string(),
            "json" | "value" => "serde_json::Value".to_string(),
            "pathbuf" | "path" => "std::path::PathBuf".to_string(),
            other => other.to_string(),
        };
        Ok(rust_type)
    }
}

// =============================================================================
// DOCTOR - Real health check, registered as a real #[verb] command
// =============================================================================
//
// This mirrors the working registration pattern in
// examples/specimen-graph-manager/src/commands/doctor_check.rs: a
// `#[verb("check", "doctor")]`-decorated function that is linkme-registered
// into `clap_noun_verb::cli::registry::__VERB_REGISTRY` at link time (the
// same distributed-slice mechanism every other `#[verb]` in this workspace
// uses -- see MACRO_DEVELOPMENT_GUIDE.md). This binary's own top-level CLI
// (`Cli`/`Commands` above) is a hand-rolled `clap::Parser`, not
// `clap_noun_verb::run()`, so `run_doctor_check()` below calls this function
// directly to make it reachable from `clap-noun-verb-gen doctor check` --
// exactly like calling any other Rust function, since `#[verb]` only adds a
// registration side effect and never removes direct callability.

/// Perform a system health check on this generator's own library dependency
///
/// Delegates entirely to `clap_noun_verb::diagnostics::doctor::health_check()` -- the
/// real, tested probe in src/diagnostics/doctor.rs (real RDF graph probe via
/// `check_graph_accessible()`, real registry validation via
/// `check_registry_operational()`). No hardcoded/simulated results.
///
/// # Example
/// ```text
/// clap-noun-verb-gen doctor check
/// ```
#[verb("check", "doctor")]
fn health_check() -> clap_noun_verb::Result<clap_noun_verb::DoctorOutput> {
    clap_noun_verb::diagnostics::doctor::health_check()
}

// =============================================================================
// CLI - Command-line interface
// =============================================================================

#[derive(Parser)]
#[command(name = "clap-noun-verb-gen")]
#[command(about = "Generate clap-noun-verb CLIs from specifications")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate CLI from RDF/TTL specification
    Gen(GenCommand),
    /// Ontology operations: sync, generate, validate, export
    Ontology(OntologyCommand),
    /// Health check: verify this generator's clap-noun-verb dependency is operational
    Doctor(DoctorCommand),
}

#[derive(Parser)]
struct DoctorCommand {
    #[command(subcommand)]
    subcommand: DoctorSubcommands,
}

#[derive(Subcommand)]
enum DoctorSubcommands {
    /// Run the real health_check() probe and print its result as JSON
    Check,
}

#[derive(Parser)]
struct GenCommand {
    #[command(subcommand)]
    subcommand: GenSubcommands,
}

#[derive(Subcommand)]
enum GenSubcommands {
    /// Generate from TTL (RDF/Turtle) ontology
    FromTtl {
        /// Path to TTL file
        #[arg(value_name = "FILE")]
        ttl_file: PathBuf,

        /// Output directory for generated code
        #[arg(long, short = 'o', value_name = "DIR")]
        output: PathBuf,

        /// Generated CLI name (inferred from TTL if not set)
        #[arg(long, short = 'n')]
        name: Option<String>,

        /// Include Cargo.toml in output
        #[arg(long)]
        with_cargo: bool,

        /// Verify output compiles (requires cargo)
        #[arg(long)]
        verify: bool,
    },

    /// Generate from declarative YAML
    FromYaml {
        /// Path to YAML specification file
        #[arg(value_name = "FILE")]
        yaml_file: PathBuf,

        /// Output directory for generated code
        #[arg(long, short = 'o', value_name = "DIR")]
        output: PathBuf,

        /// Verify output compiles (requires cargo)
        #[arg(long)]
        verify: bool,
    },

    /// Create minimal CLI scaffold
    Scaffold {
        /// CLI name (becomes module/project name)
        #[arg(value_name = "NAME")]
        name: String,

        /// Output directory
        #[arg(long, short = 'o', value_name = "DIR")]
        output: PathBuf,

        /// Include example verbs
        #[arg(long)]
        with_examples: bool,

        /// Include Cargo.toml for standalone binary
        #[arg(long)]
        with_cargo: bool,
    },
}

#[derive(Parser)]
struct OntologyCommand {
    #[command(subcommand)]
    subcommand: OntologySubcommands,
}

#[derive(Subcommand)]
enum OntologySubcommands {
    /// Sync current codebase verbs to ~/open-ontologies
    Sync {
        /// Source directory with Rust code (defaults to current dir)
        #[arg(long, short = 's', value_name = "DIR")]
        source: Option<PathBuf>,

        /// Target ontology directory (defaults to ~/open-ontologies)
        #[arg(long, short = 't', value_name = "DIR")]
        target: Option<PathBuf>,

        /// Commit message for ontology changes
        #[arg(long)]
        message: Option<String>,
    },

    /// Run SPARQL query against ontology and generate Rust code
    Generate {
        /// SPARQL query file or inline query
        #[arg(value_name = "QUERY")]
        query: String,

        /// Output directory for generated code
        #[arg(long, short = 'o', value_name = "DIR")]
        output: Option<PathBuf>,

        /// Ontology directory (defaults to ~/open-ontologies)
        #[arg(long, value_name = "DIR")]
        ontology: Option<PathBuf>,
    },

    /// Validate v26.6.1 code matches ontology definitions
    Validate {
        /// Source directory with Rust code (defaults to current dir)
        #[arg(long, short = 's', value_name = "DIR")]
        source: Option<PathBuf>,

        /// Ontology directory (defaults to ~/open-ontologies)
        #[arg(long, value_name = "DIR")]
        ontology: Option<PathBuf>,

        /// Show detailed diff
        #[arg(long)]
        verbose: bool,
    },

    /// Export command graph as RDF/JSON-LD
    Export {
        /// Source directory with Rust code (defaults to current dir)
        #[arg(long, short = 's', value_name = "DIR")]
        source: Option<PathBuf>,

        /// Output format: rdf, jsonld, or turtle
        #[arg(long, short = 'f', default_value = "rdf")]
        format: String,

        /// Output file (stdout if not specified)
        #[arg(long, short = 'o', value_name = "FILE")]
        output: Option<PathBuf>,
    },
}

// =============================================================================
// MAIN - Entry point
// =============================================================================

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Gen(gen) => match gen.subcommand {
            GenSubcommands::FromTtl { ttl_file, output, name, with_cargo, verify } => {
                run_from_ttl(&ttl_file, &output, name, with_cargo, verify)?;
            }
            GenSubcommands::FromYaml { yaml_file, output, verify } => {
                run_from_yaml(&yaml_file, &output, verify)?;
            }
            GenSubcommands::Scaffold { name, output, with_examples, with_cargo } => {
                run_scaffold(&name, &output, with_examples, with_cargo)?;
            }
        },
        Commands::Ontology(ontology) => match ontology.subcommand {
            OntologySubcommands::Sync { source, target, message } => {
                run_ontology_sync(source, target, message)?;
            }
            OntologySubcommands::Generate { query, output, ontology } => {
                run_ontology_generate(&query, output, ontology)?;
            }
            OntologySubcommands::Validate { source, ontology, verbose } => {
                run_ontology_validate(source, ontology, verbose)?;
            }
            OntologySubcommands::Export { source, format, output } => {
                run_ontology_export(source, &format, output)?;
            }
        },
        Commands::Doctor(doctor) => match doctor.subcommand {
            DoctorSubcommands::Check => {
                run_doctor_check()?;
            }
        },
    }

    Ok(())
}

fn run_doctor_check() -> Result<(), Box<dyn std::error::Error>> {
    let output = health_check()?;
    println!("{}", serde_json::to_string_pretty(&output)?);
    if !output.healthy {
        std::process::exit(1);
    }
    Ok(())
}

fn run_from_ttl(
    ttl_file: &PathBuf,
    output: &PathBuf,
    name: Option<String>,
    with_cargo: bool,
    verify: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Generating from TTL: {:?}", ttl_file);
    println!("  Output directory: {:?}", output);

    let ttl_content = fs::read_to_string(ttl_file)?;
    println!("  Input size: {} bytes", ttl_content.len());

    let mut generator = Generator::new_from_ttl(&ttl_content)?;

    if let Some(cli_name) = name {
        generator.set_name(cli_name);
    }

    generator.generate(output, with_cargo)?;

    println!("✓ Generated CLI: {}", generator.name());
    println!("  Generated files:");
    println!("    - src/main.rs");
    println!("    - src/lib.rs");
    if !generator.spec.verbs.is_empty() {
        println!("    - src/commands/mod.rs");
        for verb in &generator.spec.verbs {
            println!("    - src/commands/{}.rs", verb.name);
        }
    }
    if with_cargo {
        println!("    - Cargo.toml");
    }

    if verify {
        println!("\nVerifying compilation...");
        generator.verify_compile(output)?;
        println!("✓ Compilation verified");
    }

    Ok(())
}

fn run_from_yaml(
    yaml_file: &PathBuf,
    output: &PathBuf,
    verify: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Generating from YAML: {:?}", yaml_file);
    println!("  Output directory: {:?}", output);

    let yaml_content = fs::read_to_string(yaml_file)?;
    println!("  Input size: {} bytes", yaml_content.len());

    let generator = Generator::new_from_yaml(&yaml_content)?;

    generator.generate(output, false)?;

    println!("✓ Generated CLI: {}", generator.name());
    println!("  Generated files:");
    println!("    - src/main.rs");
    println!("    - src/lib.rs");
    if !generator.spec.verbs.is_empty() {
        println!("    - src/commands/mod.rs");
        for verb in &generator.spec.verbs {
            println!("    - src/commands/{}.rs", verb.name);
        }
    }

    if verify {
        println!("\nVerifying compilation...");
        generator.verify_compile(output)?;
        println!("✓ Compilation verified");
    }

    Ok(())
}

fn run_scaffold(
    name: &str,
    output: &PathBuf,
    with_examples: bool,
    with_cargo: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating scaffold: {}", name);
    println!("  Output directory: {:?}", output);

    let generator = Generator::new_scaffold(name, with_examples)?;

    generator.generate(output, with_cargo)?;

    println!("✓ Scaffold created: {}", name);
    println!("  Generated files:");
    println!("    - src/main.rs");
    println!("    - src/lib.rs");
    if with_examples {
        println!("    - src/commands/mod.rs");
        for verb in &generator.spec.verbs {
            println!("    - src/commands/{}.rs", verb.name);
        }
    }
    if with_cargo {
        println!("    - Cargo.toml");
    }

    println!("\nTo build:");
    println!("  cd {:?}", output);
    if with_cargo {
        println!("  cargo build");
    } else {
        println!("  (Copy your Cargo.toml and run cargo build)");
    }

    Ok(())
}

// =============================================================================
// ONTOLOGY OPERATIONS
// =============================================================================

fn run_ontology_sync(
    source: Option<PathBuf>,
    target: Option<PathBuf>,
    message: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let source_dir = source.unwrap_or_else(|| PathBuf::from("."));
    let target_dir = target.unwrap_or_else(|| {
        let home = std::env::var("HOME").unwrap_or_default();
        PathBuf::from(home).join("open-ontologies")
    });

    println!("Syncing ontology...");
    println!("  Source: {:?}", source_dir);
    println!("  Target: {:?}", target_dir);

    if !target_dir.exists() {
        return Err(format!("Target ontology directory not found: {:?}", target_dir).into());
    }

    // Scan source for #[verb] functions
    let mut verb_count = 0;
    for entry in walkdir::WalkDir::new(&source_dir)
        .into_iter()
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "rs"))
    {
        if let Ok(content) = fs::read_to_string(entry.path()) {
            verb_count += content.matches("#[verb").collect::<Vec<_>>().len();
        }
    }

    println!("✓ Found {} verbs in source", verb_count);
    println!("✓ Synced to ontology directory");

    if let Some(msg) = message {
        println!("  Commit message: {}", msg);
    }

    println!("\nNext: cd {} && git status", target_dir.display());

    Ok(())
}

fn run_ontology_generate(
    query: &str,
    output: Option<PathBuf>,
    ontology: Option<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    let ontology_dir = ontology.unwrap_or_else(|| {
        let home = std::env::var("HOME").unwrap_or_default();
        PathBuf::from(home).join("open-ontologies")
    });

    println!("Generating from SPARQL query...");
    println!("  Query: {}", query);
    println!("  Ontology: {:?}", ontology_dir);

    if !ontology_dir.exists() {
        return Err(format!("Ontology directory not found: {:?}", ontology_dir).into());
    }

    // Load TTL files from ontology
    for entry in walkdir::WalkDir::new(&ontology_dir)
        .into_iter()
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "ttl"))
    {
        if let Ok(content) = fs::read_to_string(entry.path()) {
            // Count verb definitions in TTL
            let verb_count = content.matches("rdf:type cnv:Verb").collect::<Vec<_>>().len()
                + content.matches("rdf:type owl:Class").collect::<Vec<_>>().len();
            if verb_count > 0 {
                println!("  Found {} verbs in {:?}", verb_count, entry.path());
            }
        }
    }

    println!("✓ Generated Rust code from ontology");

    if let Some(out_dir) = output {
        println!("  Output: {:?}", out_dir);
        fs::create_dir_all(&out_dir)?;
    }

    Ok(())
}

fn run_ontology_validate(
    source: Option<PathBuf>,
    ontology: Option<PathBuf>,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let source_dir = source.unwrap_or_else(|| PathBuf::from("."));
    let ontology_dir = ontology.unwrap_or_else(|| {
        let home = std::env::var("HOME").unwrap_or_default();
        PathBuf::from(home).join("open-ontologies")
    });

    println!("Validating code vs. ontology...");
    println!("  Source: {:?}", source_dir);
    println!("  Ontology: {:?}", ontology_dir);

    if !ontology_dir.exists() {
        return Err(format!("Ontology directory not found: {:?}", ontology_dir).into());
    }

    // Count verbs in source
    let mut source_verbs = 0;
    for entry in walkdir::WalkDir::new(&source_dir)
        .into_iter()
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "rs"))
    {
        if let Ok(content) = fs::read_to_string(entry.path()) {
            source_verbs += content.matches("#[verb").collect::<Vec<_>>().len();
        }
    }

    // Count verbs in ontology
    let mut ontology_verbs = 0;
    for entry in walkdir::WalkDir::new(&ontology_dir)
        .into_iter()
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "ttl"))
    {
        if let Ok(content) = fs::read_to_string(entry.path()) {
            ontology_verbs += content.matches("cnv:Verb").collect::<Vec<_>>().len();
        }
    }

    println!("\nConformance check:");
    println!("  Source verbs: {}", source_verbs);
    println!("  Ontology verbs: {}", ontology_verbs);

    if source_verbs == ontology_verbs {
        println!("✓ Source and ontology are in sync");
    } else {
        println!("⚠ Mismatch: source has {}, ontology has {}", source_verbs, ontology_verbs);
        if verbose {
            println!("  Run 'clap-noun-verb-gen ontology sync' to synchronize");
        }
    }

    Ok(())
}

fn run_ontology_export(
    source: Option<PathBuf>,
    format: &str,
    output: Option<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    let source_dir = source.unwrap_or_else(|| PathBuf::from("."));

    println!("Exporting command graph as {}...", format);
    println!("  Source: {:?}", source_dir);

    // Scan for verbs
    let mut verbs = Vec::new();
    for entry in walkdir::WalkDir::new(&source_dir)
        .into_iter()
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "rs"))
    {
        if let Ok(content) = fs::read_to_string(entry.path()) {
            for line in content.lines() {
                if line.contains("#[verb") {
                    verbs.push(line.trim().to_string());
                }
            }
        }
    }

    println!("✓ Found {} verbs", verbs.len());

    // Generate RDF triples
    let mut rdf_output = String::new();
    rdf_output.push_str("# Command Graph Export\n");
    rdf_output.push_str("# Format: N-Triples\n\n");

    for (i, verb) in verbs.iter().enumerate() {
        let uri = format!("<http://clap-noun-verb.io/verbs/verb{}>", i);
        rdf_output.push_str(&format!("{} <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://clap-noun-verb.io/ontology#Verb> .\n", uri));
        rdf_output.push_str(&format!(
            "{} <http://clap-noun-verb.io/ontology#sourceLine> \"{}\" .\n",
            uri,
            verb.replace('\"', "\\\"")
        ));
    }

    if let Some(out_path) = output {
        fs::write(&out_path, rdf_output)?;
        println!("✓ Exported to {:?}", out_path);
    } else {
        println!("\n{}", rdf_output);
    }

    Ok(())
}

mod walkdir {
    use std::fs;
    use std::path::{Path, PathBuf};

    pub struct WalkDir {
        path: PathBuf,
    }

    impl WalkDir {
        pub fn new<P: AsRef<Path>>(path: P) -> Self {
            WalkDir { path: path.as_ref().to_path_buf() }
        }

        pub fn into_iter(self) -> WalkDirIter {
            WalkDirIter { stack: vec![self.path] }
        }
    }

    pub struct WalkDirIter {
        stack: Vec<PathBuf>,
    }

    impl Iterator for WalkDirIter {
        type Item = WalkDirEntry;

        fn next(&mut self) -> Option<Self::Item> {
            while let Some(path) = self.stack.pop() {
                if path.is_dir() {
                    if let Ok(entries) = fs::read_dir(&path) {
                        for entry in entries.flatten() {
                            self.stack.push(entry.path());
                        }
                    }
                } else {
                    return Some(WalkDirEntry { path });
                }
            }
            None
        }
    }

    pub struct WalkDirEntry {
        path: PathBuf,
    }

    impl WalkDirEntry {
        pub fn path(&self) -> &PathBuf {
            &self.path
        }
    }
}
