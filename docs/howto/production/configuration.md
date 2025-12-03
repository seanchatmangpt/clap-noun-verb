# How-To: Production Configuration

**Problem:** You need to manage configuration across multiple environments (dev, staging, production) without hardcoding values.

**Solution:** Use a layered configuration approach with environment variables, config files, and runtime overrides following 12-Factor App principles.

---

## Prerequisites

- clap-noun-verb CLI application
- Multiple deployment environments
- Secret management system (Kubernetes Secrets, AWS Secrets Manager, Vault)

---

## Quick Start (5 Minutes)

```bash
# 1. Add config dependencies
cargo add config serde

# 2. Create configuration structure
# See Step 1 below

# 3. Create environment-specific config files
# config/development.toml
# config/production.toml

# 4. Use in CLI commands
# See Step 3 below
```

---

## Step 1: Configuration Structure

### Define Config Types

```rust
// src/config/mod.rs
use serde::{Deserialize, Serialize};
use config::{Config, File, Environment};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub logging: LoggingConfig,
    pub features: FeatureFlags,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub timeout_seconds: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub connection_timeout_seconds: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String, // "json" or "text"
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FeatureFlags {
    pub enable_telemetry: bool,
    pub enable_caching: bool,
    pub enable_rate_limiting: bool,
}

impl AppConfig {
    pub fn load() -> Result<Self, config::ConfigError> {
        let environment = std::env::var("ENVIRONMENT")
            .unwrap_or_else(|_| "development".to_string());

        Config::builder()
            // Start with defaults
            .set_default("server.host", "0.0.0.0")?
            .set_default("server.port", 8080)?
            .set_default("server.timeout_seconds", 30)?
            .set_default("database.max_connections", 10)?
            .set_default("database.connection_timeout_seconds", 5)?
            .set_default("logging.level", "info")?
            .set_default("logging.format", "json")?
            .set_default("features.enable_telemetry", true)?
            .set_default("features.enable_caching", true)?
            .set_default("features.enable_rate_limiting", false)?

            // Load default config file
            .add_source(
                File::with_name("config/default")
                    .required(false)
            )

            // Load environment-specific config
            .add_source(
                File::with_name(&format!("config/{}", environment))
                    .required(false)
            )

            // Override with environment variables (APP_ prefix)
            // Example: APP_SERVER__PORT=3000
            .add_source(
                Environment::with_prefix("APP")
                    .separator("__")
            )

            .build()?
            .try_deserialize()
    }
}
```

---

## Step 2: Environment-Specific Configuration

### Development Config

```toml
# config/development.toml
[server]
host = "localhost"
port = 8080
timeout_seconds = 60

[database]
url = "postgres://localhost:5432/myapp_dev"
max_connections = 5
connection_timeout_seconds = 10

[logging]
level = "debug"
format = "text"

[features]
enable_telemetry = false
enable_caching = false
enable_rate_limiting = false
```

### Staging Config

```toml
# config/staging.toml
[server]
host = "0.0.0.0"
port = 3000
timeout_seconds = 30

[database]
url = "postgres://staging-db.internal:5432/myapp_staging"
max_connections = 20
connection_timeout_seconds = 5

[logging]
level = "info"
format = "json"

[features]
enable_telemetry = true
enable_caching = true
enable_rate_limiting = true
```

### Production Config

```toml
# config/production.toml
[server]
host = "0.0.0.0"
port = 3000
timeout_seconds = 30

[database]
# Use environment variable for sensitive data
# url will be set via APP_DATABASE__URL env var
max_connections = 50
connection_timeout_seconds = 5

[logging]
level = "warn"
format = "json"

[features]
enable_telemetry = true
enable_caching = true
enable_rate_limiting = true
```

---

## Step 3: Use Configuration in Commands

### Load Config at Startup

```rust
// src/main.rs
use crate::config::AppConfig;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration
    let config = AppConfig::load()?;

    // Initialize logging with config
    init_logging(&config.logging)?;

    // Make config available globally (if needed)
    set_global_config(config.clone())?;

    clap_noun_verb::run()
}
```

### Access Config in Commands

```rust
use clap_noun_verb_macros::verb;
use serde::Serialize;

#[derive(Serialize)]
pub struct ConfigStatus {
    environment: String,
    server_port: u16,
    database_connections: u32,
    log_level: String,
    features: Vec<String>,
}

#[verb(help = "Show current configuration")]
pub fn show_config() -> Result<ConfigStatus, Box<dyn std::error::Error>> {
    let config = get_global_config()?;

    let mut features = Vec::new();
    if config.features.enable_telemetry {
        features.push("telemetry".to_string());
    }
    if config.features.enable_caching {
        features.push("caching".to_string());
    }
    if config.features.enable_rate_limiting {
        features.push("rate_limiting".to_string());
    }

    Ok(ConfigStatus {
        environment: std::env::var("ENVIRONMENT").unwrap_or("unknown".to_string()),
        server_port: config.server.port,
        database_connections: config.database.max_connections,
        log_level: config.logging.level.clone(),
        features,
    })
}
```

---

## Step 4: Secrets Management

### Kubernetes Secrets

```yaml
# secret.yaml
apiVersion: v1
kind: Secret
metadata:
  name: my-cli-secrets
  namespace: production
type: Opaque
stringData:
  DATABASE_URL: "postgres://user:password@prod-db:5432/myapp"
  API_KEY: "prod-api-key-xyz"
```

### Mount Secrets as Environment Variables

```yaml
# deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: my-cli
spec:
  template:
    spec:
      containers:
      - name: my-cli
        image: my-cli:latest
        env:
        - name: APP_DATABASE__URL
          valueFrom:
            secretKeyRef:
              name: my-cli-secrets
              key: DATABASE_URL
        - name: APP_API_KEY
          valueFrom:
            secretKeyRef:
              name: my-cli-secrets
              key: API_KEY
```

---

## Step 5: Feature Flags

### Runtime Feature Toggle

```rust
#[verb(help = "Process with optional caching")]
pub fn process(
    #[arg] data: String,
) -> Result<ProcessResult, Box<dyn std::error::Error>> {
    let config = get_global_config()?;

    // Check feature flag
    if config.features.enable_caching {
        // Try cache first
        if let Some(cached) = check_cache(&data)? {
            return Ok(cached);
        }
    }

    // Process data
    let result = crate::domain::process(&data)?;

    // Store in cache if enabled
    if config.features.enable_caching {
        store_in_cache(&data, &result)?;
    }

    Ok(result)
}
```

### LaunchDarkly Integration

```rust
use launchdarkly_server_sdk::{Client, ConfigBuilder};

pub struct FeatureFlags {
    client: Client,
}

impl FeatureFlags {
    pub fn new(sdk_key: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let config = ConfigBuilder::new(sdk_key).build();
        let client = Client::build(config)?;

        Ok(Self { client })
    }

    pub fn is_enabled(&self, flag: &str, user_id: &str) -> bool {
        self.client
            .bool_variation(flag, &user_context(user_id), false)
    }
}
```

---

## Step 6: Validation

### Config Validation

```rust
impl AppConfig {
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate port range
        if self.server.port < 1024 {
            return Err(ConfigError::InvalidPort(self.server.port));
        }

        // Validate database URL
        if !self.database.url.starts_with("postgres://") {
            return Err(ConfigError::InvalidDatabaseUrl);
        }

        // Validate connection pool
        if self.database.max_connections == 0 {
            return Err(ConfigError::InvalidConnectionPool);
        }

        // Validate log level
        let valid_levels = ["trace", "debug", "info", "warn", "error"];
        if !valid_levels.contains(&self.logging.level.as_str()) {
            return Err(ConfigError::InvalidLogLevel(self.logging.level.clone()));
        }

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Invalid port: {0} (must be >= 1024)")]
    InvalidPort(u16),

    #[error("Invalid database URL")]
    InvalidDatabaseUrl,

    #[error("Invalid connection pool size")]
    InvalidConnectionPool,

    #[error("Invalid log level: {0}")]
    InvalidLogLevel(String),
}
```

---

## Step 7: Configuration Hot Reload

### Watch Config Files

```rust
use notify::{Watcher, RecursiveMode, Result as NotifyResult};
use std::sync::{Arc, RwLock};

pub struct ConfigWatcher {
    config: Arc<RwLock<AppConfig>>,
}

impl ConfigWatcher {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config = Arc::new(RwLock::new(AppConfig::load()?));

        let config_clone = config.clone();
        let mut watcher = notify::recommended_watcher(move |res: NotifyResult<_>| {
            if let Ok(event) = res {
                // Reload config on file change
                if let Ok(new_config) = AppConfig::load() {
                    if let Ok(mut config) = config_clone.write() {
                        *config = new_config;
                        tracing::info!("Configuration reloaded");
                    }
                }
            }
        })?;

        watcher.watch("config".as_ref(), RecursiveMode::Recursive)?;

        Ok(Self { config })
    }

    pub fn get_config(&self) -> AppConfig {
        self.config.read().unwrap().clone()
    }
}
```

---

## Step 8: Environment Detection

### Auto-Detect Environment

```rust
pub fn detect_environment() -> String {
    // Check environment variable first
    if let Ok(env) = std::env::var("ENVIRONMENT") {
        return env;
    }

    // Detect from Kubernetes
    if std::path::Path::new("/var/run/secrets/kubernetes.io").exists() {
        if let Ok(namespace) = std::fs::read_to_string("/var/run/secrets/kubernetes.io/serviceaccount/namespace") {
            if namespace.contains("prod") {
                return "production".to_string();
            } else if namespace.contains("staging") {
                return "staging".to_string();
            }
        }
    }

    // Detect from AWS metadata
    if let Ok(response) = reqwest::blocking::get("http://169.254.169.254/latest/meta-data/instance-id") {
        if response.status().is_success() {
            // Running in AWS
            if let Ok(tags) = get_ec2_tags() {
                if let Some(env) = tags.get("Environment") {
                    return env.clone();
                }
            }
        }
    }

    // Default to development
    "development".to_string()
}
```

---

## Step 9: Configuration as Code

### Terraform Configuration

```hcl
# terraform/main.tf
resource "kubernetes_config_map" "my_cli_config" {
  metadata {
    name      = "my-cli-config"
    namespace = "production"
  }

  data = {
    "production.toml" = file("${path.module}/../config/production.toml")
  }
}

resource "kubernetes_secret" "my_cli_secrets" {
  metadata {
    name      = "my-cli-secrets"
    namespace = "production"
  }

  data = {
    DATABASE_URL = var.database_url
    API_KEY      = var.api_key
  }
}
```

---

## Best Practices

✅ **Use layered configuration** - Defaults → Files → Environment variables
✅ **Never commit secrets** - Use secret management systems
✅ **Validate configuration at startup** - Fail fast on invalid config
✅ **Environment-specific files** - Separate configs per environment
✅ **12-Factor App principles** - Config in environment, not code
✅ **Type-safe configuration** - Use strong types, not strings
✅ **Document all config options** - Clear descriptions and examples
✅ **Provide sensible defaults** - Make config optional where possible

---

## Troubleshooting

### Config Not Loading

```bash
# Check config file exists
ls -la config/

# Check environment variable
echo $ENVIRONMENT

# Enable debug logging
export RUST_LOG=config=debug
./my-cli config show-config
```

### Secret Not Found

```bash
# Check Kubernetes secret exists
kubectl get secret my-cli-secrets -n production

# Check secret is mounted
kubectl describe pod <pod-name> -n production | grep -A 10 "Environment"
```

---

## Next Steps

- **[How-To: Production Security](security.md)** - Secure configuration and secrets
- **[How-To: Production Deployment](deployment.md)** - Deploy with configuration
- **[Tutorial 09: Deployment Basics](../../tutorial/09-deployment-basics.md)** - Configuration fundamentals

---

*Part of the [clap-noun-verb How-To Guides](../README.md) - Problem-solving documentation*
