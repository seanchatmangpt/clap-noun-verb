# ggen v2.0 Refactoring Script

This script uses [claude-flow](https://github.com/ruvnet/claude-flow) to automate the refactoring of ggen from v1.x to v2.0 using clap-noun-verb v3.0.0.

## Prerequisites

### 1. Install Node.js and npm
- Node.js 18+ required
- npm 9+ required

### 2. Install claude-flow
```bash
npm install -g claude-flow@alpha
```

Verify installation:
```bash
claude-flow --version
```

### 3. Install Claude Code CLI
```bash
npm install -g @anthropic-ai/claude-code
```

Activate Claude Code:
```bash
claude --dangerously-skip-permissions
```

## Usage

### Basic Usage

```bash
./refactor_ggen_v2.sh [ggen-project-path]
```

**Example:**
```bash
# Use default path (~/ggen)
./refactor_ggen_v2.sh

# Or specify custom path
./refactor_ggen_v2.sh /path/to/ggen
```

### What the Script Does

1. **Validates Prerequisites**
   - Checks ggen project directory exists
   - Verifies claude-flow is installed
   - Validates Node.js and npm versions

2. **Initializes claude-flow**
   - Creates `.claude-flow/` directory if needed
   - Sets up configuration files

3. **Creates Refactoring Context**
   - Generates comprehensive refactoring plan
   - Creates phase-by-phase task breakdown
   - Sets up swarm instructions

4. **Starts Refactoring Process**
   - Launches claude-flow swarm
   - Provides all necessary context files
   - Monitors progress

### Context Files Created

The script creates several context files in `.claude-flow/`:

- **refactor-v2-context.md**: Overall refactoring strategy and constraints
- **refactor-v2-tasks.md**: Phase-by-phase task breakdown
- **swarm-instructions.txt**: Direct instructions for claude-flow

### Manual Monitoring

After the script starts the swarm, you can monitor progress:

```bash
# Check status
claude-flow status

# View logs
claude-flow logs

# View specific task
claude-flow task <task-id>
```

## Refactoring Phases

The script follows a 5-phase approach:

### Phase 1: Dependencies & Foundation
- Update Cargo.toml files
- Add clap-noun-verb v3.0.0 dependencies
- Create new directory structure (`commands/`, `domain/`)

### Phase 2: Proof of Concept
- Migrate one command (`utils/doctor`) fully
- Test auto-discovery works
- Verify business logic separation

### Phase 3: Core Migration
- Migrate all command groups systematically
- Apply async/sync wrapper pattern
- Update command names (`market` → `marketplace`, etc.)

### Phase 4: Core Engine Updates
- Remove frontmatter RDF/vars parsing
- Add RDF loading via CLI parameter
- Add frozen section support
- Add filesystem routing

### Phase 5: Cleanup
- Delete old command structure
- Remove v1.x patterns
- Update tests and documentation

## Critical Constraints

The script enforces these constraints (documented in context files):

### 1. Async/Sync Compatibility ⚠️ CRITICAL
- clap-noun-verb v3.0.0 uses sync-only functions
- ggen has 94 async functions
- Solution: Sync CLI wrappers that spawn async runtimes

### 2. Error Handling
- NEVER use `unwrap()` or `expect()` in production code
- Always use `Result<T>` types
- Proper error propagation with `?` operator

### 3. Business Logic Separation
- CLI layer: `cli/src/commands/` (sync wrappers)
- Business logic: `cli/src/domain/` (async functions)

### 4. Command Renames (v2.0 Breaking Changes)
- `market` → `marketplace` (all 14 commands)
- `doctor` → `utils doctor`
- `help-me` → `utils help-me`
- `ggen gen` → `ggen template generate`
- `--vars` flags → `--rdf` flag

## Verification Steps

After refactoring completes, verify:

```bash
# 1. Code compiles
cd ~/ggen
cargo check

# 2. Tests pass
cargo test

# 3. No linting errors
cargo clippy

# 4. Check for unwrap/expect
grep -r "\.unwrap()\|\.expect(" cli/src/commands/

# 5. Test commands
cargo run -- utils doctor
cargo run -- marketplace search "rust"
cargo run -- template generate --template test.tmpl --rdf test.ttl
```

## Troubleshooting

### claude-flow not found
```bash
npm install -g claude-flow@alpha
```

### Claude Code not activated
```bash
claude --dangerously-skip-permissions
```

### Project directory not found
```bash
# Provide absolute path
./refactor_ggen_v2.sh /absolute/path/to/ggen
```

### Refactoring stalls
```bash
# Check status
claude-flow status

# View logs
claude-flow logs

# Restart if needed
claude-flow swarm --restart
```

## Documentation

For detailed information, see:
- [GGEN_V2_REFACTORING_PLAN.md](docs/book/GGEN_V2_REFACTORING_PLAN.md) - File-by-file refactoring plan
- [ASYNC_SYNC_COMPATIBILITY.md](docs/book/ASYNC_SYNC_COMPATIBILITY.md) - Async/sync compatibility guide
- [GGEN_V2_PLAN.md](docs/book/GGEN_V2_PLAN.md) - Overall v2.0 architecture plan

## References

- [claude-flow GitHub](https://github.com/ruvnet/claude-flow) - claude-flow documentation
- [clap-noun-verb v3.0.0](https://crates.io/crates/clap-noun-verb) - Framework documentation

---

**Status**: Script ready for use  
**Last Updated**: Pre-refactor readiness check

