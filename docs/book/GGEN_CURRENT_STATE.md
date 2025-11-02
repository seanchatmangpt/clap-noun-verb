# ggen Current State Analysis

## Repository Structure

```
~/ggen/
├── cli/                    # CLI interface (Clap-based)
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs          # Main CLI entry point
│       └── cmds/           # Command modules
│           ├── mod.rs       # Commands enum
│           ├── ai/          # AI commands
│           ├── market/      # Marketplace commands
│           ├── project/     # Project commands
│           ├── template/    # Template commands
│           ├── hook/        # Hook commands
│           ├── doctor.rs    # Doctor command
│           └── help_progressive.rs  # Help-me command
├── ggen-core/              # Template engine, RDF/SPARQL processing
├── ggen-ai/                # AI providers
├── ggen-marketplace/       # Package management
└── src/
    └── main.rs             # Application entry point
```

## Current CLI Architecture

### Entry Point

**File**: `cli/src/lib.rs`

```rust
#[derive(Parser, Debug)]
#[command(name = "ggen", author, about = "Graph-aware code generator", version)]
pub struct Cli {
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<PathBuf>,
    
    #[clap(subcommand)]
    pub command: cmds::Commands,  // Enum-based commands
}

pub async fn cli_match() -> Result<()> {
    let cli = Cli::parse();
    // ... setup ...
    cli.command.run().await
}
```

### Command Enum Structure

**File**: `cli/src/cmds/mod.rs`

```rust
#[derive(Subcommand, Debug)]
pub enum Commands {
    #[command(name = "ai", about = "AI-powered template generation and analysis")]
    Ai(ai::AiArgs),
    
    #[command(name = "market", about = "Marketplace operations for gpacks")]
    Market(market::MarketCmd),
    
    #[command(name = "project", about = "Project scaffolding and generation")]
    Project(project::ProjectCmd),
    
    #[command(name = "template", about = "Template management")]
    Template(template::TemplateCmd),
    
    #[command(name = "doctor", about = "Check system prerequisites and environment health")]
    Doctor(doctor::DoctorArgs),
    
    #[command(name = "help-me", about = "Get personalized help based on your experience level")]
    HelpProgressive(help_progressive::HelpProgressiveArgs),
    
    #[command(name = "hook", about = "Knowledge hooks for autonomic graph regeneration")]
    Hook(hook::HookCmd),
    
    #[command(name = "graph", about = "RDF graph operations")]
    Graph(graph::GraphCmd),
    
    #[command(name = "audit", about = "Security and performance auditing")]
    Audit(audit::AuditCmd),
    
    #[command(name = "ci", about = "CI/CD operations and GitHub integration")]
    Ci(ci::CiCmd),
    
    #[command(name = "shell", about = "Shell integration and completion")]
    Shell(shell::ShellCmd),
    
    #[command(name = "lifecycle", about = "Universal lifecycle management")]
    Lifecycle(lifecycle::LifecycleArgs),
}

impl Commands {
    pub async fn run(&self) -> Result<()> {
        match self {
            Commands::Ai(args) => ai::run(args).await,
            Commands::Market(cmd) => cmd.run().await,
            Commands::Project(cmd) => cmd.run().await,
            Commands::Template(cmd) => cmd.run().await,
            Commands::Doctor(args) => doctor::run(args).await,
            Commands::HelpProgressive(args) => help_progressive::run(args).await,
            Commands::Hook(cmd) => cmd.run().await,
            // ... other commands
        }
    }
}
```

### Command Modules

#### AI Commands (`cli/src/cmds/ai/`)

**Files**:
- `mod.rs` - AI command structure
- `project.rs` - `ggen ai project` command
- `generate.rs` - `ggen ai generate` command
- `graph.rs` - `ggen ai graph` command
- `sparql.rs` - `ggen ai sparql` command

**Structure**:
```rust
// cli/src/cmds/ai/mod.rs
#[derive(Parser, Debug)]
pub struct AiArgs {
    #[clap(subcommand)]
    pub command: AiCommand,
}

#[derive(Subcommand, Debug)]
pub enum AiCommand {
    Project(project::ProjectArgs),
    Generate(generate::GenerateArgs),
    Graph(graph::GraphArgs),
    Sparql(sparql::SparqlArgs),
}

pub async fn run(args: &AiArgs) -> Result<()> {
    match &args.command {
        AiCommand::Project(args) => project::run(args).await,
        AiCommand::Generate(args) => generate::run(args).await,
        // ...
    }
}
```

#### Marketplace Commands (`cli/src/cmds/market/`)

**Files**:
- `mod.rs` - Marketplace command structure
- `search.rs` - `ggen market search` command
- `add.rs` - `ggen market add` command
- `list.rs` - `ggen market list` command
- `publish.rs` - `ggen market publish` command
- `remove.rs` - `ggen market remove` command
- `update.rs` - `ggen market update` command

**Current Structure**:
```rust
// cli/src/cmds/market/mod.rs
#[derive(Parser, Debug)]
pub struct MarketCmd {
    #[clap(subcommand)]
    pub command: MarketCommand,
}

#[derive(Subcommand, Debug)]
pub enum MarketCommand {
    Search(search::SearchArgs),
    Add(add::AddArgs),
    List(list::ListArgs),
    Publish(publish::PublishArgs),
    // ...
}
```

## Current Issues

### 1. **Verbose Enum-Based Structure**
- Large `Commands` enum with many variants
- Large match statement in `run()` method
- Manual command registration required

### 2. **Manual Command Registration**
- Each command must be added to enum
- Each command must be added to match statement
- Harder to extend with new commands

### 3. **Scattered Command Definitions**
- Commands defined in separate modules
- No centralized registration
- Harder to discover available commands

### 4. **Mixed CLI and Business Logic**
- CLI handlers contain business logic
- Can't test business logic independently
- Harder to reuse logic in other contexts

### 5. **No Automatic JSON Output**
- Output is plain text
- Harder to script/automate
- Not ideal for agent/MCP integration

### 6. **No Type Inference**
- Arguments must be explicitly defined
- More boilerplate code
- Harder to maintain

## Current Dependencies

**Cargo.toml**:
```toml
[dependencies]
clap = { version = "4.5.48", features = ["cargo", "derive"] }
clap_complete = "4.5.58"
genai = "0.4"
tera = "1.20"
oxigraph = "0.5"
tokio = { version = "1.47", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
```

## Command Count

**Total Commands**: **12 top-level commands** (more than originally estimated)

1. `ai` - AI-powered generation (4 subcommands)
2. `market` - Marketplace operations (12+ subcommands)
3. `project` - Project scaffolding (10+ subcommands)
4. `template` - Template management (7 subcommands)
5. `doctor` - Diagnostics
6. `help-me` - Progressive help
7. `hook` - Knowledge hooks (5 subcommands)
8. `graph` - RDF graph operations (7 subcommands)
9. `audit` - Security auditing (3 subcommands)
10. `ci` - CI/CD operations (4 subcommands)
11. `shell` - Shell integration (2 subcommands)
12. `lifecycle` - Lifecycle management

**Total Subcommands**: **50+ commands** (much more than the 13 originally estimated)

## Migration Complexity

### High Complexity Commands

1. **AI Commands** (`ai/`) - 4 subcommands
   - `project` - Complex project generation
   - `generate` - Template generation
   - `graph` - RDF graph generation
   - `sparql` - SPARQL query generation

2. **Marketplace Commands** (`market/`) - 12+ subcommands
   - `search`, `add`, `list`, `publish`, `remove`, `update`
   - `info`, `categories`, `recommend`, `sync`, `offline`
   - `cache`, `lockfile`, `registry`, `natural`, `unpublish`

3. **Project Commands** (`project/`) - 10+ subcommands
   - `new`, `gen`, `apply`, `validate`, `test`, `watch`
   - `plan`, `diff`, `freeze`, `inject`

4. **Template Commands** (`template/`) - 7 subcommands
   - `generate_tree`, `list`, `show`, `new`, `lint`
   - `regenerate`, `generate`

5. **Hook Commands** (`hook/`) - 5 subcommands
   - `create`, `list`, `remove`, `run`, `validate`

6. **Graph Commands** (`graph/`) - 7 subcommands
   - `load`, `query`, `validate`, `stats`, `export`
   - `snapshot`, `diff`

## Migration Strategy

### Phase 1: Foundation (Week 1-2)

**Priority**: Start with simpler commands first

1. **Utils Commands** (2 commands):
   - `doctor` - Simple diagnostics
   - `help-me` - Progressive help

2. **Simple Project Commands** (2 commands):
   - `project new` - Project creation
   - `hook create` - Hook creation

3. **Core Commands** (8 commands):
   - `ai project` - Most used
   - `ai generate` - Most used
   - `market search` - Most used
   - `market add` - Most used
   - `template generate` - Most used
   - `project gen` - Most used
   - `ai graph` - Medium complexity
   - `ai sparql` - Medium complexity

4. **Remaining Commands** (40+ commands):
   - Migrate incrementally
   - One command group at a time
   - Maintain backward compatibility

### Phase 2: Separation of Concerns (Week 2-3)

- Extract business logic from CLI handlers
- Organize by domain
- Create clear boundaries

### Phase 3: Module Boundaries (Week 3-4)

- Define trait interfaces
- Implement dependency inversion
- Enable testing with mocks

### Phase 4: Core Abstraction (Week 4)

- Extract common patterns
- Apply consistently
- Document usage patterns

## Next Steps

1. **Review Current State** ✅ - Completed
2. **Start Phase 1 Migration** - Migrate utils commands first
3. **Create Test Matrix** - Test all commands before/after
4. **Set Performance Baselines** - Benchmark current performance
5. **Create Feature Branch** - `refactor/80-20-cli-migration`

## Reference Implementation

See `examples/ggen-refactor-phase1.rs` in clap-noun-verb repository for:
- Complete Phase 1 reference implementation
- All 13 core commands demonstrated
- Separation of concerns pattern
- Auto-discovery with `#[verb]` attributes
- Type inference from function signatures
- JSON output with `Serialize`

---

**Last Updated**: Current state analysis completed. Ready to begin Phase 1 migration.

