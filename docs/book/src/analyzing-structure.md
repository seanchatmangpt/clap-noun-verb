# Analyzing ggen's Current Structure

Before porting, we need to understand ggen's current CLI structure and map it to the noun-verb pattern. This chapter analyzes the existing commands and identifies how they should be organized.

## Overview of ggen commands

Based on the ggen documentation, here are the main command categories:

### AI Commands

```bash
ggen ai project "E-commerce API with Stripe" --name shop-api --rust
ggen ai generate -d "Database repository pattern" -o repo.tmpl
ggen ai graph -d "User management ontology" -o users.ttl
ggen ai sparql -d "Find all active users" -g schema.ttl
```

### Marketplace Commands

```bash
ggen search "rust web"
ggen add io.ggen.rust.axum
ggen list
ggen update
```

### Template Generation Commands

```bash
ggen gen example.tmpl --vars name=my_module
```

### Utility Commands

There may be additional commands like:
- `ggen --help`
- `ggen --version`
- `ggen doctor` (diagnostics)
- `ggen help-me` (tips)

## Identifying nouns

**Nouns** are entities or concepts that group related actions. In ggen's case, we can identify:

### 1. `ai` - AI-powered generation

This groups all AI-related commands:
- `project` - Generate complete projects
- `generate` - Generate templates from descriptions
- `graph` - Generate RDF ontologies
- `sparql` - Generate SPARQL queries

### 2. `marketplace` - Template marketplace

This groups marketplace-related operations:
- `search` - Find packages
- `add` - Install package
- `list` - List installed packages
- `update` - Update packages

### 3. `template` - Template operations (potential)

If there are template-specific commands beyond generation:
- `generate` - Generate from template
- `validate` - Validate template
- `list` - List templates

### 4. Root-level commands

Some commands might not need a noun wrapper if they're standalone:
- `doctor` - Diagnostics
- `help-me` - Get tips

However, we could group them under:
- `utils` - Utility commands

Or keep them as root-level if that makes more sense.

## Identifying verbs

**Verbs** are actions performed on nouns. For each noun identified above, here are the verbs:

### AI Verbs

| Verb | Description | Arguments |
|------|-------------|-----------|
| `project` | Generate complete projects | `name`, `--rust` flag |
| `generate` | Generate templates | `-d/--description`, `-o/--output` |
| `graph` | Generate RDF ontologies | `-d/--description`, `-o/--output` |
| `sparql` | Generate SPARQL queries | `-d/--description`, `-g/--graph`, `-o/--output` |

### Marketplace Verbs

| Verb | Description | Arguments |
|------|-------------|-----------|
| `search` | Find packages | `query` (required) |
| `add` | Install package | `package` (required) |
| `list` | List installed packages | None |
| `update` | Update packages | None |

### Template Verbs (if applicable)

| Verb | Description | Arguments |
|------|-------------|-----------|
| `generate` | Generate from template | `template`, `--vars` |
| `validate` | Validate template | `template` |
| `list` | List templates | None |

## Command hierarchy analysis

Let's analyze the current structure and how it maps to noun-verb:

### Current Structure (Regular clap)

Assuming a typical clap structure, ggen likely uses:

```
ggen (root)
├── ai (subcommand)
│   ├── project (subcommand)
│   ├── generate (subcommand)
│   ├── graph (subcommand)
│   └── sparql (subcommand)
├── search (subcommand)
├── add (subcommand)
├── list (subcommand)
└── update (subcommand)
```

### Target Structure (clap-noun-verb)

```
ggen (root)
├── ai (noun)
│   ├── project (verb)
│   ├── generate (verb)
│   ├── graph (verb)
│   └── sparql (verb)
├── marketplace (noun)
│   ├── search (verb)
│   ├── add (verb)
│   ├── list (verb)
│   └── update (verb)
└── template (noun, optional)
    ├── generate (verb)
    ├── validate (verb)
    └── list (verb)
```

### Decisions to Make

1. **Marketplace grouping**: Should `search`, `add`, `list`, `update` be grouped under `marketplace` noun, or stay as root-level commands?

   **Recommendation**: Group under `marketplace` for consistency and scalability. If users prefer root-level, we can alias them.

2. **Template vs Generation**: Is there a distinction between AI generation (`ai generate`) and template generation (`gen`), or should they be unified?

   **Recommendation**: Keep separate - `ai generate` uses LLMs, while `gen` uses existing templates.

3. **Global arguments**: What global arguments does ggen support?
   - `--verbose` / `-v` (verbosity)
   - `--config` / `-c` (config file)
   - Potentially others

## Mapping commands to noun-verb structure

Here's the complete mapping:

### AI Commands

```rust,no_run
noun!("ai", "AI-powered generation", [
    verb!("project", "Generate complete projects", handler, args: [
        Arg::new("name").required(true),
        Arg::new("rust").long("rust"),
        // ... project description, etc.
    ]),
    verb!("generate", "Generate templates from descriptions", handler, args: [
        Arg::new("description").short('d').long("description").required(true),
        Arg::new("output").short('o').long("output"),
    ]),
    verb!("graph", "Generate RDF ontologies", handler, args: [
        Arg::new("description").short('d').long("description").required(true),
        Arg::new("output").short('o').long("output"),
    ]),
    verb!("sparql", "Generate SPARQL queries", handler, args: [
        Arg::new("description").short('d').long("description").required(true),
        Arg::new("graph").short('g').long("graph").required(true),
        Arg::new("output").short('o').long("output"),
    ]),
])
```

### Marketplace Commands

```rust,no_run
noun!("marketplace", "Template marketplace", [
    verb!("search", "Find packages", handler, args: [
        Arg::new("query").required(true),
    ]),
    verb!("add", "Install package", handler, args: [
        Arg::new("package").required(true),
    ]),
    verb!("list", "List installed packages", handler),
    verb!("update", "Update packages", handler),
])
```

### Template Commands (if applicable)

```rust,no_run
noun!("template", "Template operations", [
    verb!("generate", "Generate from template", handler, args: [
        Arg::new("template").required(true),
        Arg::new("vars").long("vars").num_args(1..),
    ]),
    verb!("validate", "Validate template", handler, args: [
        Arg::new("template").required(true),
    ]),
    verb!("list", "List templates", handler),
])
```

## Command grouping strategy

When grouping commands, consider:

### 1. Semantic Cohesion

Commands should naturally belong together:
- ✅ `ai project`, `ai generate` - All use AI/LLMs
- ✅ `marketplace search`, `marketplace add` - All manage marketplace packages
- ❌ Don't force unrelated commands into the same noun

### 2. User Mental Model

How do users think about the commands?
- "I want to use AI to generate something" → `ai` noun
- "I want to manage marketplace packages" → `marketplace` noun

### 3. Scalability

Consider future commands:
- `ai` might get `refine`, `explain`, `debug` verbs
- `marketplace` might get `remove`, `info`, `publish` verbs

### 4. Consistency

Use consistent naming patterns:
- All verbs within a noun should follow similar patterns
- Similar verbs across nouns should have similar behavior

## Next Steps

Now that we've analyzed the structure, we're ready to:

1. [Getting Started with Porting](getting-started.md) - Set up the project and understand the framework APIs
2. [Porting Commands Step-by-Step](porting-commands.md) - Implement each command group

