# Reference: Configuration Options

Complete configuration reference for clap-noun-verb environments.

## Cargo.toml Features

### RDF Composition Features

```toml
[dependencies]
clap-noun-verb = { version = "5.3.4", features = [
    "rdf-composition",    # Core RDF/Turtle support
    "sparql-executor",    # SPARQL 1.1 querying
    "code-generator",     # Rust code generation
    "mcp-integration",    # MCP tool support
    "async",              # Async/await runtime
    "crypto",             # Cryptographic features
] }
```

### Benchmark Features

```toml
[dev-dependencies]
criterion = "0.5"          # Benchmarking
proptest = "1.0"           # Property testing
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.0", features = ["full"] }
```

## Environment Variables

### Logging Configuration

```bash
# Set log level
RUST_LOG=debug             # Debug level logging
RUST_LOG=trace             # Verbose trace logging
RUST_LOG=info              # Info and above
RUST_LOG=warn              # Warnings and errors

# Module-specific logging
RUST_LOG=clap_noun_verb=debug,hyper=info
```

### Performance Tuning

```bash
# Enable SPARQL query caching
CLAP_NOUN_VERB_CACHE=1

# Cache TTL in seconds
CLAP_NOUN_VERB_CACHE_TTL=3600

# Maximum cache entries
CLAP_NOUN_VERB_CACHE_SIZE=1000
```

### Development

```bash
# Enable verbose error messages
RUST_BACKTRACE=1           # Short backtrace
RUST_BACKTRACE=full        # Full backtrace

# Profile information
RUST_PROFILE=debug         # Keep debug symbols
```

## Runtime Configuration File

Create `config.toml`:

```toml
[cli]
name = "my-cli"
version = "1.0.0"
description = "My awesome CLI"

[ontology]
path = "./ontology/main.ttl"
validation = true           # Validate on load
cache = true               # Cache parsed ontology

[sparql]
caching = true
cache_ttl_seconds = 3600
max_cache_entries = 1000
query_timeout_ms = 5000

[code_generation]
output_directory = "./src/generated"
enable_docs = true
format_generated_code = true

[server]
listen = "0.0.0.0:8080"
workers = 4
request_timeout_seconds = 30

[logging]
level = "info"
format = "json"
output = "stderr"

[performance]
max_concurrent_commands = 10
command_timeout_seconds = 30
# These affect Criterion benchmarks
# small_sample_size = 10
# large_sample_size = 100
```

## Feature Flags

### Compile-Time Options

```rust
// In your code
#[cfg(feature = "rdf-composition")]
pub fn use_rdf_features() { }

#[cfg(feature = "mcp-integration")]
pub fn use_mcp_tools() { }
```

### Build with Specific Features

```bash
# Build with all features
cargo build --all-features

# Build with specific feature set
cargo build --features rdf-composition,async

# Build without optional features
cargo build --no-default-features
```

## Performance Configuration

### Optimization Profile

```toml
[profile.release]
opt-level = 3              # Full optimization
lto = true                 # Link-time optimization
codegen-units = 1          # Single codegen unit for better optimization
strip = true               # Strip debug symbols

[profile.release-with-debug]
inherits = "release"
debug = true               # Keep debug symbols
strip = false              # Don't strip
```

### Benchmark Configuration

```toml
[profile.bench]
opt-level = 3
lto = true
codegen-units = 1
```

### Test Configuration

```toml
[profile.test]
opt-level = 2              # Faster tests
debug = true               # Debug symbols for debugging
```

## SLO Configuration

```toml
[slos]
# Turtle parsing - must complete in 50ms for 100 triples
turtle_parse_ms = 50

# Validation - must complete in 10ms
validation_ms = 10

# SPARQL queries - must complete in 10ms
sparql_simple_ms = 10
sparql_join_ms = 20

# Code generation - must complete in 500ms for 100 verbs
code_generation_ms = 500

# Memory usage - must not exceed 20MB
memory_mb = 20
```

## Deployment Configuration

### Production

```toml
[deployment.production]
log_level = "info"
cache_enabled = true
cache_ttl_seconds = 3600
max_concurrent = 50
timeout_seconds = 30
enable_metrics = true
enable_profiling = false
```

### Staging

```toml
[deployment.staging]
log_level = "debug"
cache_enabled = true
cache_ttl_seconds = 1800
max_concurrent = 20
timeout_seconds = 60
enable_metrics = true
enable_profiling = true
```

### Development

```toml
[deployment.development]
log_level = "trace"
cache_enabled = false
cache_ttl_seconds = 60
max_concurrent = 5
timeout_seconds = 120
enable_metrics = false
enable_profiling = true
```

## MCP Tool Configuration

```toml
[mcp]
enabled = true
port = 9090
max_request_size_bytes = 1000000
request_timeout_seconds = 30

[mcp.tools]
GenerateCliFromTurtle = true
QueryCapabilities = true
ExportToTurtle = false              # Not yet implemented

[mcp.auth]
enabled = false
token_header = "Authorization"
```

## Docker Configuration

```dockerfile
FROM rust:latest as builder
WORKDIR /app
COPY . .
RUN cargo build --release --features rdf-composition,async

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libssl3
COPY --from=builder /app/target/release/my_cli /usr/local/bin/
ENV RUST_LOG=info
ENV CLAP_NOUN_VERB_CACHE=1
ENTRYPOINT ["my_cli"]
```

### Environment Variables in Docker

```bash
# .env file
RUST_LOG=info
CLAP_NOUN_VERB_CACHE=1
CLAP_NOUN_VERB_CACHE_TTL=3600
```

## Systemd Service

```ini
[Unit]
Description=My CLI Service
After=network.target

[Service]
Type=simple
User=myapp
WorkingDirectory=/opt/myapp
ExecStart=/opt/myapp/my_cli serve --config /opt/myapp/config.toml

# Environment
Environment="RUST_LOG=info"
Environment="CLAP_NOUN_VERB_CACHE=1"

# Resource limits
MemoryLimit=256M
CPUQuota=50%

# Restart policy
Restart=on-failure
RestartSec=5s
StartLimitInterval=60s
StartLimitBurst=3

[Install]
WantedBy=multi-user.target
```

## Kubernetes Configuration

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: my-cli-config
data:
  config.toml: |
    [cli]
    name = "my-cli"
    version = "1.0.0"

    [sparql]
    caching = true
    cache_ttl_seconds = 3600

    [logging]
    level = "info"
    format = "json"
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: my-cli
spec:
  replicas: 3
  template:
    spec:
      containers:
      - name: my-cli
        image: my-cli:latest
        env:
        - name: RUST_LOG
          value: "info"
        - name: CLAP_NOUN_VERB_CACHE
          value: "1"
        resources:
          requests:
            memory: "128Mi"
            cpu: "250m"
          limits:
            memory: "256Mi"
            cpu: "500m"
        volumeMounts:
        - name: config
          mountPath: /etc/my-cli
      volumes:
      - name: config
        configMap:
          name: my-cli-config
```

## Configuration Loading Priority

Configurations are loaded in this order (later overrides earlier):

1. **Default values** - Hardcoded defaults
2. **config.toml** - File in working directory
3. **Environment variables** - Prefixed with `CLAP_NOUN_VERB_`
4. **Command-line arguments** - Highest priority

Example:
```bash
# Uses config.toml but overrides cache TTL via env var
CLAP_NOUN_VERB_CACHE_TTL=600 cargo run --config custom-config.toml
```

## Configuration Validation

All configurations are validated on load:

```rust
pub fn load_config(path: &str) -> Result<Config, ConfigError> {
    let config = toml::from_str(&std::fs::read_to_string(path)?)?;
    config.validate()?;  // Validate ranges, types, etc.
    Ok(config)
}
```

---

**Related**:
- [How-to: Deploy Production CLIs](../tutorials/tutorial-5-deployment.md)
- [Reference: Performance SLOs](performance-slos.md)
- [Explanation: Agent Architecture Patterns](../explanation/agent-architecture.md)
