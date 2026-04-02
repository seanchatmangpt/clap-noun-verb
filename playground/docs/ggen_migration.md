# ggen Migration Guide: v5 to v26.4.2

## Overview

**ggen v26.4.2** represents a major architectural evolution from v5, introducing a **7-domain capability model** that provides fine-grained control over code generation, template management, and build orchestration.

### Key Changes

| Aspect | v5 | v26.4.2 |
|--------|-----|---------|
| **Architecture** | Monolithic generator | 7-domain capability model |
| **Capabilities** | Fixed feature set | Modular, enable/disable domains |
| **Packs** | Built-in templates | External, community-contributed |
| **Sync** | N/A | Sacred command for state reconciliation |
| **Receipts** | N/A | Cryptographic verification of operations |
| **Projections** | Fixed output formats | Multi-format (Rust, Go, TypeScript, etc.) |
| **Surfaces** | CLI only | CLI, MCP, WASM, HTTP APIs |

---

## The 7-Domain Capability Model

ggen v26.4.2 organizes functionality into **7 domains**:

### 1. **Generator** (`gen`)
Core code generation engine.
- **Status**: Always enabled
- **Features**: Template expansion, code synthesis, AST transformation
- **Packs**: `rust-core`, `go-core`, `typescript-core`

### 2. **Template** (`template`)
Template management and rendering.
- **Status**: Enable via `ggen capability enable --surface template`
- **Features**: Hot-reload, validation, inheritance
- **Packs**: `tera-templates`, `handlebar-templates`, `jinja-templates`

### 3. **Builder** (`builder`)
Build orchestration and compilation.
- **Status**: Enable via `ggen capability enable --surface builder`
- **Features**: Incremental builds, caching, dependency resolution
- **Packs**: `cargo-builder`, `make-builder`, `webpack-builder`

### 4. **MCP** (Model Context Protocol) (`mcp`)
AI agent integration and coordination.
- **Status**: Enable via `ggen capability enable --surface mcp`
- **Features**: Agent spawning, message passing, resource management
- **Packs**: `mcp-server`, `mcp-client`, `mcp-tools`

### 5. **Validator** (`validator`)
Code quality and validation.
- **Status**: Enable via `ggen capability enable --surface validator`
- **Features**: Linting, formatting, type checking
- **Packs**: `clippy-validator`, `eslint-validator`, `golangci-lint`

### 6. **Pack** (`pack`)
Package management and distribution.
- **Status**: Always enabled
- **Features**: Add, remove, update packs
- **Packs**: Community-contributed packages

### 7. **Receipt** (`receipt`)
Cryptographic verification and audit trails.
- **Status**: Always enabled
- **Features**: Operation signing, verification, history
- **Packs**: `crypto-receipt`, `audit-log`

---

## Command Mapping Table

### v5 → v26.4.2 Command Translation

| v5 Command | v26.4.2 Equivalent | Notes |
|------------|-------------------|-------|
| `ggen generate` | `ggen gen run` | Now part of Generator domain |
| `ggen template add` | `ggen pack add <template>` | Use pack system for templates |
| `ggen build` | `ggen builder run` | Now part of Builder domain |
| `ggen validate` | `ggen validator run` | Now part of Validator domain |
| `ggen agent spawn` | `ggen mcp spawn` | Now part of MCP domain |
| N/A | `ggen capability enable` | NEW: Enable capabilities |
| N/A | `ggen pack add` | NEW: Add packs |
| N/A | `ggen sync` | NEW: Sacred sync command |
| N/A | `ggen receipt verify` | NEW: Verify operations |

---

## Feature Flags

### Capability Flags

Enable/disable entire capability domains:

```bash
# Enable MCP capability with Rust projection
ggen capability enable --surface mcp --projection rust

# Enable Builder capability
ggen capability enable --surface builder

# Disable a capability
ggen capability disable --surface validator
```

### Projection Flags

Specify output format for generated code:

```bash
# Generate Rust code
ggen gen run --projection rust

# Generate Go code
ggen gen run --projection go

# Generate TypeScript code
ggen gen run --projection typescript
```

### Pack Flags

Add community-contributed packs:

```bash
# Add a pack
ggen pack add mcp-server

# Remove a pack
ggen pack remove mcp-server

# Update a pack
ggen pack update mcp-server

# List installed packs
ggen pack list
```

---

## Migration Timeline

### Phase 1: Assessment (Week 1)
- [ ] Audit current ggen v5 usage
- [ ] Identify capabilities in use
- [ ] Map v5 commands to v26.4.2 equivalents
- [ ] Test v26.4.2 in development environment

### Phase 2: Capability Migration (Week 2)
- [ ] Enable required capabilities
- [ ] Add required packs
- [ ] Update CI/CD pipelines
- [ ] Migrate custom templates to pack system

### Phase 3: Sync Integration (Week 3)
- [ ] Replace build scripts with `ggen sync`
- [ ] Configure receipt verification
- [ ] Update deployment workflows
- [ ] Train team on new workflow

### Phase 4: Validation (Week 4)
- [ ] Run full test suite
- [ ] Performance benchmarking
- [ ] Security audit
- [ ] Production rollout

---

## Migration Steps

### Step 1: Install ggen v26.4.2

```bash
# Via cargo
cargo install ggen --version 26.4.2

# Via binary release
curl -sSL https://github.com/ruvnet/ggen/releases/download/v26.4.2/ggen-x86_64-apple-darwin.tar.gz | tar xz
sudo mv ggen /usr/local/bin/
```

### Step 2: Enable Required Capabilities

```bash
# Enable MCP capability (most common)
ggen capability enable --surface mcp --projection rust

# Enable Builder capability
ggen capability enable --surface builder

# Enable Validator capability
ggen capability enable --surface validator
```

### Step 3: Add Required Packs

```bash
# Add MCP server pack
ggen pack add mcp-server

# Add cargo builder pack
ggen pack add cargo-builder

# Add clippy validator pack
ggen pack add clippy-validator
```

### Step 4: Run Initial Sync

```bash
# This is the "sacred" command - it reconciles all state
ggen sync
```

### Step 5: Verify Receipts

```bash
# Verify the last operation
ggen receipt verify --file receipts/<latest-id>.json
```

### Step 6: Update Build Scripts

Replace v5 commands:

```bash
# OLD (v5)
ggen generate --template my-template
ggen build
ggen validate

# NEW (v26.4.2)
ggen gen run --template my-template
ggen builder run
ggen validator run
```

---

## Rollback Instructions

If you need to rollback to v5:

### Step 1: Uninstall v26.4.2

```bash
# If installed via cargo
cargo uninstall ggen

# If installed via binary
sudo rm /usr/local/bin/ggen
```

### Step 2: Reinstall v5

```bash
cargo install ggen --version 5.3.3
```

### Step 3: Restore v5 Configuration

```bash
# Restore from backup
cp ~/.ggen.v5.backup/config.toml ~/.ggen/config.toml

# Or manually reconfigure
ggen config set template_dir ~/.ggen/templates
ggen config set output_dir ./output
```

### Step 4: Update CI/CD

Revert CI/CD pipelines to use v5 commands.

---

## Common Issues and Solutions

### Issue 1: Capability Not Found

**Error**: `Error: Capability 'mcp' not found`

**Solution**:
```bash
# Enable the capability first
ggen capability enable --surface mcp --projection rust
```

### Issue 2: Pack Not Found

**Error**: `Error: Pack 'mcp-server' not found`

**Solution**:
```bash
# Update pack index
ggen pack update-index

# Then add the pack
ggen pack add mcp-server
```

### Issue 3: Sync Fails

**Error**: `Error: Sync failed - receipt verification failed`

**Solution**:
```bash
# Verify the receipt
ggen receipt verify --file receipts/<id>.json

# Force sync (use with caution)
ggen sync --force
```

### Issue 4: Projection Not Supported

**Error**: `Error: Projection 'python' not supported for capability 'mcp'`

**Solution**:
```bash
# Check supported projections
ggen capability list --surface mcp

# Use a supported projection
ggen gen run --projection rust
```

---

## Best Practices

### 1. Enable Only Required Capabilities

```bash
# Good: Enable only what you need
ggen capability enable --surface mcp --projection rust

# Bad: Enable everything
ggen capability enable --surface all
```

### 2. Use Sync as Single Source of Truth

```bash
# Good: Always run sync after changes
ggen pack add mcp-server
ggen sync

# Bad: Skip sync
ggen pack add mcp-server
# ... forget to sync
```

### 3. Verify Receipts After Critical Operations

```bash
# Good: Verify after important changes
ggen sync
ggen receipt verify --file receipts/<latest-id>.json

# Bad: Never verify
ggen sync
# ... assume it worked
```

### 4. Use Projection Flags for Multi-Language Projects

```bash
# Good: Explicit projection
ggen gen run --projection rust
ggen gen run --projection typescript

# Bad: Rely on defaults
ggen gen run
```

---

## Further Reading

- **ggen v26.4.2 Documentation**: https://docs.ggen.dev
- **7-Domain Model**: https://docs.ggen.dev/architecture/7-domains
- **Pack Development**: https://docs.ggen.dev/packs/development
- **MCP Integration**: https://docs.ggen.dev/capabilities/mcp

---

## Summary

**ggen v26.4.2** is a **major architectural upgrade** from v5, introducing:

- ✅ **7-domain capability model** for fine-grained control
- ✅ **Pack system** for community contributions
- ✅ **Sync command** for state reconciliation
- ✅ **Receipt verification** for cryptographic proofs
- ✅ **Multi-projection support** for multiple languages

**Migration is straightforward** when following the phased approach outlined in this guide.

**Questions?** Open an issue at https://github.com/ruvnet/ggen/issues
