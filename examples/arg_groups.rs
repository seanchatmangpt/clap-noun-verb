//! Example: Argument Groups and Conflicts
//!
//! This example demonstrates argument groups, requires, and conflicts_with support in v3.2.0:
//! - Argument groups: Exclusive arguments (e.g., `--json` OR `--yaml`, but not both)
//! - Requires: Arguments that require other arguments
//! - Conflicts: Arguments that conflict with each other

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

fn create_export_config(format: Option<String>, output: Option<String>, filename: Option<String>, raw: bool) -> ExportConfig {
    ExportConfig { format, output, filename, raw }
}

// CLI Layer with Argument Groups and Conflicts

/// Export data with argument groups
///
/// This command demonstrates argument groups and conflicts:
/// - `json` and `yaml` are in an exclusive group (can't use both)
/// - `filename` requires `format` (if filename is used, format must be specified)
/// - `raw` conflicts with `format` (if raw is used, format cannot be used)
///
/// # Arguments
/// * `json` - Export as JSON (exclusive group: format)
/// * `yaml` - Export as YAML (exclusive group: format)
/// * `format` - Export format (conflicts with raw, required by filename)
/// * `output` - Output file path
/// * `filename` - Filename (requires format)
/// * `raw` - Raw output (conflicts with format)
///
/// **Note**: In v3.2.0, you can use argument groups and relationships:
/// - `#[arg(group = "format")]` - Exclusive groups (json OR yaml, not both)
/// - `#[arg(requires = "format")]` - filename requires format
/// - `#[arg(conflicts_with = "format")]` - raw conflicts with format
#[noun("export", "Export commands")]
#[verb("data")]
fn export_data(
    // In v3.2.0: #[arg(group = "format")] - Export as JSON (exclusive group: format)
    json: bool,
    // In v3.2.0: #[arg(group = "format")] - Export as YAML (exclusive group: format)
    yaml: bool,
    // In v3.2.0: #[arg(short = 'f', conflicts_with = "raw")] - Export format (conflicts with raw, required by filename)
    format: Option<String>,
    // In v3.2.0: #[arg(short = 'o')] - Output file path
    output: Option<String>,
    // In v3.2.0: #[arg(requires = "format")] - Filename (requires format)
    filename: Option<String>,
    // In v3.2.0: #[arg(conflicts_with = "format")] - Raw output (conflicts with format)
    raw: bool,
) -> Result<ExportConfig> {
    Ok(create_export_config(format, output, filename, raw))
}

fn main() -> Result<()> {
    // Usage examples:
    //
    // 1. Exclusive group (json OR yaml, not both):
    //    $ cargo run --example arg_groups -- export data --json
    //    Output: {"format":null,"output":null,"filename":null,"raw":false}
    //
    //    $ cargo run --example arg_groups -- export data --yaml
    //    Output: {"format":null,"output":null,"filename":null,"raw":false}
    //
    //    # Error: Cannot use both --json and --yaml
    //    $ cargo run --example arg_groups -- export data --json --yaml
    //
    // 2. Requires (filename requires format):
    //    # Error: filename requires format
    //    $ cargo run --example arg_groups -- export data --filename test.json
    //
    //    # OK: filename with format
    //    $ cargo run --example arg_groups -- export data --format json --filename test.json
    //    Output: {"format":Some("json"),"output":null,"filename":Some("test.json"),"raw":false}
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
    //    $ cargo run --example arg_groups -- export data --json -o output.json -f json --filename test.json
    //    Output: {"format":Some("json"),"output":Some("output.json"),"filename":Some("test.json"),"raw":false}

    // Auto-discover all registered commands and run
    clap_noun_verb::run()
}

