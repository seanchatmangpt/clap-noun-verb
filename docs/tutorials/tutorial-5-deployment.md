# Tutorial 5: Deploy Production CLIs

**Duration**: 25-30 minutes
**Level**: Advanced
**Prerequisites**: Completed [Tutorial 4: Query Ontologies with SPARQL](tutorial-4-sparql.md)
**Goals**:
- Package generated CLIs as distributable binaries
- Configure agents for automated deployment
- Monitor CLI performance in production
- Update ontologies and redeploy seamlessly

## Step 1: Set Up Production Build

Create a build configuration in `Cargo.toml`:

```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true

[profile.release-with-debug]
inherits = "release"
debug = true
strip = false
```

Build for release:

```bash
cargo make release
```

## Step 2: Create Deployment Package Structure

```
deployment/
├── build.sh              # Build script
├── config.toml           # Deployment configuration
├── ontology/
│   └── services-cli.ttl  # Runtime ontology
├── handlers/
│   ├── services.rs       # Service handlers
│   └── config.rs         # Config handlers
└── systemd/
    └── mycli.service     # Systemd unit
```

### Build Script (`deployment/build.sh`)

```bash
#!/bin/bash
set -e

# Configuration
CLI_NAME="mycli"
VERSION=$(grep "^version" Cargo.toml | head -n1 | cut -d'"' -f2)
BUILD_DIR="target/release"
DIST_DIR="dist/$CLI_NAME-$VERSION"

echo "🔨 Building $CLI_NAME v$VERSION..."

# Clean and build
cargo make clean
cargo make release

# Create distribution
mkdir -p "$DIST_DIR"
cp "$BUILD_DIR/$CLI_NAME" "$DIST_DIR/"
cp deployment/config.toml "$DIST_DIR/"
cp -r ontology "$DIST_DIR/"
cp deployment/systemd/mycli.service "$DIST_DIR/"

# Create tarball
tar -czf "dist/$CLI_NAME-$VERSION.tar.gz" -C dist "$CLI_NAME-$VERSION"

echo "✅ Built $CLI_NAME-$VERSION.tar.gz"
ls -lh "dist/$CLI_NAME-$VERSION.tar.gz"
```

## Step 3: Runtime Configuration

Create `deployment/config.toml`:

```toml
[cli]
name = "mycli"
version = "1.0.0"
ontology = "./ontology/services-cli.ttl"

[server]
listen = "127.0.0.1:8080"
workers = 4

[logging]
level = "info"
format = "json"

[performance]
max_concurrent_commands = 10
timeout_seconds = 30
cache_sparql = true
```

## Step 4: Agent Deployment Configuration

Create an agent deployment manifest:

```rust
// deployment/agent_deployment.rs
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct CliDeploymentSpec {
    pub name: String,
    pub version: String,
    pub ontology_iri: String,
    pub handlers: Vec<HandlerSpec>,
    pub resources: ResourceSpec,
    pub health_check: HealthCheckSpec,
}

#[derive(Serialize, Deserialize)]
pub struct HandlerSpec {
    pub verb_name: String,
    pub noun_name: String,
    pub handler_fn: String,
    pub timeout_ms: u64,
}

#[derive(Serialize, Deserialize)]
pub struct ResourceSpec {
    pub cpu_limit: String,
    pub memory_limit: String,
    pub max_concurrent: usize,
}

#[derive(Serialize, Deserialize)]
pub struct HealthCheckSpec {
    pub endpoint: String,
    pub interval_seconds: u64,
    pub timeout_seconds: u64,
}

pub fn load_deployment_spec(path: &str) -> Result<CliDeploymentSpec, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(path)?;
    let spec: CliDeploymentSpec = toml::from_str(&content)?;
    Ok(spec)
}
```

## Step 5: Automated Deployment Workflow

Create an agent that handles deployment:

```rust
// deployment/deploy.rs
pub struct AgentDeployer {
    spec: CliDeploymentSpec,
}

impl AgentDeployer {
    pub fn new(spec_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let spec = load_deployment_spec(spec_path)?;
        Ok(AgentDeployer { spec })
    }

    pub async fn deploy(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("📦 Deploying {} v{}", self.spec.name, self.spec.version);

        // Step 1: Build
        self.build().await?;

        // Step 2: Test
        self.run_tests().await?;

        // Step 3: Package
        self.package().await?;

        // Step 4: Push to registry
        self.push_to_registry().await?;

        // Step 5: Update production
        self.update_production().await?;

        println!("✅ Deployment complete!");
        Ok(())
    }

    async fn build(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("  🔨 Building...");
        std::process::Command::new("cargo")
            .args(&["make", "release"])
            .status()?;
        Ok(())
    }

    async fn run_tests(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("  ✅ Testing...");
        std::process::Command::new("cargo")
            .args(&["make", "test"])
            .status()?;
        Ok(())
    }

    async fn package(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("  📦 Packaging...");
        std::process::Command::new("bash")
            .arg("deployment/build.sh")
            .status()?;
        Ok(())
    }

    async fn push_to_registry(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("  📤 Pushing to registry...");
        // Implementation depends on your registry (Artifactory, S3, etc.)
        Ok(())
    }

    async fn update_production(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("  🚀 Updating production...");
        // Implementation depends on your deployment platform
        Ok(())
    }
}
```

## Step 6: Health Checks and Monitoring

Create a health check endpoint:

```rust
pub async fn health_check() -> Result<HealthStatus, Box<dyn std::error::Error>> {
    Ok(HealthStatus {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: get_uptime()?,
        commands_processed: get_command_count()?,
        last_error: None,
    })
}

pub async fn metrics() -> Result<Metrics, Box<dyn std::error::Error>> {
    Ok(Metrics {
        total_commands: get_command_count()?,
        average_latency_ms: get_avg_latency()?,
        error_rate: get_error_rate()?,
        memory_usage_mb: get_memory_usage()?,
        uptime_seconds: get_uptime()?,
    })
}
```

## Step 7: Zero-Downtime Updates

Update ontologies without restarting:

```rust
pub struct HotReloadManager {
    current_ontology: Arc<RwLock<ParsedTurtle>>,
    reload_signal: Arc<Notify>,
}

impl HotReloadManager {
    pub async fn reload_ontology(&self, new_ontology_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Parse new ontology
        let content = std::fs::read_to_string(new_ontology_path)?;
        let parser = TurtleParser::new();
        let new_ontology = parser.parse(&content)?;
        new_ontology.validate_ontology()?;

        // Swap in new version
        {
            let mut current = self.current_ontology.write().await;
            *current = new_ontology;
        }

        // Notify readers
        self.reload_signal.notify_waiters();

        println!("✅ Ontology reloaded successfully");
        Ok(())
    }

    pub async fn get_ontology(&self) -> Arc<RwLock<ParsedTurtle>> {
        self.current_ontology.clone()
    }
}
```

## Step 8: Production Checklist

Before deploying to production:

**Code Quality**
- ✅ All tests pass: `cargo make test`
- ✅ No clippy warnings: `cargo make lint`
- ✅ No compiler warnings: `cargo make check`
- ✅ Performance SLOs met: `cargo make slo-check`

**Configuration**
- ✅ Ontology file verified and validated
- ✅ Configuration file reviewed
- ✅ Environment variables set correctly
- ✅ Logging configured appropriately

**Deployment**
- ✅ Binary built with release optimizations
- ✅ Health check endpoint working
- ✅ Metrics endpoint accessible
- ✅ Graceful shutdown configured

**Monitoring**
- ✅ Metrics collection enabled
- ✅ Error logging configured
- ✅ Alert thresholds set
- ✅ Health check URL known

## Step 9: Systemd Service Integration

Create `deployment/systemd/mycli.service`:

```ini
[Unit]
Description=MyApp CLI Service
After=network.target

[Service]
Type=simple
User=mycli
WorkingDirectory=/opt/mycli
ExecStart=/opt/mycli/mycli serve --config /opt/mycli/config.toml
Restart=on-failure
RestartSec=5s
StandardOutput=journal
StandardError=journal
Environment="RUST_LOG=info"

[Install]
WantedBy=multi-user.target
```

Install:
```bash
sudo cp deployment/systemd/mycli.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable mycli.service
sudo systemctl start mycli.service
```

## Step 10: Agent-Driven Deployment Pipeline

Agents can trigger deployments through MCP:

```rust
pub struct DeploymentMcpTool {
    deployer: AgentDeployer,
}

impl DeploymentMcpTool {
    pub async fn deploy_from_ontology(
        &self,
        ontology_path: &str,
        target_environment: &str,
    ) -> Result<DeploymentResult, Box<dyn std::error::Error>> {
        println!("🤖 Agent-initiated deployment to {}", target_environment);

        // 1. Load and validate ontology
        let content = std::fs::read_to_string(ontology_path)?;
        let parser = TurtleParser::new();
        let ontology = parser.parse(&content)?;
        ontology.validate_ontology()?;

        // 2. Generate CLI from ontology
        let generator = CliCodeGenerator::new()?;
        let generated = generator.generate_from_ontology(&ontology)?;

        // 3. Build and test
        self.deployer.build().await?;
        self.deployer.run_tests().await?;

        // 4. Deploy to target
        if target_environment == "production" {
            self.deployer.update_production().await?;
        }

        Ok(DeploymentResult {
            success: true,
            version: get_version()?,
            deployed_at: chrono::Utc::now(),
            commands_deployed: generated.verb_count(),
        })
    }
}
```

## Deployment Workflow Diagram

```
Ontology Update
      ↓
[Parse & Validate]
      ↓
[Generate Code]
      ↓
[Compile Release Binary]
      ↓
[Run Tests]
      ↓
[Build Package]
      ↓
[Push to Registry]
      ↓
[Hot-Reload Ontology]
      ↓
[Health Check]
      ↓
✅ Production Update Complete
```

## What You Learned

✅ Setting up production build configurations
✅ Creating deployment packages and automation
✅ Implementing health checks and monitoring
✅ Zero-downtime ontology updates
✅ Agent-driven deployment pipelines
✅ Systemd service integration
✅ Production readiness checklist

## Summary: Zero to Hero

You've now completed the full journey:

1. **Tutorial 1**: Set up your first agent with MCP
2. **Tutorial 2**: Created RDF Turtle ontologies
3. **Tutorial 3**: Generated Rust CLI code from ontologies
4. **Tutorial 4**: Queried ontologies with SPARQL
5. **Tutorial 5**: Deployed production CLIs with agent automation

**You can now:**
- Create semantic ontologies for any CLI system
- Generate production-grade Rust code from specifications
- Query ontologies to discover capabilities
- Deploy CLIs with automated agent workflows
- Update systems zero-downtime through ontology changes

## Next Steps

Explore advanced topics:
- [How-to: Integrate with Agents](../howto/agent-integration.md)
- [How-to: Optimize Performance](../howto/performance-optimization.md)
- [Explanation: Agent Architecture Patterns](../explanation/agent-architecture.md)

---

**Congratulations! You're now a clap-noun-verb expert with production deployment skills.** 🚀
