# How-To: Production Security

**Problem:** You need to secure your clap-noun-verb CLI against common vulnerabilities and threats in production.

**Solution:** Implement defense-in-depth security with input validation, secrets management, dependency scanning, and runtime protection.

---

## Prerequisites

- clap-noun-verb CLI deployed to production
- Container runtime (Docker)
- Kubernetes or similar orchestration (optional)
- Secret management system

---

## Quick Start (10 Minutes)

```bash
# 1. Audit dependencies
cargo audit

# 2. Add security dependencies
cargo add secrecy argon2

# 3. Implement input validation
# See Step 1 below

# 4. Secure secrets
# See Step 2 below

# 5. Run as non-root
# See Step 4 below
```

---

## Step 1: Input Validation

### Validate All Inputs

```rust
use clap_noun_verb_macros::verb;
use regex::Regex;
use serde::Serialize;

#[derive(Serialize)]
pub struct UserCreated {
    user_id: String,
    email: String,
}

#[verb(help = "Create user with validation")]
pub fn create_user(
    #[arg(help = "User email")] email: String,
    #[arg(help = "Username")] username: String,
) -> Result<UserCreated, Box<dyn std::error::Error>> {
    // Validate email format
    validate_email(&email)?;

    // Validate username (alphanumeric only)
    validate_username(&username)?;

    // Prevent SQL injection
    if contains_sql_injection(&username) {
        return Err("Invalid username: potential SQL injection detected".into());
    }

    let user_id = crate::domain::users::create(&email, &username)?;

    Ok(UserCreated {
        user_id,
        email,
    })
}

fn validate_email(email: &str) -> Result<(), Box<dyn std::error::Error>> {
    let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$")?;

    if !email_regex.is_match(email) {
        return Err(format!("Invalid email format: {}", email).into());
    }

    // Check length
    if email.len() > 254 {
        return Err("Email too long".into());
    }

    Ok(())
}

fn validate_username(username: &str) -> Result<(), Box<dyn std::error::Error>> {
    let username_regex = Regex::new(r"^[a-zA-Z0-9_-]{3,32}$")?;

    if !username_regex.is_match(username) {
        return Err("Invalid username: must be 3-32 alphanumeric characters".into());
    }

    Ok(())
}

fn contains_sql_injection(input: &str) -> bool {
    let patterns = ["--", ";", "/*", "*/", "xp_", "sp_", "DROP", "SELECT", "INSERT", "DELETE"];

    patterns.iter().any(|pattern| input.to_uppercase().contains(pattern))
}
```

### Path Traversal Prevention

```rust
use std::path::{Path, PathBuf};

fn validate_file_path(input_path: &str, base_dir: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let path = Path::new(input_path);

    // Canonicalize to resolve .. and symlinks
    let canonical = path.canonicalize()
        .map_err(|_| "Invalid file path")?;

    // Ensure path is within allowed directory
    if !canonical.starts_with(base_dir) {
        return Err(format!("Path traversal detected: {:?}", input_path).into());
    }

    Ok(canonical)
}

#[verb(help = "Read file with path validation")]
pub fn read_file(
    #[arg(help = "File path")] path: String,
) -> Result<FileContent, Box<dyn std::error::Error>> {
    let base_dir = Path::new("/var/data");
    let safe_path = validate_file_path(&path, base_dir)?;

    let content = std::fs::read_to_string(safe_path)?;

    Ok(FileContent { content })
}
```

---

## Step 2: Secrets Management

### Never Store Secrets in Code

```rust
// ❌ BAD: Hardcoded secret
const API_KEY: &str = "sk-1234567890abcdef"; // NEVER DO THIS!

// ✅ GOOD: Load from environment
fn get_api_key() -> Result<String, Box<dyn std::error::Error>> {
    std::env::var("API_KEY")
        .map_err(|_| "API_KEY environment variable not set".into())
}
```

### Use secrecy Crate

```rust
use secrecy::{Secret, ExposeSecret};

pub struct AppSecrets {
    database_password: Secret<String>,
    api_key: Secret<String>,
}

impl AppSecrets {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            database_password: Secret::new(std::env::var("DATABASE_PASSWORD")?),
            api_key: Secret::new(std::env::var("API_KEY")?),
        })
    }

    pub fn database_url(&self) -> String {
        format!(
            "postgres://user:{}@localhost/db",
            self.database_password.expose_secret()
        )
    }
}

// Secrets are automatically redacted in debug output
// Debug output: AppSecrets { database_password: Secret([REDACTED]), api_key: Secret([REDACTED]) }
```

### Kubernetes Secrets

```yaml
# secret.yaml
apiVersion: v1
kind: Secret
metadata:
  name: my-cli-secrets
  namespace: production
type: Opaque
data:
  # Base64 encoded values
  database-password: cGFzc3dvcmQxMjM=
  api-key: c2stMTIzNDU2Nzg5MA==
```

---

## Step 3: Cryptography

### Hash Passwords (Never Store Plaintext)

```rust
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

pub fn hash_password(password: &str) -> Result<String, Box<dyn std::error::Error>> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)?
        .to_string();

    Ok(password_hash)
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let parsed_hash = PasswordHash::new(hash)?;
    let argon2 = Argon2::default();

    Ok(argon2.verify_password(password.as_bytes(), &parsed_hash).is_ok())
}

#[verb(help = "Create user with hashed password")]
pub fn create_user_secure(
    #[arg] username: String,
    #[arg] password: String,
) -> Result<UserCreated, Box<dyn std::error::Error>> {
    // Hash password before storing
    let password_hash = hash_password(&password)?;

    let user_id = crate::domain::users::create_with_hash(&username, &password_hash)?;

    Ok(UserCreated { user_id, username })
}
```

---

## Step 4: Container Security

### Run as Non-Root User

```dockerfile
# Dockerfile
FROM rust:1.75-slim as builder

WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim

# Create non-root user
RUN groupadd -r appuser && \
    useradd -r -g appuser -u 1000 appuser

# Copy binary
COPY --from=builder /app/target/release/my-cli /usr/local/bin/my-cli

# Set ownership
RUN chown appuser:appuser /usr/local/bin/my-cli

# Switch to non-root user
USER appuser

CMD ["/usr/local/bin/my-cli"]
```

### Security Context (Kubernetes)

```yaml
# deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: my-cli
spec:
  template:
    spec:
      securityContext:
        runAsNonRoot: true
        runAsUser: 1000
        fsGroup: 1000
        seccompProfile:
          type: RuntimeDefault

      containers:
      - name: my-cli
        image: my-cli:latest

        securityContext:
          allowPrivilegeEscalation: false
          readOnlyRootFilesystem: true
          capabilities:
            drop:
            - ALL
```

---

## Step 5: Dependency Scanning

### Audit Dependencies

```bash
# Install cargo-audit
cargo install cargo-audit

# Run audit
cargo audit

# Auto-fix known issues
cargo audit fix
```

### Automated Scanning (CI/CD)

```yaml
# .github/workflows/security.yml
name: Security Scan

on: [push, pull_request]

jobs:
  audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install cargo-audit
        run: cargo install cargo-audit

      - name: Run security audit
        run: cargo audit

      - name: Check for vulnerabilities
        run: cargo audit --deny warnings
```

---

## Step 6: Rate Limiting

### Command Rate Limiting

```rust
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::time::{Duration, Instant};

pub struct RateLimiter {
    requests: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
    max_requests: usize,
    window: Duration,
}

impl RateLimiter {
    pub fn new(max_requests: usize, window: Duration) -> Self {
        Self {
            requests: Arc::new(Mutex::new(HashMap::new())),
            max_requests,
            window,
        }
    }

    pub fn check_rate_limit(&self, user_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut requests = self.requests.lock().unwrap();
        let now = Instant::now();

        let user_requests = requests.entry(user_id.to_string()).or_insert_with(Vec::new);

        // Remove old requests outside window
        user_requests.retain(|&time| now.duration_since(time) < self.window);

        if user_requests.len() >= self.max_requests {
            return Err(format!(
                "Rate limit exceeded: {} requests in {:?}",
                self.max_requests, self.window
            ).into());
        }

        user_requests.push(now);

        Ok(())
    }
}

#[verb(help = "API call with rate limiting")]
pub fn api_call(
    #[arg] endpoint: String,
    #[arg(env = "USER_ID")] user_id: String,
) -> Result<ApiResponse, Box<dyn std::error::Error>> {
    let rate_limiter = get_rate_limiter();

    // Check rate limit
    rate_limiter.check_rate_limit(&user_id)?;

    // Make API call
    let response = crate::domain::api::call(&endpoint)?;

    Ok(response)
}
```

---

## Step 7: Audit Logging

### Log Security Events

```rust
use tracing::{warn, error};

#[verb]
pub fn delete_resource(
    #[arg] resource_id: String,
    #[arg(env = "USER_ID")] user_id: String,
) -> Result<DeleteResult, Box<dyn std::error::Error>> {
    // Log security-sensitive operation
    tracing::info!(
        user_id = %user_id,
        resource_id = %resource_id,
        action = "delete_resource",
        "Security-sensitive operation initiated"
    );

    // Check authorization
    if !is_authorized(&user_id, &resource_id)? {
        error!(
            user_id = %user_id,
            resource_id = %resource_id,
            "Unauthorized delete attempt"
        );
        return Err("Unauthorized".into());
    }

    // Perform deletion
    crate::domain::resources::delete(&resource_id)?;

    tracing::info!(
        user_id = %user_id,
        resource_id = %resource_id,
        action = "delete_resource",
        status = "success",
        "Resource deleted successfully"
    );

    Ok(DeleteResult { resource_id })
}
```

---

## Step 8: Network Security

### TLS/HTTPS Only

```rust
use reqwest::Client;

pub async fn make_secure_request(url: &str) -> Result<Response, Box<dyn std::error::Error>> {
    // Enforce HTTPS
    if !url.starts_with("https://") {
        return Err("Only HTTPS URLs allowed".into());
    }

    let client = Client::builder()
        .min_tls_version(reqwest::tls::Version::TLS_1_2)
        .build()?;

    let response = client.get(url).send().await?;

    Ok(response)
}
```

---

## Step 9: Secure Defaults

### Security Checklist

```rust
pub struct SecurityConfig {
    pub enforce_https: bool,
    pub require_authentication: bool,
    pub enable_rate_limiting: bool,
    pub max_request_size: usize,
    pub session_timeout_minutes: u64,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            enforce_https: true,          // ✅ Always HTTPS
            require_authentication: true, // ✅ Always auth
            enable_rate_limiting: true,   // ✅ Rate limiting on
            max_request_size: 1_048_576,  // 1MB max
            session_timeout_minutes: 30,  // 30 minute sessions
        }
    }
}
```

---

## Best Practices

✅ **Validate all inputs** - Never trust user input
✅ **Never hardcode secrets** - Use environment variables or secret managers
✅ **Run as non-root** - Minimize privilege escalation risk
✅ **Audit dependencies** - Scan for known vulnerabilities
✅ **Use HTTPS only** - Encrypt all network communication
✅ **Implement rate limiting** - Prevent abuse
✅ **Log security events** - Audit trail for incidents
✅ **Hash passwords** - Never store plaintext passwords
✅ **Principle of least privilege** - Minimal permissions needed
✅ **Defense in depth** - Multiple layers of security

---

## Troubleshooting

### cargo audit Failures

```bash
# Check which crates have vulnerabilities
cargo audit

# See detailed advisory
cargo audit --db /path/to/advisory-db

# Ignore specific advisory (NOT recommended)
cargo audit --ignore RUSTSEC-YYYY-NNNN
```

### Permission Denied Errors

```bash
# Check file permissions
ls -la /var/data

# Fix ownership
chown -R appuser:appuser /var/data

# Check security context
kubectl get pod <pod-name> -o yaml | grep securityContext -A 10
```

---

## Next Steps

- **[How-To: Production Deployment](deployment.md)** - Secure deployment practices
- **[How-To: Production Configuration](configuration.md)** - Secure configuration management
- **[Tutorial 08: Error Handling](../../tutorial/08-error-handling.md)** - Secure error handling

---

*Part of the [clap-noun-verb How-To Guides](../README.md) - Problem-solving documentation*
