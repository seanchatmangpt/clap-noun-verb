# ggen Template Generation Guide

This guide shows you how to use `ggen` to work with templates for generating commands, code, and configurations. ggen provides tools for template validation, inspection, and management.

## Quick Reference: ggen Template Commands

### List Available Templates
```bash
ggen template list
```
Lists all available templates with descriptions as JSON output.

### Show Template Metadata
```bash
ggen template show --template <template-name>
```
View details about a specific template including variables and output paths.

### Lint a Template
```bash
ggen template lint --template <template-name>
```
Validate template syntax and structure.

### Generate File Tree
```bash
ggen template generate_tree --template <template-name>
```
Show the file structure that the template would generate.

### Create New Template
```bash
ggen template new --name <template-name>
```
Create a new template with ggen scaffold.

## Template Syntax

Templates use YAML front matter with Handlebars-style variable substitution:

```handlebars
---
to: src/{{ name | snake_case }}.rs
vars:
  name: "UserService"
  description: "A service for managing users"
  language: "rust"
---

// {{ description }}
pub struct {{ name }} {
    // implementation
}
```

### Key Features:
- **Front matter**: YAML block between `---` markers
- **`to:` field**: Output file path (supports template variables)
- **`vars:` section**: Default variable definitions
- **Filters**: `{{ variable | snake_case }}`, `{{ variable | kebab_case }}`, etc.
- **Nested handlebars**: Use `{{` and `}}` for template variables

## Available Templates

### Basic Templates
- **hello.tmpl** - Simple "Hello World" example
- **rust.tmpl** - Basic Rust project structure
- **python.tmpl** - Basic Python script
- **bash.tmpl** - Basic Bash script

### Complex/Production Templates
- **ai-generated.tmpl** - AI-generated CRUD service template
- **ai-client-wrapper.tmpl** - AI-powered client wrapper
- **ai-generators.tmpl** - Code generation assistant template
- **rust-service-with-placeholders.tmpl** - Production-ready service
- **safe-error-handling.tmpl** - Error handling patterns
- **database-with-migrations.tmpl** - Database schema with migrations
- **production-readiness-demo.tmpl** - Production tracking patterns

### Domain-Specific Templates
- **ai-ontology.tmpl** - E-commerce domain ontology
- **ai-sparql.tmpl** - SPARQL query templates

## Practical Examples

### Example 1: Generate a Simple Rust Service

```bash
cd ~/output-dir
ggen template generate \
  --template ai-generated.tmpl \
  --vars name=ProductService,description="Product catalog service",framework=axum \
  --output .
```

This generates a `ai_generated_product_service.rs` file with:
- Struct definitions for Product and CreateProductRequest
- Service implementation with CRUD methods
- Axum router with HTTP handlers
- Full async/await pattern

### Example 2: Generate from Hello Template

```bash
ggen template generate \
  --template hello.tmpl \
  --vars name=MyProject \
  --output ./generated
```

Output: `hello.rs` with "Hello, MyProject!" program

### Example 3: Generate Database Schema

```bash
ggen template generate \
  --template database-with-migrations.tmpl \
  --vars db_name=myapp,tables=users,posts,comments \
  --output ./migrations
```

### Example 4: Generate Error Handling Pattern

```bash
ggen template generate \
  --template safe-error-handling.tmpl \
  --vars project_name=MyApp,error_types=Database,Network,Validation \
  --output ./src
```

## Workflow Integration

### 1. Template-First Development
```bash
# 1. List available templates
ggen template list | jq '.templates[] | {name, description}'

# 2. Show specific template
ggen template show --template ai-generated.tmpl

# 3. Validate before generating
ggen template lint --template ai-generated.tmpl

# 4. Preview file tree
ggen template generate_tree --template ai-generated.tmpl

# 5. Generate code
ggen template generate \
  --template ai-generated.tmpl \
  --vars name=MyService \
  --output ./src
```

### 2. Batch Generation Script
```bash
#!/bin/bash
# Generate multiple services from templates

for service in UserService ProductService OrderService; do
  ggen template generate \
    --template ai-generated.tmpl \
    --vars name=$service,framework=axum \
    --output ./src/services
done
```

### 3. Custom Template Creation
```bash
# Create your own template
cat > templates/my-cli.tmpl << 'EOF'
---
to: "src/{{ name | snake_case }}.rs"
vars:
  name: "MyCli"
  description: "My custom CLI tool"
---

use clap::Parser;

#[derive(Parser)]
#[command(name = "{{ name }}")]
#[command(about = "{{ description }}", long_about = None)]
struct Args {
    #[arg(short, long)]
    name: String,
}

fn main() {
    let args = Args::parse();
    println!("Hello, {}!", args.name);
}
EOF

# Generate from custom template
ggen template generate \
  --template my-cli.tmpl \
  --vars name=MyTool \
  --output .
```

## Template Variable Filters

Common filters available:
- `snake_case` - Convert to snake_case
- `kebab_case` - Convert to kebab-case
- `PascalCase` - Convert to PascalCase
- `SCREAMING_SNAKE_CASE` - Convert to SCREAMING_SNAKE_CASE

Example:
```handlebars
pub struct {{ name | PascalCase }} {
    fn {{ name | snake_case }}() {}
    const {{ name | SCREAMING_SNAKE_CASE }}: &str = "";
}
```

## Best Practices

### 1. **Validate Before Generation**
```bash
ggen template lint --template my-template.tmpl
```

### 2. **Preview Output First**
```bash
ggen template generate_tree --template ai-generated.tmpl
```

### 3. **Use Meaningful Variable Names**
```bash
# Good - clear variable names
ggen template generate \
  --template ai-generated.tmpl \
  --vars name=OrderProcessor,framework=axum,database=postgres

# Avoid - unclear variables
ggen template generate \
  --template ai-generated.tmpl \
  --vars a=x,b=y
```

### 4. **Organize Output**
```bash
# Generate to proper directory structure
ggen template generate \
  --template ai-generated.tmpl \
  --vars name=UserService \
  --output ./src/services

ggen template generate \
  --template safe-error-handling.tmpl \
  --output ./src/errors
```

### 5. **Version Control Generated Code**
```bash
git add generated-code/
git commit -m "chore: Generate code from templates"
```

## Troubleshooting

### Template Not Found
```bash
# List templates to find correct name
ggen template list

# Use exact template name
ggen template show --template ai-generated.tmpl
```

### Variable Substitution Issues
```bash
# Check template syntax
ggen template lint --template my-template.tmpl

# Verify variables match template expectations
ggen template show --template ai-generated.tmpl | grep -A 10 "vars:"
```

### File Output Issues
```bash
# Preview what files will be created
ggen template generate_tree --template ai-generated.tmpl

# Ensure output directory exists
mkdir -p ./output
ggen template generate --template ai-generated.tmpl --output ./output
```

## Integration with clap-noun-verb

### Generating CLI Commands with Templates

1. **Design command structure using templates**
```bash
# Generate a base CLI structure
ggen template generate \
  --template ai-generated.tmpl \
  --vars name=NounVerbCli,description="Clap noun-verb command generator" \
  --output ./generated
```

2. **Customize for your needs**
- Edit generated files to match your requirements
- Add your custom logic on top of template output

3. **Repeat for different command modules**
```bash
# Generate multiple command modules
for cmd in parse generate validate; do
  ggen template generate \
    --template ai-generated.tmpl \
    --vars name="${cmd^}Command" \
    --output "./src/commands"
done
```

## See Also

- `ggen template --help` - Full command documentation
- `/Users/sac/ggen/templates/` - Template directory
- Template examples in the ggen project

## Quick Commands Cheatsheet

```bash
# List all templates
ggen template list

# Generate from template
ggen template generate --template hello.tmpl --vars name=MyApp --output .

# Validate template
ggen template lint --template my-template.tmpl

# Preview file structure
ggen template generate_tree --template ai-generated.tmpl

# Show template details
ggen template show --template ai-generated.tmpl

# Create custom template (vim)
vim ~/ggen/templates/my-new-template.tmpl

# Generate multiple services
for s in User Product Order; do
  ggen template generate --template ai-generated.tmpl \
    --vars name="${s}Service" --output ./src/services
done
```

---

**Note**: All template generation uses the ggen binary. Ensure it's installed:
```bash
which ggen  # Should show /Users/sac/.asdf/shims/ggen
```
