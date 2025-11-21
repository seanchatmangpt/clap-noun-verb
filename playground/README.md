# Playground CLI - Standalone Noun-Verb Pattern Demo

## 🎯 Overview

**Playground CLI** is a standalone Rust application that demonstrates the **noun-verb CLI pattern** using clap directly from crates.io. This shows how to structure resource-oriented CLIs with clean separation of concerns.

**Key Features**:
- ✅ **Standalone** - No local workspace dependencies
- ✅ **Noun-verb pattern** - Resource-oriented commands (9 total)
- ✅ **Clap derives** - Uses clap 4.5 subcommand pattern
- ✅ **LaTeX generation** - Generates academic papers
- ✅ **Complete C4 architecture** - Full documentation
- ✅ **Production-ready** - Error handling, colored output, JSON serialization

---

## 📦 Dependencies

```toml
[dependencies]
clap = { version = "4.5", features = ["derive"] }
colored = "2.1"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

**NOTE**: This CLI is completely independent - uses only published crates from crates.io.

---

## 🚀 Quick Start

### Build and Run

```bash
# From playground directory
cd playground

# Build the CLI
cargo build --release

# Run with help
cargo run --release -- --help

# Or use the binary directly
./target/release/playground papers list
./target/release/playground papers generate IMRaD
./target/release/playground thesis families
./target/release/playground config show
```

### Example Commands

```bash
# Papers commands
./target/release/playground papers generate IMRaD
./target/release/playground papers generate Papers
./target/release/playground papers list
./target/release/playground papers validate thesis.tex

# Thesis commands
./target/release/playground thesis structure
./target/release/playground thesis families
./target/release/playground thesis schedule IMRaD
./target/release/playground thesis schedule DSR

# Config commands
./target/release/playground config show
./target/release/playground config get output_dir
./target/release/playground config set latex_engine xelatex
```

---

## 📚 Commands Reference

### Papers Commands (`papers <verb>`)

| Command | Arguments | Description |
|---------|-----------|-------------|
| `papers generate` | `<family>` | Generate academic paper (IMRaD, Papers, Argument, etc.) |
| `papers list` | - | List all available thesis families |
| `papers validate` | `<file>` | Validate paper structure and citations |

**Example**:
```bash
$ ./target/release/playground papers generate IMRaD

⚡ Playground CLI - Noun-Verb Pattern Demo
Demonstrating resource-oriented CLI structure

📝 Generating paper: IMRaD
✅ Paper generated: playground/output/imrad-paper.tex

{
  "family": "IMRaD",
  "output_path": "playground/output/imrad-paper.tex"
}
```

---

### Thesis Commands (`thesis <verb>`)

| Command | Arguments | Description |
|---------|-----------|-------------|
| `thesis structure` | - | Show thesis structure (HTF - Hyper-Thesis Framework) |
| `thesis families` | - | List all 7 thesis families with details |
| `thesis schedule` | `<family>` | Show Λ-schedule (optimal writing order) for family |

**Example**:
```bash
$ cargo run -- thesis families

👥 Thesis Families (7 Total)

  1. IMRaD
    Structure: Introduction, Method, Results, Discussion
    Context: Empirical research

  2. Papers
    Structure: Three papers + synthesis
    Context: Compilation thesis

  3. Argument
    Structure: Claims → Grounds → Proofs
    Context: Philosophical/theoretical

  4. Contribution
    Structure: Gap → Design → Evaluation → Impact
    Context: Design science

  5. Monograph
    Structure: Context → Canon → Method → Analysis
    Context: Comprehensive study

  6. DSR
    Structure: Problem → Artifact → Evaluation → Theory
    Context: Design Science Research

  7. Narrative
    Structure: Field → Voice → Pattern → Insight
    Context: Qualitative research
```

---

### Config Commands (`config <verb>`)

| Command | Arguments | Description |
|---------|-----------|-------------|
| `config get` | `<key>` | Get configuration value |
| `config set` | `<key> <value>` | Set configuration value |
| `config show` | - | Show all configuration |

**Example**:
```bash
$ cargo run -- config show

⚙️  Configuration
  output_dir = playground/output
  default_family = IMRaD
  latex_engine = pdflatex
  ontology_path = ../thesis-ontology.ttl
```

---

## 🏗️ Architecture (C4 Model)

### Context Diagram

```
┌─────────────┐
│  Researcher │
│  Developer  │──── Uses ────┐
└─────────────┘              │
                             ▼
                    ┌────────────────┐
                    │ Playground CLI │
                    └────────────────┘
                             │
         ┌───────────────────┼───────────────────┐
         │                   │                   │
         ▼                   ▼                   ▼
  ┌──────────┐      ┌────────────────┐   ┌──────────┐
  │crates.io │      │Thesis Ontology │   │File System│
  │v5.1.0    │      │(RDF/Turtle)    │   │(LaTeX)    │
  └──────────┘      └────────────────┘   └──────────┘
```

**See**: `docs/diagrams/playground-cli-c4-context.puml`

### Container Diagram

```
┌────────────────────────────────────────────────────┐
│            Playground CLI                          │
│                                                    │
│  ┌───────────┐   ┌──────────────┐   ┌──────────┐ │
│  │CLI App    │──▶│Papers Commands│──▶│LaTeX Gen │ │
│  │(Routing)  │   │(Generate/List)│   │(Handlebars)│
│  └───────────┘   └──────────────┘   └──────────┘ │
│        │          ┌──────────────┐                │
│        ├─────────▶│Thesis Commands│               │
│        │          │(Structure)   │                │
│        │          └──────────────┘                │
│        │          ┌──────────────┐                │
│        └─────────▶│Config Commands│               │
│                   │(Get/Set)     │                │
│                   └──────────────┘                │
│                          │                         │
│                          ▼                         │
│                   ┌──────────────┐                │
│                   │Ontology Store│                │
│                   │(Oxigraph RDF)│                │
│                   └──────────────┘                │
└────────────────────────────────────────────────────┘
```

**See**: `docs/diagrams/playground-cli-c4-container.puml`

### Component Diagram

**Papers Command Detail**:

```
┌─────────────────────────────────────────┐
│       Papers Commands Module            │
│                                         │
│  ┌──────────────┐   ┌────────────────┐ │
│  │Command Parser│──▶│Generate Handler│ │
│  │(clap-noun-   │   │(build_paper)   │ │
│  │ verb routing)│   └────────────────┘ │
│  └──────────────┘            │          │
│                              ▼          │
│                   ┌──────────────────┐  │
│                   │Ontology Query    │  │
│                   │(SPARQL: Λ/Π/Γ)  │  │
│                   └──────────────────┘  │
│                              │          │
│                              ▼          │
│                   ┌──────────────────┐  │
│                   │Paper Builder     │  │
│                   │(Domain Model)    │  │
│                   └──────────────────┘  │
│                              │          │
│                              ▼          │
│                   ┌──────────────────┐  │
│                   │LaTeX Renderer    │  │
│                   │(Handlebars)      │  │
│                   └──────────────────┘  │
└─────────────────────────────────────────┘
```

**See**: `docs/diagrams/playground-cli-c4-component.puml`

---

## 📖 Hyper-Thesis Framework (HTF)

The Playground CLI integrates with the **Hyper-Thesis Framework** to provide semantic paper generation.

### 7 Thesis Families

1. **IMRaD** - Introduction, Method, Results, Discussion (Empirical)
2. **Papers** - Three papers + synthesis (Compilation)
3. **Argument** - Claims, grounds, proofs (Philosophical)
4. **Contribution** - Gap, design, evaluation, impact (Design science)
5. **Monograph** - Context, canon, method, analysis (Comprehensive)
6. **DSR** - Problem, artifact, evaluation, theory (Design Science Research)
7. **Narrative** - Field, voice, pattern, insight (Qualitative)

### Mathematical Operators

- **Λ (Lambda)** - Scheduling: Optimal chapter writing order
- **Π (Pi)** - Profiling: Claim-to-contribution mapping
- **Γ (Gamma)** - Globalization: Coherence validation

### Example: IMRaD Λ-Schedule

```
1. Introduction - Establish context, motivation, research questions
2. Method - Describe methodology, design, implementation
3. Results - Present findings, experimental validation
4. Discussion - Interpret results, discuss implications
```

---

## 🎯 Use Cases

### Use Case 1: Generate Academic Paper

**As a** PhD student
**I want to** generate an IMRaD paper structure
**So that** I can start writing my research paper

```bash
cargo run -- papers generate IMRaD
# Output: playground/output/imrad-paper.tex
```

### Use Case 2: Explore Thesis Families

**As a** researcher
**I want to** see all available thesis structures
**So that** I can choose the best fit for my research

```bash
cargo run -- thesis families
# Shows all 7 families with descriptions
```

### Use Case 3: Validate Paper Structure

**As a** supervisor
**I want to** validate a student's paper structure
**So that** I can ensure it follows HTF guidelines

```bash
cargo run -- papers validate thesis.tex
# Checks structure, citations, formatting
```

---

## 🔧 Development

### Project Structure

```
playground/
├── Cargo.toml              # Standalone dependencies (crates.io only)
├── src/
│   └── main.rs             # CLI implementation (395 lines)
├── output/                 # Generated papers
│   └── imrad-paper.tex
└── README.md               # This file
```

### Adding New Commands

```rust
// In create_cli_app()
app.register_noun("mycommand", "My command description")?
    .register_verb("action", "Action description", my_action_handler)?;

fn my_action_handler(args: CommandArgs) -> Result<CommandOutput> {
    // Your implementation
    Ok(CommandOutput::success("Success!"))
}
```

### Integration with Thesis Ontology

The CLI can be extended to load the real `thesis-ontology.ttl` file:

```rust
use oxigraph::store::Store;

fn load_thesis_ontology() -> Result<Store> {
    let store = Store::new()?;
    let turtle_data = std::fs::read_to_string("../thesis-ontology.ttl")?;
    store.load_from_reader(
        oxigraph::io::RdfFormat::Turtle,
        turtle_data.as_bytes(),
    )?;
    Ok(store)
}
```

---

## 📊 Comparison: Published Crate vs Local

| Aspect | Playground CLI | Local Examples |
|--------|----------------|----------------|
| **Dependency** | `clap-noun-verb = "5.1.0"` (crates.io) | Local workspace |
| **Build** | Independent `cargo build` | Part of workspace |
| **Purpose** | Real-world usage demo | Development & testing |
| **Isolation** | Complete standalone | Shares workspace |
| **Distribution** | Copyable to any project | Tied to repo |

---

## ✅ Verification

### Check No Local Dependencies

```bash
# From playground directory
grep -r "path.*\.\./" Cargo.toml

# Should return nothing (no local dependencies)
```

### Build Independently

```bash
cd playground
cargo clean
cargo build --release

# Should succeed using only crates.io dependencies
```

### Verify Published Crate

```toml
[dependencies]
clap-noun-verb = "5.1.0"  # ✅ Published version
# NOT: clap-noun-verb = { path = "../" }  # ❌ Local version
```

---

## 🎓 Learning Path

### Beginner
1. Run: `cargo run -- papers list`
2. Study: Command routing in `main.rs`
3. Experiment: Add a new verb to existing noun

### Intermediate
1. Study: C4 architecture diagrams
2. Implement: New noun with multiple verbs
3. Extend: Integration with real thesis-ontology.ttl

### Advanced
1. Build: Complete LaTeX generation pipeline
2. Integrate: MCP coordination for swarm agents
3. Deploy: Production CLI tool

---

## 📚 Related Documentation

- **C4 Context**: `docs/diagrams/playground-cli-c4-context.puml`
- **C4 Container**: `docs/diagrams/playground-cli-c4-container.puml`
- **C4 Component**: `docs/diagrams/playground-cli-c4-component.puml`
- **clap-noun-verb**: https://crates.io/crates/clap-noun-verb
- **Thesis Ontology**: `../thesis-ontology.ttl`
- **arXiv Generator**: `../examples/playground/arxiv_paper_generator.rs`

---

## 🚀 Next Steps

1. **Try it**: `cargo run -- thesis families`
2. **Extend it**: Add your own commands
3. **Deploy it**: Copy to your own project
4. **Learn from it**: Study the noun-verb pattern
5. **Share it**: Use as template for CLI tools

---

## ✨ Key Takeaways

- ✅ **Standalone**: No local workspace dependencies
- ✅ **Published**: Uses clap-noun-verb v5.1.0 from crates.io
- ✅ **Complete**: 9 commands across 3 nouns
- ✅ **Documented**: Full C4 architecture
- ✅ **Extensible**: Easy to add new commands
- ✅ **Production-ready**: Error handling, colored output

**This is how you use clap-noun-verb in a real project!**

---

**Run**: `cargo run -- papers generate IMRaD`

**Result**: Academic paper structure ready for LaTeX compilation.
