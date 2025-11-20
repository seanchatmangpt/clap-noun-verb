# Data Flow - RDF MCP CLI Swarm Coordination

## 🌊 Information Flow Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                    1. ONTOLOGY DISCOVERY PHASE                  │
└─────────────────────────────────────────────────────────────────┘

    Queen Seraphina
         │
         ├─→ [SPARQL Query] → RDF MCP Server
         │                         │
         │                         ├─→ CLI Ontology (TTL/RDF)
         │                         │   • Noun taxonomy
         │                         │   • Verb taxonomy
         │                         └─→ SHACL Shapes
         │
         ├─→ Scout Alpha   → Memory[swarm/shared/nouns]
         │   "Find all CLI nouns (services, config, logs)"
         │
         ├─→ Scout Beta    → Memory[swarm/shared/verbs]
         │   "Discover verbs (status, show, tail, start, stop)"
         │
         └─→ Scout Gamma   → Memory[swarm/shared/constraints]
             "Extract SHACL validation rules"

┌─────────────────────────────────────────────────────────────────┐
│                    2. COMMAND VALIDATION PHASE                  │
└─────────────────────────────────────────────────────────────────┘

    Validator Sentinel
         │
         ├─→ Retrieve: Memory[swarm/shared/constraints]
         │   • SHACL shapes for "services status"
         │   • Required fields: noun=services, verb=status
         │   • Type constraints: both are strings
         │
         ├─→ Validate Command:
         │   ✅ sh:property [
         │        sh:path cli:noun ;
         │        sh:minCount 1 ;
         │        sh:datatype xsd:string ;
         │        sh:in ("services" "config" "logs")
         │      ]
         │   ✅ sh:property [
         │        sh:path cli:verb ;
         │        sh:minCount 1 ;
         │        sh:in ("status" "show" "tail")
         │      ]
         │
         └─→ Result → Memory[swarm/validation/result]
             • Status: VALID ✅
             • Andon Signal: GREEN 🟢
             • Timestamp: 2025-11-20T05:03:00Z

┌─────────────────────────────────────────────────────────────────┐
│                    3. COMMAND EXECUTION PHASE                   │
└─────────────────────────────────────────────────────────────────┘

    Queen Seraphina
         │
         ├─→ Check: Memory[swarm/validation/result]
         │   ✅ Validation passed
         │
         ├─→ Assign Task → Worker One
         │   Command: "clnrm services status"
         │
         └─→ Worker One Executes:
             │
             ├─→ Execute: clnrm services status
             │   Output: {
             │     "docker": "running",
             │     "otel-collector": "running",
             │     "weaver": "running",
             │     "testcontainers": "available"
             │   }
             │
             ├─→ Generate Receipt:
             │   receipt = {
             │     "command": "services status",
             │     "timestamp": "2025-11-20T05:03:05.123Z",
             │     "executor": "worker-one",
             │     "result": {...output...},
             │     "hash": blake3(command + timestamp + result)
             │   }
             │   hash = "a7f5c2e8d9b1..." (64 chars)
             │
             └─→ Store: Memory[swarm/receipts/services-status]
                        Memory[swarm/lockchain/latest]

┌─────────────────────────────────────────────────────────────────┐
│                    4. PROVENANCE TRACKING PHASE                 │
└─────────────────────────────────────────────────────────────────┘

    Worker Two (Provenance Tracker)
         │
         ├─→ Retrieve: Memory[swarm/receipts/services-status]
         │
         ├─→ Build Provenance Chain:
         │   chain = {
         │     "receipt_id": "receipt_1763614985_001",
         │     "command": "services status",
         │     "validation_hash": "b3e8f7a2c9...",
         │     "execution_hash": "a7f5c2e8d9b1...",
         │     "previous_hash": "0000000000..." (genesis),
         │     "chain_hash": blake3(all_hashes),
         │     "proof": "PoE (Proof of Execution)"
         │   }
         │
         └─→ Store: Memory[swarm/lockchain/chain]

┌─────────────────────────────────────────────────────────────────┐
│                    5. METRICS AGGREGATION PHASE                 │
└─────────────────────────────────────────────────────────────────┘

    Worker Three (Metrics Collector)
         │
         ├─→ Collect Metrics:
         │   • Execution time: 45ms
         │   • Memory usage: 2.3MB
         │   • Validation time: 12ms
         │   • Receipt generation: 8ms
         │
         ├─→ Aggregate Results:
         │   metrics = {
         │     "total_time": "65ms",
         │     "agents_involved": 5,
         │     "validations": 1,
         │     "executions": 1,
         │     "receipts_generated": 1,
         │     "lockchain_blocks": 1
         │   }
         │
         └─→ Store: Memory[swarm/metrics/summary]

┌─────────────────────────────────────────────────────────────────┐
│                    6. CONSENSUS & REPORTING PHASE               │
└─────────────────────────────────────────────────────────────────┘

    Queen Seraphina
         │
         ├─→ Gather from Memory:
         │   • swarm/shared/nouns
         │   • swarm/shared/verbs
         │   • swarm/validation/result
         │   • swarm/receipts/services-status
         │   • swarm/lockchain/chain
         │   • swarm/metrics/summary
         │
         ├─→ Build Consensus:
         │   ✅ All scouts agree on ontology
         │   ✅ Validator approved execution
         │   ✅ Workers generated valid receipts
         │   ✅ Lockchain integrity verified
         │
         └─→ Generate Report:
             📊 Swarm Coordination Report
             • Operation: "services status"
             • Validation: PASSED ✅
             • Execution: SUCCESS ✅
             • Receipt Hash: a7f5c2e8d9b1...
             • Lockchain Block: #1
             • Total Time: 65ms
```

## 🔐 Blake3 Hashing Strategy

### Receipt Hashing
```rust
fn generate_receipt_hash(command: &str, timestamp: &str, result: &str) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(command.as_bytes());
    hasher.update(timestamp.as_bytes());
    hasher.update(result.as_bytes());
    hasher.finalize().to_hex().to_string()
}
```

### Lockchain Linking
```rust
fn build_lockchain_block(
    receipt_hash: &str,
    validation_hash: &str,
    previous_hash: &str
) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(receipt_hash.as_bytes());
    hasher.update(validation_hash.as_bytes());
    hasher.update(previous_hash.as_bytes());
    hasher.finalize().to_hex().to_string()
}
```

## 📡 Memory Coordination Protocol

### Namespace: `hive-mind-coordination`
- **TTL**: 3600 seconds (1 hour)
- **Storage**: SQLite
- **Keys**:
  - `swarm/queen/status` - Coordination state
  - `swarm/scouts/{alpha,beta,gamma}/status` - Scout progress
  - `swarm/workers/{one,two,three}/status` - Worker state
  - `swarm/validator/status` - Guard state
  - `swarm/shared/ontology` - RDF discoveries
  - `swarm/shared/nouns` - Noun taxonomy
  - `swarm/shared/verbs` - Verb taxonomy
  - `swarm/shared/constraints` - SHACL shapes
  - `swarm/validation/result` - Validation outcomes
  - `swarm/receipts/*` - Execution receipts
  - `swarm/lockchain/chain` - Provenance chain
  - `swarm/metrics/summary` - Performance metrics

## 🎯 Data Flow Summary

1. **Discovery**: Scouts query RDF ontology via SPARQL → Memory
2. **Validation**: Sentinel validates command against SHACL → Memory
3. **Execution**: Workers execute validated command → Receipt
4. **Hashing**: Blake3 hash of receipt → Lockchain block
5. **Provenance**: Link blocks with previous hash → Chain
6. **Consensus**: Queen aggregates all results → Report
