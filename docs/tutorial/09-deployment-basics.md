# Tutorial 09: Deployment Basics - From Development to Production

**Learning Path:** Development CLI → Production-Ready Deployment
**Time:** 20 minutes
**Prerequisites:** [Tutorial 08: Error Handling](08-error-handling.md)

---

## What You'll Learn

How to deploy clap-noun-verb CLIs to production:
- Building release binaries
- Configuration management
- Logging and observability
- Packaging and distribution
- Cross-platform builds

---

## Building Release Binaries

### Development Build (Fast, Large, Debuggable)

```bash
cargo build
# Binary: target/debug/my-cli
# Size: ~15MB (includes debug symbols)
# Speed: Fast compilation, slow execution
```

### Release Build (Optimized, Small, Fast)

```bash
cargo build --release
# Binary: target/release/my-cli
# Size: ~2-3MB (optimized, stripped)
# Speed: Slow compilation, fast execution
```

### Size Optimization

Add to `Cargo.toml`:

```toml
[profile.release]
opt-level = 3           # Maximum optimization
lto = true              # Link-time optimization
codegen-units = 1       # Better optimization, slower build
strip = true            # Strip debug symbols
panic = "abort"         # Smaller binary (no panic unwinding)
```

**Result:**
- Before: ~15MB (debug)
- After: ~1.5MB (release, optimized)

---

## Configuration Management

### Environment Variables

```rust
use clap_noun_verb_macros::verb;
use serde::Serialize;

#[derive(Serialize)]
pub struct ConfigStatus {
    database_url: String,
    log_level: String,
    environment: String,
}

#[verb(help = "Show current configuration")]
pub fn show_config(
    #[arg(
        env = "DATABASE_URL",
        help = "Database connection string"
    )]
    database_url: String,

    #[arg(
        env = "LOG_LEVEL",
        default = "info",
        help = "Logging level"
    )]
    log_level: String,

    #[arg(
        env = "ENVIRONMENT",
        default = "production",
        help = "Deployment environment"
    )]
    environment: String,
) -> Result<ConfigStatus, Box<dyn std::error::Error>> {
    Ok(ConfigStatus {
        database_url,
        log_level,
        environment,
    })
}
```

**Usage:**
```bash
# From environment
export DATABASE_URL="postgres://localhost/mydb"
export LOG_LEVEL="debug"
myapp config show-config

# Override with arguments
myapp config show-config --database-url "postgres://prod/mydb"
```

---

### Config Files (12-Factor App Pattern)

```rust
use serde::Deserialize;
use config::{Config, File, Environment};

#[derive(Deserialize, Debug)]
pub struct AppConfig {
    pub database_url: String,
    pub port: u16,
    pub log_level: String,
    pub environment: String,
}

pub fn load_config() -> Result<AppConfig, config::ConfigError> {
    Config::builder()
        // Start with defaults
        .set_default("port", 8080)?
        .set_default("log_level", "info")?
        .set_default("environment", "production")?
        // Load from config file
        .add_source(File::with_name("config/default").required(false))
        .add_source(File::with_name(&format!("config/{}", env!("ENVIRONMENT"))).required(false))
        // Override with environment variables
        .add_source(Environment::with_prefix("APP"))
        .build()?
        .try_deserialize()
}
```

**Config files:**
```
config/
├── default.toml         # Default settings
├── development.toml     # Dev overrides
├── staging.toml         # Staging overrides
└── production.toml      # Production settings
```

**config/production.toml:**
```toml
database_url = "postgres://prod-db:5432/mydb"
port = 3000
log_level = "warn"
environment = "production"
```

**Dependencies:**
```toml
[dependencies]
config = "0.14"
```

---

## Logging and Observability

### Structured Logging with tracing

```rust
use tracing::{info, warn, error, instrument};
use tracing_subscriber;

// Initialize logging on startup
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup tracing subscriber
    tracing_subscriber::fmt()
        .with_env_filter("my_cli=info")
        .json() // JSON output for log aggregators
        .init();

    clap_noun_verb::run()
}

// Add instrumentation to functions
#[instrument]
pub fn process_order(order_id: &str) -> Result<Order, OrderError> {
    info!("Processing order: {}", order_id);

    let order = load_order(order_id)?;

    if order.amount > 10000 {
        warn!("Large order detected: {} (amount: {})", order_id, order.amount);
    }

    process(&order)?;

    info!("Order processed successfully: {}", order_id);
    Ok(order)
}
```

**Log output (JSON):**
```json
{
  "timestamp": "2025-12-03T18:00:00Z",
  "level": "INFO",
  "target": "my_cli",
  "message": "Processing order: order-123",
  "order_id": "order-123"
}
```

**Dependencies:**
```toml
[dependencies]
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["json"] }
```

---

### OpenTelemetry Integration

```rust
use opentelemetry::{global, sdk::trace as sdktrace};
use opentelemetry_otlp::WithExportConfig;
use tracing_opentelemetry;
use tracing_subscriber::layer::SubscriberExt;

fn init_telemetry() -> Result<(), Box<dyn std::error::Error>> {
    // Setup OpenTelemetry OTLP exporter
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint("http://localhost:4317")
        )
        .install_batch(opentelemetry::runtime::Tokio)?;

    // Setup tracing subscriber with OTEL layer
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
    let subscriber = tracing_subscriber::Registry::default()
        .with(telemetry)
        .with(tracing_subscriber::fmt::layer());

    tracing::subscriber::set_global_default(subscriber)?;

    Ok(())
}
```

**Dependencies:**
```toml
[dependencies]
opentelemetry = "0.21"
opentelemetry-otlp = "0.14"
tracing-opentelemetry = "0.22"
```

---

## Docker Packaging

### Multi-Stage Dockerfile

```dockerfile
# Build stage
FROM rust:1.75-slim as builder

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source
COPY src ./src

# Build release binary
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /app/target/release/my-cli /usr/local/bin/my-cli

# Set entrypoint
ENTRYPOINT ["my-cli"]
CMD ["--help"]
```

**Build and run:**
```bash
# Build image
docker build -t my-cli:latest .

# Run container
docker run --rm my-cli:latest services status
```

---

## Cross-Platform Builds

### Linux, macOS, Windows

```bash
# Install cross-compilation tools
cargo install cross

# Build for Linux (x86_64)
cross build --release --target x86_64-unknown-linux-gnu

# Build for macOS (Apple Silicon)
cross build --release --target aarch64-apple-darwin

# Build for Windows
cross build --release --target x86_64-pc-windows-gnu
```

**Binaries:**
```
target/
├── x86_64-unknown-linux-gnu/release/my-cli
├── aarch64-apple-darwin/release/my-cli
└── x86_64-pc-windows-gnu/release/my-cli.exe
```

---

## Exercise: Production Deployment Checklist

**Goal:** Prepare CLI for production deployment

**Arrange:** Add production optimizations

```toml
# Cargo.toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true
panic = "abort"

[dependencies]
clap-noun-verb = "5.2"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["json"] }
config = "0.14"
```

**Act:** Implement production features

```rust
// src/main.rs
use tracing_subscriber;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize structured logging
    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("LOG_LEVEL")
                .unwrap_or_else(|_| "info".to_string())
        )
        .json()
        .init();

    // Load configuration
    let config = load_config()?;

    // Run CLI
    clap_noun_verb::run()
}

fn load_config() -> Result<AppConfig, config::ConfigError> {
    config::Config::builder()
        .add_source(config::Environment::with_prefix("APP"))
        .build()?
        .try_deserialize()
}
```

**Assert:** Verify production build

```bash
# Build optimized binary
cargo build --release

# Check binary size
ls -lh target/release/my-cli
# Expected: ~1.5-2MB

# Test with production config
export APP_DATABASE_URL="postgres://prod/mydb"
export LOG_LEVEL="warn"
./target/release/my-cli services status
```

---

## Deployment Strategies

### 1. Binary Distribution

```bash
# Upload to GitHub Releases
gh release create v1.0.0 target/release/my-cli

# Users download and install
curl -L https://github.com/user/my-cli/releases/download/v1.0.0/my-cli -o /usr/local/bin/my-cli
chmod +x /usr/local/bin/my-cli
```

---

### 2. Container Registry

```bash
# Build and tag
docker build -t ghcr.io/user/my-cli:v1.0.0 .

# Push to registry
docker push ghcr.io/user/my-cli:v1.0.0

# Users pull and run
docker pull ghcr.io/user/my-cli:v1.0.0
docker run ghcr.io/user/my-cli:v1.0.0 services status
```

---

### 3. Package Managers

**Homebrew (macOS):**
```ruby
# Formula/my-cli.rb
class MyCli < Formula
  desc "My awesome CLI tool"
  homepage "https://github.com/user/my-cli"
  url "https://github.com/user/my-cli/archive/v1.0.0.tar.gz"
  sha256 "..."

  depends_on "rust" => :build

  def install
    system "cargo", "build", "--release"
    bin.install "target/release/my-cli"
  end

  test do
    system "#{bin}/my-cli", "--version"
  end
end
```

**Install:**
```bash
brew install my-cli
```

---

## Health Checks

Add health check endpoint:

```rust
#[verb(
    help = "Health check for container orchestration",
    effects = ["reads_state"]
)]
pub fn health() -> Result<HealthStatus, Box<dyn std::error::Error>> {
    // Check critical dependencies
    let db_healthy = check_database_connection()?;
    let cache_healthy = check_cache_connection()?;

    Ok(HealthStatus {
        status: if db_healthy && cache_healthy { "healthy" } else { "unhealthy" },
        checks: vec![
            Check { name: "database", healthy: db_healthy },
            Check { name: "cache", healthy: cache_healthy },
        ],
    })
}
```

**Kubernetes readiness probe:**
```yaml
readinessProbe:
  exec:
    command:
    - /usr/local/bin/my-cli
    - services
    - health
  initialDelaySeconds: 5
  periodSeconds: 10
```

---

## Key Takeaways

✅ **Release builds** - Optimized binaries for production
✅ **Configuration** - Environment variables and config files
✅ **Structured logging** - JSON logs for aggregation
✅ **Docker packaging** - Multi-stage builds for efficiency
✅ **Cross-platform** - Support Linux, macOS, Windows

---

## Next Steps

- **[Tutorial 10: Next Steps](10-next-steps.md)** - Advanced topics and resources
- **[How-To: Production Deployment](../howto/production/deployment.md)** - Comprehensive deployment guide
- **[How-To: Monitoring](../howto/production/monitoring.md)** - Production observability

**Estimated time to next tutorial:** 10 minutes

---

*Part of the [clap-noun-verb Tutorial Series](README.md) - Learning-oriented documentation*
