//! Example: Argument Groups and Conflicts (Typer-like Doc Comment Syntax)
//!
//! This example demonstrates argument relationships using Typer-like doc comment syntax:
//!
//! ## Relationship Tags
//! - `[group: name]` - Argument belongs to exclusive group "name"
//! - `[requires: arg]` - Argument requires "arg" to be present
//! - `[conflicts: arg]` - Argument conflicts with "arg"
//!
//! ## Value Configuration Tags
//! - `[env: VAR]` - Read value from environment variable VAR
//! - `[default: value]` - Default value if not provided
//! - `[value_hint: type]` - Shell completion hint (FilePath, DirPath, Url, etc.)
//!
//! ## Display Tags
//! - `[hide]` - Hide argument from help output
//! - `[help_heading: name]` - Group arguments under heading in help
//!
//! ## Behavior Tags
//! - `[global]` - Propagate argument to all subcommands
//! - `[exclusive]` - Argument cannot be used with any other arguments
//!
//! This approach follows Python Typer's philosophy: relationships are declared in doc comments,
//! not in code attributes, keeping the function signature clean and readable.

use clap_noun_verb::Result;
use clap_noun_verb_macros::{noun, verb};
use serde::Serialize;

// Business Logic Layer (Pure Functions - Reusable)
#[derive(Serialize, Debug)]
struct ExportConfig {
    format: Option<String>,
    output: Option<String>,
    filename: Option<String>,
    raw: bool,
}

#[derive(Serialize, Debug)]
struct AdvancedConfig {
    input: Option<String>,
    output: Option<String>,
    verbose: bool,
    log_level: String,
    config: Option<String>,
    version_check: bool,
}

fn create_export_config(
    format: Option<String>,
    output: Option<String>,
    filename: Option<String>,
    raw: bool,
) -> ExportConfig {
    ExportConfig { format, output, filename, raw }
}

// CLI Layer with Typer-like Doc Comment Syntax for Relationships

/// Export data with argument groups
///
/// This command demonstrates argument relationships using Typer-like doc comment syntax.
/// Relationships are declared inline with argument descriptions using bracket tags.
///
/// # Arguments
/// * `json` - Export as JSON [group: format_group]
/// * `yaml` - Export as YAML [group: format_group]
/// * `format` - Export format string [conflicts: raw]
/// * `output` - Output file path
/// * `filename` - Output filename [requires: format]
/// * `raw` - Raw output mode [conflicts: format]
#[noun("export", "Export commands")]
#[verb("data")]
fn export_data(
    json: bool,
    yaml: bool,
    format: Option<String>,
    output: Option<String>,
    filename: Option<String>,
    raw: bool,
) -> Result<ExportConfig> {
    // Determine actual format from group flags
    let actual_format = if json {
        Some("json".to_string())
    } else if yaml {
        Some("yaml".to_string())
    } else {
        format
    };

    Ok(create_export_config(actual_format, output, filename, raw))
}

fn create_advanced_config(
    input: Option<String>,
    output: Option<String>,
    verbose: bool,
    log_level: String,
    config: Option<String>,
    version_check: bool,
) -> AdvancedConfig {
    AdvancedConfig { input, output, verbose, log_level, config, version_check }
}

/// Process data with advanced configuration options
///
/// This command demonstrates all Typer-like doc comment tags for argument configuration.
///
/// # Arguments
/// * `input` - Input file path [value_hint: FilePath] [help_heading: Input/Output]
/// * `output` - Output file path [value_hint: FilePath] [help_heading: Input/Output]
/// * `verbose` - Enable verbose output [conflicts: quiet] [help_heading: Logging]
/// * `quiet` - Suppress output [conflicts: verbose] [help_heading: Logging]
/// * `log_level` - Log level to use [env: LOG_LEVEL] [default: info] [help_heading: Logging]
/// * `config` - Config file path [value_hint: FilePath] [env: CONFIG_PATH]
/// * `debug_info` - Internal debugging [hide]
/// * `version_check` - Check for updates [exclusive]
#[verb("process", "export")]
fn process_data(
    input: Option<String>,
    output: Option<String>,
    verbose: bool,
    #[allow(unused_variables)]
    quiet: bool,
    log_level: Option<String>,
    config: Option<String>,
    #[allow(unused_variables)]
    debug_info: bool,
    version_check: bool,
) -> Result<AdvancedConfig> {
    Ok(create_advanced_config(
        input,
        output,
        verbose,
        log_level.unwrap_or_else(|| "info".to_string()),
        config,
        version_check,
    ))
}

fn main() -> Result<()> {
    // Usage examples:
    //
    // ============================================
    // EXPORT DATA (Argument Relationships)
    // ============================================
    //
    // 1. Exclusive group (json OR yaml, not both):
    //    $ cargo run --example arg_groups -- export data --json
    //    Output: {"format":"json","output":null,"filename":null,"raw":false}
    //
    //    $ cargo run --example arg_groups -- export data --yaml
    //    Output: {"format":"yaml","output":null,"filename":null,"raw":false}
    //
    //    # Error: Cannot use both --json and --yaml (exclusive group)
    //    $ cargo run --example arg_groups -- export data --json --yaml
    //
    // 2. Requires (filename requires format):
    //    # Error: filename requires format
    //    $ cargo run --example arg_groups -- export data --filename test.json
    //
    //    # OK: filename with format
    //    $ cargo run --example arg_groups -- export data --format json --filename test.json
    //    Output: {"format":"json","output":null,"filename":"test.json","raw":false}
    //
    // 3. Conflicts (raw conflicts with format):
    //    # Error: raw conflicts with format
    //    $ cargo run --example arg_groups -- export data --format json --raw
    //
    //    # OK: raw without format
    //    $ cargo run --example arg_groups -- export data --raw
    //    Output: {"format":null,"output":null,"filename":null,"raw":true}
    //
    // 4. Combined:
    //    $ cargo run --example arg_groups -- export data --json -o output.json --filename test.json
    //    Output: {"format":"json","output":"output.json","filename":"test.json","raw":false}
    //
    // ============================================
    // PROCESS DATA (Advanced Tag Features)
    // ============================================
    //
    // 5. Environment variables and defaults:
    //    # Uses LOG_LEVEL env var if set, otherwise "info"
    //    $ LOG_LEVEL=debug cargo run --example arg_groups -- export process --input data.csv
    //    Output: {"input":"data.csv","output":null,"verbose":false,"log_level":"debug",...}
    //
    // 6. Value hints (shell completion):
    //    # --input and --output get file path completion in supported shells
    //    $ cargo run --example arg_groups -- export process --input <TAB>
    //
    // 7. Conflicts (verbose conflicts with quiet):
    //    # Error: Cannot use both --verbose and --quiet
    //    $ cargo run --example arg_groups -- export process --verbose --quiet
    //
    // 8. Help headings (organized help output):
    //    $ cargo run --example arg_groups -- export process --help
    //    # Shows arguments grouped under "Input/Output" and "Logging" headings
    //
    // 9. Hidden arguments (--debug-info not shown in help):
    //    $ cargo run --example arg_groups -- export process --help
    //    # --debug-info is hidden from help output
    //
    // 10. Exclusive arguments (--version-check cannot be combined):
    //    # Error: --version-check cannot be used with other arguments
    //    $ cargo run --example arg_groups -- export process --version-check --verbose

    // Auto-discover all registered commands and run
    clap_noun_verb::run()
}
