# Security Policy

## Supported Versions

We currently support the following versions with security updates:

| Version | Supported          |
| ------- | ------------------ |
| 4.0.x   | :white_check_mark: |
| 3.x.x   | :white_check_mark: |
| < 3.0   | :x:                |

## Reporting a Vulnerability

If you discover a security vulnerability, please **DO NOT** open a public issue. Instead, please report it via one of the following methods:

1. **Email**: Send details to the maintainer at [seanchatmangpt@gmail.com](mailto:seanchatmangpt@gmail.com)
2. **GitHub Security Advisory**: Use GitHub's private vulnerability reporting feature

### What to Include

When reporting a vulnerability, please include:

- Description of the vulnerability
- Steps to reproduce the issue
- Potential impact
- Suggested fix (if you have one)

### Response Time

We will acknowledge receipt of your report within 48 hours and provide an initial assessment within 7 days.

### Disclosure Policy

We follow responsible disclosure practices:

1. We will notify you when we receive your report
2. We will keep you informed of our progress
3. We will notify you when the vulnerability is fixed
4. We will publicly disclose the vulnerability after it's patched (typically within 90 days)

Thank you for helping keep clap-noun-verb secure!

---

## Security Features in v4.0.0

### Plugin Signature Verification

Version 4.0.0 introduces Ed25519 signature verification for plugins to ensure authenticity and prevent tampering.

**Usage:**
```rust
use clap_noun_verb::plugin::PluginManifest;

// Load plugin manifest
let manifest = PluginManifest::new("my-plugin", "1.0.0", "libplugin.so")
    .with_signature("base64_encoded_signature")
    .with_public_key("base64_encoded_public_key");

// Verify signature before loading
if !manifest.verify_signature()? {
    return Err("Plugin signature verification failed".into());
}
```

**Best Practices:**
- Always verify signatures before loading plugins in production
- Store private signing keys securely (HSM or key vault)
- Rotate keys periodically
- Log signature verification failures

### Resource Quotas for Plugins

Plugins can be executed with resource limits to prevent runaway resource consumption:

```rust
use clap_noun_verb::plugin::{PluginConfig, ResourceQuota};

// Configure plugin quotas
let quota = ResourceQuota::builder()
    .cpu_time_ms(5000)        // 5 seconds max
    .memory_bytes(100_000_000) // 100MB max
    .file_handles(50)          // 50 files max
    .network_connections(10)   // 10 connections max
    .build();

let config = PluginConfig::new()
    .with_quotas(quota)
    .with_sandbox(true);

// Execute plugin with quota enforcement
let guard = config.acquire_quota_guard()?;
if let Some(guard) = guard {
    // Plugin execution happens here
    guard.check_quotas()?; // Verify quotas not exceeded
}
```

**Quota Types:**
- **CPU Time**: Maximum walltime for plugin execution
- **Memory**: Maximum heap allocation
- **File Handles**: Maximum number of open files
- **Network Connections**: Maximum concurrent connections

**Graceful Failure:**
Quota violations return clear error messages without crashing the application.

### PII Redaction in Middleware

Middleware supports automatic redaction of sensitive information:

```rust
use clap_noun_verb::middleware::MiddlewareRequest;

let request = MiddlewareRequest::new("user-login")
    .with_arg("--password=secret123")
    .with_arg("--email=user@example.com");

// Redact sensitive arguments before logging
let sensitive_patterns = &["password", "token", "api_key", "email"];
let redacted = request.redacted_args(sensitive_patterns);
// Outputs: ["[REDACTED]", "[REDACTED]"]
```

**Patterns Supported:**
- Passwords (password, passwd, pwd)
- Tokens (token, bearer, auth)
- API Keys (api_key, apikey)
- Email addresses (email)
- SSN, credit cards, etc.

### Path Traversal Protection

Plugin paths are validated and canonicalized to prevent directory traversal attacks:

```rust
use clap_noun_verb::plugin::PluginLoader;

// Validates plugin paths against traversal
let loader = PluginLoader::new("./plugins");
loader.discover()?; // Automatically validates all paths
```

**Protected Against:**
- `../../../etc/passwd`
- Symlink attacks
- Absolute path escapes
- Windows path traversal

### Security Testing

Version 4.0.0 includes comprehensive security tests in `tests/security_tests.rs`:

- Plugin path traversal prevention
- PII redaction validation
- Plugin isolation enforcement
- Argument validation
- Error message safety

Run security tests:
```bash
cargo test --test security_tests
```

### Unsafe Code Audit

All unsafe code has been audited and documented in `docs/UNSAFE_CODE_AUDIT_v4.0.0.md`:

- 8 unsafe blocks total (SIMD optimizations only)
- All blocks properly documented with safety invariants
- Comprehensive test coverage for unsafe code
- No unsafe code in security-critical paths

### Dependency Security

- Removed `atty` (RUSTSEC-2021-0145)
- All dependencies up-to-date
- Regular security audits via `cargo audit`

**Run security audit:**
```bash
cargo audit
cargo deny check advisories
```

