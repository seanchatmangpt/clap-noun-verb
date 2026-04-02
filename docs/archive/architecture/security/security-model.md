# Security Model for Federated Semantic Network

## Threat Model

### Adversary Capabilities

**Assumed Adversary Powers**:
1. **Network Control**: Can intercept, modify, replay, or drop network messages
2. **Node Compromise**: Can compromise up to f < n/3 nodes (Byzantine tolerance threshold)
3. **Cryptographic Resources**: Cannot break Ed25519 signatures or SHA-256 hashes (computationally bounded)
4. **Knowledge**: Knows all public information (ontologies, endpoints, public keys)

**Adversary Goals**:
1. Unauthorized command execution
2. Data exfiltration or corruption
3. Denial of service
4. Impersonation of legitimate CLIs
5. Capability token forgery or theft

### Attack Surfaces

| Surface              | Attack Vectors                          | Mitigations                          |
|----------------------|-----------------------------------------|--------------------------------------|
| SPARQL Endpoints     | Query injection, DOS                    | Query sanitization, rate limiting    |
| gRPC Invocation      | MITM, replay attacks                    | TLS, signature verification          |
| Capability Tokens    | Token theft, forgery                    | Short expiration, binding to TLS cert|
| Discovery (DHT)      | Sybil attack, eclipse attack            | Proof-of-work join, diverse peer set |
| Consensus (BFT)      | Byzantine nodes, fork attacks           | HotStuff consensus (f < n/3)         |
| Type System          | Type confusion, deserialization attacks | SHACL validation, runtime checks     |

## Security Principles

### 1. Zero-Trust Architecture

**Principle**: Never trust, always verify.

**Implementation**:
- Every request requires capability token
- Every message requires cryptographic signature
- Every response is validated against type schema
- No ambient authority (possession of credentials = access)

```rust
async fn zero_trust_invoke(
    request: CommandRequest,
    context: &SecurityContext,
) -> Result<CommandResponse, SecurityError> {
    // 1. Verify request signature
    verify_signature(&request, &request.signature)?;

    // 2. Verify capability token
    verify_capability_token(&request.capability_token, &request.command_name)?;

    // 3. Validate request types (SHACL)
    validate_request_types(&request, &context.ontology)?;

    // 4. Check revocation lists
    check_revocation(&request.capability_token)?;

    // 5. Execute command
    let response = execute_command(&request).await?;

    // 6. Validate response types
    validate_response_types(&response, &context.ontology)?;

    // 7. Sign response
    let signed_response = sign_response(response, &context.private_key)?;

    Ok(signed_response)
}
```

### 2. Defense in Depth

**Layers of Security**:
1. **Network Layer**: TLS 1.3 encryption, mutual authentication
2. **Transport Layer**: gRPC with ALTS (Application Layer Transport Security)
3. **Message Layer**: Ed25519 signatures on every request/response
4. **Authorization Layer**: JWT capability tokens with SHACL constraints
5. **Application Layer**: SHACL type validation, business logic checks
6. **Audit Layer**: Immutable append-only logs

```
┌────────────────────────────────────────────────┐
│ Application Layer                              │
│ - Business logic validation                    │
└────────────────┬───────────────────────────────┘
                 │
┌────────────────▼───────────────────────────────┐
│ Audit Layer                                    │
│ - Append-only logs, distributed tracing        │
└────────────────┬───────────────────────────────┘
                 │
┌────────────────▼───────────────────────────────┐
│ Authorization Layer                            │
│ - Capability tokens, SHACL constraints         │
└────────────────┬───────────────────────────────┘
                 │
┌────────────────▼───────────────────────────────┐
│ Message Layer                                  │
│ - Ed25519 signatures, timestamp verification   │
└────────────────┬───────────────────────────────┘
                 │
┌────────────────▼───────────────────────────────┐
│ Transport Layer                                │
│ - gRPC, HTTP/2, ALTS                           │
└────────────────┬───────────────────────────────┘
                 │
┌────────────────▼───────────────────────────────┐
│ Network Layer                                  │
│ - TLS 1.3, mutual authentication               │
└────────────────────────────────────────────────┘
```

### 3. Principle of Least Privilege

**Implementation**:
- Capability tokens grant minimum necessary permissions
- Tokens can be attenuated (narrowed) but never amplified
- Tokens expire automatically (default: 1 hour)
- No long-lived credentials

```rust
#[derive(Serialize, Deserialize)]
struct CapabilityToken {
    jti: Uuid,
    iss: String,
    sub: String,

    // Minimal capabilities
    cap: Vec<String>, // Only what's needed

    // Constraints (further restrict permissions)
    constraints: Option<ShaclShape>,

    // Time bounds
    exp: i64, // Expiration (short-lived)
    nbf: i64, // Not before

    // Attenuation chain
    delegation_chain: Vec<String>,
    parent_token_id: Option<Uuid>, // Can trace to root
}

impl CapabilityToken {
    // Attenuate: narrow permissions, cannot amplify
    pub fn attenuate(&self, new_cap: Vec<String>) -> Result<Self, Error> {
        // Verify new capabilities are subset of current
        for cap in &new_cap {
            if !self.cap.contains(cap) {
                return Err(Error::CapabilityEscalation {
                    requested: cap.clone(),
                    available: self.cap.clone(),
                });
            }
        }

        // Create attenuated token
        Ok(Self {
            jti: Uuid::new_v4(),
            iss: self.sub.clone(), // Attenuator becomes issuer
            sub: "...".to_string(), // New subject
            cap: new_cap,
            constraints: self.constraints.clone(), // Inherit constraints
            exp: std::cmp::min(self.exp, current_timestamp() + 3600), // Cannot extend
            nbf: current_timestamp(),
            delegation_chain: {
                let mut chain = self.delegation_chain.clone();
                chain.push(self.jti.to_string());
                chain
            },
            parent_token_id: Some(self.jti),
        })
    }
}
```

## Cryptographic Primitives

### Ed25519 Digital Signatures

**Why Ed25519**:
- Fast: 64K signatures/sec, 40K verifications/sec (single core)
- Small: 32-byte public key, 64-byte signature
- Secure: 128-bit security level, deterministic (no RNG failures)
- Widely supported: libsodium, ring, ed25519-dalek

**Signature Scheme**:
```rust
use ed25519_dalek::{Keypair, Signature, Signer, Verifier};

// Key generation (once per CLI)
let mut csprng = OsRng{};
let keypair = Keypair::generate(&mut csprng);

// Signing
let message = b"canonical request representation";
let signature = keypair.sign(message);

// Verification
assert!(keypair.public.verify(message, &signature).is_ok());
```

**Key Management**:
- Private keys stored in OS keychain (macOS Keychain, Windows Credential Manager, Linux Secret Service)
- Public keys published in RDF ontology
- Key rotation every 90 days (automated)

### TLS 1.3 with Mutual Authentication

**Configuration**:
```rust
use tonic::transport::{Certificate, Identity, ServerTlsConfig};

// Server side
let cert = std::fs::read_to_string("server-cert.pem")?;
let key = std::fs::read_to_string("server-key.pem")?;
let ca_cert = std::fs::read_to_string("ca-cert.pem")?;

let tls_config = ServerTlsConfig::new()
    .identity(Identity::from_pem(&cert, &key))
    .client_ca_root(Certificate::from_pem(&ca_cert));

// Client side
let client_cert = std::fs::read_to_string("client-cert.pem")?;
let client_key = std::fs::read_to_string("client-key.pem")?;

let tls_config = ClientTlsConfig::new()
    .identity(Identity::from_pem(&client_cert, &client_key))
    .ca_certificate(Certificate::from_pem(&ca_cert));
```

**Certificate Binding**:
Bind capability tokens to TLS client certificates to prevent token theft:

```rust
#[derive(Serialize, Deserialize)]
struct CapabilityToken {
    // ...existing fields...

    // Certificate binding
    cnf: Option<ConfirmationClaim>,
}

#[derive(Serialize, Deserialize)]
struct ConfirmationClaim {
    // SHA-256 hash of TLS client certificate
    x5t_s256: String,
}

fn verify_certificate_binding(
    token: &CapabilityToken,
    tls_cert: &Certificate,
) -> Result<(), Error> {
    if let Some(cnf) = &token.cnf {
        let cert_hash = sha256(tls_cert.as_bytes());
        let expected_hash = hex::decode(&cnf.x5t_s256)?;

        if cert_hash != expected_hash {
            return Err(Error::CertificateMismatch {
                expected: cnf.x5t_s256.clone(),
                actual: hex::encode(&cert_hash),
            });
        }
    }

    Ok(())
}
```

## Access Control

### Capability-Based Security (Object Capabilities)

**Model**:
- Capabilities are unforgeable tokens that grant authority
- Possession of capability = authorization to invoke
- No confused deputy problem (caller cannot misuse deputy's authority)

**Example**:
```rust
// Traditional ACL (confused deputy problem)
fn process_file_acl(user: &User, file_path: &str) -> Result<Data, Error> {
    // Check if user has permission to read file
    if !user.has_permission("read", file_path) {
        return Err(Error::PermissionDenied);
    }

    // BUG: Attacker can trick this function into reading any file
    // by passing file_path = "/etc/passwd" if this function runs with elevated privileges
    read_file(file_path)
}

// Capability-based (no confused deputy)
fn process_file_cap(file_cap: &FileReadCapability) -> Result<Data, Error> {
    // Capability directly references the file; cannot be tricked
    file_cap.read()
}
```

### RBAC Mapping (Optional)

For organizations that need traditional RBAC, map roles to capability sets:

```turtle
# Role definition
:DeveloperRole a clicap:Role ;
    clicap:includesCapability clicap:ReadImageFiles ;
    clicap:includesCapability clicap:ExecuteConvert ;
    clicap:excludesCapability clicap:DeleteFiles . # Principle of least privilege

# User-Role assignment
:alice a foaf:Person ;
    clicap:hasRole :DeveloperRole .

# Capability token generation from role
# When Alice authenticates, issue token with capabilities from DeveloperRole
```

## Security Protocols

### Authentication Protocol

```
┌──────────┐                                    ┌──────────┐
│ CLI A    │                                    │ CLI B    │
└────┬─────┘                                    └────┬─────┘
     │                                                │
     │ 1. TLS ClientHello (with client cert)         │
     ├───────────────────────────────────────────────►│
     │                                                │
     │ 2. TLS ServerHello (mutual auth)               │
     │◄───────────────────────────────────────────────┤
     │                                                │
     │ 3. gRPC Request + Capability Token + Signature │
     ├───────────────────────────────────────────────►│
     │                                                │
     │                         4. Verify:             │
     │                            - TLS cert valid    │
     │                            - Token signature   │
     │                            - Token not expired │
     │                            - Token not revoked │
     │                            - Token binds to cert│
     │                                                │
     │ 5. gRPC Response + Signature                   │
     │◄───────────────────────────────────────────────┤
     │                                                │
     │ 6. Verify response signature                   │
     │                                                │
```

### Revocation Protocol

**Certificate Revocation Lists (CRLs)**:

```turtle
# Revocation list (published via RDF)
:RevocationList a clicap:RevocationList ;
    dcterms:created "2026-01-05T12:00:00Z"^^xsd:dateTime ;
    dcterms:issuer <https://example.com/cli> ;
    clicap:revokedToken [
        clicap:tokenId "550e8400-e29b-41d4-a716-446655440000" ;
        clicap:revokedAt "2026-01-05T11:30:00Z"^^xsd:dateTime ;
        clicap:reason "Token compromised"
    ] .
```

```rust
async fn check_revocation(token_id: &Uuid) -> Result<bool, Error> {
    // 1. Fetch CRL from issuer's SPARQL endpoint
    let issuer = extract_issuer_from_token(token_id)?;
    let crl_uri = format!("{}/revocation-list", issuer);

    let crl = fetch_crl(&crl_uri).await?;

    // 2. Check if token ID in revocation list
    let is_revoked = crl.revoked_tokens.contains(token_id);

    // 3. Cache result (TTL: 5 minutes)
    cache_revocation_status(token_id, is_revoked).await;

    Ok(is_revoked)
}
```

**OCSP (Online Certificate Status Protocol)** (Future):
- Real-time revocation checking
- Lower latency than CRL (no download entire list)

## Audit & Compliance

### Immutable Audit Log

```rust
use sha2::{Sha256, Digest};

#[derive(Serialize, Deserialize)]
struct AuditLogEntry {
    // Entry metadata
    entry_id: Uuid,
    timestamp: i64,
    previous_hash: String, // Hash of previous entry (blockchain-style)

    // Event data
    event_type: AuditEventType,
    invoker_uri: String,
    provider_uri: String,
    command_name: String,

    // Security data
    capability_token_id: Uuid,
    request_signature: String,
    response_signature: String,

    // Result
    success: bool,
    error_code: Option<String>,
}

enum AuditEventType {
    CommandInvocation,
    CapabilityIssued,
    CapabilityRevoked,
    SecurityViolation,
}

impl AuditLogEntry {
    fn compute_hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(serde_json::to_string(self).unwrap());
        hex::encode(hasher.finalize())
    }

    fn verify_chain(&self, previous_entry: &AuditLogEntry) -> bool {
        self.previous_hash == previous_entry.compute_hash()
    }
}

// Append-only log
async fn append_audit_log(entry: AuditLogEntry) -> Result<(), Error> {
    // 1. Get hash of previous entry
    let previous_hash = get_latest_entry_hash().await?;

    // 2. Set previous_hash in new entry
    let mut entry = entry;
    entry.previous_hash = previous_hash;

    // 3. Append to log (immutable storage)
    append_to_log(&entry).await?;

    // 4. Optional: Publish to distributed ledger for tamper-evidence
    publish_to_ledger(&entry).await?;

    Ok(())
}
```

### Compliance Reporting

Generate compliance reports for auditors:

```rust
async fn generate_compliance_report(
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
) -> ComplianceReport {
    let logs = query_audit_logs(start_date, end_date).await?;

    ComplianceReport {
        period: (start_date, end_date),
        total_invocations: logs.len(),
        unauthorized_attempts: logs.iter()
            .filter(|e| e.event_type == AuditEventType::SecurityViolation)
            .count(),
        capabilities_issued: logs.iter()
            .filter(|e| e.event_type == AuditEventType::CapabilityIssued)
            .count(),
        capabilities_revoked: logs.iter()
            .filter(|e| e.event_type == AuditEventType::CapabilityRevoked)
            .count(),
        success_rate: logs.iter().filter(|e| e.success).count() as f64 / logs.len() as f64,
    }
}
```

## Security Incident Response

### Incident Types & Responses

| Incident                 | Detection                   | Response                        |
|--------------------------|-----------------------------|---------------------------------|
| Invalid signature        | Signature verification fail | Reject request, log incident    |
| Expired token            | Token exp < now             | Reject, require re-auth         |
| Revoked token            | CRL check                   | Reject, log security violation  |
| Rate limit exceeded      | Request counter             | Throttle, return 429            |
| Type validation failed   | SHACL violation             | Reject, log malicious payload   |
| Byzantine node detected  | BFT consensus divergence    | Isolate node, alert admin       |

### Security Monitoring

```rust
use prometheus::{Counter, Histogram};

lazy_static! {
    static ref SECURITY_VIOLATIONS: Counter = Counter::new(
        "security_violations_total",
        "Total number of security violations"
    ).unwrap();

    static ref SIGNATURE_VERIFICATION_TIME: Histogram = Histogram::new(
        "signature_verification_seconds",
        "Time to verify signatures"
    ).unwrap();
}

async fn invoke_with_monitoring(request: CommandRequest) -> Result<CommandResponse, Error> {
    let timer = SIGNATURE_VERIFICATION_TIME.start_timer();

    match verify_signature(&request) {
        Ok(_) => {
            timer.observe_duration();
            execute_command(&request).await
        }
        Err(e) => {
            SECURITY_VIOLATIONS.inc();
            log::error!("Signature verification failed: {:?}", e);
            Err(e.into())
        }
    }
}
```

## Security Checklist

- [ ] Implement Ed25519 signature generation and verification
- [ ] Implement JWT capability token issuance and validation
- [ ] Implement TLS 1.3 mutual authentication
- [ ] Implement certificate binding for tokens
- [ ] Implement CRL checking with caching
- [ ] Implement SHACL type validation
- [ ] Implement immutable audit logging
- [ ] Implement rate limiting per token
- [ ] Implement security violation alerting
- [ ] Implement key rotation automation
- [ ] Add security integration tests (penetration testing)
- [ ] Add property tests for cryptographic primitives
- [ ] Conduct third-party security audit

## References

- [Zero Trust Architecture (NIST SP 800-207)](https://nvlpubs.nist.gov/nistpubs/SpecialPublications/NIST.SP.800-207.pdf)
- [OAuth 2.0 Token Binding](https://datatracker.ietf.org/doc/html/rfc8473)
- [Ed25519 Signature Scheme](https://ed25519.cr.yp.to/)
- [Object Capability Security](http://erights.org/elib/capability/ode/ode.pdf)
