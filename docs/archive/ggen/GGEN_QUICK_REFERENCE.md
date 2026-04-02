# ggen Template Generation - Quick Reference

One-page reference for generating commands, code, and configurations using ggen templates.

## Core Commands

```bash
# List all available templates
ggen template list

# Generate from template
ggen template generate --template <name> --vars key=value --output <dir>

# Show template details
ggen template show --template <name>

# Validate template syntax
ggen template lint --template <name>

# Preview file structure
ggen template generate_tree --template <name>
```

## Common Templates

| Template | Purpose | Example |
|----------|---------|---------|
| `hello.tmpl` | Hello world program | Quick test |
| `rust.tmpl` | Rust project | Basic structure |
| `python.tmpl` | Python script | Python code |
| `bash.tmpl` | Bash script | Shell scripts |
| `ai-generated.tmpl` | CRUD service | Production services |
| `safe-error-handling.tmpl` | Error patterns | Error types |
| `database-with-migrations.tmpl` | DB schema | Database setup |

## One-Liners

### Generate Hello World
```bash
ggen template generate --template hello.tmpl --vars name=MyApp --output .
```

### Generate Rust Service
```bash
ggen template generate --template ai-generated.tmpl \
  --vars name=UserService,framework=axum --output ./src/services
```

### Generate Error Handling
```bash
ggen template generate --template safe-error-handling.tmpl \
  --vars project=MyApp --output ./src/errors
```

### Validate Before Generating
```bash
ggen template lint --template my-template.tmpl && \
ggen template generate --template my-template.tmpl --vars name=MyApp --output .
```

## Batch Generate Multiple

```bash
# Generate multiple services
for service in User Product Order; do
  ggen template generate \
    --template ai-generated.tmpl \
    --vars name="${service}Service" \
    --output ./src/services
done
```

## Template Syntax

```handlebars
---
to: src/{{ name | snake_case }}.rs
vars:
  name: "MyService"
  description: "Service description"
---

pub struct {{ name }} {
    // {{ description }}
}
```

### Key Elements:
- **Front matter**: YAML between `---` markers
- **`to:`**: Output file path (supports variables)
- **`vars:`**: Default variables
- **Filters**: `snake_case`, `kebab_case`, `PascalCase`, etc.

## Generate Commands for clap-noun-verb

### Generate All Noun Commands
```bash
for NOUN in User Product Order; do
  ggen template generate \
    --template noun-verb-cmd.tmpl \
    --vars noun=$NOUN \
    --output ./src/nouns
done
```

### Generate All Verbs
```bash
for VERB in Create Read Update Delete; do
  ggen template generate \
    --template verb.tmpl \
    --vars verb=$VERB \
    --output ./src/verbs
done
```

### Generate Full CLI Structure
```bash
ggen template generate --template ai-generated.tmpl \
  --vars name=NounVerbCli,description="CLI tool" \
  --output ./src

ggen template generate --template safe-error-handling.tmpl \
  --output ./src/errors

ggen template generate --template ai-ontology.tmpl \
  --vars name=CommandStructure \
  --output ./src/models
```

## Workflow

```bash
# 1. List templates
ggen template list | jq '.templates[] | {name, description}'

# 2. Choose template
TEMPLATE="ai-generated.tmpl"

# 3. Validate
ggen template lint --template $TEMPLATE

# 4. Preview
ggen template generate_tree --template $TEMPLATE

# 5. Generate
ggen template generate \
  --template $TEMPLATE \
  --vars name=MyService,description="My service" \
  --output ./generated

# 6. Review and commit
ls -la ./generated
git add ./generated
git commit -m "chore: Generate code from templates"
```

## Variable Naming Convention

```bash
# Use descriptive names matching template expectations
--vars name=UserService,description="User management"

# Avoid generic names
--vars a=x,b=y  # ❌ Don't do this
```

## Common Issues & Solutions

| Issue | Solution |
|-------|----------|
| Template not found | Run `ggen template list` to find correct name |
| Variables not substituting | Check template with `ggen template show --template <name>` |
| Wrong output path | Use `ggen template generate_tree` to preview |
| Permission errors | Ensure output directory exists: `mkdir -p <dir>` |

## Useful Patterns

### Preview-First Approach
```bash
# Always preview before generating
ggen template generate_tree --template ai-generated.tmpl
ggen template lint --template ai-generated.tmpl
ggen template show --template ai-generated.tmpl
# Then generate
ggen template generate --template ai-generated.tmpl --vars name=Test --output .
```

### Batch Generation Script
```bash
#!/bin/bash
SERVICES=(UserService ProductService OrderService)
for s in "${SERVICES[@]}"; do
  ggen template generate --template ai-generated.tmpl \
    --vars name=$s --output ./src/services
done
```

### Generate and Version
```bash
ggen template generate --template hello.tmpl --output ./gen
git add ./gen
git commit -m "chore: Generate from templates"
git push
```

## File Paths

- **Templates location**: `/Users/sac/ggen/templates/`
- **ggen binary**: `/Users/sac/.asdf/shims/ggen`
- **Current project**: `/Users/sac/clap-noun-verb/`

## See More

- **Full Guide**: `/Users/sac/clap-noun-verb/docs/GGEN_TEMPLATE_GUIDE.md`
- **Examples**: `/Users/sac/clap-noun-verb/docs/GGEN_EXAMPLES.md`
- **ggen Help**: `ggen template --help`

## Environment

```bash
# Check ggen installation
which ggen
ggen --version

# List all available templates
ggen template list

# Quick test
ggen template generate --template hello.tmpl --vars name=Test --output /tmp
cat /tmp/hello.rs
```

---

**TL;DR**: Use `ggen template generate --template <name> --vars key=value --output <dir>` to create code from templates!
