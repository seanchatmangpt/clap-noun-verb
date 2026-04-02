# ADR-004: Capability-Based Security Model

## Status
Accepted

## Context

Cross-CLI invocation requires security guarantees:
- **Authentication**: Verify caller identity
- **Authorization**: Verify caller permissions
- **Integrity**: Verify request not tampered
- **Non-repudiation**: Audit trail of invocations

In a federated network with no central authority, traditional ACL/RBAC models don't work.

## Decision

We will implement **Capability-Based Security** using signed JWT tokens with embedded permissions.

## Rationale

### Security Model Comparison

| Model               | Central Auth | Delegation | Revocation | Offline | Zero-Trust |
|---------------------|--------------|------------|------------|---------|------------|
| ACL (Access Control)| Required     | Hard       | Easy       | ✗       | ✗          |
| RBAC (Role-Based)   | Required     | Moderate   | Easy       | ✗       | ✗          |
| OAuth2              | Required     | ✓          | Moderate   | ✗       | ~          |
| Capabilities (JWT)  | ✗            | ✓          | Hard       | ✓       | ✓          |
| Macaroons           | ✗            | ✓          | ✓          | ✓       | ✓          |

### Key Design Principles

1. **Ambient Authority Rejection**: Caller doesn't get permissions by identity alone; must present capability token
2. **Least Privilege**: Tokens grant minimum necessary permissions
3. **Delegation**: Tokens can be attenuated (narrowed) and passed on
4. **Unforgeable**: Cryptographic signatures prevent token forgery
5. **Time-Bounded**: Tokens expire automatically (no perpetual access)

### Capability Token Structure

```rust
#[derive(Serialize, Deserialize)]
struct CapabilityToken {
    /// Token identifier (for revocation)
    jti: Uuid,

    /// Issuer (CLI that granted capability)
    iss: String, // URI of issuing CLI

    /// Subject (CLI receiving capability)
    sub: String, // URI of subject CLI

    /// Capabilities granted (semantic RDF URIs)
    cap: Vec<String>, // e.g., ["clicap:convert#execute"]

    /// Constraints (SHACL shapes)
    constraints: Option<ShaclShape>,

    /// Expiration timestamp
    exp: i64,

    /// Not before timestamp
    nbf: i64,

    /// Delegation chain (for audit)
    delegation_chain: Vec<String>,
}
```

**Example Token**:
```json
{
  "jti": "550e8400-e29b-41d4-a716-446655440000",
  "iss": "cli://system-a.example.com/convert-tool",
  "sub": "cli://system-b.example.com/workflow-engine",
  "cap": [
    "https://clicap.org/capabilities#execute-convert",
    "https://clicap.org/capabilities#read-image-files"
  ],
  "constraints": {
    "maxFileSize": "10MB",
    "allowedFormats": ["PNG", "JPEG"]
  },
  "exp": 1735689600,
  "nbf": 1735603200,
  "delegation_chain": ["cli://admin.example.com/root"]
}
```

### How It Works

1. **Capability Advertisement**: CLI publishes capabilities in RDF ontology
   ```turtle
   :ConvertCapability a clicap:Capability ;
     clicap:grants clicap:execute-convert ;
     clicap:requiredSignature :AdminKey .
   ```

2. **Token Issuance**: Admin grants capability to workflow engine
   ```rust
   let token = CapabilityToken::new()
       .issuer("cli://system-a/convert-tool")
       .subject("cli://system-b/workflow-engine")
       .capability("clicap:execute-convert")
       .expires_in(Duration::days(30))
       .sign(&admin_private_key);
   ```

3. **Invocation with Capability**: Workflow engine includes token in gRPC metadata
   ```rust
   let mut request = tonic::Request::new(ConvertRequest { ... });
   request.metadata_mut().insert(
       "authorization",
       format!("Bearer {}", token).parse().unwrap()
   );
   ```

4. **Verification**: Convert tool validates signature and checks capabilities
   ```rust
   fn verify_capability(token: &str, required_cap: &str) -> Result<(), SecurityError> {
       let claims = jwt::decode::<CapabilityToken>(
           token,
           &public_key,
           &Validation::default()
       )?;

       if !claims.cap.contains(&required_cap.to_string()) {
           return Err(SecurityError::InsufficientPermissions);
       }

       // Check constraints (SHACL validation)
       validate_constraints(&claims.constraints)?;

       Ok(())
   }
   ```

### Trade-offs

**Costs**:
- Revocation hard (tokens are bearer; can't be revoked after issuance)
  - Mitigation: Short expiration times + revocation lists (CRLs)
- Key management complexity (each CLI has public/private keypair)
- Token size larger than simple API keys

**Benefits**:
- No central authentication service (fully decentralized)
- Delegation without server round-trip (tokens can be attenuated offline)
- Zero-trust by default (every request must present capability)
- Fine-grained permissions (RDF URIs allow arbitrary granularity)
- Audit trail built-in (delegation chain in token)

## Consequences

### Positive
- True zero-trust security (no ambient authority)
- Works offline (no need to contact central auth server)
- Natural delegation model (tokens can be attenuated and passed)
- Semantic permissions (capabilities are RDF URIs with formal meaning)
- Audit-friendly (delegation chain traces permission provenance)

### Negative
- Revocation requires distributed revocation lists (CRLs)
- Key management burden on CLI operators
- Token validation overhead on every request (~1ms)
- Potential for token theft (bearer tokens; possession = access)

### Mitigation Strategies

1. **Revocation**: Publish CRLs via RDF; cache with 5-minute TTL
2. **Key Management**: Integrate with OS keystores (macOS Keychain, Windows Credential Manager, Linux keyrings)
3. **Performance**: Cache verified tokens for 60 seconds
4. **Token Theft**:
   - Short expiration (1 hour default)
   - Bind tokens to TLS client certificate (MTLS)
   - Rate limiting per token

## Architecture Integration

```
┌─────────────────────────────────────────────────────┐
│ Capability Issuance Flow                            │
│                                                     │
│  1. Admin CLI                                       │
│     ↓                                               │
│  2. Discover "convert" capability via SPARQL        │
│     ↓                                               │
│  3. Fetch capability requirements from RDF          │
│     ↓                                               │
│  4. Generate signed JWT capability token            │
│     ↓                                               │
│  5. Distribute token to authorized CLIs             │
│     (via secure channel)                            │
│                                                     │
│ Invocation Flow                                     │
│                                                     │
│  1. CLI B receives capability token from admin      │
│     ↓                                               │
│  2. CLI B invokes CLI A's "convert" command         │
│     ↓                                               │
│  3. CLI B includes token in gRPC metadata           │
│     ↓                                               │
│  4. CLI A validates:                                │
│     - Signature (Ed25519 verify)                    │
│     - Expiration (exp > now)                        │
│     - Capabilities (required cap in token.cap)      │
│     - Constraints (SHACL validation)                │
│     - Revocation (check CRL)                        │
│     ↓                                               │
│  5. If valid: execute command                       │
│     If invalid: return 403 Forbidden                │
└─────────────────────────────────────────────────────┘
```

## Alternatives Considered

### 1. OAuth2 with Central Authorization Server
- **Pros**: Industry standard, excellent tooling
- **Cons**: Requires central server (violates decentralization goal)
- **Rejected**: Single point of failure

### 2. Macaroons (Contextual Caveats)
- **Pros**: Better revocation, contextual constraints
- **Cons**: Less mature tooling, harder to integrate with RDF
- **Rejected**: Immaturity; JWT more widely supported

### 3. Mutual TLS (MTLS) Only
- **Pros**: Strong authentication, no token management
- **Cons**: No fine-grained permissions, all-or-nothing access
- **Rejected**: Too coarse-grained; need capability-level control

### 4. Public Key Infrastructure (PKI) Certificates
- **Pros**: Standard X.509, built into TLS
- **Cons**: Certificate attributes not semantic (no RDF integration)
- **Rejected**: Can't express semantic capabilities in X.509

## Validation

Success metrics:
- Zero unauthorized access incidents
- Token validation latency p99 < 5ms
- Revocation propagation time p99 < 5 minutes
- 100% of invocations include valid capability tokens
- Audit trail completeness: 100% (all invocations logged)

## References

- [Capability-Based Security](https://en.wikipedia.org/wiki/Capability-based_security)
- [JWT RFC 7519](https://datatracker.ietf.org/doc/html/rfc7519)
- [Macaroons: Cookies with Contextual Caveats](https://research.google/pubs/pub41892/)
- [ZCAP-LD: Authorization Capabilities for Linked Data](https://w3c-ccg.github.io/zcap-spec/)
