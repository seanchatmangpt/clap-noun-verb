# clap-noun-verb Examples

**Organized by [Diataxis](https://diataxis.fr/) Framework**

---

## Quick Navigation

| I want to... | Go to... | Examples |
|--------------|----------|----------|
| **Learn the basics** | [tutorial/](tutorial/) | basic, arguments, positional |
| **Solve a specific task** | [howto/](howto/) | arg_groups, validation, env_vars |
| **See complete API usage** | [reference/](reference/) | attribute_macro, framework, nested |
| **Explore advanced features** | [advanced/](advanced/) | autonomic, async, io_advanced |
| **Experiment with RDF/MCP** | [playground/](playground/) | rdf_mcp, sparql, semantic |

---

## 🎓 Tutorial Examples (Learning-Oriented)

**For:** Developers learning clap-noun-verb from scratch

```bash
# Start here - simplest working CLI
cargo run --example tutorial_basic

# Add arguments step by step
cargo run --example tutorial_arguments

# Positional arguments
cargo run --example tutorial_positional -- file.txt --verbose
```

**Examples:**
- `tutorial/basic.rs` - 5-minute "Hello World" CLI
- `tutorial/arguments.rs` - Adding typed arguments
- `tutorial/positional.rs` - Positional vs named arguments
- `tutorial/services.rs` - Multi-noun CLI structure

---

## 🔧 How-To Examples (Task-Oriented)

**For:** Developers solving specific problems

```bash
# Argument relationships (groups, requires, conflicts)
cargo run --example howto_arg_groups -- export data --json

# Input validation
cargo run --example howto_validation

# Environment variables
LOG_LEVEL=debug cargo run --example howto_env_vars
```

**Examples:**
- `howto/arg_groups.rs` - Mutually exclusive arguments, dependencies
- `howto/validation.rs` - Custom validators, constraints
- `howto/env_vars.rs` - Environment variable integration
- `howto/arg_actions.rs` - Count, append, set actions
- `howto/completion.rs` - Shell completion generation
- `howto/deprecation.rs` - Marking deprecated commands

---

## 📚 Reference Examples (Information-Oriented)

**For:** Developers looking for exhaustive API demonstrations

```bash
# Full attribute macro API
cargo run --example ref_attribute_macro -- services status

# Framework patterns
cargo run --example ref_framework

# Nested commands
cargo run --example ref_nested -- level1 level2 action
```

**Examples:**
- `reference/attribute_macro.rs` - Complete #[noun]/#[verb] usage
- `reference/framework.rs` - Full framework integration
- `reference/nested.rs` - Multi-level command nesting
- `reference/collector.rs` - Command collection patterns
- `reference/format.rs` - Output format handling
- `reference/context.rs` - AppContext and state

---

## 🚀 Advanced Examples (Expert-Level)

**For:** Production systems, agents, and advanced use cases

```bash
# Machine-grade introspection
cargo run --example adv_autonomic -- --capabilities

# Async operations
cargo run --example adv_async

# Advanced I/O
cargo run --example adv_io
```

**Examples:**
- `advanced/autonomic.rs` - Agent2028 capabilities, introspection
- `advanced/async.rs` - Async command handlers
- `advanced/io_basic.rs` - File I/O with clio
- `advanced/io_advanced.rs` - Advanced I/O patterns
- `advanced/swarm_*.rs` - Multi-agent patterns
- `advanced/thesis_*.rs` - Research/academic examples

---

## 🧪 Playground (Experimental)

**For:** RDF/MCP experimentation and research

```bash
# Interactive RDF playground
cargo run --example rdf_interactive_playground

# SPARQL queries
cargo run --example rdf_oxigraph_sparql

# MCP integration
cargo run --example rdf_mcp_core
```

**Examples:**
- `playground/rdf_interactive_playground.rs` - Interactive RDF queries
- `playground/rdf_oxigraph_sparql.rs` - SPARQL with Oxigraph
- `playground/rdf_mcp_core.rs` - MCP protocol integration
- `playground/rdf_mcp_lean.rs` - Minimal MCP example
- `playground/thesis_rdf_mcp_80_20.rs` - 80/20 principle example
- `playground/arxiv_paper_generator.rs` - Academic paper generation

---

## Running Examples

### Basic Run
```bash
cargo run --example <name> -- [ARGS]
```

### With Features
```bash
cargo run --example <name> --features experimental -- [ARGS]
```

### List All Examples
```bash
cargo run --example 2>&1 | grep -E "^\s+"
```

---

## Example Categories by Feature

### Core Features (v5.0+)
| Feature | Example | Description |
|---------|---------|-------------|
| Basic CLI | `tutorial/basic` | Minimal working example |
| Arguments | `tutorial/arguments` | Typed argument handling |
| Groups | `howto/arg_groups` | Mutually exclusive args |
| Validation | `howto/validation` | Custom validators |

### v5.2.0 Phase 2 Features
| Feature | Example | Description |
|---------|---------|-------------|
| `[group:]` | `howto/arg_groups` | Exclusive groups |
| `[requires:]` | `howto/arg_groups` | Argument dependencies |
| `[conflicts:]` | `howto/arg_groups` | Mutually exclusive |
| `[env:]` | `howto/env_vars` | Environment variables |
| `[default:]` | `howto/arg_groups` | Default values |
| `[value_hint:]` | `howto/completion` | Shell completion hints |
| `[hide]` | `howto/arg_groups` | Hidden arguments |
| `[help_heading:]` | `howto/arg_groups` | Organized help |
| `[global]` | `reference/nested` | Global arguments |
| `[exclusive]` | `howto/arg_groups` | Exclusive arguments |

### Advanced Features
| Feature | Example | Description |
|---------|---------|-------------|
| Autonomic | `advanced/autonomic` | Agent introspection |
| Async | `advanced/async` | Async handlers |
| MCP | `playground/rdf_mcp_*` | Model Context Protocol |
| SPARQL | `playground/rdf_oxigraph_*` | RDF queries |

---

## Migrating from Flat Structure

Old path → New path:
```
examples/basic.rs           → examples/tutorial/basic.rs
examples/arg_groups.rs      → examples/howto/arg_groups.rs
examples/attribute_macro.rs → examples/reference/attribute_macro.rs
examples/autonomic_example.rs → examples/advanced/autonomic.rs
examples/playground/*.rs    → examples/playground/*.rs (unchanged)
```

---

## See Also

- [docs/tutorial/](../docs/tutorial/) - Learning documentation
- [docs/howto/](../docs/howto/) - Problem-solving guides
- [docs/reference/](../docs/reference/) - API reference
- [docs/explanation/](../docs/explanation/) - Architecture docs
