# ADR-003: gRPC for Cross-CLI Invocation Transport

## Status
Accepted

## Context

After discovering capabilities via SPARQL, CLIs need a transport protocol to invoke each other's commands. Requirements:

- Type-safe serialization
- Bidirectional streaming
- Low latency (p99 < 100ms)
- Cross-platform support
- Supports authentication/authorization

## Decision

We will use **gRPC with Protocol Buffers** for CLI-to-CLI invocation.

## Rationale

### Protocol Comparison

| Protocol      | Latency | Streaming | Type Safety | Ecosystem | HTTP/2 |
|---------------|---------|-----------|-------------|-----------|--------|
| gRPC          | Low     | ✓         | Strong      | Excellent | ✓      |
| REST/JSON     | Moderate| ✗         | Weak        | Excellent | ~      |
| GraphQL       | Moderate| ~         | Moderate    | Good      | ✗      |
| WebSocket     | Low     | ✓         | Weak        | Good      | ✗      |
| QUIC          | Lowest  | ✓         | Custom      | Limited   | ✗      |

### Key Decision Factors

1. **Type Safety**: Protocol Buffers provide compile-time type checking
2. **Performance**: HTTP/2 multiplexing + binary serialization = low latency
3. **Streaming**: Bidirectional streaming for long-running CLI commands
4. **Code Generation**: Auto-generate Rust clients/servers from `.proto` files
5. **Ecosystem**: Mature Rust support via `tonic` crate
6. **Interoperability**: gRPC clients exist for 10+ languages

### Integration with RDF Schemas

RDF ontologies define **what** capabilities exist (semantic layer).
Protocol Buffers define **how** to invoke them (transport layer).

**Workflow**:
1. CLI publishes RDF ontology: `:ConvertCommand clicap:signature "ImageFile -> JSON"`
2. Auto-generate `.proto` from RDF types:
   ```protobuf
   service ConvertService {
     rpc Convert(ImageFileRequest) returns (JSONResponse);
   }
   ```
3. gRPC handles invocation; RDF handles discovery

### Trade-offs

**Costs**:
- Requires Protocol Buffers schema maintenance (in addition to RDF)
- HTTP/2 dependency (not all networks support)
- Binary protocol harder to debug than JSON (no human readability)

**Benefits**:
- 5-10x faster than REST/JSON
- Strongly typed contracts (prevents runtime errors)
- Streaming support for `tail -f`, `watch`, long-running commands
- Built-in authentication (TLS mutual auth)
- Automatic retry/timeout handling

## Architecture Integration

```
┌──────────────────────────────────────────────────┐
│ CLI A (Invoker)                                  │
│                                                  │
│  1. SPARQL Discovery                             │
│     ↓                                            │
│  2. Fetch RDF Ontology from CLI B                │
│     ↓                                            │
│  3. Validate Type Compatibility                  │
│     ↓                                            │
│  4. Generate gRPC Client from RDF Types          │
│     ↓                                            │
│  5. gRPC Call to CLI B                           │
│     │                                            │
│     └─────────────────────┐                      │
│                           ▼                      │
│                  ┌─────────────────────┐         │
│                  │ CLI B (Provider)    │         │
│                  │                     │         │
│                  │ gRPC Server         │         │
│                  │ (auto-generated     │         │
│                  │  from RDF ontology) │         │
│                  └─────────────────────┘         │
└──────────────────────────────────────────────────┘
```

## Consequences

### Positive
- Type-safe cross-CLI invocation with zero runtime type errors
- Low latency (p99 < 50ms for small payloads)
- Streaming support for interactive CLI commands
- Automatic code generation reduces boilerplate
- Built-in load balancing and service discovery (gRPC-LB)

### Negative
- Two schema systems to maintain (RDF + Protobuf)
- Debugging harder than JSON (binary format)
- HTTP/2 not universally supported (requires modern infrastructure)
- Learning curve for developers unfamiliar with gRPC

### Mitigation Strategies

1. **Schema Sync**: Automated RDF → Protobuf code generation
2. **Debugging**: Use `grpcurl` for manual testing; structured logging
3. **HTTP/2 Support**: Fallback to gRPC-Web (HTTP/1.1) for legacy networks
4. **Learning**: Provide example CLI implementations with gRPC

## Alternatives Considered

### 1. REST with JSON-LD
- **Pros**: Human-readable, RDF-native (JSON-LD)
- **Cons**: No streaming, slower than gRPC, weak type safety
- **Rejected**: Performance and type safety requirements not met

### 2. GraphQL
- **Pros**: Flexible querying, good for data fetching
- **Cons**: Not designed for RPC, no bidirectional streaming
- **Rejected**: RPC pattern better fit for CLI invocation

### 3. Apache Thrift
- **Pros**: Similar to gRPC, cross-language support
- **Cons**: Smaller ecosystem, less Rust support than gRPC
- **Rejected**: gRPC has better Rust integration (`tonic`)

### 4. Cap'n Proto
- **Pros**: Zero-copy serialization, extremely fast
- **Cons**: Immature ecosystem, no native RDF integration
- **Rejected**: Ecosystem too immature for production use

## Validation

Success metrics:
- gRPC invocation latency p99 < 100ms
- Zero type-related runtime errors (100% caught at compile-time)
- Streaming support for 95% of long-running CLI commands
- Cross-language compatibility: 5+ languages supported

## References

- [gRPC Core Concepts](https://grpc.io/docs/what-is-grpc/core-concepts/)
- [tonic - Rust gRPC Implementation](https://github.com/hyperium/tonic)
- [Protocol Buffers Language Guide](https://protobuf.dev/programming-guides/proto3/)
- [HTTP/2 Specification](https://httpwg.org/specs/rfc7540.html)
