# Conflict Resolution Protocol

## Overview

In a federated semantic network, conflicts arise when:
1. **Naming Collisions**: Multiple CLIs claim the same name/URI
2. **Capability Conflicts**: Two CLIs provide incompatible versions of same capability
3. **Type Conflicts**: Incompatible type definitions for same concept
4. **Consensus Conflicts**: Byzantine nodes propose conflicting updates

## Conflict Types & Resolution Strategies

### 1. Naming Collisions

**Problem**: Two CLIs both claim URI `cli://example.com/converter`

#### Resolution Strategy: Hierarchical Namespace + DNS-Based Authority

**Principle**: URIs use DNS-based authority; DNS owner has ultimate authority.

```turtle
# CLI A claims:
<cli://example.com/converter> a clicap:CLI ;
    clicap:publicKey "ed25519:KEY_A" .

# CLI B claims (malicious):
<cli://example.com/converter> a clicap:CLI ;
    clicap:publicKey "ed25519:KEY_B" .

# Resolution:
# 1. Query DNS TXT record for example.com
# 2. Verify public key against DNS record
# 3. Accept only CLI with matching public key
```

**DNS TXT Record**:
```
example.com.  IN  TXT  "clicap-pubkey=ed25519:KEY_A"
```

**Verification Algorithm**:
```rust
async fn resolve_naming_conflict(
    conflicting_claims: Vec<CLIAdvertisement>,
) -> Result<CLIAdvertisement, Error> {
    // Extract domain from CLI URI
    let uri = &conflicting_claims[0].uri;
    let domain = extract_domain(uri)?;

    // Fetch DNS TXT record
    let dns_pubkey = query_dns_txt_record(&domain, "clicap-pubkey").await?;

    // Find claim with matching public key
    for claim in conflicting_claims {
        if claim.public_key == dns_pubkey {
            return Ok(claim);
        }
    }

    Err(Error::NamingConflictUnresolved {
        uri: uri.clone(),
        candidates: conflicting_claims,
    })
}
```

**Fallback: Content-Addressed URIs**:
If DNS resolution fails, use content-addressed URIs:
```
cli://sha256:abc123.../converter
```
where `abc123...` is SHA-256 hash of CLI's public key.

### 2. Capability Conflicts

**Problem**: Two CLIs provide "image conversion" but with incompatible semantics.

#### Resolution Strategy: Semantic Versioning + Explicit Selection

**Principle**: Clients explicitly select capability version; system never auto-resolves ambiguity.

```turtle
# CLI A: Version 1.0.0
<https://example.com/cli-a> a clicap:CLI ;
    clicap:hasCommand [
        clicap:commandName "convert" ;
        clicap:semanticVersion "1.0.0" ;
        clicap:typeSignature "ImageFile -> ImageFile" ;
        clicap:outputFormat clicap:LosslessCompression
    ] .

# CLI B: Version 2.0.0 (breaking change)
<https://example.com/cli-b> a clicap:CLI ;
    clicap:hasCommand [
        clicap:commandName "convert" ;
        clicap:semanticVersion "2.0.0" ;
        clicap:typeSignature "ImageFile -> (ImageFile, Metadata)" ;
        clicap:outputFormat clicap:LossyCompression
    ] .
```

**Client-Side Selection**:
```rust
async fn select_capability(
    capability_name: &str,
    version_constraint: &VersionReq,
) -> Result<CLICapability, Error> {
    // Query for all providers of capability
    let providers = discover_capability_providers(capability_name).await?;

    // Filter by version constraint (SemVer)
    let compatible: Vec<_> = providers.into_iter()
        .filter(|p| version_constraint.matches(&p.version))
        .collect();

    match compatible.len() {
        0 => Err(Error::NoCompatibleProvider {
            capability: capability_name.to_string(),
            constraint: version_constraint.clone(),
        }),

        1 => Ok(compatible[0].clone()),

        _ => {
            // Multiple compatible providers: require explicit selection
            Err(Error::AmbiguousCapability {
                capability: capability_name.to_string(),
                candidates: compatible,
            })
        }
    }
}

// Usage
let capability = select_capability("convert", &VersionReq::parse("^1.0.0")?).await?;
```

**User Intervention Required**:
```bash
# Ambiguous: multiple providers match constraint
$ clap-noun-verb invoke convert --input image.png

Error: Multiple providers found for 'convert ^1.0.0':
  1. cli://example.com/converter-a (version 1.2.3)
  2. cli://example.com/converter-b (version 1.5.0)

Please specify explicitly:
  $ clap-noun-verb invoke --cli cli://example.com/converter-a convert --input image.png
```

### 3. Type Conflicts

**Problem**: Two ontologies define `ImageFile` differently.

#### Resolution Strategy: Namespace Isolation + Explicit Mappings

**Principle**: Each CLI has isolated namespace; clients explicitly map between types.

```turtle
# CLI A's ontology
<https://example.com/cli-a/types#ImageFile> a owl:Class ;
    clicap:hasProperty [
        clicap:propertyName "format" ;
        clicap:allowedValues ("PNG", "JPEG")
    ] .

# CLI B's ontology
<https://example.com/cli-b/types#ImageFile> a owl:Class ;
    clicap:hasProperty [
        clicap:propertyName "format" ;
        clicap:allowedValues ("WebP", "AVIF")  # Different formats!
    ] .
```

**Type Mapping**:
```turtle
# Client defines explicit mapping
:TypeMapping a clicap:TypeMapping ;
    clicap:sourceType <https://example.com/cli-a/types#ImageFile> ;
    clicap:targetType <https://example.com/cli-b/types#ImageFile> ;
    clicap:transformation [
        a clicap:Transformation ;
        clicap:transforms "format" ;
        clicap:mapping [
            clicap:from "PNG" ;
            clicap:to "WebP"
        ] ;
        clicap:mapping [
            clicap:from "JPEG" ;
            clicap:to "WebP"
        ]
    ] .
```

```rust
fn transform_type(
    value: &Value,
    source_type: &str,
    target_type: &str,
    mappings: &[TypeMapping],
) -> Result<Value, Error> {
    // Find applicable mapping
    let mapping = mappings.iter()
        .find(|m| m.source_type == source_type && m.target_type == target_type)
        .ok_or(Error::NoTypeMappingFound)?;

    // Apply transformation
    let mut transformed = value.clone();

    for transform in &mapping.transformations {
        match transform {
            Transformation::RenameField { from, to } => {
                transformed.rename_field(from, to)?;
            }
            Transformation::MapValue { field, mapping } => {
                let old_value = transformed.get_field(field)?;
                let new_value = mapping.get(old_value)
                    .ok_or(Error::UnmappedValue {
                        field: field.clone(),
                        value: old_value.clone(),
                    })?;
                transformed.set_field(field, new_value.clone())?;
            }
        }
    }

    Ok(transformed)
}
```

### 4. Consensus Conflicts (Byzantine Failures)

**Problem**: Byzantine nodes propose conflicting state updates.

#### Resolution Strategy: BFT Consensus (HotStuff)

**Principle**: Require 2f+1 votes for commit; honest majority always agrees.

```rust
async fn resolve_consensus_conflict(
    proposals: Vec<StateUpdate>,
) -> Result<StateUpdate, Error> {
    // HotStuff phases: Prepare, PreCommit, Commit, Decide

    let mut votes = HashMap::new();

    // Collect votes from all nodes
    for node in all_nodes() {
        let vote = node.vote_on_proposals(&proposals).await?;
        *votes.entry(vote.proposal_hash).or_insert(0) += 1;
    }

    // Find proposal with 2f+1 votes (quorum)
    for (proposal_hash, vote_count) in votes {
        if vote_count >= 2 * MAX_BYZANTINE_NODES + 1 {
            let winning_proposal = proposals.iter()
                .find(|p| hash(p) == proposal_hash)
                .ok_or(Error::ProposalNotFound)?;

            return Ok(winning_proposal.clone());
        }
    }

    // No quorum reached
    Err(Error::ConsensusFailure {
        proposals,
        votes,
    })
}
```

**Safety Guarantee**:
If f < n/3 nodes are Byzantine, honest nodes never commit different values.

**Proof (Sketch)**:
- Quorum = 2f+1
- Two quorums overlap by ≥ f+1 nodes
- At least 1 honest node in overlap
- Honest nodes vote consistently
- ∴ Cannot have two conflicting quorums □

### 5. Schema Evolution Conflicts

**Problem**: Old clients vs. new schemas (forward/backward compatibility).

#### Resolution Strategy: Schema Versioning + Compatibility Checks

```turtle
# Version 1.0.0
:ImageFileV1 a owl:Class ;
    owl:versionInfo "1.0.0" ;
    clicap:hasProperty [
        clicap:propertyName "data" ;
        clicap:required true
    ] .

# Version 2.0.0 (breaking: added required field)
:ImageFileV2 a owl:Class ;
    owl:versionInfo "2.0.0" ;
    owl:backwardCompatibleWith :ImageFileV1 ;
    clicap:hasProperty [
        clicap:propertyName "data" ;
        clicap:required true
    ] ;
    clicap:hasProperty [
        clicap:propertyName "metadata" ;
        clicap:required false ;  # Optional for backward compat
        clicap:defaultValue "{}"
    ] .
```

```rust
fn check_schema_compatibility(
    client_version: &Version,
    server_version: &Version,
) -> CompatibilityResult {
    match (client_version.major, server_version.major) {
        (c, s) if c == s => {
            // Same major version: backward compatible
            CompatibilityResult::Compatible
        }

        (c, s) if c < s => {
            // Client older: check backward compatibility
            if server_has_backward_compat_for(client_version) {
                CompatibilityResult::CompatibleWithUpgrade {
                    suggested_version: server_version.clone(),
                }
            } else {
                CompatibilityResult::Incompatible {
                    reason: "Server version requires client upgrade".to_string(),
                }
            }
        }

        (c, s) if c > s => {
            // Client newer: check forward compatibility
            if client_has_forward_compat_for(server_version) {
                CompatibilityResult::CompatibleWithDowngrade
            } else {
                CompatibilityResult::Incompatible {
                    reason: "Client version too new for server".to_string(),
                }
            }
        }

        _ => unreachable!(),
    }
}
```

## Conflict Detection

### Proactive Conflict Detection

```rust
#[derive(Debug)]
struct ConflictReport {
    conflict_type: ConflictType,
    severity: Severity,
    participants: Vec<String>,
    detected_at: i64,
    resolution_strategy: ResolutionStrategy,
}

enum ConflictType {
    NamingCollision,
    CapabilityConflict,
    TypeConflict,
    ConsensusConflict,
    SchemaEvolution,
}

enum Severity {
    Low,      // Warning, no action required
    Medium,   // Manual intervention recommended
    High,     // Automatic resolution attempted
    Critical, // Service degradation possible
}

async fn detect_conflicts(network_state: &NetworkState) -> Vec<ConflictReport> {
    let mut conflicts = Vec::new();

    // Check for naming collisions
    for (uri, claims) in group_by_uri(&network_state.cli_advertisements) {
        if claims.len() > 1 {
            conflicts.push(ConflictReport {
                conflict_type: ConflictType::NamingCollision,
                severity: Severity::High,
                participants: claims.iter().map(|c| c.uri.clone()).collect(),
                detected_at: current_timestamp(),
                resolution_strategy: ResolutionStrategy::DNSVerification,
            });
        }
    }

    // Check for capability conflicts
    for (cap_name, providers) in group_by_capability(&network_state.capabilities) {
        let versions: HashSet<_> = providers.iter().map(|p| &p.version).collect();

        if versions.len() > 1 {
            // Check for breaking changes
            let has_breaking_change = versions.iter()
                .any(|v| providers.iter().any(|p| p.version.major != v.major));

            if has_breaking_change {
                conflicts.push(ConflictReport {
                    conflict_type: ConflictType::CapabilityConflict,
                    severity: Severity::Medium,
                    participants: providers.iter().map(|p| p.uri.clone()).collect(),
                    detected_at: current_timestamp(),
                    resolution_strategy: ResolutionStrategy::ExplicitSelection,
                });
            }
        }
    }

    // Check for type conflicts
    // ... (similar pattern)

    conflicts
}
```

### Reactive Conflict Resolution

```rust
async fn handle_conflict(conflict: ConflictReport) -> Result<Resolution, Error> {
    match conflict.resolution_strategy {
        ResolutionStrategy::DNSVerification => {
            let resolution = resolve_naming_conflict(&conflict.participants).await?;
            Ok(Resolution::Automatic(resolution))
        }

        ResolutionStrategy::ExplicitSelection => {
            // Require user intervention
            Ok(Resolution::RequiresUserInput {
                message: format!(
                    "Multiple providers for capability: {:?}. Please select explicitly.",
                    conflict.participants
                ),
                options: conflict.participants,
            })
        }

        ResolutionStrategy::BFTConsensus => {
            let resolution = resolve_consensus_conflict(&conflict.participants).await?;
            Ok(Resolution::Automatic(resolution))
        }

        ResolutionStrategy::TypeMapping => {
            // Attempt automatic type mapping
            match attempt_automatic_type_mapping(&conflict).await {
                Ok(mapping) => Ok(Resolution::Automatic(mapping)),
                Err(_) => Ok(Resolution::RequiresUserInput {
                    message: "Type conflict detected. Please provide type mapping.".to_string(),
                    options: conflict.participants,
                }),
            }
        }
    }
}
```

## Conflict Metrics

Track conflict frequency for network health monitoring:

```rust
use prometheus::{Counter, Histogram};

lazy_static! {
    static ref CONFLICTS_TOTAL: Counter = Counter::new(
        "conflicts_total",
        "Total number of conflicts detected"
    ).unwrap();

    static ref CONFLICT_RESOLUTION_TIME: Histogram = Histogram::new(
        "conflict_resolution_seconds",
        "Time to resolve conflicts"
    ).unwrap();
}

async fn monitor_conflict_resolution(conflict: ConflictReport) {
    CONFLICTS_TOTAL.inc();

    let timer = CONFLICT_RESOLUTION_TIME.start_timer();

    match handle_conflict(conflict).await {
        Ok(Resolution::Automatic(_)) => {
            timer.observe_duration();
            log::info!("Conflict resolved automatically");
        }

        Ok(Resolution::RequiresUserInput { .. }) => {
            timer.stop_and_discard();
            log::warn!("Conflict requires user intervention");
        }

        Err(e) => {
            timer.stop_and_discard();
            log::error!("Conflict resolution failed: {:?}", e);
        }
    }
}
```

## Conflict Prevention

### Best Practices

1. **Use DNS-Based URIs**: Prevents naming collisions
   ```
   ✓ cli://example.com/converter
   ✗ cli://converter (ambiguous)
   ```

2. **Semantic Versioning**: Clearly indicate breaking changes
   ```
   1.0.0 → 1.1.0  (backward compatible)
   1.0.0 → 2.0.0  (breaking change)
   ```

3. **Namespace Isolation**: Use unique prefixes for types
   ```turtle
   <https://example.com/cli-a/types#ImageFile>  # Namespaced
   vs.
   <clicap:ImageFile>  # Shared namespace (conflict risk)
   ```

4. **Explicit Versioning**: Always specify version constraints
   ```rust
   // ✓ Explicit
   select_capability("convert", &VersionReq::parse("^1.0.0")?)

   // ✗ Implicit (latest)
   select_capability("convert", &VersionReq::STAR)  // Dangerous
   ```

5. **Schema Evolution Discipline**:
   - Never remove required fields
   - Always add optional fields with defaults
   - Document breaking changes in `CHANGELOG.md`

## Implementation Checklist

- [ ] Implement DNS TXT record verification for naming conflicts
- [ ] Implement semantic versioning parser and comparator
- [ ] Implement type mapping DSL and transformation engine
- [ ] Implement BFT consensus conflict resolution
- [ ] Implement schema compatibility checker
- [ ] Implement proactive conflict detection
- [ ] Add conflict resolution metrics (Prometheus)
- [ ] Add conflict resolution UI for manual intervention
- [ ] Add conflict prevention linter (checks ontologies)
- [ ] Document conflict resolution procedures for operators

## References

- [Semantic Versioning 2.0.0](https://semver.org/)
- [DNS-Based Service Discovery](https://datatracker.ietf.org/doc/html/rfc6763)
- [Byzantine Consensus](https://pmg.csail.mit.edu/papers/osdi99.pdf)
- [Schema Evolution in Databases](https://en.wikipedia.org/wiki/Schema_evolution)
