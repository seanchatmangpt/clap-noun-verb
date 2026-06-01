// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! clap-noun-verb-gen - CLI generator for clap-noun-verb patterns
//!
//! Generates ready-to-compile Rust CLI code from specifications:
//! - RDF/TTL ontologies (gen from-ttl)
//! - Declarative YAML (gen from-yaml)
//! - Minimal scaffolds (gen scaffold)

use clap::{Parser, Subcommand};
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

    fn new_scaffold(
        name: &str,
        with_examples: bool,
    ) -> Result<Self, Box<dyn std::error::Error>> {
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

    fn generate(&self, output_dir: &PathBuf, with_cargo: bool) -> Result<(), Box<dyn std::error::Error>> {
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

        let output = Command::new("cargo")
            .arg("check")
            .current_dir(output_dir)
            .output()?;

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
                        .last()
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
        let mut code = "// Copyright (c) 2024\n// SPDX-License-Identifier: MIT OR Apache-2.0\n\n".to_string();
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
        s.to_lowercase()
            .replace('-', "_")
            .replace(' ', "_")
    }

    fn to_pascal_case(s: &str) -> String {
        s.split(|c: char| !c.is_alphanumeric())
            .filter(|s| !s.is_empty())
            .map(|s| {
                let mut chars = s.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => {
                        first.to_uppercase().collect::<String>() + chars.as_str()
                    }
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

    generator.generate(&output, with_cargo)?;

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
        generator.verify_compile(&output)?;
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

    generator.generate(&output, false)?;

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
        generator.verify_compile(&output)?;
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

    generator.generate(&output, with_cargo)?;

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
