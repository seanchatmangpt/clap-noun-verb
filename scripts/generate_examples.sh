#!/bin/bash
#
# ggen Template Generation Examples Script
# Demonstrates practical usage of ggen for code generation
#
# Usage:
#   ./scripts/generate_examples.sh
#   ./scripts/generate_examples.sh hello
#   ./scripts/generate_examples.sh rust-service
#

set -e

GGEN_BINARY=$(which ggen)
OUTPUT_DIR="./examples/generated"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Helper functions
print_header() {
    echo -e "\n${BLUE}===================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}===================================${NC}\n"
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

print_info() {
    echo -e "${YELLOW}ℹ $1${NC}"
}

# Main functions

validate_ggen() {
    if ! command -v ggen &> /dev/null; then
        print_error "ggen not found. Please install: https://github.com/sac/ggen"
        exit 1
    fi
    print_success "ggen found at: $GGEN_BINARY"
}

list_templates() {
    print_header "Available Templates"
    ggen template list | jq -r '.templates[] | "\(.name)\t\(.description // "N/A")"' | column -t -s $'\t'
}

example_hello_world() {
    print_header "Example 1: Hello World"

    local output="$OUTPUT_DIR/hello_world"
    mkdir -p "$output"

    print_info "Generating hello world program..."
    ggen template generate \
        --template hello.tmpl \
        --vars name=NounVerb \
        --output "$output"

    print_success "Generated hello world program"

    if [ -f "$output/hello.rs" ]; then
        print_info "Output file: $output/hello.rs"
        echo -e "\n${BLUE}Content:${NC}"
        cat "$output/hello.rs"
    fi
}

example_rust_basic() {
    print_header "Example 2: Basic Rust Project"

    local output="$OUTPUT_DIR/rust_basic"
    mkdir -p "$output"

    print_info "Generating basic Rust project..."
    ggen template generate \
        --template rust.tmpl \
        --vars name=MyApp \
        --output "$output"

    print_success "Generated Rust project"

    if [ -f "$output/src/main.rs" ]; then
        print_info "Output file: $output/src/main.rs"
        echo -e "\n${BLUE}Content:${NC}"
        cat "$output/src/main.rs"
    fi
}

example_ai_service() {
    print_header "Example 3: AI-Generated Service"

    local output="$OUTPUT_DIR/ai_service"
    mkdir -p "$output"

    print_info "Generating AI-powered service template..."
    ggen template generate \
        --template ai-generated.tmpl \
        --vars name=UserService,description="User management service",framework=axum \
        --output "$output"

    print_success "Generated AI service"

    if [ -f "$output/ai_generated_user_service.rs" ]; then
        print_info "Output file: $output/ai_generated_user_service.rs"
        echo -e "\n${BLUE}Preview (first 50 lines):${NC}"
        head -50 "$output/ai_generated_user_service.rs"
        echo "..."
    fi
}

example_error_handling() {
    print_header "Example 4: Error Handling Patterns"

    local output="$OUTPUT_DIR/error_handling"
    mkdir -p "$output"

    print_info "Generating error handling template..."
    ggen template generate \
        --template safe-error-handling.tmpl \
        --vars project=NounVerbCli \
        --output "$output"

    print_success "Generated error handling patterns"

    if [ -f "$output/safe_error_handling.rs" ]; then
        print_info "Output file: $output/safe_error_handling.rs"
        echo -e "\n${BLUE}Preview (first 40 lines):${NC}"
        head -40 "$output/safe_error_handling.rs"
        echo "..."
    fi
}

example_database() {
    print_header "Example 5: Database Schema with Migrations"

    local output="$OUTPUT_DIR/database"
    mkdir -p "$output"

    print_info "Generating database template..."
    ggen template generate \
        --template database-with-migrations.tmpl \
        --vars db_name=myapp \
        --output "$output"

    print_success "Generated database schema"

    if [ -f "$output/database_with_migrations.sql" ]; then
        print_info "Output file: $output/database_with_migrations.sql"
        echo -e "\n${BLUE}Preview (first 40 lines):${NC}"
        head -40 "$output/database_with_migrations.sql"
        echo "..."
    fi
}

example_batch_services() {
    print_header "Example 6: Batch Generate Multiple Services"

    local output="$OUTPUT_DIR/batch_services"
    mkdir -p "$output"

    local services=("User" "Product" "Order")

    for service in "${services[@]}"; do
        print_info "Generating ${service}Service..."
        ggen template generate \
            --template ai-generated.tmpl \
            --vars name="${service}Service",description="${service} management service",framework=axum \
            --output "$output"
    done

    print_success "Generated ${#services[@]} services"

    print_info "Generated files:"
    ls -1 "$output"/*.rs 2>/dev/null | head -5
}

example_preview_tree() {
    print_header "Example 7: Preview File Tree Before Generation"

    print_info "Previewing file structure for ai-generated.tmpl..."
    ggen template generate_tree --template ai-generated.tmpl
}

example_validate_template() {
    print_header "Example 8: Validate Template Syntax"

    print_info "Validating hello.tmpl..."
    if ggen template lint --template hello.tmpl 2>&1; then
        print_success "Template validation passed"
    else
        print_error "Template validation failed"
    fi
}

example_show_metadata() {
    print_header "Example 9: Show Template Metadata"

    print_info "Template: ai-generated.tmpl"
    ggen template show --template ai-generated.tmpl | head -20
}

example_clap_noun_verb() {
    print_header "Example 10: Generate clap-noun-verb Commands"

    local output="$OUTPUT_DIR/noun_verb_commands"
    mkdir -p "$output"

    print_info "Generating noun-verb CLI structure..."

    # Create a temporary template for noun-verb structure
    cat > /tmp/noun-verb-template.tmpl << 'EOF'
---
to: "{{ name | snake_case }}_command.rs"
vars:
  name: "UserCommand"
  description: "User noun command"
---

use clap::Parser;

#[derive(Parser)]
#[command(name = "{{ name | snake_case }}")]
#[command(about = "{{ description }}")]
pub struct {{ name }} {
    #[arg(value_name = "ACTION")]
    pub action: String,
}

impl {{ name }} {
    pub fn execute(&self) -> Result<(), String> {
        match self.action.as_str() {
            "list" => Ok(println!("Listing")),
            "create" => Ok(println!("Creating")),
            "delete" => Ok(println!("Deleting")),
            _ => Err(format!("Unknown action: {}", self.action)),
        }
    }
}
EOF

    # Generate for multiple nouns
    local nouns=("User" "Product" "Order")

    for noun in "${nouns[@]}"; do
        print_info "Generating ${noun}Command..."
        ggen template generate \
            --template /tmp/noun-verb-template.tmpl \
            --vars name="${noun}Command",description="${noun} command" \
            --output "$output"
    done

    print_success "Generated ${#nouns[@]} noun-verb commands"

    print_info "Generated files:"
    ls -1 "$output"/*.rs 2>/dev/null

    print_info "Sample content:"
    head -20 "$output"/*_command.rs | head -25
}

show_summary() {
    print_header "Generated Files Summary"

    if [ -d "$OUTPUT_DIR" ]; then
        print_info "Output directory: $OUTPUT_DIR"
        echo -e "\n${BLUE}File structure:${NC}"
        find "$OUTPUT_DIR" -type f | sort | sed 's|^|  |'

        echo -e "\n${BLUE}Statistics:${NC}"
        local total_files=$(find "$OUTPUT_DIR" -type f | wc -l)
        local total_lines=$(find "$OUTPUT_DIR" -type f -exec wc -l {} + 2>/dev/null | tail -1 | awk '{print $1}')

        echo "  Total files generated: $total_files"
        echo "  Total lines of code: $total_lines"
    else
        print_error "Output directory not found"
    fi
}

show_help() {
    cat << EOF
${BLUE}ggen Template Generation Examples${NC}

Usage: $0 [EXAMPLE]

Examples:
  none              Run all examples
  hello             Generate hello world
  rust              Generate basic Rust project
  service           Generate AI service
  errors            Generate error handling patterns
  database          Generate database schema
  batch             Batch generate services
  tree              Preview file tree
  validate          Validate template
  metadata          Show template metadata
  noun-verb         Generate noun-verb commands
  list              List available templates
  all               Run all examples (default)
  help              Show this help message

${YELLOW}Environment:${NC}
  ggen binary: $GGEN_BINARY
  output dir:  $OUTPUT_DIR

${YELLOW}Quick Start:${NC}
  $0 hello          # Generate hello world example
  $0 service        # Generate service example
  $0 all            # Run all examples

EOF
}

# Main execution
main() {
    validate_ggen

    local example="${1:-all}"

    case "$example" in
        hello)
            example_hello_world
            show_summary
            ;;
        rust)
            example_rust_basic
            show_summary
            ;;
        service|ai)
            example_ai_service
            show_summary
            ;;
        errors|error)
            example_error_handling
            show_summary
            ;;
        database|db)
            example_database
            show_summary
            ;;
        batch)
            example_batch_services
            show_summary
            ;;
        tree)
            example_preview_tree
            ;;
        validate)
            example_validate_template
            ;;
        metadata)
            example_show_metadata
            ;;
        noun-verb|clap)
            example_clap_noun_verb
            show_summary
            ;;
        list)
            list_templates
            ;;
        all)
            example_hello_world
            example_rust_basic
            example_ai_service
            example_error_handling
            example_batch_services
            example_clap_noun_verb
            show_summary
            ;;
        help|--help|-h)
            show_help
            ;;
        *)
            print_error "Unknown example: $example"
            show_help
            exit 1
            ;;
    esac

    echo -e "\n${GREEN}Done!${NC}\n"
}

main "$@"
