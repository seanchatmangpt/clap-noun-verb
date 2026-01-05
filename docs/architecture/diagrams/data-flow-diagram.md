# Data Flow Diagram - Federated Semantic Network

## End-to-End Cross-CLI Invocation Flow

```
┌────────────────────────────────────────────────────────────────────────────┐
│ Phase 1: Capability Discovery                                             │
└────────────────────────────────────────────────────────────────────────────┘

Developer                 CLI A                    Discovery Layer        External
   │                        │                            │                Systems
   │                        │                            │                   │
   │ 1. Invoke command      │                            │                   │
   ├───────────────────────►│                            │                   │
   │  "convert image.png"   │                            │                   │
   │                        │                            │                   │
   │                        │ 2. SPARQL query            │                   │
   │                        │  "Find CLIs with           │                   │
   │                        │   convert capability"      │                   │
   │                        ├───────────────────────────►│                   │
   │                        │                            │                   │
   │                        │                            │ 3. Query DHT      │
   │                        │                            ├──────────────────►│
   │                        │                            │                   │
   │                        │                            │ 4. DHT results    │
   │                        │                            │◄──────────────────┤
   │                        │                            │  [CLI B, CLI C]   │
   │                        │                            │                   │
   │                        │ 5. Federated SPARQL        │                   │
   │                        │  query to CLI B, C         │                   │
   │                        │◄───────────────────────────┤                   │
   │                        │                            │                   │
   │                        │ 6. Fetch ontologies        │                   │
   │                        ├────────────────────────────┼──────────────────►│
   │                        │                            │  GET ontology.ttl │
   │                        │                            │                   │
   │                        │ 7. Ontologies (RDF)        │                   │
   │                        │◄───────────────────────────┼───────────────────┤
   │                        │                            │                   │

┌────────────────────────────────────────────────────────────────────────────┐
│ Phase 2: Type Validation & Client Generation                              │
└────────────────────────────────────────────────────────────────────────────┘

   │                        │                            │                   │
   │                        │ 8. Parse RDF ontologies    │                   │
   │                        │  Extract type signatures   │                   │
   │                        │                            │                   │
   │                        │ 9. Validate argument types │                   │
   │                        │  image.png : ImageFile ✓   │                   │
   │                        │                            │                   │
   │                        │ 10. Generate gRPC client   │                   │
   │                        │   from RDF types           │                   │
   │                        │                            │                   │
   │                        │ 11. Select provider        │                   │
   │                        │   (CLI B chosen)           │                   │
   │                        │                            │                   │

┌────────────────────────────────────────────────────────────────────────────┐
│ Phase 3: Security - Token & Signature Preparation                         │
└────────────────────────────────────────────────────────────────────────────┘

   │                        │                            │                   │
   │                        │ 12. Load capability token  │                   │
   │                        │   from local store         │                   │
   │                        │                            │                   │
   │                        │ 13. Verify token not       │                   │
   │                        │   expired (exp > now)      │                   │
   │                        │                            │                   │
   │                        │ 14. Construct canonical    │                   │
   │                        │   request representation   │                   │
   │                        │                            │                   │
   │                        │ 15. Sign request with      │                   │
   │                        │   Ed25519 private key      │                   │
   │                        │                            │                   │

┌────────────────────────────────────────────────────────────────────────────┐
│ Phase 4: Network Invocation (gRPC)                                        │
└────────────────────────────────────────────────────────────────────────────┘

   │                        │                            │                   │
CLI A                       Network                    CLI B             Storage
   │                        │                            │                   │
   │ 16. gRPC Request       │                            │                   │
   │  (TLS encrypted)       │                            │                   │
   ├────────────────────────┼───────────────────────────►│                   │
   │                        │                            │                   │
   │  Request contains:     │                            │                   │
   │  - Command metadata    │                            │                   │
   │  - Arguments (typed)   │                            │                   │
   │  - Capability token    │                            │                   │
   │  - Signature           │                            │                   │
   │                        │                            │                   │

┌────────────────────────────────────────────────────────────────────────────┐
│ Phase 5: Server-Side Verification                                         │
└────────────────────────────────────────────────────────────────────────────┘

   │                        │                            │                   │
   │                        │                            │ 17. TLS handshake │
   │                        │                            │   (mutual auth)   │
   │                        │                            │                   │
   │                        │                            │ 18. Extract client│
   │                        │                            │   certificate     │
   │                        │                            │                   │
   │                        │                            │ 19. Verify request│
   │                        │                            │   signature       │
   │                        │                            │   (Ed25519)       │
   │                        │                            │                   │
   │                        │                            │ 20. Decode JWT    │
   │                        │                            │   capability token│
   │                        │                            │                   │
   │                        │                            │ 21. Verify token  │
   │                        │                            │   signature       │
   │                        │                            │                   │
   │                        │                            │ 22. Check token   │
   │                        │                            │   expiration      │
   │                        │                            │                   │
   │                        │                            │ 23. Verify cert   │
   │                        │                            │   binding (TLS)   │
   │                        │                            │                   │
   │                        │                            │ 24. Check CRL     │
   │                        │                            ├──────────────────►│
   │                        │                            │  (Query revocation│
   │                        │                            │   list)           │
   │                        │                            │◄──────────────────┤
   │                        │                            │  Token valid ✓    │
   │                        │                            │                   │
   │                        │                            │ 25. SHACL type    │
   │                        │                            │   validation      │
   │                        │                            │                   │

┌────────────────────────────────────────────────────────────────────────────┐
│ Phase 6: Command Execution                                                │
└────────────────────────────────────────────────────────────────────────────┘

   │                        │                            │                   │
   │                        │                            │ 26. Execute       │
   │                        │                            │   convert command │
   │                        │                            │                   │
   │                        │                            │ 27. Process image │
   │                        │                            │   (business logic)│
   │                        │                            │                   │
   │                        │                            │ 28. Generate      │
   │                        │                            │   result          │
   │                        │                            │                   │

┌────────────────────────────────────────────────────────────────────────────┐
│ Phase 7: Response & Audit                                                 │
└────────────────────────────────────────────────────────────────────────────┘

   │                        │                            │                   │
   │                        │                            │ 29. Validate      │
   │                        │                            │   result type     │
   │                        │                            │   (SHACL)         │
   │                        │                            │                   │
   │                        │                            │ 30. Sign response │
   │                        │                            │   (Ed25519)       │
   │                        │                            │                   │
   │                        │                            │ 31. Append audit  │
   │                        │                            │   log entry       │
   │                        │                            ├──────────────────►│
   │                        │                            │  (Immutable log)  │
   │                        │                            │                   │
   │                        │  32. gRPC Response         │                   │
   │◄───────────────────────┼────────────────────────────┤                   │
   │  (TLS encrypted)       │                            │                   │
   │                        │                            │                   │
   │  Response contains:    │                            │                   │
   │  - Result (typed)      │                            │                   │
   │  - Status              │                            │                   │
   │  - Signature           │                            │                   │
   │                        │                            │                   │

┌────────────────────────────────────────────────────────────────────────────┐
│ Phase 8: Client-Side Result Validation                                    │
└────────────────────────────────────────────────────────────────────────────┘

   │                        │                            │                   │
   │ 33. Verify response    │                            │                   │
   │   signature (Ed25519)  │                            │                   │
   │                        │                            │                   │
   │ 34. Validate result    │                            │                   │
   │   type matches         │                            │                   │
   │   expected output      │                            │                   │
   │                        │                            │                   │
   │ 35. Return result to   │                            │                   │
   │   developer            │                            │                   │
   │◄───────────────────────┤                            │                   │
   │                        │                            │                   │

Developer
   │
   │ 36. Receive converted image
   ▼
```

## Data Transformations

### Data States Through Pipeline

| Phase | Data Form                      | Schema/Format           | Validation             |
|-------|--------------------------------|-------------------------|------------------------|
| 1     | User command string            | Plain text              | CLI parsing            |
| 2     | Structured arguments           | Rust types              | Type inference         |
| 3     | RDF ontology queries           | SPARQL                  | SPARQL syntax          |
| 4     | RDF ontology results           | Turtle/RDF-XML          | RDF parsing            |
| 5     | Typed gRPC request             | Protocol Buffers        | Protobuf schema        |
| 6     | Capability token (JWT)         | JSON Web Token          | JWT signature          |
| 7     | Signed request (canonical)     | JSON (deterministic)    | Ed25519 signature      |
| 8     | Encrypted network payload      | TLS 1.3                 | TLS verification       |
| 9     | Server-side validated request  | Rust types              | SHACL validation       |
| 10    | Business logic execution       | Domain types            | Business rules         |
| 11    | Result (typed)                 | Rust types              | Type constraints       |
| 12    | Signed response                | Protocol Buffers + sig  | Ed25519 signature      |
| 13    | Verified result                | Rust types              | Type validation        |

### Data Security Transformations

```
Plain Arguments
    ↓
[Type Checking: Rust compiler]
    ↓
Typed Arguments
    ↓
[Serialization: Protocol Buffers]
    ↓
Binary Payload
    ↓
[Signing: Ed25519]
    ↓
Signed Payload (tamper-evident)
    ↓
[Encryption: TLS 1.3]
    ↓
Encrypted Payload (confidential)
    ↓
[Network Transmission]
    ↓
[Decryption: TLS 1.3]
    ↓
Signed Payload
    ↓
[Signature Verification: Ed25519]
    ↓
Verified Payload
    ↓
[Deserialization: Protocol Buffers]
    ↓
Typed Arguments
    ↓
[SHACL Validation]
    ↓
Validated Typed Arguments
```

## Data Stores

### Persistent Data

| Data Store         | Data Stored                    | Technology      | Access Pattern    |
|--------------------|--------------------------------|-----------------|-------------------|
| Triple Store       | RDF ontology, capabilities     | Oxigraph        | SPARQL queries    |
| Capability Cache   | JWT tokens, trust certificates | Redis/In-memory | Key-value lookups |
| Trust Graph DB     | Trust relationships, scores    | RocksDB         | Graph queries     |
| Audit Log          | Invocation history, events     | Append-only log | Sequential reads  |
| Merkle DAG         | Consensus state history        | IPFS/Custom     | Content-addressed |
| Revocation Lists   | Revoked tokens, certificates   | RDF (published) | Periodic fetch    |

### Data Flow to Storage

```
Invocation Request
    ↓
Audit Log Writer ────────► [Audit Log Storage]
                            (Immutable, append-only)
    ↓
Capability Verifier ──────► [Revocation List Cache]
                            (5-minute TTL)
    ↓
Type Validator ───────────► [Triple Store]
                            (RDF ontology lookup)
    ↓
Trust Evaluator ──────────► [Trust Graph DB]
                            (Compute trust score)
    ↓
Command Executor
```

## Performance Optimizations in Data Flow

### Caching Layers

```
Developer Request
    ↓
┌──────────────────────┐
│ Local Ontology Cache │  ← Avoid SPARQL federation
│ (5-minute TTL)       │
└──────────────────────┘
    ↓ (cache miss)
┌──────────────────────┐
│ Proximity-Routed     │  ← Query geographically close
│ SPARQL Endpoints     │
└──────────────────────┘
    ↓
┌──────────────────────┐
│ Type Validation      │  ← Pre-compiled SHACL shapes
│ (compiled shapes)    │
└──────────────────────┘
    ↓
┌──────────────────────┐
│ gRPC Connection Pool │  ← Reuse connections
└──────────────────────┘
```

### Parallel Processing

```
Discovery Phase:
    ├─► DHT Query (async)
    ├─► mDNS Query (async)
    └─► Bootstrap Nodes Query (async)
            ↓
    Wait for first N responses

Validation Phase:
    ├─► Signature Verification (async)
    ├─► Token Validation (async)
    ├─► CRL Check (async, cached)
    └─► SHACL Type Validation (async)
            ↓
    Fail-fast on first error
```

## Data Flow Metrics

| Metric                      | Target        | Measurement Point              |
|-----------------------------|---------------|--------------------------------|
| Discovery latency           | p99 < 500ms   | SPARQL query completion        |
| Type validation latency     | p99 < 10ms    | SHACL validation duration      |
| Signature verification      | p99 < 5ms     | Ed25519 verify duration        |
| Total invocation latency    | p99 < 1s      | Request sent → Response recv   |
| Network serialization size  | < 10KB/request| Protobuf message size          |
| Audit log throughput        | 10K writes/sec| Log append operations          |

## References

- [Data Flow Diagrams](https://en.wikipedia.org/wiki/Data-flow_diagram)
- [gRPC Data Flow](https://grpc.io/docs/what-is-grpc/core-concepts/)
- [TLS 1.3 Handshake](https://tls13.xargs.org/)
