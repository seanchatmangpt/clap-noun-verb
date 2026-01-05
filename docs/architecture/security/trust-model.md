# Trust Model for Federated Semantic Network

## Overview

In a decentralized federated network, **trust cannot be assumed**. The trust model defines how CLIs establish, verify, and revoke trust relationships without a central authority.

## Trust Architecture

### Web of Trust (Decentralized)

Instead of a centralized Certificate Authority (CA), we use a **Web of Trust** model where:
- CLIs directly sign each other's public keys
- Trust is transitive (with decay)
- No single root of trust

```
┌─────────────────────────────────────────────────┐
│ Web of Trust                                    │
│                                                 │
│         ┌───────┐                               │
│         │ CLI A │                               │
│         └───┬───┘                               │
│             │ signs                             │
│             ▼                                   │
│         ┌───────┐         ┌───────┐            │
│         │ CLI B │────────►│ CLI C │            │
│         └───┬───┘  signs  └───┬───┘            │
│             │                 │                 │
│             │ signs           │ signs           │
│             ▼                 ▼                 │
│         ┌───────┐         ┌───────┐            │
│         │ CLI D │         │ CLI E │            │
│         └───────┘         └───────┘            │
│                                                 │
│ Trust Path: A → B → C (2 hops)                 │
│ Trust Path: A → B → D (2 hops)                 │
│ Trust Path: A → B → C → E (3 hops, weaker)     │
└─────────────────────────────────────────────────┘
```

### Trust Anchors (Bootstrap Trust)

For initial bootstrapping, pre-established **trust anchors** provide starting points:

```turtle
@prefix clicap: <https://clicap.org/ontology#> .

# Trust anchor (root)
:CLICAPFoundation a clicap:TrustAnchor ;
    foaf:name "CLICAP Foundation"@en ;
    clicap:publicKey "ed25519:FOUNDATIONKEY..." ;
    clicap:establishedDate "2026-01-01"^^xsd:date ;
    clicap:trustLevel clicap:Root . # Maximum trust

# CLI signed by trust anchor
:VerifiedCLI a clicap:CLI ;
    clicap:publicKey "ed25519:CLIPUBKEY..." ;
    clicap:signedBy :CLICAPFoundation ;
    clicap:signature [
        clicap:signatureValue "..." ;
        clicap:signedAt "2026-01-05"^^xsd:date ;
        clicap:expiresAt "2027-01-05"^^xsd:date
    ] .
```

## Trust Establishment Protocol

### 1. Direct Trust (Signature Exchange)

**Scenario**: CLI A wants to trust CLI B directly.

**Protocol**:
```
CLI A                                    CLI B
  │                                        │
  │ 1. Request public key                 │
  ├───────────────────────────────────────►│
  │                                        │
  │ 2. Public key + proof of possession    │
  │◄───────────────────────────────────────┤
  │                                        │
  │ 3. Verify proof (sign challenge)       │
  │                                        │
  │ 4. Sign B's public key with A's key   │
  │    (creates trust certificate)         │
  │                                        │
  │ 5. Send trust certificate to B         │
  ├───────────────────────────────────────►│
  │                                        │
  │ 6. B reciprocates (optional)           │
  │◄───────────────────────────────────────┤
```

**Implementation**:
```rust
use ed25519_dalek::{Keypair, PublicKey, Signature, Signer, Verifier};

#[derive(Serialize, Deserialize)]
struct TrustCertificate {
    issuer: String,        // URI of signing CLI
    subject: String,       // URI of trusted CLI
    subject_public_key: PublicKey,
    trust_level: TrustLevel,
    issued_at: i64,
    expires_at: i64,
    signature: Signature,  // Issuer's signature over certificate
}

enum TrustLevel {
    Root,          // Trust anchor (100%)
    Direct,        // Directly verified (90%)
    OneHop,        // Trusted via one intermediary (70%)
    TwoHops,       // Trusted via two intermediaries (50%)
    Untrusted,     // Not trusted (0%)
}

impl TrustCertificate {
    fn issue(
        issuer_uri: &str,
        issuer_keypair: &Keypair,
        subject_uri: &str,
        subject_public_key: &PublicKey,
        trust_level: TrustLevel,
    ) -> Self {
        let issued_at = current_timestamp();
        let expires_at = issued_at + 365 * 24 * 3600; // 1 year

        let canonical = CanonicalTrustCertificate {
            issuer: issuer_uri.to_string(),
            subject: subject_uri.to_string(),
            subject_public_key: subject_public_key.as_bytes(),
            trust_level: &trust_level,
            issued_at,
            expires_at,
        };

        let signature = issuer_keypair.sign(&canonical.to_bytes());

        TrustCertificate {
            issuer: issuer_uri.to_string(),
            subject: subject_uri.to_string(),
            subject_public_key: *subject_public_key,
            trust_level,
            issued_at,
            expires_at,
            signature,
        }
    }

    fn verify(&self, issuer_public_key: &PublicKey) -> Result<(), Error> {
        // Verify signature
        let canonical = CanonicalTrustCertificate {
            issuer: &self.issuer,
            subject: &self.subject,
            subject_public_key: self.subject_public_key.as_bytes(),
            trust_level: &self.trust_level,
            issued_at: self.issued_at,
            expires_at: self.expires_at,
        };

        issuer_public_key.verify(&canonical.to_bytes(), &self.signature)?;

        // Check expiration
        if current_timestamp() > self.expires_at {
            return Err(Error::CertificateExpired);
        }

        Ok(())
    }
}
```

### 2. Transitive Trust (Trust Chains)

**Scenario**: CLI A trusts CLI B, CLI B trusts CLI C. Can A trust C?

**Trust Propagation**:
```rust
fn compute_trust_score(
    source: &str,
    target: &str,
    trust_graph: &TrustGraph,
) -> f64 {
    // Find shortest trust path from source to target
    let path = trust_graph.shortest_path(source, target)?;

    if path.is_empty() {
        return 0.0; // No trust path
    }

    // Compute trust score with decay
    let mut score = 1.0;

    for edge in path {
        score *= edge.trust_level.as_float();
        score *= 0.8; // Decay factor (20% reduction per hop)
    }

    score
}

impl TrustLevel {
    fn as_float(&self) -> f64 {
        match self {
            TrustLevel::Root => 1.0,
            TrustLevel::Direct => 0.9,
            TrustLevel::OneHop => 0.7,
            TrustLevel::TwoHops => 0.5,
            TrustLevel::Untrusted => 0.0,
        }
    }
}

// Example:
// A -> B (Direct: 0.9) -> C (Direct: 0.9)
// Trust score for A->C: 0.9 * 0.9 * 0.8 (decay) = 0.648

// Maximum hop limit to prevent infinite chains
const MAX_TRUST_HOPS: usize = 3;
```

### 3. Trust Revocation

**Scenario**: CLI A revokes trust in CLI B (e.g., B's key compromised).

**Revocation List**:
```turtle
:TrustRevocationList a clicap:TrustRevocationList ;
    dcterms:issuer <https://example.com/cli-a> ;
    dcterms:created "2026-01-05T15:00:00Z"^^xsd:dateTime ;
    clicap:revokedCertificate [
        clicap:subject <https://example.com/cli-b> ;
        clicap:revokedAt "2026-01-05T14:30:00Z"^^xsd:dateTime ;
        clicap:reason "Key compromise suspected"
    ] .
```

```rust
async fn check_trust_revocation(
    issuer_uri: &str,
    subject_uri: &str,
) -> Result<bool, Error> {
    // Fetch revocation list from issuer
    let revocation_list = fetch_trust_revocation_list(issuer_uri).await?;

    // Check if subject in revocation list
    let is_revoked = revocation_list.revoked_certificates
        .iter()
        .any(|cert| cert.subject == subject_uri);

    Ok(is_revoked)
}
```

## Trust Graph

### Graph Structure

```rust
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::algo::dijkstra;

struct TrustGraph {
    graph: DiGraph<String, TrustEdge>,
    cli_to_node: HashMap<String, NodeIndex>,
}

struct TrustEdge {
    certificate: TrustCertificate,
    trust_score: f64,
}

impl TrustGraph {
    fn add_cli(&mut self, cli_uri: String) -> NodeIndex {
        if let Some(&node) = self.cli_to_node.get(&cli_uri) {
            return node;
        }

        let node = self.graph.add_node(cli_uri.clone());
        self.cli_to_node.insert(cli_uri, node);
        node
    }

    fn add_trust_certificate(&mut self, cert: TrustCertificate) {
        let issuer_node = self.add_cli(cert.issuer.clone());
        let subject_node = self.add_cli(cert.subject.clone());

        let edge = TrustEdge {
            trust_score: cert.trust_level.as_float(),
            certificate: cert,
        };

        self.graph.add_edge(issuer_node, subject_node, edge);
    }

    fn shortest_trust_path(
        &self,
        source: &str,
        target: &str,
    ) -> Option<Vec<&TrustEdge>> {
        let source_node = *self.cli_to_node.get(source)?;
        let target_node = *self.cli_to_node.get(target)?;

        // Dijkstra with custom weight (inverse of trust score)
        let result = dijkstra(
            &self.graph,
            source_node,
            Some(target_node),
            |edge| 1.0 / edge.weight().trust_score, // Invert trust score for shortest path
        );

        // Reconstruct path
        // ... (path reconstruction logic)

        None // Placeholder
    }

    fn compute_trust_score(&self, source: &str, target: &str) -> f64 {
        if source == target {
            return 1.0; // Trust self
        }

        if let Some(path) = self.shortest_trust_path(source, target) {
            if path.len() > MAX_TRUST_HOPS {
                return 0.0; // Too many hops
            }

            let mut score = 1.0;
            for edge in path {
                score *= edge.trust_score;
                score *= 0.8; // Decay
            }

            score
        } else {
            0.0 // No path
        }
    }
}
```

### Trust Graph Synchronization

Trust graphs are synchronized via BFT consensus:

```rust
async fn synchronize_trust_graph(local_graph: &mut TrustGraph) -> Result<(), Error> {
    // 1. Fetch trust certificates from all known CLIs
    let certificates = fetch_all_trust_certificates().await?;

    // 2. Propose trust graph update via BFT consensus
    let update = TrustGraphUpdate {
        added_certificates: certificates,
        revoked_certificates: fetch_all_revocations().await?,
    };

    let consensus_result = bft_consensus_propose(update).await?;

    // 3. Apply consensus result to local graph
    for cert in consensus_result.added_certificates {
        local_graph.add_trust_certificate(cert);
    }

    for revocation in consensus_result.revoked_certificates {
        local_graph.remove_trust_certificate(&revocation.subject);
    }

    Ok(())
}
```

## Trust Policies

### Minimum Trust Threshold

```rust
struct TrustPolicy {
    min_trust_score: f64,
    max_trust_hops: usize,
    require_trust_anchor: bool,
}

impl TrustPolicy {
    fn default_strict() -> Self {
        Self {
            min_trust_score: 0.7,    // Require 70% trust
            max_trust_hops: 2,       // Maximum 2 intermediaries
            require_trust_anchor: true, // Must trace to root
        }
    }

    fn default_permissive() -> Self {
        Self {
            min_trust_score: 0.5,
            max_trust_hops: 3,
            require_trust_anchor: false,
        }
    }

    fn evaluate(
        &self,
        source: &str,
        target: &str,
        trust_graph: &TrustGraph,
    ) -> Result<(), TrustError> {
        let trust_score = trust_graph.compute_trust_score(source, target);

        if trust_score < self.min_trust_score {
            return Err(TrustError::InsufficientTrust {
                required: self.min_trust_score,
                actual: trust_score,
            });
        }

        if self.require_trust_anchor {
            if !trust_graph.has_path_to_trust_anchor(target)? {
                return Err(TrustError::NoTrustAnchorPath);
            }
        }

        Ok(())
    }
}
```

### Context-Dependent Trust

Different operations may require different trust levels:

```rust
enum Operation {
    ReadPublicData,      // Low trust required
    InvokeCommand,       // Medium trust required
    IssueCapability,     // High trust required
    ManageTrustGraph,    // Maximum trust required
}

impl Operation {
    fn required_trust_score(&self) -> f64 {
        match self {
            Operation::ReadPublicData => 0.3,
            Operation::InvokeCommand => 0.7,
            Operation::IssueCapability => 0.9,
            Operation::ManageTrustGraph => 1.0,
        }
    }
}

async fn authorize_operation(
    invoker: &str,
    operation: Operation,
    trust_graph: &TrustGraph,
) -> Result<(), Error> {
    let self_uri = get_self_uri();
    let trust_score = trust_graph.compute_trust_score(&self_uri, invoker);

    if trust_score < operation.required_trust_score() {
        return Err(Error::InsufficientTrust {
            operation: format!("{:?}", operation),
            required: operation.required_trust_score(),
            actual: trust_score,
        });
    }

    Ok(())
}
```

## Reputation System (Optional Enhancement)

Track CLI behavior over time to adjust trust dynamically:

```rust
#[derive(Serialize, Deserialize)]
struct ReputationScore {
    cli_uri: String,
    successful_invocations: u64,
    failed_invocations: u64,
    security_violations: u64,
    uptime_percentage: f64,
    last_updated: i64,
}

impl ReputationScore {
    fn compute_score(&self) -> f64 {
        let success_rate = self.successful_invocations as f64
            / (self.successful_invocations + self.failed_invocations) as f64;

        let security_penalty = 1.0 - (self.security_violations as f64 * 0.1).min(0.9);

        success_rate * security_penalty * self.uptime_percentage
    }

    fn adjust_trust_score(&self, base_trust: f64) -> f64 {
        let reputation = self.compute_score();

        // Blend base trust with reputation
        base_trust * 0.7 + reputation * 0.3
    }
}

// Example:
// Base trust (from certificates): 0.8
// Reputation score: 0.95 (excellent track record)
// Adjusted trust: 0.8 * 0.7 + 0.95 * 0.3 = 0.845
```

## Trust Bootstrapping

### Initial Trust Setup

For new CLI joining network:

1. **Option 1: Trust Anchor Signature**
   - CLI operator requests signature from trust anchor
   - Trust anchor verifies identity (out-of-band)
   - Trust anchor issues certificate

2. **Option 2: Proof of Work**
   - CLI solves computational puzzle
   - Puzzle difficulty makes Sybil attacks expensive
   - Automatic admission after proof

3. **Option 3: Vouching**
   - Existing trusted CLI vouches for new CLI
   - Voucher's reputation at stake
   - Transitive trust from voucher

```rust
async fn bootstrap_trust(new_cli_uri: &str) -> Result<TrustCertificate, Error> {
    // Option 1: Trust anchor
    if let Some(anchor_cert) = request_trust_anchor_signature(new_cli_uri).await? {
        return Ok(anchor_cert);
    }

    // Option 2: Proof of work
    let pow_cert = perform_proof_of_work(new_cli_uri).await?;
    if verify_proof_of_work(&pow_cert)? {
        return Ok(pow_cert);
    }

    // Option 3: Vouching
    let voucher = find_willing_voucher(new_cli_uri).await?;
    let vouched_cert = voucher.issue_certificate(new_cli_uri).await?;

    Ok(vouched_cert)
}
```

## Trust Visualization

Operators can visualize trust relationships:

```rust
fn visualize_trust_graph(graph: &TrustGraph) -> String {
    let mut dot = String::from("digraph TrustGraph {\n");

    for node in graph.graph.node_indices() {
        let cli_uri = &graph.graph[node];
        dot.push_str(&format!("  \"{}\";\n", cli_uri));
    }

    for edge in graph.graph.edge_indices() {
        let (source, target) = graph.graph.edge_endpoints(edge).unwrap();
        let trust_edge = &graph.graph[edge];

        let color = match trust_edge.trust_score {
            s if s > 0.8 => "green",
            s if s > 0.5 => "orange",
            _ => "red",
        };

        dot.push_str(&format!(
            "  \"{}\" -> \"{}\" [label=\"{:.2}\", color={}];\n",
            graph.graph[source],
            graph.graph[target],
            trust_edge.trust_score,
            color
        ));
    }

    dot.push_str("}\n");
    dot
}

// Render with Graphviz:
// dot -Tpng trust_graph.dot -o trust_graph.png
```

## References

- [Web of Trust](https://en.wikipedia.org/wiki/Web_of_trust)
- [PGP Trust Model](https://www.gnupg.org/gph/en/manual/x334.html)
- [Decentralized Trust Management](https://www.cs.virginia.edu/~evans/pubs/oakland2002/)
- [Trust Metrics](http://www.levien.com/thesis/compact.pdf)
