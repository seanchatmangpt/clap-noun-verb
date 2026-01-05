# C4 Model: Context Diagram

## System Context - Federated Semantic Network

```
                         ┌─────────────────────────────────────────────┐
                         │      External Systems & Actors              │
                         └─────────────────────────────────────────────┘

           ┌──────────┐                                      ┌──────────┐
           │Developer │                                      │ Operator │
           │          │                                      │          │
           └────┬─────┘                                      └────┬─────┘
                │                                                 │
                │ Invokes CLIs                                   │ Manages
                │ Discovers capabilities                         │ Monitors
                │                                                 │
                ▼                                                 ▼
    ┌───────────────────────────────────────────────────────────────────┐
    │                                                                   │
    │          Federated Semantic Network for CLI Composition          │
    │                                                                   │
    │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐             │
    │  │   CLI A     │  │   CLI B     │  │   CLI C     │             │
    │  │             │  │             │  │             │             │
    │  │ RDF Endpoint│  │ RDF Endpoint│  │ RDF Endpoint│             │
    │  │ gRPC Server │  │ gRPC Server │  │ gRPC Server │             │
    │  └─────────────┘  └─────────────┘  └─────────────┘             │
    │                                                                   │
    │  ┌──────────────────────────────────────────────────────┐        │
    │  │ SPARQL Federation Layer                              │        │
    │  │ - Distributed query execution                        │        │
    │  │ - Capability discovery                               │        │
    │  └──────────────────────────────────────────────────────┘        │
    │                                                                   │
    │  ┌──────────────────────────────────────────────────────┐        │
    │  │ BFT Consensus Layer (HotStuff)                       │        │
    │  │ - Capability registry updates                        │        │
    │  │ - Trust certificate management                       │        │
    │  └──────────────────────────────────────────────────────┘        │
    │                                                                   │
    │  ┌──────────────────────────────────────────────────────┐        │
    │  │ Security & Trust Management                          │        │
    │  │ - Capability-based access control                    │        │
    │  │ - Cryptographic verification (Ed25519)               │        │
    │  │ - Trust graph synchronization                        │        │
    │  └──────────────────────────────────────────────────────┘        │
    │                                                                   │
    └───────────────────────────────────────────────────────────────────┘
                │                              │                  │
                │ Uses                         │ Publishes to     │ Queries
                ▼                              ▼                  ▼
    ┌────────────────────┐       ┌────────────────────┐  ┌────────────────┐
    │ DNS Infrastructure │       │ DHT Network        │  │ SPARQL         │
    │                    │       │ (Kademlia)         │  │ Endpoints      │
    │ - TXT records for  │       │ - Bootstrap nodes  │  │                │
    │   public keys      │       │ - Peer discovery   │  │ - Triple stores│
    │ - CLI verification │       │ - Capability index │  │ - RDF ontology │
    └────────────────────┘       └────────────────────┘  └────────────────┘
```

## Relationships

### Primary Actors

1. **Developer**
   - Uses federated network to discover and invoke CLI capabilities
   - Composes workflows from multiple CLIs
   - Trusts network to provide type-safe, secure invocations

2. **Operator**
   - Manages CLI nodes in the federation
   - Monitors network health and security
   - Responds to conflicts and security incidents

### System Boundaries

**Inside the System**:
- Individual CLI tools with RDF endpoints
- SPARQL federation layer
- BFT consensus mechanism
- Security and trust management

**Outside the System**:
- DNS infrastructure (relied upon but not controlled)
- DHT network (decentralized, no single owner)
- External SPARQL endpoints (operated by third parties)

### Key Integration Points

1. **DNS Infrastructure**
   - System queries DNS TXT records to verify CLI ownership
   - DNS provides root of trust for naming

2. **DHT Network**
   - System uses DHT for peer discovery
   - System publishes capability indices to DHT
   - DHT provides decentralized registry

3. **SPARQL Endpoints**
   - System publishes RDF ontologies to endpoints
   - System queries endpoints for capability discovery
   - Endpoints provide semantic query interface

## Context Diagram Notes

**Technology Choices**:
- **RDF/OWL**: Semantic capability description
- **SPARQL**: Federated query execution
- **gRPC**: High-performance invocation protocol
- **Ed25519**: Cryptographic signatures
- **HotStuff**: Byzantine fault tolerance

**Quality Attributes**:
- **Availability**: 99.9% (no single point of failure)
- **Security**: Zero-trust, capability-based
- **Scalability**: 100K+ nodes via DHT
- **Type Safety**: 100% verified (compile + runtime)

**Constraints**:
- Requires DNS for naming authority
- Requires at least 4 nodes for BFT (f=1)
- Requires public internet for wide-area federation
