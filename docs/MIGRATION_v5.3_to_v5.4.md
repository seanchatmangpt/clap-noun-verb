# Migration Guide: v5.3.x → v5.4.0

**TL;DR**: No breaking changes. Update your `Cargo.toml` and optionally enable new frontier features.

---

## Quick Start

### 1. Update Cargo.toml

```toml
[dependencies]
clap-noun-verb = "5.4"  # or "5.4.0" for exact version
```

### 2. No Code Changes Required

All existing code using `#[noun]` and `#[verb]` macros works without modification.

### 3. (Optional) Enable Frontier Features

```toml
[dependencies]
clap-noun-verb = { version = "5.4", features = [
    "frontier-semantic",   # RDF + Federated Network
    "frontier-quality",    # Testing + Specs
]}
```

---

## What's New

### 🆕 Root-Level Verbs

Define commands without noun prefixes:

```rust
#[verb]
fn deploy(args: VerbArgs) -> Result<()> {
    // No noun prefix needed
    // `deploy --help` instead of `app deploy --help`
}
```

### 🆕 10 Frontier Packages

Advanced capabilities available via feature flags:

**Meta-framework**: Type-erased agent systems
```toml
features = ["meta-framework"]
```

**RDF Composition**: Semantic ontologies with SPARQL
```toml
features = ["rdf-composition"]
```

**And 8 more...** See `docs/ADVANCED_FEATURES_GUIDE.md`

### 🆕 ggen Integration

Generate CLIs from Turtle specifications:

```rust
// See examples/generated-from-turtle/ for examples
```

---

## Detailed Migration

### Scenario 1: Basic CLI User

**Before (v5.3.x)**:
```rust
use clap_noun_verb::{noun, verb, VerbArgs};

#[noun]
pub struct App;

#[verb]
fn process(args: VerbArgs) -> Result<()> {
    Ok(())
}
```

**After (v5.4.0)**:
```rust
use clap_noun_verb::{noun, verb, VerbArgs};

#[noun]
pub struct App;

#[verb]
fn process(args: VerbArgs) -> Result<()> {
    // Works exactly the same
    Ok(())
}
```

**Nothing to change!** Your code is forward compatible.

### Scenario 2: Upgrading to Root-Level Verbs

**New in v5.4.0**:
```rust
#[verb]
fn migrate(args: VerbArgs) -> Result<()> {
    // Root verb without noun prefix
    // Usage: `app migrate --version`
    println!("Starting migration...");
    Ok(())
}

#[noun]
struct MyApp;

#[verb]
fn backup(args: VerbArgs) -> Result<()> {
    // Noun-verb combo still works
    // Usage: `app backup create`
    println!("Backing up...");
    Ok(())
}
```

### Scenario 3: Using Frontier Features

**Minimal Dependency Footprint (Default)**:
```toml
clap-noun-verb = "5.4"  # ~10 dependencies
```

**Add Semantic Capabilities**:
```toml
clap-noun-verb = { version = "5.4", features = ["frontier-semantic"] }
```

**Add Testing Capabilities**:
```toml
clap-noun-verb = { version = "5.4", features = ["frontier-quality"] }
```

**Add Everything**:
```toml
clap-noun-verb = { version = "5.4", features = ["frontier-all"] }
```

---

## Feature Activation Guide

### frontier-all (100 dependencies)
```toml
features = ["frontier-all"]
```
Includes all 10 frontier packages. Use for maximum capabilities.

### frontier-semantic (30 dependencies)
```toml
features = ["frontier-semantic"]
```
- Meta-framework (type erasure)
- RDF Composition (semantic ontologies)
- Federated Network (distributed agents)

**Use case**: Building semantic CLIs with ontology support

### frontier-intelligence (40 dependencies)
```toml
features = ["frontier-intelligence"]
```
- Discovery Engine (dynamic capabilities)
- Learning Trajectories (ReasoningBank learning)
- Economic Simulation (agent economies)

**Use case**: Agent-based systems with learning

### frontier-quality (35 dependencies)
```toml
features = ["frontier-quality"]
```
- Executable Specs (BDD testing)
- Reflexive Testing (property-based testing)

**Use case**: Specification-driven development

---

## Breaking Changes

**None in v5.4.0**

The following remain unchanged:
- ✅ `#[noun]` macro signature and behavior
- ✅ `#[verb]` macro signature and behavior
- ✅ `VerbArgs` API and methods
- ✅ Argument extraction functions
- ✅ Default feature set (~10 dependencies)
- ✅ MSRV (Rust 1.74)
- ✅ Error handling patterns

---

## Removed Features

**None**. All v5.3.x features remain available in v5.4.0.

---

## Dependency Changes

### New Dependencies (optional, feature-gated)

Adding frontier features introduces new optional dependencies:

```
frontier-semantic:
  + oxrdf, oxigraph, json-ld, sophia_api
  + libp2p, quinn, rustls

frontier-intelligence:
  + ndarray, smartcore, linfa
  + priority-queue, ordered-float, bevy_ecs

frontier-quality:
  + cucumber, gherkin, libtest-mimic
  + quickcheck, arbitrary

meta-framework:
  + erased-serde, typetag, inventory, paste
```

**None of these are included by default.** Opt-in via feature flags.

---

## Performance Impact

### Default Build (5.4.0 vs 5.3.x)

- **Compilation time**: No change (~13s incremental)
- **Binary size**: No change (~2.5MB release)
- **Runtime**: No change (zero-cost abstractions)

### With frontier-all

- **Compilation time**: +30-40s (first build only)
- **Binary size**: +4-5MB (full features)
- **Runtime**: No change (features are compile-time)

---

## Documentation Updates

### New Documentation
- `docs/ADVANCED_FEATURES_GUIDE.md` - Frontier packages overview
- `examples/generated-from-turtle/` - ggen integration examples
- `docs/CAPABILITY_DISCOVERY_ENGINE_ARCHITECTURE.md` - Agent discovery

### Updated Documentation
- `CHANGELOG.md` - Full v5.4.0 release notes
- `README.md` - Feature overview updated
- `docs.rs` - API documentation with new examples

---

## Common Upgrade Paths

### Path A: Minimal (No New Features)
```toml
# v5.3.x
clap-noun-verb = "5.3"

# v5.4.0 - just update version
clap-noun-verb = "5.4"
```

### Path B: Add Semantic Capabilities
```toml
# v5.3.x with autonomic
clap-noun-verb = { version = "5.3", features = ["autonomic"] }

# v5.4.0 with semantic capabilities
clap-noun-verb = { version = "5.4", features = ["frontier-semantic"] }
```

### Path C: Full Agent Stack
```toml
# v5.3.x
clap-noun-verb = "5.3"

# v5.4.0 with all frontier packages
clap-noun-verb = { version = "5.4", features = ["frontier-all"] }
```

---

## Testing After Upgrade

### 1. Verify Compilation
```bash
cargo check
cargo build
```

### 2. Run Existing Tests
```bash
cargo test
```

### 3. Test with New Features (if enabled)
```bash
cargo build --all-features
cargo test --all-features
```

### 4. Check Examples
```bash
cargo build --examples
./target/debug/examples/tutorial_basic
```

---

## Troubleshooting

### Issue: `error: failed to resolve: use of undeclared crate`

**Cause**: Using frontier feature without enabling it
**Solution**: Add feature to `Cargo.toml`:
```toml
clap-noun-verb = { version = "5.4", features = ["frontier-semantic"] }
```

### Issue: Compilation takes too long

**Cause**: Building with `frontier-all` enables 100+ dependencies
**Solution**: Use specific feature bundle:
```toml
# Instead of frontier-all
clap-noun-verb = { version = "5.4", features = ["frontier-quality"] }
```

### Issue: Old dependency version conflicts

**Cause**: v5.4.0 uses newer versions of dependencies
**Solution**: Update other crates or use cargo's dependency resolution:
```bash
cargo update
cargo tree  # Verify no conflicts
```

---

## Reporting Issues

If you encounter issues upgrading to v5.4.0:

1. Check this migration guide
2. Review `CHANGELOG.md` for v5.4.0 changes
3. Check examples in `examples/` and `examples/tutorial/`
4. Open GitHub issue with:
   - Rust version (`rustc --version`)
   - Full error message
   - Minimal reproduction case
   - Your `Cargo.toml`

---

## Questions & Support

- **GitHub Discussions**: Ask questions and discuss API design
- **Documentation**: Full API docs at https://docs.rs/clap-noun-verb/5.4.0/
- **Examples**: 30+ working examples in `examples/`
- **Issues**: Report bugs with detailed information

---

**Migration Complete!** Welcome to v5.4.0 🎉
