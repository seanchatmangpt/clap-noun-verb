# ggen Template Generation - Complete Documentation Index

Comprehensive guides for using ggen templates with the clap-noun-verb project.

## 📚 Documentation Files

### 1. [GGEN_QUICK_REFERENCE.md](./GGEN_QUICK_REFERENCE.md) ⚡
**One-page quick reference for busy developers**
- Core ggen commands at a glance
- Common templates and their uses
- One-liners for quick generation
- Quick reference tables

**Start here if**: You just need to remember a command quickly

### 2. [GGEN_PRACTICAL_GUIDE.md](./GGEN_PRACTICAL_GUIDE.md) 🎯
**Real-world integration guide for clap-noun-verb**
- How ggen actually works in this project
- Template inspection and validation workflows
- Using templates as design references
- Custom template creation
- Integration patterns

**Start here if**: You're using ggen with clap-noun-verb

### 3. [GGEN_TEMPLATE_GUIDE.md](./GGEN_TEMPLATE_GUIDE.md) 📖
**Comprehensive template syntax and reference**
- Template YAML front matter explanation
- Handlebars variable substitution
- Available template filters
- Complete command reference
- Best practices

**Start here if**: You want to understand template syntax deeply

### 4. [GGEN_EXAMPLES.md](./GGEN_EXAMPLES.md) 💡
**Practical examples and batch generation patterns**
- Real-world example code
- Generate multiple services
- Generate error types
- Generate noun-verb commands
- Complete workflows

**Start here if**: You need to see actual working examples

### 5. [GGEN_INTEGRATION_ANALYSIS.md](./GGEN_INTEGRATION_ANALYSIS.md) 🔍
**Analysis of ggen integration with clap-noun-verb**
- Integration points and opportunities
- Current capabilities
- Integration patterns for the project
- Implementation recommendations

**Start here if**: You want to deeply integrate ggen into the project

## 🚀 Quick Start

### First Time Users
1. Read [GGEN_QUICK_REFERENCE.md](./GGEN_QUICK_REFERENCE.md) (5 min)
2. Try one example from [GGEN_EXAMPLES.md](./GGEN_EXAMPLES.md) (10 min)
3. Check [GGEN_PRACTICAL_GUIDE.md](./GGEN_PRACTICAL_GUIDE.md) for your use case (10 min)

### Integration Work
1. Read [GGEN_PRACTICAL_GUIDE.md](./GGEN_PRACTICAL_GUIDE.md)
2. Study [GGEN_INTEGRATION_ANALYSIS.md](./GGEN_INTEGRATION_ANALYSIS.md)
3. Review [GGEN_EXAMPLES.md](./GGEN_EXAMPLES.md) for patterns
4. Implement using [GGEN_TEMPLATE_GUIDE.md](./GGEN_TEMPLATE_GUIDE.md) as reference

### Template Development
1. Start with [GGEN_QUICK_REFERENCE.md](./GGEN_QUICK_REFERENCE.md)
2. Study [GGEN_TEMPLATE_GUIDE.md](./GGEN_TEMPLATE_GUIDE.md)
3. Review examples in [GGEN_EXAMPLES.md](./GGEN_EXAMPLES.md)
4. Create custom templates in `~/ggen/templates/`

## 📋 Core ggen Commands Summary

```bash
# List all templates
ggen template list

# Show template details
ggen template show --template <name>

# Validate template
ggen template lint --template <name>

# Preview file structure
ggen template generate_tree --template <name>

# Create custom template
ggen template new --name <template-name>
```

## 🎯 Use Cases

### Use Case: Design Phase
**Goal**: Understand code structure and patterns

**Files to Read**:
- [GGEN_QUICK_REFERENCE.md](./GGEN_QUICK_REFERENCE.md) - Quick lookup
- [GGEN_PRACTICAL_GUIDE.md](./GGEN_PRACTICAL_GUIDE.md) - Design patterns

**Key Commands**:
```bash
ggen template list
ggen template show --template <template>
ggen template generate_tree --template <template>
```

### Use Case: Code Generation
**Goal**: Generate code from templates

**Files to Read**:
- [GGEN_EXAMPLES.md](./GGEN_EXAMPLES.md) - Real examples
- [GGEN_TEMPLATE_GUIDE.md](./GGEN_TEMPLATE_GUIDE.md) - Syntax reference

**Pattern**:
1. Choose template
2. Review structure
3. Validate template
4. Generate code

### Use Case: Custom Template Creation
**Goal**: Create and use custom templates

**Files to Read**:
- [GGEN_QUICK_REFERENCE.md](./GGEN_QUICK_REFERENCE.md) - Command reference
- [GGEN_TEMPLATE_GUIDE.md](./GGEN_TEMPLATE_GUIDE.md) - Template syntax
- [GGEN_EXAMPLES.md](./GGEN_EXAMPLES.md) - Template examples

**Steps**:
1. Create `.tmpl` file
2. Add YAML front matter
3. Add Handlebars templates
4. Validate with `ggen template lint`
5. Use with generation tools

### Use Case: Integration with clap-noun-verb
**Goal**: Integrate ggen into development workflow

**Files to Read**:
- [GGEN_PRACTICAL_GUIDE.md](./GGEN_PRACTICAL_GUIDE.md) - Integration patterns
- [GGEN_INTEGRATION_ANALYSIS.md](./GGEN_INTEGRATION_ANALYSIS.md) - Deep analysis
- [GGEN_EXAMPLES.md](./GGEN_EXAMPLES.md) - Practical patterns

**Topics**:
- Template-driven architecture
- Noun-verb CLI generation
- Error type generation
- Service structure design

## 📊 Available Templates

### Basic Templates
- `hello.tmpl` - Hello world example
- `rust.tmpl` - Basic Rust project
- `python.tmpl` - Python script
- `bash.tmpl` - Bash script

### Service Templates
- `ai-generated.tmpl` - CRUD service (most useful)
- `ai-client-wrapper.tmpl` - Client wrapper pattern
- `ai-generators.tmpl` - Code generator pattern

### Infrastructure Templates
- `database-with-migrations.tmpl` - Database schema
- `safe-error-handling.tmpl` - Error patterns
- `production-readiness-demo.tmpl` - Production patterns

### Domain-Specific Templates
- `ai-ontology.tmpl` - Ontology modeling
- `ai-sparql.tmpl` - SPARQL queries

### Custom Project Templates
- `cli/` - CLI-specific templates
- `cleanroom/` - Cleanroom patterns
- `papers/` - Research paper templates

## 🔧 Utilities

### Scripts

#### `scripts/generate_examples.sh`
Automated example generation script

```bash
# Run all examples
./scripts/generate_examples.sh

# Run specific example
./scripts/generate_examples.sh hello
./scripts/generate_examples.sh service
./scripts/generate_examples.sh noun-verb

# List examples
./scripts/generate_examples.sh list
```

**Available Examples**:
- `hello` - Hello world
- `rust` - Rust project
- `service` - AI service
- `errors` - Error handling
- `database` - Database schema
- `batch` - Batch services
- `noun-verb` - Noun-verb commands
- `all` - All examples (default)

## 📖 Reading Path by Goal

### "I want to understand ggen"
1. [GGEN_QUICK_REFERENCE.md](./GGEN_QUICK_REFERENCE.md) (5 min)
2. [GGEN_TEMPLATE_GUIDE.md](./GGEN_TEMPLATE_GUIDE.md) (20 min)
3. [GGEN_EXAMPLES.md](./GGEN_EXAMPLES.md) (15 min)

### "I want to generate code NOW"
1. [GGEN_QUICK_REFERENCE.md](./GGEN_QUICK_REFERENCE.md) (5 min)
2. [GGEN_EXAMPLES.md](./GGEN_EXAMPLES.md) - Find your use case (10 min)
3. Copy and adapt the example

### "I want to create custom templates"
1. [GGEN_QUICK_REFERENCE.md](./GGEN_QUICK_REFERENCE.md)
2. [GGEN_TEMPLATE_GUIDE.md](./GGEN_TEMPLATE_GUIDE.md)
3. [GGEN_EXAMPLES.md](./GGEN_EXAMPLES.md) - Review template examples
4. Create your template in `~/ggen/templates/`

### "I want to integrate with clap-noun-verb"
1. [GGEN_PRACTICAL_GUIDE.md](./GGEN_PRACTICAL_GUIDE.md)
2. [GGEN_INTEGRATION_ANALYSIS.md](./GGEN_INTEGRATION_ANALYSIS.md)
3. [GGEN_EXAMPLES.md](./GGEN_EXAMPLES.md) - Review patterns
4. Implement custom templates for your commands

## 🗂️ File Organization

```
docs/
├── GGEN_INDEX.md                      # This file
├── GGEN_QUICK_REFERENCE.md            # Quick lookup
├── GGEN_PRACTICAL_GUIDE.md            # Real-world usage
├── GGEN_TEMPLATE_GUIDE.md             # Syntax reference
├── GGEN_EXAMPLES.md                   # Working examples
└── GGEN_INTEGRATION_ANALYSIS.md       # Deep integration analysis

scripts/
└── generate_examples.sh               # Example generation script

templates/                             # ggen templates (in ~/ggen)
├── hello.tmpl
├── rust.tmpl
├── ai-generated.tmpl
├── safe-error-handling.tmpl
└── ... (37+ more templates)
```

## 💡 Key Concepts

### Template Structure
- **YAML Front Matter**: Configuration (to, vars)
- **Handlebars**: Variable substitution
- **Filters**: Case conversion (snake_case, PascalCase, etc.)

### Common Patterns
- **CRUD Services**: Use `ai-generated.tmpl`
- **Error Types**: Use `safe-error-handling.tmpl`
- **Noun Commands**: Create custom templates
- **Verb Actions**: Design from patterns

### Integration Points
- Template inspection for design
- Custom template creation for patterns
- Batch generation for multiple entities
- Reference templates for documentation

## 🎓 Learning Resources

### Official Resources
- ggen GitHub: https://github.com/sac/ggen
- ggen CLI Help: `ggen template --help`
- Template Directory: `~/ggen/templates/`

### Project-Specific Resources
- This documentation set
- Example templates in project
- Generated examples in `examples/generated/`

## ✅ Checklist: Using ggen Effectively

- [ ] Read GGEN_QUICK_REFERENCE for commands
- [ ] Explore templates with `ggen template list`
- [ ] Review structure with `ggen template show`
- [ ] Validate templates with `ggen template lint`
- [ ] Preview output with `ggen template generate_tree`
- [ ] Create custom templates as needed
- [ ] Document template usage in your project
- [ ] Version control template definitions
- [ ] Update docs when adding new templates

## 🚀 Next Steps

1. **Get Started**: Read [GGEN_QUICK_REFERENCE.md](./GGEN_QUICK_REFERENCE.md)
2. **Explore**: Use `ggen template list` to see what's available
3. **Learn**: Review relevant guide based on your use case
4. **Practice**: Try examples from [GGEN_EXAMPLES.md](./GGEN_EXAMPLES.md)
5. **Create**: Design custom templates for clap-noun-verb
6. **Integrate**: Use templates in your development workflow

---

**Last Updated**: November 2024
**ggen Version**: Latest (from `~/ggen`)
**Project**: clap-noun-verb
