//! ggen CLI Example - Enhanced Error Handling
//!
//! This example demonstrates production-grade error handling for a CLI application
//! built with clap-noun-verb. It shows how to provide user-friendly, actionable
//! error messages that reduce support requests.
//!
//! ## Key Features
//!
//! 1. **User-Friendly Errors**: Clear problem descriptions with recovery steps
//! 2. **Input Validation**: Comprehensive validation with helpful suggestions
//! 3. **Documentation Links**: Context-sensitive help links
//! 4. **Error Categories**: Structured error types for metrics and monitoring
//!
//! ## Commands Implemented
//!
//! ### AI Commands
//! - `ggen ai generate` - Generate code with AI
//! - `ggen ai project` - Generate complete projects
//! - `ggen ai graph` - Generate RDF ontologies
//! - `ggen ai sparql` - Generate SPARQL queries
//!
//! ### Marketplace Commands
//! - `ggen marketplace search` - Search for packages
//! - `ggen marketplace install` - Install packages
//! - `ggen marketplace list` - List packages
//! - `ggen marketplace publish` - Publish packages
//!
//! ### Template Commands
//! - `ggen template generate` - Generate from template
//! - `ggen template render` - Render template
//! - `ggen template validate` - Validate template
//! - `ggen template list` - List templates
//!
//! ## Running the Example
//!
//! ```bash
//! # Build the example
//! cargo build --example ggen_cli
//!
//! # Try commands (they will show helpful errors)
//! cargo run --example ggen_cli -- ai generate -d "Create REST API"
//! cargo run --example ggen_cli -- marketplace search rust
//! cargo run --example ggen_cli -- template list
//! ```

// Module declarations
#[path = "ggen/mod.rs"]
mod ggen;

use clap_noun_verb::Result;

fn main() -> Result<()> {
    // Initialize tracing for development
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    // Run the CLI with auto-discovered commands
    // The framework will:
    // 1. Discover all #[verb] functions across modules
    // 2. Build the command structure (noun-verb pattern)
    // 3. Parse arguments and validate inputs
    // 4. Execute the appropriate handler
    // 5. Handle errors with user-friendly messages
    // 6. Serialize output to JSON

    clap_noun_verb::run()
}
