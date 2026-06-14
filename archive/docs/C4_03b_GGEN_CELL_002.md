# C4-03b: ggen Manufacturing Cell

## Mission

Show how ggen transforms law graphs into artifacts. ggen emits evidence. wasm4pm adjudicates.

## Manufacturing Pipeline

```
┌───────────────────────────────────────────────────────────────────────┐
│                        GGEN MANUFACTURING CELL                        │
│                                                                       │
│  INPUT: Law Graphs (RDF/Turtle) + Selection Rules + Templates       │
│  OUTPUT: Artifacts (typed files) + Audit Trail + Receipts           │
│                                                                       │
│  ┌─────────────────────────────────────────────────────────────────┐ │
│  │                    KNOWLEDGE INPUT LAYER                        │ │
│  │                                                                 │ │
│  │  .ttl files (RDF Turtle)                                       │ │
│  │  ├─ canon.ttl         (predicates; what is lawful)            │ │
│  │  ├─ doctrine.ttl      (rules; how work proceeds)              │ │
│  │  ├─ schema.nt         (audit schema; event structure)         │ │
│  │  └─ ontology.ttl      (concepts; noun/verb/arg defs)          │ │
│  │                                                                 │ │
│  └────────────────────────┬──────────────────────────────────────┘ │
│                           │                                         │
│  ┌────────────────────────▼──────────────────────────────────────┐ │
│  │                    SELECTION LAYER                            │ │
│  │                                                                 │ │
│  │  .rq files (SPARQL SELECT queries)                            │ │
│  │  ├─ which_predicates_apply.rq                                 │ │
│  │  ├─ which_noun_types.rq                                       │ │
│  │  ├─ which_verbs_allowed.rq                                    │ │
│  │  └─ which_gates_required.rq                                   │ │
│  │                                                                 │ │
│  │  Selection Result: Subset of applicable rules for this build  │ │
│  │                                                                 │ │
│  └────────────────────────┬──────────────────────────────────────┘ │
│                           │                                         │
│  ┌────────────────────────▼──────────────────────────────────────┐ │
│  │                    RENDERING LAYER                            │ │
│  │                                                                 │ │
│  │  .tera files (Jinja2 templates)                               │ │
│  │  ├─ cli_command.rs.tera                                       │ │
│  │  ├─ audit_trail.json.tera                                     │ │
│  │  ├─ receipt_schema.nt.tera                                    │ │
│  │  └─ conformance_check.sparql.tera                             │ │
│  │                                                                 │ │
│  │  Template context: { selected_rules, predicates, gates, ... } │ │
│  │                                                                 │ │
│  └────────────────────────┬──────────────────────────────────────┘ │
│                           │                                         │
│  ┌────────────────────────▼──────────────────────────────────────┐ │
│  │                  CONTROL PLANE LAYER                          │ │
│  │                                                                 │ │
│  │  ggen.toml (manufacturing rules)                              │ │
│  │  ├─ [inputs]                                                  │ │
│  │  │  ├─ ttl_graph = "canon.ttl"                               │ │
│  │  │  └─ sparql_selection = "which_predicates_apply.rq"       │ │
│  │  ├─ [outputs]                                                │ │
│  │  │  ├─ artifact_dir = "./target/ggen"                       │ │
│  │  │  └─ audit_dir = "./target/ggen/audits"                   │ │
│  │  ├─ [rules]                                                  │ │
│  │  │  ├─ rule_1 = { if: "predicate == audit", then: ".." }   │ │
│  │  │  ├─ rule_2 = { if: "noun_type == receipt", then: ".." }  │ │
│  │  │  └─ rule_N = { ... }                                      │ │
│  │  └─ [features]                                               │ │
│  │     ├─ cold_path_only = true                                 │ │
│  │     └─ no_hot_path_dependency = true                          │ │
│  │                                                                 │ │
│  │  Control Loop:                                                │ │
│  │  1. Load graph + selections                                  │ │
│  │  2. Apply rules to each predicate                            │ │
│  │  3. Render template for each match                           │ │
│  │  4. Emit artifact + audit trail                              │ │
│  │  5. Sign artifact hash to receipt ledger                     │ │
│  │                                                                 │ │
│  └────────────────────────┬──────────────────────────────────────┘ │
│                           │                                         │
│  ┌────────────────────────▼──────────────────────────────────────┐ │
│  │                    ARTIFACT OUTPUTS                           │ │
│  │                                                                 │ │
│  │  .rs files (Rust source code)                                │ │
│  │  .yaml files (configuration schemas)                         │ │
│  │  .sol files (Solidity contracts)                             │ │
│  │  .json files (data structures)                               │ │
│  │  .nt files (OCEL event schema instances)                     │ │
│  │                                                                 │ │
│  └────────────────────────┬──────────────────────────────────────┘ │
│                           │                                         │
│  ┌────────────────────────▼──────────────────────────────────────┐ │
│  │                   AUDIT TRAIL OUTPUTS                         │ │
│  │                                                                 │ │
│  │  ggen_audit.json                                             │ │
│  │  {                                                             │ │
│  │    "artifact_id": "uuid",                                    │ │
│  │    "timestamp": "ISO8601",                                   │ │
│  │    "source_graph": "canon.ttl",                              │ │
│  │    "selection_query": "which_predicates_apply.rq",           │ │
│  │    "rules_applied": [ ... ],                                 │ │
│  │    "template_context": { ... },                              │ │
│  │    "artifact_hash": "sha256:...",                            │ │
│  │    "why": "predicate=audit AND noun=receipt, so render.."   │ │
│  │  }                                                             │ │
│  │                                                                 │ │
│  │  Human-readable explanation: "We chose rule_7 because the   │ │
│  │  predicate 'audit' applies and noun_type is 'receipt'."      │ │
│  │                                                                 │
│  │  Receipt (to ledger):                                        │ │
│  │  {                                                             │ │
│  │    "receipt_id": "uuid",                                    │ │
│  │    "artifact_hash": "sha256:...",                            │ │
│  │    "timestamp": "ISO8601",                                   │ │
│  │    "status": "emitted",                                      │ │
│  │    "adjudicator": "pending_wasm4pm"                          │ │
│  │  }                                                             │ │
│  │                                                                 │ │
│  └─────────────────────────────────────────────────────────────┘ │
│                                                                       │
│  CRITICAL BOUNDARY:                                                 │
│  ┌────────────────────────────────────────────────────────────────┐ │
│  │  ggen EMITS EVIDENCE.                                          │ │
│  │  ggen DOES NOT DECIDE what is lawful.                         │ │
│  │  wasm4pm ADJUDICATES.                                         │ │
│  │                                                                 │ │
│  │  ggen follows rules. wasm4pm checks rules.                    │ │
│  │  ggen produces artifacts. wasm4pm admits or rejects.          │ │
│  │                                                                 │ │
│  │  If wasm4pm rejects an artifact, ggen must re-render with     │ │
│  │  different selection or rules. ggen does not override wasm4pm.│ │
│  └────────────────────────────────────────────────────────────────┘ │
│                                                                       │
└───────────────────────────────────────────────────────────────────────┘
                              │
                              │ Artifacts
                              │ + Audit Trail
                              │ + Evidence Receipts
                              ↓
                        ┌──────────────────┐
                        │  wasm4pm-compat  │
                        │  (Admission Gate)│
                        │                  │
                        │ Check artifact   │
                        │ against doctrine │
                        │ Emit verdict     │
                        └──────────────────┘
```

## Component Details

### Knowledge Input Layer
- **canon.ttl**: Predicates (what is lawful, what is forbidden, what is required)
- **doctrine.ttl**: Rules (how work proceeds, gate sequences, approval conditions)
- **schema.nt**: OCEL audit schema (event structure, object types, attributes)
- **ontology.ttl**: Concept definitions (nouns: receipt, petition, motion; verbs: approve, emit, sign; args: timestamp, hash)

### Selection Layer
SPARQL SELECT queries that answer: "Given this artifact scope, which rules apply?"

Example:
```sparql
SELECT ?rule
WHERE {
  ?artifact a noun:Receipt .
  ?rule doctrine:appliesTo noun:Receipt .
  ?rule doctrine:requiredWhen ?condition .
  FILTER (?condition = "event_type=receipt_emitted")
}
```

Result: A list of rule IDs (rule_7, rule_12, etc.)

### Rendering Layer
Jinja2 templates that generate artifacts. Template context includes selected rules, predicates, and audit metadata.

Example `receipt_schema.nt.tera`:
```
{% for rule in selected_rules %}
# Rule: {{ rule.id }}
{% if rule.emits_event %}
_:event_{{ loop.index0 }} a ocel:Event ;
  ocel:eventType "{{ rule.event_type }}" ;
  ocel:timestamp ?timestamp ;
  ocel:objects ( {% for obj in rule.objects %} ?{{ obj.name }} {% endfor %} ) .
{% endif %}
{% endfor %}
```

### Control Plane Layer
**ggen.toml** orchestrates the entire pipeline:

```toml
[inputs]
ttl_graph = "docs/canon.ttl"
sparql_selection = "queries/which_predicates_apply.rq"

[outputs]
artifact_dir = "target/ggen/artifacts"
audit_dir = "target/ggen/audits"
receipt_ledger = "target/ggen/receipts.jsonl"

[[rules]]
id = "rule_1"
description = "If artifact is a receipt, render OCEL schema"
if = "artifact_type == noun:Receipt"
then = "render(receipt_schema.nt.tera)"

[[rules]]
id = "rule_2"
description = "If rule requires signature, emit audit trail"
if = "doctrine:requiresSignature = true"
then = "render(audit_trail.json.tera)"

[features]
cold_path_only = true
no_runtime_lvm_in_hot_path = true
```

### Artifact Outputs
- **.rs files**: Rust code (command handlers, gate implementations)
- **.yaml files**: Configuration schemas (noun/verb definitions)
- **.sol files**: Solidity contracts (if blockchain auditing is required)
- **.json files**: Data structures (gate specifications, routing rules)
- **.nt files**: OCEL event schema instances (receipts in N-Triples format)

### Audit Trail Outputs
**ggen_audit.json** explains every artifact:
```json
{
  "artifact_id": "0x1a2b3c4d",
  "timestamp": "2026-06-02T10:30:00Z",
  "source_graph": "canon.ttl",
  "selection_query": "which_predicates_apply.rq",
  "rules_applied": ["rule_1", "rule_12"],
  "template_context": {
    "predicate": "audit",
    "noun": "receipt",
    "required_gates": ["inspection", "repair"]
  },
  "artifact_hash": "sha256:abc123...",
  "why": "Predicate 'audit' AND noun_type 'receipt' → rule_1 applies → render receipt_schema.nt.tera"
}
```

## Critical Boundary: ggen ↔ wasm4pm

```
┌──────────────────────────┐        ┌──────────────────────────┐
│        GGEN              │        │      wasm4pm-compat      │
│   (Manufacturing)        │        │    (Admission Gate)      │
├──────────────────────────┤        ├──────────────────────────┤
│ • Renders artifacts      │        │ • Checks artifact vs.    │
│ • Emits evidence trail   │        │   doctrine               │
│ • Signs to receipt       │        │ • Emits verdict          │
│ • Does NOT decide lawful │        │ • DOES decide lawful     │
│                          │        │                          │
│ Input: Rules             │        │ Input: Artifact          │
│ Output: Artifacts        │   ───→ │ Output: Admit/Reject     │
│                          │        │                          │
│ Role: Evidence Maker     │        │ Role: Adjudicator        │
└──────────────────────────┘        └──────────────────────────┘
```

**Rule:** If wasm4pm rejects an artifact, ggen re-renders (with different selection or rules). ggen never overrides wasm4pm.

## Architecturally Forbidden

- ❌ ggen executes admitted artifacts (only renders them)
- ❌ ggen decides what is lawful (only follows rules)
- ❌ ggen runs in hot path (cold path CI only)
- ❌ ggen skips audit trail (every artifact has audit trail)
- ❌ ggen emits receipts without evidence (receipt must link to artifact hash)
