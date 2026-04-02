# CLI Capability Discovery Protocol

## Overview

The Discovery Protocol enables CLIs to find each other's capabilities in a federated semantic network using SPARQL federation. CLIs publish RDF ontologies describing their capabilities and query the network to discover compatible services.

## Protocol Layers

```
┌─────────────────────────────────────────────┐
│ Application Layer (CLI Tools)              │
│ - Issue SPARQL queries                     │
│ - Consume RDF ontology results             │
└─────────────┬───────────────────────────────┘
              │
┌─────────────▼───────────────────────────────┐
│ SPARQL Federation Layer                    │
│ - Distribute queries across endpoints      │
│ - Aggregate results                        │
│ - Handle network failures                  │
└─────────────┬───────────────────────────────┘
              │
┌─────────────▼───────────────────────────────┐
│ Local Discovery Layer                      │
│ - mDNS/DNS-SD (local network)              │
│ - Cache discovered endpoints               │
└─────────────┬───────────────────────────────┘
              │
┌─────────────▼───────────────────────────────┐
│ Global Discovery Layer                     │
│ - DHT (Kademlia) for wide-area discovery   │
│ - Bootstrap nodes                          │
└─────────────────────────────────────────────┘
```

## Discovery Phases

### Phase 1: Bootstrap Discovery

**Objective**: Find initial SPARQL endpoints in the network

**Mechanisms**:

1. **Local Network Discovery (mDNS/DNS-SD)**:
   ```
   Service Type: _clicap._tcp.local

   Example advertisement:
   image-converter._clicap._tcp.local IN PTR
       TXT "sparql=https://192.168.1.100:8080/sparql"
       TXT "version=1.2.3"
       TXT "capabilities=convert,resize,optimize"
   ```

2. **Global Network Discovery (DHT)**:
   ```
   DHT Key: SHA-256("clicap:bootstrap")
   DHT Value: List of bootstrap SPARQL endpoints

   Example:
   {
     "endpoints": [
       "https://registry.clicap.org/sparql",
       "https://eu.clicap.org/sparql",
       "https://us.clicap.org/sparql"
     ],
     "updated": "2026-01-05T10:30:00Z"
   }
   ```

3. **Well-Known URIs**:
   ```
   GET https://example.com/.well-known/clicap

   Response:
   {
     "sparql_endpoint": "https://example.com/clicap/sparql",
     "rdf_ontology": "https://example.com/clicap/ontology.ttl",
     "public_key": "ed25519:A1B2C3D4..."
   }
   ```

### Phase 2: Capability Advertisement

**Objective**: Publish CLI capabilities in RDF format

**Advertisement Components**:

1. **RDF Ontology Publication**:
   ```turtle
   # Published at: https://example.com/clicap/ontology.ttl

   @prefix clicap: <https://clicap.org/ontology#> .

   <https://example.com/tools/converter> a clicap:CLI ;
       clicap:version "1.2.3" ;
       clicap:endpoint <grpc://converter.example.com:50051> ;
       clicap:sparqlEndpoint <https://converter.example.com/sparql> ;
       clicap:publicKey "ed25519:ABC123..." ;
       clicap:hasCommand <https://example.com/tools/converter/commands/convert> .
   ```

2. **SPARQL Endpoint Setup**:
   ```rust
   // Serve RDF ontology via SPARQL endpoint
   use oxigraph::store::Store;
   use oxigraph::sparql::QueryResults;

   let store = Store::new()?;

   // Load CLI ontology
   store.load_graph(
       std::fs::read("ontology.ttl")?,
       GraphFormat::Turtle,
       &GraphName::DefaultGraph,
       None,
   )?;

   // Serve SPARQL endpoint
   let query = "SELECT ?command WHERE { ?cli clicap:hasCommand ?command }";
   let results = store.query(query)?;
   ```

3. **Automatic Updates**:
   - Publish ontology updates to DHT when capabilities change
   - Notify subscribers via WebSub protocol
   - Maintain version history for backward compatibility

### Phase 3: Semantic Query

**Objective**: Find CLIs matching semantic criteria

**Example Queries**:

1. **Find All Image Processing CLIs**:
   ```sparql
   PREFIX clicap: <https://clicap.org/ontology#>
   PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>

   SELECT ?cli ?command ?endpoint
   WHERE {
       # Federated query across all known endpoints
       SERVICE ?sparqlEndpoint {
           ?cli a clicap:CLI ;
                clicap:hasCommand ?command ;
                clicap:endpoint ?endpoint .

           ?command clicap:accepts ?inputType .

           # Semantic inference: find subclasses of ImageFile
           ?inputType rdfs:subClassOf* clicap:ImageFile .
       }
   }
   ```

2. **Find CLIs by Type Signature**:
   ```sparql
   PREFIX clicap: <https://clicap.org/ontology#>

   SELECT ?cli ?command
   WHERE {
       SERVICE ?sparqlEndpoint {
           ?cli clicap:hasCommand ?command .

           ?command clicap:typeSignature "ImageFile -> JSONData" .
       }
   }
   ```

3. **Find CLIs with Specific Capabilities**:
   ```sparql
   PREFIX clicap: <https://clicap.org/ontology#>

   SELECT ?cli ?capability
   WHERE {
       SERVICE ?sparqlEndpoint {
           ?cli clicap:hasCapability ?capability .

           ?capability clicap:grants <https://clicap.org/permissions/batch-process> .
       }
   }
   ```

### Phase 4: Federated Query Execution

**Objective**: Execute SPARQL queries across multiple endpoints efficiently

**Algorithm** (Simplified):

```rust
async fn execute_federated_query(query: &str) -> Result<Vec<Solution>, Error> {
    // 1. Discover all SPARQL endpoints
    let endpoints = discover_endpoints().await?;

    // 2. Parallelize query across endpoints
    let futures = endpoints.iter().map(|endpoint| {
        async move {
            let client = reqwest::Client::new();
            client.post(endpoint)
                .header("Content-Type", "application/sparql-query")
                .body(query.to_string())
                .send()
                .await?
                .json::<QueryResults>()
                .await
        }
    });

    // 3. Aggregate results
    let results = futures::future::join_all(futures).await;

    // 4. Merge and deduplicate
    let merged = results.into_iter()
        .filter_map(|r| r.ok())
        .flatten()
        .collect::<HashSet<_>>() // Deduplicate
        .into_iter()
        .collect();

    Ok(merged)
}
```

**Optimizations**:

1. **Query Rewriting**:
   - Detect which SERVICE clauses target which endpoints
   - Only send relevant sub-queries to each endpoint
   - Example: `SERVICE <https://example.com/sparql>` → send only to example.com

2. **Caching**:
   ```rust
   struct DiscoveryCache {
       cache: HashMap<String, (Vec<Solution>, Instant)>,
       ttl: Duration,
   }

   impl DiscoveryCache {
       fn get_or_query(&mut self, query: &str) -> Result<Vec<Solution>> {
           if let Some((cached, timestamp)) = self.cache.get(query) {
               if timestamp.elapsed() < self.ttl {
                   return Ok(cached.clone());
               }
           }

           let results = execute_federated_query(query)?;
           self.cache.insert(query.to_string(), (results.clone(), Instant::now()));
           Ok(results)
       }
   }
   ```

3. **Proximity Routing**:
   - Prefer geographically close endpoints (lower latency)
   - Use DHT routing table to estimate network distance
   - Fallback to farther endpoints if close ones timeout

## Network Topology

### Hierarchical Discovery

```
┌────────────────────────────────────────────────────┐
│ Global Bootstrap Nodes                             │
│ - registry.clicap.org                              │
│ - eu.clicap.org                                    │
│ - us.clicap.org                                    │
└───────────┬────────────────────────────────────────┘
            │
┌───────────▼────────────────────────────────────────┐
│ Regional Aggregators                               │
│ - Aggregate endpoints from organizations           │
│ - Provide regional SPARQL federation               │
└───────────┬────────────────────────────────────────┘
            │
┌───────────▼────────────────────────────────────────┐
│ Organizational CLI Clusters                        │
│ - company-a.clicap.org/sparql                      │
│ - company-b.clicap.org/sparql                      │
└───────────┬────────────────────────────────────────┘
            │
┌───────────▼────────────────────────────────────────┐
│ Individual CLI Instances                           │
│ - image-converter.company-a.com/sparql             │
│ - video-processor.company-b.com/sparql             │
└────────────────────────────────────────────────────┘
```

### Peer-to-Peer Discovery (DHT)

```
Using Kademlia DHT:

1. Each CLI has a Node ID (SHA-256 of public key)
2. CLIs maintain routing table to other nodes
3. Capability lookups via DHT key lookups:

   Key = SHA-256("capability:convert-image")
   Value = List of CLI endpoints providing this capability

4. DHT operations:
   - FIND_NODE(id): Find closest nodes to target ID
   - STORE(key, value): Store capability advertisement
   - FIND_VALUE(key): Retrieve capability providers
```

## Protocol State Machine

```
┌──────────┐
│  INIT    │
└────┬─────┘
     │
     │ 1. Bootstrap discovery
     ▼
┌──────────┐
│DISCOVERING│─────► (timeout) ──────┐
└────┬─────┘                         │
     │                               │
     │ 2. Found endpoints            │
     ▼                               │
┌──────────┐                         │
│ QUERYING │─────► (no results) ────┤
└────┬─────┘                         │
     │                               │
     │ 3. Received results           │
     ▼                               │
┌──────────┐                         │
│ VALIDATED│                         │
└────┬─────┘                         │
     │                               │
     │ 4. Selected provider          │
     ▼                               │
┌──────────┐                         │
│ CONNECTED│                         │
└────┬─────┘                         │
     │                               │
     │ 5. Connection failed          │
     └───────────────────────────────┤
                                     │
                                     ▼
                                ┌─────────┐
                                │ FAILED  │
                                └─────────┘
```

## Error Handling

### Failure Modes

| Failure              | Detection           | Recovery Strategy            |
|----------------------|---------------------|------------------------------|
| Endpoint unreachable | HTTP timeout        | Try next endpoint in list    |
| Invalid RDF          | Parse error         | Skip endpoint, log warning   |
| SPARQL syntax error  | Query error response| Fix query, retry             |
| No results           | Empty result set    | Broaden query, try bootstrap |
| Network partition    | All endpoints fail  | Fall back to cached results  |

### Fallback Strategies

```rust
async fn robust_discover(query: &str) -> Result<Vec<Solution>> {
    // 1. Try cached results first
    if let Some(cached) = cache.get(query) {
        return Ok(cached);
    }

    // 2. Try federated query
    match execute_federated_query(query).await {
        Ok(results) if !results.is_empty() => return Ok(results),
        _ => {}
    }

    // 3. Fall back to bootstrap nodes
    for bootstrap in BOOTSTRAP_NODES {
        if let Ok(results) = query_endpoint(bootstrap, query).await {
            return Ok(results);
        }
    }

    // 4. Final fallback: local-only discovery (mDNS)
    discover_local_mdns().await
}
```

## Security Considerations

1. **Endpoint Verification**:
   - Verify HTTPS certificates for SPARQL endpoints
   - Check public key signatures on RDF ontologies
   - Reject untrusted endpoints

2. **Query Injection Prevention**:
   - Parameterize SPARQL queries (use bindings)
   - Validate query syntax before execution
   - Rate limit queries per endpoint

3. **DOS Protection**:
   - Limit number of federated endpoints per query (max 100)
   - Timeout individual endpoint queries (5 seconds)
   - Implement exponential backoff for retries

## Performance Metrics

| Metric                     | Target      | Measurement Method        |
|----------------------------|-------------|---------------------------|
| Bootstrap discovery time   | p99 < 5s    | Prometheus histogram      |
| SPARQL query latency       | p99 < 100ms | Per-endpoint timing       |
| Federated query latency    | p99 < 500ms | End-to-end measurement    |
| Cache hit rate             | > 80%       | Cache hits / total queries|
| Discovery success rate     | > 99%       | Successful / total queries|

## Implementation Checklist

- [ ] Implement mDNS/DNS-SD publisher and resolver
- [ ] Integrate Kademlia DHT library (e.g., `libp2p-kad`)
- [ ] Implement SPARQL federation client (Oxigraph)
- [ ] Implement discovery cache with TTL
- [ ] Implement proximity-based endpoint selection
- [ ] Add query result validation (SHACL)
- [ ] Implement exponential backoff retry logic
- [ ] Add Prometheus metrics for discovery latency
- [ ] Implement endpoint health checking
- [ ] Add integration tests for federated queries
