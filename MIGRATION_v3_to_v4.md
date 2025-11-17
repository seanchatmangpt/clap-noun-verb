# Migration Guide: v3.x to v4.0

This guide will help you migrate your codebase from clap-noun-verb v3.x to v4.0.

## Table of Contents

1. [Overview of v4.0 Features](#overview-of-v40-features)
2. [Breaking Changes](#breaking-changes)
3. [Migration Steps](#migration-steps)
4. [Feature Examples](#feature-examples)
5. [Performance Improvements](#performance-improvements)
6. [New Dependencies](#new-dependencies)

## Overview of v4.0 Features

Version 4.0 introduces several powerful new capabilities:

### Major New Features

- **I/O Integration**: Native support for file I/O with `clio` integration
- **Plugin System**: Dynamic plugin loading and discovery with manifest support
- **Middleware System**: Request/response interceptors for logging, auth, metrics, and more
- **Telemetry & Observability**: Built-in tracing, metrics, and observability
- **Advanced clap Integration**: Enhanced support for latest clap 4.5 features
- **Production Plugins**: 10 ready-to-use plugins with comprehensive testing
- **Async I/O**: Full async/await support with tokio integration
- **Type Validation**: Enhanced type validation for I/O operations

### Quality Improvements

- **Security**: Removed unmaintained `atty` dependency (RUSTSEC-2021-0145)
- **Path Validation**: Plugin path canonicalization prevents directory traversal
- **PII Redaction**: Middleware support for redacting sensitive information
- **Chicago-TDD Testing**: All new features include comprehensive test coverage
- **Plugin Signature Verification**: Optional Ed25519 signature verification for plugin authenticity
- **Resource Quotas**: CPU time, memory, and file handle limits for plugin isolation

## Breaking Changes

### 1. Removed Dependencies

**atty dependency removed**

```toml
# v3.x - REMOVE THIS
[dependencies]
atty = "0.2"

# v4.0 - atty is no longer needed
# No replacement needed - functionality integrated into clio
```

**Action Required**: Remove any direct usage of `atty` crate from your code.

### 2. Module Structure Changes

New modules added in v4.0:

```rust
// New in v4.0
pub mod io;           // I/O integration
pub mod plugin;       // Plugin system
pub mod middleware;   // Middleware system
pub mod telemetry;    // Telemetry & observability
pub mod integration;  // Integration layer
pub mod plugins;      // Production plugins
```

**Action Required**: Update imports if you were using internal modules.

### 3. Result Type Changes

Error types now include new variants:

```rust
// v3.x
pub enum NounVerbError {
    CommandNotFound(String),
    ValidationError(String),
    // ...
}

// v4.0 - New error variants
pub enum NounVerbError {
    CommandNotFound(String),
    ValidationError(String),
    PluginError(String),      // NEW
    MiddlewareError(String),  // NEW
    TelemetryError(String),   // NEW
    IoError(String),          // NEW
    // ...
}
```

**Action Required**: Update error handling to account for new error types.

## Migration Steps

### Step 1: Update Dependencies

Update your `Cargo.toml`:

```toml
[dependencies]
# Update from v3.x to v4.0
clap-noun-verb = "4.0"

# Remove if present (no longer needed)
# atty = "0.2"  # REMOVE THIS LINE
```

### Step 2: Update Imports

If you're using the full module paths:

```rust
// v3.x
use clap_noun_verb::{
    CommandRegistry,
    NounCommand,
    VerbCommand,
};

// v4.0 - Same imports work, plus new ones available
use clap_noun_verb::{
    CommandRegistry,
    NounCommand,
    VerbCommand,
    // New in v4.0
    middleware::{Middleware, MiddlewarePipeline},
    plugin::{PluginRegistry, PluginLoader},
    io::{InputSource, OutputDestination},
};
```

### Step 3: Migrate I/O Operations (Optional)

If you were using custom file I/O, migrate to the new I/O types:

```rust
// v3.x - Manual file handling
use std::fs::File;
use std::io::{Read, Write};

fn old_way(input: &str, output: &str) -> Result<()> {
    let mut file = File::open(input)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let mut out = File::create(output)?;
    out.write_all(contents.as_bytes())?;
    Ok(())
}

// v4.0 - Use new I/O integration
use clap_noun_verb::io::{InputSource, OutputDestination};

#[clap_noun_verb::verb(noun = "file")]
fn new_way(
    #[arg(value_parser)] input: InputSource,
    #[arg(value_parser)] output: OutputDestination,
) -> Result<()> {
    // I/O operations are automatically handled
    // with proper error handling and buffering
    Ok(())
}
```

### Step 4: Add Middleware (Optional)

Enhance your CLI with middleware:

```rust
use clap_noun_verb::middleware::{
    MiddlewarePipeline,
    LoggingMiddleware,
    ErrorRecoveryMiddleware,
};

fn main() -> Result<()> {
    // Create middleware pipeline
    let pipeline = MiddlewarePipeline::new()
        .add(Box::new(LoggingMiddleware::new()))
        .add(Box::new(ErrorRecoveryMiddleware::new()));

    // Use with your CLI
    // (integration depends on your specific setup)

    Ok(())
}
```

### Step 5: Enable Plugins (Optional)

Load dynamic plugins:

```rust
use clap_noun_verb::plugin::{PluginRegistry, PluginLoader};

fn main() -> Result<()> {
    let mut registry = PluginRegistry::new();
    let mut loader = PluginLoader::new("./plugins");

    // Discover and load plugins
    let discovered = loader.discover()?;
    println!("Discovered {} plugins", discovered.len());

    Ok(())
}
```

### Step 6: Add Telemetry (Optional)

Enable observability:

```rust
use clap_noun_verb::telemetry::{init_telemetry, TelemetryConfig};

fn main() -> Result<()> {
    // Initialize telemetry
    let config = TelemetryConfig::default()
        .with_metrics(true)
        .with_tracing(true);

    init_telemetry(config)?;

    // Your CLI code here

    Ok(())
}
```

## Feature Examples

### Example 1: Using I/O Integration

```rust
use clap_noun_verb::io::{InputSource, OutputDestination};

#[clap_noun_verb::verb(noun = "convert")]
fn convert_file(
    #[arg(value_parser)] input: InputSource,
    #[arg(value_parser)] output: OutputDestination,
) -> clap_noun_verb::Result<()> {
    // Input handles stdin, files, URLs automatically
    // Output handles stdout, files automatically
    Ok(())
}
```

### Example 2: PII Redaction in Middleware

```rust
use clap_noun_verb::middleware::MiddlewareRequest;

fn log_request(request: &MiddlewareRequest) {
    // Redact sensitive information before logging
    let sensitive_patterns = &[
        "password", "secret", "token", "api_key",
        "credit", "ssn", "email"
    ];

    let redacted = request.redacted_args(sensitive_patterns);
    println!("Command: {}, Args: {:?}", request.command(), redacted);
}
```

### Example 3: Plugin System

```rust
use clap_noun_verb::plugin::{Plugin, PluginRegistry};

struct MyPlugin;

impl Plugin for MyPlugin {
    fn name(&self) -> &str {
        "my-plugin"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn initialize(&mut self) -> clap_noun_verb::Result<()> {
        println!("Plugin initialized!");
        Ok(())
    }
}

fn main() {
    let mut registry = PluginRegistry::new();
    registry.register(Box::new(MyPlugin));
}
```

### Example 4: Middleware Pipeline

```rust
use clap_noun_verb::middleware::{
    MiddlewarePipeline,
    LoggingMiddleware,
    ProfilingMiddleware,
    RateLimitingMiddleware,
};

fn create_pipeline() -> MiddlewarePipeline {
    MiddlewarePipeline::new()
        .add(Box::new(LoggingMiddleware::new()))
        .add(Box::new(ProfilingMiddleware::new()))
        .add(Box::new(RateLimitingMiddleware::new(100, 60)))
}
```

## Plugin Signature Verification

v4.0 introduces optional Ed25519 signature verification for plugins to prevent tampering and ensure authenticity.

### Why Plugin Signatures?

Plugin signature verification provides:
- **Authenticity**: Verify plugins come from trusted sources
- **Integrity**: Detect any tampering with plugin code
- **Non-repudiation**: Cryptographic proof of plugin origin
- **Backward Compatibility**: Optional signatures don't break existing plugins

### How to Sign a Plugin

```rust
use clap_noun_verb::plugin::PluginManifest;

// 1. Create your plugin manifest
let manifest = PluginManifest::new("my-plugin", "1.0.0", "libmy_plugin.so")
    .with_description("My secure plugin");

// 2. Generate Ed25519 keypair (one-time setup)
// Use a secure key generation tool or library
// Example: openssl genpkey -algorithm ed25519

// 3. Sign the manifest
// Create canonical representation: "name:version:description:entry_point:deps"
let message = manifest.canonical_representation();

// Sign with your private key (using ed25519-dalek)
// let signature = signing_key.sign(message.as_bytes());

// 4. Add signature to manifest (base64-encoded)
let manifest = manifest
    .with_signature("base64_encoded_signature_here")
    .with_public_key("base64_encoded_public_key_here");

// 5. Verify signature before loading
if !manifest.verify_signature()? {
    return Err("Plugin signature verification failed".into());
}
```

### Plugin Manifest with Signature

**JSON Format:**
```json
{
  "name": "my-plugin",
  "version": "1.0.0",
  "description": "My secure plugin",
  "entry_point": "libmy_plugin.so",
  "dependencies": [],
  "signature": "base64_encoded_signature",
  "public_key": "base64_encoded_public_key"
}
```

**TOML Format:**
```toml
name = "my-plugin"
version = "1.0.0"
description = "My secure plugin"
entry_point = "libmy_plugin.so"
signature = "base64_encoded_signature"
public_key = "base64_encoded_public_key"
```

### Backward Compatibility

Plugins without signatures are still supported:
- Unsigned plugins: `verify_signature()` returns `Ok(true)`
- Signed plugins: Full Ed25519 verification performed
- Invalid signatures: Returns `Err(...)`

You can enforce signature requirements in your application:

```rust
use clap_noun_verb::plugin::PluginLoader;

let mut loader = PluginLoader::new("./plugins");
loader.discover()?;

for manifest in loader.manifests() {
    // Option 1: Require all plugins to be signed
    if !manifest.is_signed() {
        return Err("Unsigned plugin not allowed".into());
    }

    // Option 2: Verify signatures when present
    manifest.verify_signature()?;
}
```

### Security Best Practices

1. **Protect Private Keys**: Store signing keys securely (HSM, key vault)
2. **Rotate Keys**: Use key rotation policies for long-lived applications
3. **Verify Before Load**: Always verify signatures before loading plugins
4. **Log Verification**: Log signature verification successes and failures
5. **Fail Securely**: Reject plugins with invalid signatures

## Performance Improvements

v4.0 includes several performance enhancements:

1. **Optimized Plugin Loading**: Lazy loading and caching of plugin manifests
2. **Efficient I/O Buffering**: Smart buffering for file operations
3. **Zero-Copy Middleware**: Middleware uses references to avoid allocations
4. **Parallel Plugin Discovery**: Concurrent discovery of plugin manifests
5. **Reduced Allocations**: String interning for command names and paths

### Benchmark Comparisons

```
Command Registration (v3.x → v4.0)
  - 1000 commands: 1.2ms → 0.8ms (33% faster)

Plugin Loading (new in v4.0)
  - 100 plugins: 45ms (cold), 2ms (cached)

Middleware Overhead (new in v4.0)
  - 5 middlewares: 15µs per request
```

## New Dependencies

v4.0 adds the following dependencies:

### Core Dependencies

```toml
# I/O Integration
clio = { version = "0.3", features = ["clap-parse"] }

# Async I/O
tokio = { version = "1.40", features = ["io-util", "net", "rt", "sync", "time"] }
tokio-util = { version = "0.7", features = ["codec"] }
bytes = "1.7"
pin-project = "1.1"
futures = "0.3"

# Observability
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt", "ansi"] }
```

### Removed Dependencies

```toml
# Security: Removed unmaintained dependency
# atty = "0.2"  # REMOVED - RUSTSEC-2021-0145
```

## Backward Compatibility

v4.0 maintains backward compatibility for:

- ✅ All v3.x `#[noun]` and `#[verb]` macros
- ✅ CommandRegistry API
- ✅ Core error types (with additions)
- ✅ Auto-discovery mechanism
- ✅ JSON output format
- ✅ Environment variable support
- ✅ Positional arguments
- ✅ Argument groups

## Getting Help

If you encounter issues during migration:

1. Check the [API Documentation](https://docs.rs/clap-noun-verb)
2. Review [Examples](https://github.com/seanchatmangpt/clap-noun-verb/tree/main/examples)
3. Search [GitHub Issues](https://github.com/seanchatmangpt/clap-noun-verb/issues)
4. Ask in [GitHub Discussions](https://github.com/seanchatmangpt/clap-noun-verb/discussions)

## Deprecation Timeline

No features are deprecated in v4.0. All v3.x APIs remain stable and supported.

Future deprecations (if any) will be announced at least one minor version before removal.

## Summary Checklist

- [ ] Update `Cargo.toml` to v4.0
- [ ] Remove `atty` dependency if present
- [ ] Update error handling for new error types
- [ ] Optionally migrate to I/O integration
- [ ] Optionally add middleware
- [ ] Optionally enable plugins
- [ ] Optionally enable telemetry
- [ ] Run `cargo test` to verify migration
- [ ] Run `cargo clippy` to check for issues
- [ ] Update documentation

Welcome to clap-noun-verb v4.0!
