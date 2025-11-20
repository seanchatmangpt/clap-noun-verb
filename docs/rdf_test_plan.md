# RDF-Based CLAUDE.md Configuration System - Test Plan

## Overview

This test plan covers comprehensive testing of the RDF-based CLAUDE.md configuration system, including RDF ontology validation, JSON-LD serialization, CLI integration, SHACL constraints, swarm coordination, and performance benchmarks.

## Test Coverage Matrix

| Test Category | Test Count | Coverage | Success Criteria |
|---------------|------------|----------|------------------|
| RDF Ontology Tests | 4 | SPARQL queries, triple validation | All queries return expected results |
| JSON-LD Serialization | 5 | Round-trip, context expansion | Data preserved across conversions |
| CLI Integration | 7 | All CLI commands | Commands execute successfully |
| SHACL Validation | 7 | Constraint enforcement | Invalid data rejected, valid accepted |
| Swarm Integration | 5 | Consensus, innovation scoring | Deterministic, consistent results |
| Performance Tests | 6 | Query speed, serialization | Meets SLOs (<100ms queries) |

**Total Tests**: 34
**Overall Coverage Target**: 90%+

## 1. RDF Ontology Tests (SPARQL Validation)

### 1.1 Query All Agents with Properties

**Test**: `test_sparql_query_all_agents_with_properties`

**Objective**: Verify SPARQL query returns all 54+ agents with required properties (name, type, capabilities, useCase).

**Success Criteria**:
- Query returns >= 54 agents
- Each agent has: name, agentType, hasCapability (1+), useCase
- No duplicate agents in results

**SPARQL Query**:
```sparql
PREFIX claude: <http://claude.ai/config#>
PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>

SELECT ?agent ?name ?type ?capability ?useCase
WHERE {
    ?agent rdf:type claude:Agent .
    ?agent claude:name ?name .
    ?agent claude:agentType ?type .
    ?agent claude:hasCapability ?capability .
    ?agent claude:useCase ?useCase .
}
```

**Expected Output**: Minimum 54 agents with complete property sets.

---

### 1.2 Query SPARC Phases with Dependencies

**Test**: `test_sparql_query_sparc_phases_with_dependencies`

**Objective**: Verify SPARQL query returns all 5 SPARC phases with dependency relationships.

**Success Criteria**:
- Query returns exactly 5 phases: Specification, Pseudocode, Architecture, Refinement, Completion
- Phases ordered by `phaseOrder` property
- Dependency chain verified: Specification → Pseudocode → Architecture → Refinement → Completion

**SPARQL Query**:
```sparql
PREFIX claude: <http://claude.ai/config#>

SELECT ?phase ?name ?order ?dependency
WHERE {
    ?phase rdf:type claude:SPARCPhase .
    ?phase claude:phaseName ?name .
    ?phase claude:phaseOrder ?order .
    OPTIONAL { ?phase claude:dependsOn ?dependency }
}
ORDER BY ?order
```

**Expected Output**: 5 phases in correct order with dependencies.

---

### 1.3 Query Absolute Rules with Enforcement

**Test**: `test_sparql_query_absolute_rules_with_enforcement`

**Objective**: Verify SPARQL query returns all 9 absolute rules with mandatory=true.

**Success Criteria**:
- Query returns exactly 9 absolute rules
- All rules have `isMandatory` = true
- All rules have `ruleCategory` = "absolute"

**SPARQL Query**:
```sparql
PREFIX claude: <http://claude.ai/config#>

SELECT ?rule ?description ?mandatory ?category
WHERE {
    ?rule rdf:type claude:Rule .
    ?rule claude:ruleDescription ?description .
    ?rule claude:isMandatory ?mandatory .
    ?rule claude:ruleCategory ?category .
    FILTER(?category = "absolute")
}
```

**Expected Output**: 9 absolute mandatory rules.

---

### 1.4 Validate SHACL Constraint Coverage

**Test**: `test_sparql_query_shacl_constraint_coverage`

**Objective**: Verify all entity types (Agent, Rule, SPARCPhase, SLO) have SHACL shapes defined.

**Success Criteria**:
- SHACL NodeShapes exist for: claude:Agent, claude:Rule, claude:SPARCPhase, claude:SLO
- Each shape has at least one property constraint

**SPARQL Query**:
```sparql
PREFIX sh: <http://www.w3.org/ns/shacl#>
PREFIX claude: <http://claude.ai/config#>

SELECT ?shape ?targetClass ?property ?constraint
WHERE {
    ?shape rdf:type sh:NodeShape .
    ?shape sh:targetClass ?targetClass .
    ?shape sh:property ?propertyShape .
    ?propertyShape sh:path ?property .
    OPTIONAL { ?propertyShape sh:minCount ?constraint }
}
```

**Expected Output**: SHACL shapes covering all entity types.

---

## 2. JSON-LD Serialization Tests

### 2.1 Round-Trip Preservation

**Test**: `test_roundtrip_rdf_jsonld_rdf_preserves_data`

**Objective**: Verify RDF → JSON-LD → RDF conversion preserves all triples.

**Success Criteria**:
- Original RDF triple count = deserialized RDF triple count
- All original triples present in deserialized graph
- No extra triples introduced

**Process**:
1. Create RDF ontology with agents, rules, phases
2. Serialize to JSON-LD
3. Deserialize back to RDF
4. Compare triple sets (order-independent)

**Performance SLO**: Round-trip < 100ms

---

### 2.2 Context Expansion

**Test**: `test_jsonld_context_expansion_produces_full_uris`

**Objective**: Verify JSON-LD @context expansion converts prefixes to full URIs.

**Success Criteria**:
- Compacted form uses prefixes (e.g., `claude:Agent`)
- Expanded form uses full URIs (e.g., `http://claude.ai/config#Agent`)
- Property names expanded correctly

**Example**:
```json
{
  "@context": {
    "claude": "http://claude.ai/config#",
    "name": "claude:name"
  },
  "@type": "claude:Agent",
  "name": "production-validator"
}
```

**Expanded**:
```json
{
  "@type": "http://claude.ai/config#Agent",
  "http://claude.ai/config#name": "production-validator"
}
```

---

### 2.3 Agent Serialization Preserves Capabilities

**Test**: `test_agent_serialization_preserves_capabilities_and_usecase`

**Objective**: Verify agent with multiple capabilities serializes correctly to JSON-LD.

**Success Criteria**:
- `claude:hasCapability` array contains all capabilities
- `claude:useCase` string preserved
- Agent type preserved

**Example Agent**:
```json
{
  "@type": "claude:Agent",
  "claude:name": "production-validator",
  "claude:agentType": "hyper-advanced",
  "claude:hasCapability": [
    "production_readiness",
    "dependency_validation",
    "slo_compliance"
  ],
  "claude:useCase": "Validating deployments, infrastructure, release readiness"
}
```

---

### 2.4 Rule Serialization Preserves Mandatory Flag

**Test**: `test_rule_serialization_preserves_mandatory_flag`

**Objective**: Verify rule serialization preserves `isMandatory` boolean.

**Success Criteria**:
- `claude:isMandatory` = true for absolute rules
- `claude:ruleCategory` = "absolute"
- Description text preserved

---

### 2.5 Nested JSON-LD Serialization

**Test**: `test_nested_jsonld_serialization_preserves_structure`

**Objective**: Verify nested JSON-LD (ClaudeConfig with agents array) serializes correctly.

**Success Criteria**:
- `claude:hasAgent` array contains agent objects
- Agent objects nested with full properties
- No data loss in nested structure

---

## 3. CLI Integration Tests

### 3.1 Agent List Command

**Test**: `test_cli_agent_list_returns_all_agents`

**Objective**: Verify `claude-config agent list` returns all agents.

**Command**: `claude-config agent list`

**Success Criteria**:
- Exit code 0
- Output contains >= 54 agent names
- Hyper-advanced agents present (production-validator, code-analyzer, etc.)

**Performance SLO**: Command completes < 200ms

---

### 3.2 Agent Describe Command

**Test**: `test_cli_agent_describe_shows_correct_details`

**Objective**: Verify `claude-config agent describe <name>` shows agent details.

**Command**: `claude-config agent describe production-validator`

**Success Criteria**:
- Exit code 0
- Output shows: name, type, capabilities, use case
- Formatting is human-readable

---

### 3.3 Rules List Command

**Test**: `test_cli_rules_list_absolute_shows_nine_rules`

**Objective**: Verify `claude-config rules list --category absolute` shows 9 rules.

**Command**: `claude-config rules list --category absolute`

**Success Criteria**:
- Exit code 0
- Output shows exactly 9 rules
- All rules marked as mandatory

---

### 3.4 SLO List Command

**Test**: `test_cli_slo_list_shows_performance_targets`

**Objective**: Verify `claude-config slo list` shows all performance targets.

**Command**: `claude-config slo list`

**Success Criteria**:
- Exit code 0
- Output shows: compilation, unit tests, integration tests, CLI execution, memory SLOs
- Target values displayed correctly (e.g., "2s", "100ms")

---

### 3.5 SPARQL Query Command

**Test**: `test_cli_query_sparql_executes_correctly`

**Objective**: Verify `claude-config query sparql --file <file>` executes SPARQL queries.

**Command**: `claude-config query sparql --file tests/fixtures/agent_query.rq --format json`

**Success Criteria**:
- Exit code 0
- Output is valid JSON
- JSON contains `results` field with bindings array
- Query returns expected data

**Performance SLO**: Query execution < 100ms

---

### 3.6 Error Handling

**Test**: `test_cli_error_handling_for_invalid_commands`

**Objective**: Verify CLI handles invalid commands gracefully.

**Command**: `claude-config invalid command`

**Success Criteria**:
- Exit code != 0
- stderr contains error message
- Error message is helpful (suggests valid commands)

---

### 3.7 Help Output

**Test**: `test_cli_help_output_shows_all_commands`

**Objective**: Verify `claude-config --help` shows all subcommands.

**Command**: `claude-config --help`

**Success Criteria**:
- Exit code 0
- Output mentions: agent, rules, slo, query subcommands
- Help text is formatted correctly

---

## 4. SHACL Validation Tests

### 4.1 Reject Config Violating Absolute Rules

**Test**: `test_shacl_rejects_config_violating_absolute_rules`

**Objective**: Verify SHACL validation rejects configurations missing mandatory rules.

**Success Criteria**:
- Validation returns `isValid = false`
- Violations array contains at least one violation
- Violation message mentions "mandatory" or "required"

---

### 4.2 Enforce Hyper-Advanced Agent Has 3+ Capabilities

**Test**: `test_shacl_enforces_hyperadvanced_agent_has_three_capabilities`

**Objective**: Verify SHACL enforces minimum 3 capabilities for hyper-advanced agents.

**Success Criteria**:
- Agent with 2 capabilities fails validation
- Violation is `sh:minCount` constraint on `claude:hasCapability`
- Violation message mentions minimum 3 capabilities

**SHACL Shape**:
```turtle
ex:HyperAdvancedAgentShape
    a sh:NodeShape ;
    sh:targetClass claude:Agent ;
    sh:property [
        sh:path claude:hasCapability ;
        sh:minCount 3 ;
        sh:message "Hyper-advanced agents must have at least 3 capabilities" ;
    ] .
```

---

### 4.3 Validate SLO Values Are Positive

**Test**: `test_shacl_validates_slo_values_are_positive`

**Objective**: Verify SHACL enforces positive numbers for SLO target values.

**Success Criteria**:
- SLO with negative value fails validation
- Violation is `sh:minExclusive` constraint on `claude:targetValue`
- Violation message mentions positive value requirement

**SHACL Shape**:
```turtle
ex:SLOShape
    a sh:NodeShape ;
    sh:targetClass claude:SLO ;
    sh:property [
        sh:path claude:targetValue ;
        sh:datatype xsd:decimal ;
        sh:minExclusive 0.0 ;
        sh:message "SLO target value must be positive" ;
    ] .
```

---

### 4.4 Catch Missing Required Properties

**Test**: `test_shacl_catches_missing_required_properties`

**Objective**: Verify SHACL enforces required properties (e.g., agent name).

**Success Criteria**:
- Agent missing `claude:name` fails validation
- Violation is `sh:minCount` constraint
- Violation message mentions missing property

---

### 4.5 Validate Datatype Constraints

**Test**: `test_shacl_validates_datatype_constraints`

**Objective**: Verify SHACL enforces datatype constraints (e.g., boolean for isMandatory).

**Success Criteria**:
- Rule with non-boolean `isMandatory` fails validation
- Violation is `sh:datatype` constraint
- Violation message mentions datatype requirement

---

### 4.6 Validate Pattern Constraints

**Test**: `test_shacl_validates_pattern_constraints`

**Objective**: Verify SHACL enforces pattern constraints (e.g., agent name format).

**Success Criteria**:
- Agent name with spaces/special chars fails validation (if pattern defined)
- Violation is `sh:pattern` constraint
- Valid kebab-case names pass validation

**SHACL Shape** (example):
```turtle
ex:AgentShape
    a sh:NodeShape ;
    sh:targetClass claude:Agent ;
    sh:property [
        sh:path claude:name ;
        sh:pattern "^[a-z]+(-[a-z]+)*$" ;
        sh:message "Agent name must be kebab-case" ;
    ] .
```

---

### 4.7 Valid Configuration Passes

**Test**: `test_valid_configuration_passes_shacl_validation`

**Objective**: Verify fully valid configuration passes SHACL validation.

**Success Criteria**:
- Validation returns `isValid = true`
- No violations
- All constraints satisfied

---

## 5. Swarm Integration Tests

### 5.1 Swarm Analysis Agents Read RDF Configuration

**Test**: `test_swarm_agents_read_rdf_config`

**Objective**: Verify swarm agents can load and query RDF configuration.

**Success Criteria**:
- Swarm agents load RDF ontology successfully
- Agents query SPARQL to retrieve agent metadata
- Query results used for coordination decisions

---

### 5.2 Consensus Voting Produces Consistent Rankings

**Test**: `test_consensus_voting_consistency`

**Objective**: Verify consensus voting produces deterministic, consistent rankings.

**Success Criteria**:
- Same input produces same voting results (deterministic)
- Rankings consistent across multiple runs
- Voting algorithm uses RDF metadata (capabilities, use cases)

---

### 5.3 Innovation Scores Calculated Deterministically

**Test**: `test_innovation_scores_deterministic`

**Objective**: Verify innovation scoring is deterministic and reproducible.

**Success Criteria**:
- Same agent metadata produces same innovation score
- Scores calculated from RDF properties (capabilities count, use case complexity)
- No randomness in scoring

**Innovation Scoring Formula** (example):
```
innovation_score = (capabilities_count * 0.4) + (use_case_complexity * 0.6)
```

---

### 5.4 Top-3 Innovation Selection with Diversity

**Test**: `test_top3_innovation_selection_diversity`

**Objective**: Verify top-3 innovation selection promotes diversity.

**Success Criteria**:
- Top-3 agents have different specializations
- Diversity metric > threshold (e.g., 0.7)
- Selection algorithm considers capability overlap

---

### 5.5 Swarm Coordination Uses RDF Metadata

**Test**: `test_swarm_coordination_uses_rdf_metadata`

**Objective**: Verify swarm coordination leverages RDF metadata for task assignment.

**Success Criteria**:
- Task assignment queries RDF for agent capabilities
- Agents selected based on capability match
- SPARQL queries used for agent discovery

---

## 6. Performance Tests

### 6.1 SPARQL Query Performance

**Test**: `test_sparql_query_performance`

**Objective**: Verify SPARQL queries complete in <100ms on full dataset.

**Success Criteria**:
- All agents query: < 100ms
- SPARC phases query: < 50ms
- Absolute rules query: < 50ms
- Complex joins query: < 150ms

**Benchmark Approach**:
- Run query 100 times
- Calculate mean, median, p95, p99
- Verify p95 < SLO

---

### 6.2 JSON-LD Serialization Performance

**Test**: `test_jsonld_serialization_performance`

**Objective**: Verify JSON-LD serialization of 54 agents in <50ms.

**Success Criteria**:
- Serialize 54 agents: < 50ms
- Deserialize 54 agents: < 50ms
- Round-trip (serialize + deserialize): < 100ms

---

### 6.3 CLI Command Response Time

**Test**: `test_cli_command_response_time`

**Objective**: Verify CLI commands respond in <200ms.

**Success Criteria**:
- `agent list`: < 200ms
- `agent describe`: < 100ms
- `rules list`: < 150ms
- `slo list`: < 100ms
- `query sparql`: < 100ms (simple query)

---

### 6.4 RDF File Load Performance

**Test**: `test_rdf_file_load_performance`

**Objective**: Verify RDF ontology loads in <500ms.

**Success Criteria**:
- Load full CLAUDE.md RDF file: < 500ms
- Parse Turtle format: < 300ms
- Build in-memory graph: < 200ms

---

### 6.5 SHACL Validation Performance

**Test**: `test_shacl_validation_performance`

**Objective**: Verify SHACL validation completes in <200ms for full config.

**Success Criteria**:
- Validate all agents: < 150ms
- Validate all rules: < 50ms
- Validate all phases: < 50ms
- Full configuration validation: < 200ms

---

### 6.6 Swarm Consensus Performance

**Test**: `test_swarm_consensus_performance`

**Objective**: Verify swarm consensus voting completes in <500ms for 5 agents.

**Success Criteria**:
- 5-agent consensus: < 500ms
- 10-agent consensus: < 1000ms
- Innovation scoring: < 100ms
- Top-3 selection: < 200ms

---

## Test Implementation Strategy

### Unit Tests
- RDF triple validation
- JSON-LD property mapping
- SPARQL query parsing
- Innovation scoring logic
- SHACL constraint enforcement

**Location**: `tests/unit/`

### Integration Tests
- Full RDF load from file
- SPARQL query against loaded ontology
- CLI command execution with output validation
- Round-trip serialization
- Swarm coordination workflows

**Location**: `tests/integration/`

### Property-Based Tests (Proptest)
- Any valid SHACL shape passes validation
- Innovation scores remain consistent
- Agent capability subsets are valid
- Round-trip conversions preserve data

**Location**: `tests/property/`

### Snapshot Tests (Insta)
- CLI output format consistency
- JSON-LD serialization determinism
- SPARQL query results
- SHACL violation messages

**Location**: `tests/snapshots/`

---

## Test Execution

### Run All Tests
```bash
cargo make test
```

### Run Specific Test Category
```bash
cargo make test rdf_ontology_tests
cargo make test jsonld_serialization_tests
cargo make test cli_integration_tests
cargo make test shacl_validation_tests
cargo make test swarm_integration_tests
```

### Run Performance Benchmarks
```bash
cargo make bench
cargo make slo-check
```

### Generate Coverage Report
```bash
cargo make coverage
```

**Coverage Target**: 90%+

---

## Success Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Test Pass Rate | 100% | All tests pass on main branch |
| Code Coverage | 90%+ | Statement, branch, function coverage |
| SPARQL Query SLO | <100ms | p95 latency for common queries |
| CLI Response SLO | <200ms | p95 latency for CLI commands |
| RDF Load SLO | <500ms | Full ontology load time |
| Round-Trip SLO | <100ms | RDF → JSON-LD → RDF |
| SHACL Validation SLO | <200ms | Full configuration validation |

---

## Test Data Fixtures

### Test RDF Files
- `tests/fixtures/claude_config.ttl` - Full CLAUDE.md ontology
- `tests/fixtures/minimal_config.ttl` - Minimal valid configuration
- `tests/fixtures/invalid_config.ttl` - Invalid configuration for negative tests

### Test SHACL Shapes
- `tests/fixtures/claude_shapes.ttl` - SHACL constraints for all entity types

### Test SPARQL Queries
- `tests/fixtures/agent_query.rq` - Query all agents
- `tests/fixtures/phase_query.rq` - Query SPARC phases
- `tests/fixtures/rule_query.rq` - Query absolute rules

### Test JSON-LD Files
- `tests/fixtures/agent.jsonld` - Example agent serialization
- `tests/fixtures/config.jsonld` - Example full configuration

---

## CI/CD Integration

### GitHub Actions Workflow
```yaml
name: RDF Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Run tests
        run: cargo make ci
      - name: Check SLOs
        run: cargo make slo-check
      - name: Generate coverage
        run: cargo make coverage
```

---

## Andon Signal Monitoring

**CRITICAL**: All tests must pass before marking work complete.

### Validation Checklist
- [ ] `cargo make check` - No compiler errors
- [ ] `cargo make test` - All tests pass
- [ ] `cargo make lint` - No clippy warnings
- [ ] `cargo make slo-check` - Performance SLOs met
- [ ] `cargo make coverage` - Coverage >= 90%

**Stop the Line**: If any validation fails, fix immediately before proceeding.

---

## Future Enhancements (FUTURE: prefix)

- FUTURE: Add mutation testing to verify test quality
- FUTURE: Add fuzz testing for RDF parsing
- FUTURE: Add stress testing for large ontologies (1000+ agents)
- FUTURE: Add visual regression testing for CLI output
- FUTURE: Add security testing for SPARQL injection
- FUTURE: Add internationalization testing for error messages

---

**Document Version**: 1.0
**Last Updated**: 2025-11-19
**Owner**: QA Specialist
