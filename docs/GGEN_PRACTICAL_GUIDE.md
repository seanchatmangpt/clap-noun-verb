# ggen for clap-noun-verb: Practical Guide

This guide shows practical ways to use ggen templates with the clap-noun-verb project. ggen provides template management, validation, and inspection tools.

## What ggen Can Do

ggen is a template engine and CLI tool that can:

1. **List templates** - View all available template files
2. **Show metadata** - Inspect template variables and structure
3. **Validate templates** - Check template syntax and validity
4. **Generate trees** - Preview file structure from templates
5. **Manage templates** - Create, lint, and organize templates

## Essential ggen Template Commands

### 1. List All Templates
```bash
# View all available templates as JSON
ggen template list

# Format nicely with jq
ggen template list | jq '.templates[] | {name, description}'

# Count total templates
ggen template list | jq '.templates | length'
```

### 2. Inspect Template Details
```bash
# Show template metadata
ggen template show --template hello.tmpl

# Show as pretty JSON
ggen template show --template hello.tmpl | jq .

# Check what variables template expects
ggen template show --template ai-generated.tmpl | jq '.variables'

# Get output path for template
ggen template show --template hello.tmpl | jq '.output_path'
```

### 3. Validate Template Syntax
```bash
# Lint a single template
ggen template lint --template hello.tmpl

# Validate all templates
for t in ~/ggen/templates/*.tmpl; do
  echo "Checking $(basename $t)..."
  ggen template lint --template "$(basename $t)"
done
```

### 4. Preview File Structure
```bash
# Show what files template would generate
ggen template generate_tree --template ai-generated.tmpl

# Show for custom template
ggen template generate_tree --template hello.tmpl
```

## How to Use Templates in clap-noun-verb

### Approach 1: Template-Based Code Design

Since ggen templates are primarily for inspection and validation, use them for:

1. **Design phase** - Review template structure
2. **Documentation** - Reference template examples
3. **Code generation patterns** - Understand what should be generated
4. **Validation** - Ensure generated code matches template intent

### Approach 2: Manual Template Instantiation

When generating code from templates:

```bash
# 1. Review the template
cat ~/ggen/templates/hello.tmpl

# 2. Understand the structure
ggen template show --template hello.tmpl | jq .

# 3. Manually instantiate by:
#    - Copying template to your project
#    - Replacing {{variable}} placeholders manually
#    - Adding your custom logic

# 4. Validate your instantiation
ggen template lint --template your-instantiated-template.tmpl
```

### Approach 3: Template-Driven Architecture

Design your clap-noun-verb commands using templates as reference:

```bash
# 1. List all available templates
ggen template list | jq '.templates[] | select(.description != null)'

# 2. Choose relevant templates
# For noun-verb CLI:
#   - ai-generated.tmpl: Service/command structure
#   - safe-error-handling.tmpl: Error types
#   - ai-ontology.tmpl: Domain model

# 3. Use templates as design reference
ggen template show --template ai-generated.tmpl

# 4. Implement inspired by template structure
```

## Practical Examples for clap-noun-verb

### Example 1: Understand AI-Generated Service Structure

```bash
# View the template
cat ~/ggen/templates/ai-generated.tmpl

# Check its metadata
ggen template show --template ai-generated.tmpl | jq .

# See what variables it expects
ggen template show --template ai-generated.tmpl | jq '.variables'

# Understand output files it would create
ggen template generate_tree --template ai-generated.tmpl
```

This shows you how to structure your noun command services.

### Example 2: Error Handling Pattern

```bash
# View error handling template
cat ~/ggen/templates/safe-error-handling.tmpl

# Check its structure
ggen template show --template safe-error-handling.tmpl

# Use as reference for your error types
# Implement similar error handling in src/errors/
```

### Example 3: Database Schema (if needed)

```bash
# View database template
cat ~/ggen/templates/database-with-migrations.tmpl

# Show structure
ggen template show --template database-with-migrations.tmpl | jq .

# Use as reference for schema design
```

### Example 4: Create Custom Noun-Verb Template

```bash
# Create your custom template
mkdir -p ~/ggen/templates/custom

cat > ~/ggen/templates/custom/noun-command.tmpl << 'EOF'
---
to: src/commands/{{ noun | snake_case }}.rs
vars:
  noun: "User"
  description: "User noun command"
---

use clap::Parser;

#[derive(Parser)]
#[command(name = "{{ noun | snake_case }}")]
#[command(about = "{{ description }}")]
pub struct {{ noun }}Command {
    #[arg(value_name = "ACTION")]
    pub action: String,
}

impl {{ noun }}Command {
    pub fn execute(&self) -> Result<(), String> {
        match self.action.as_str() {
            "list" => self.list(),
            "create" => self.create(),
            "update" => self.update(),
            "delete" => self.delete(),
            _ => Err(format!("Unknown action: {}", self.action)),
        }
    }

    fn list(&self) -> Result<(), String> {
        println!("Listing {{ noun | snake_case }}");
        Ok(())
    }

    fn create(&self) -> Result<(), String> {
        println!("Creating {{ noun | snake_case }}");
        Ok(())
    }

    fn update(&self) -> Result<(), String> {
        println!("Updating {{ noun | snake_case }}");
        Ok(())
    }

    fn delete(&self) -> Result<(), String> {
        println!("Deleting {{ noun | snake_case }}");
        Ok(())
    }
}
EOF

# Validate your custom template
ggen template lint --template custom/noun-command.tmpl

# View its structure
ggen template show --template custom/noun-command.tmpl

# List both built-in and custom templates
ggen template list | jq '.templates[] | select(.name | contains("noun"))'
```

## Template File Reference Guide

### Core Templates for clap-noun-verb

| Template | Purpose | Use Case |
|----------|---------|----------|
| `ai-generated.tmpl` | CRUD service structure | Design noun services |
| `safe-error-handling.tmpl` | Error type patterns | Design error types |
| `ai-ontology.tmpl` | Domain model structure | Design data models |
| `hello.tmpl` | Simple example | Learn template syntax |
| `rust.tmpl` | Basic Rust structure | Reference Rust patterns |

### Using Templates as Design Documents

1. **Read template file**: Understand the structure and patterns
2. **Show metadata**: Check variables and expectations
3. **Validate syntax**: Ensure template is well-formed
4. **Use as blueprint**: Implement your code inspired by template
5. **Reference patterns**: Copy good patterns to your code

## Working with Templates in Your Project

### Step 1: Design Phase (Using Templates)

```bash
# List available templates
ggen template list | jq '.templates[] | {name, description}'

# Review relevant templates
ggen template show --template ai-generated.tmpl | jq .

# Validate template syntax
ggen template lint --template ai-generated.tmpl
```

### Step 2: Implementation Phase

Use templates as reference documents:

1. Read the template file
2. Understand variable placeholders
3. Adapt structure to your needs
4. Implement your own code

### Step 3: Documentation Phase

Document your design decisions referencing templates:

```bash
# Copy template for reference
cp ~/ggen/templates/ai-generated.tmpl ./docs/reference-service-template.tmpl

# Document which templates inspired your design
git add ./docs/reference-service-template.tmpl
git commit -m "docs: Add reference template for service design"
```

## Advanced: Template Filtering and Search

```bash
# Find templates with specific keywords
ggen template list | jq '.templates[] | select(.description | contains("service"))'

# Count templates by type
ggen template list | jq '.templates | length'

# Find templates with no description
ggen template list | jq '.templates[] | select(.description == null)'

# Get all template names
ggen template list | jq '.templates[] | .name' | sort

# Find templates in specific directory
ggen template list | jq '.templates[] | select(.path | contains("cli"))'
```

## Integration with CI/CD

### Validate All Templates in CI

```bash
#!/bin/bash
# ci-validate-templates.sh

echo "Validating all ggen templates..."
ggen template list | jq -r '.templates[] | .name' | while read tmpl; do
  echo "Validating $tmpl..."
  if ! ggen template lint --template "$tmpl"; then
    echo "ERROR: Template validation failed for $tmpl"
    exit 1
  fi
done

echo "All templates valid!"
```

### Generate Template Documentation

```bash
#!/bin/bash
# generate-template-docs.sh

echo "# Available Templates

Generated from: \`ggen template list\`

\`\`\`json
$(ggen template list | jq '.templates')
\`\`\`

## Template Details

" > docs/TEMPLATES.md

ggen template list | jq -r '.templates[] | .name' | while read tmpl; do
  echo "### $tmpl

\`\`\`json
$(ggen template show --template "$tmpl" | jq .)
\`\`\`

" >> docs/TEMPLATES.md
done
```

## Tips & Tricks

### Quick Template Inspection

```bash
# One-liner to see all template names
ggen template list | jq -r '.templates[].name' | sort

# Get template count
ggen template list | jq '.templates | length'

# Find recently modified templates (if applicable)
ls -lt ~/ggen/templates/*.tmpl | head -10
```

### Template Organization

```bash
# List templates by size
find ~/ggen/templates -name "*.tmpl" -exec wc -l {} + | sort -n

# Show template with syntax highlighting
cat ~/ggen/templates/ai-generated.tmpl | pygmentize -l yaml
```

### Validate Before Using

```bash
# Always validate before referencing
template="ai-generated.tmpl"
if ggen template lint --template "$template"; then
  echo "Template is valid, safe to use"
  ggen template show --template "$template"
else
  echo "Template has errors, do not use"
fi
```

## See Also

- **ggen Repository**: https://github.com/sac/ggen
- **ggen Help**: `ggen template --help`
- **Template Files**: `~/ggen/templates/`

## Common Issues & Solutions

| Issue | Solution |
|-------|----------|
| Template not found | Run `ggen template list` to find exact name |
| Validation fails | Check YAML syntax with `ggen template lint` |
| Variables unclear | Use `ggen template show` to see variable names |
| Can't find templates | Check `~/ggen/templates/` directory |

---

**Key Insight**: ggen templates are reference documents and design guides. Use them to understand patterns and structure your own implementations.
