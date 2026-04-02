# Frontier Integration - Agent Coordination Guide

**Version:** 1.0.0
**Date:** 2026-01-05
**Purpose:** Detailed coordination workflows for all agents participating in the 5-phase frontier integration

---

## Agent Roster & Responsibilities

### Core Integration Team

1. **System Architect** (This Agent)
   - **Primary Role:** Design oversight, architectural decisions, coordination
   - **Authority:** Final approval on module boundaries, trait designs, dependency choices
   - **Artifacts:** Architecture documents, ADRs, design reviews

2. **Rust Coder**
   - **Primary Role:** Implementation of Rust code following architecture
   - **Authority:** Implementation details within approved architecture
   - **Artifacts:** Source code, unit tests, documentation comments

3. **Backend Developer**
   - **Primary Role:** External package integration, adapter implementations
   - **Authority:** Package selection (with architect approval), integration patterns
   - **Artifacts:** Adapter modules, integration tests, performance benchmarks

4. **Test Engineer**
   - **Primary Role:** Chicago TDD test creation, validation
   - **Authority:** Test strategy, coverage requirements
   - **Artifacts:** Test suites, test data, coverage reports

5. **Code Reviewer**
   - **Primary Role:** Code quality review, type safety verification
   - **Authority:** Approve/reject pull requests, request changes
   - **Artifacts:** PR reviews, code quality reports, refactoring suggestions

6. **Performance Benchmarker**
   - **Primary Role:** Performance measurement, SLO validation
   - **Authority:** Performance requirements, benchmark design
   - **Artifacts:** Benchmark suites, performance reports, optimization recommendations

7. **DevOps Engineer**
   - **Primary Role:** CI/CD pipeline, build infrastructure
   - **Authority:** Build configuration, CI matrix design
   - **Artifacts:** CI workflows, build scripts, deployment automation

8. **Production Validator**
   - **Primary Role:** Production readiness assessment
   - **Authority:** Go/no-go decisions for releases
   - **Artifacts:** Validation checklists, security audits, deployment guides

9. **Code Analyzer**
   - **Primary Role:** Static analysis, security review
   - **Authority:** Security requirements, code quality standards
   - **Artifacts:** Security reports, dependency audits, complexity analysis

---

## Phase 1: Foundation & Infrastructure (Weeks 1-2)

### Week 1: Infrastructure Setup

#### Day 1: Kickoff & Planning

**System Architect:**
```
Tasks:
- [ ] Present master architecture to team
- [ ] Review feature-flag hierarchy design
- [ ] Assign initial responsibilities
- [ ] Set up communication channels

Deliverables:
- Architecture presentation
- RACI matrix approved
- Communication plan

Coordination:
- Morning: Team kickoff meeting
- Afternoon: 1:1 with DevOps for CI setup
```

**DevOps Engineer:**
```
Tasks:
- [ ] Review CI/CD requirements
- [ ] Set up 21-configuration test matrix
- [ ] Configure cargo make timeout checks
- [ ] Prepare baseline benchmark infrastructure

Deliverables:
- CI workflow file (.github/workflows/frontier-integration.yml)
- Baseline benchmark harness

Coordination:
- Afternoon: 1:1 with System Architect
- EOD: Share CI design with team
```

**Rust Coder:**
```
Tasks:
- [ ] Review module structure design
- [ ] Create /src/frontier/ directory hierarchy
- [ ] Set up module stubs (mod.rs files)
- [ ] Add feature gates to lib.rs

Deliverables:
- /src/frontier/ directory structure
- Feature-gated modules

Coordination:
- Review architecture document
- EOD: Commit module structure
```

---

#### Days 2-3: Reflexive Testing Integration

**Test Engineer:**
```
Tasks:
- [ ] Review existing proptest integration
- [ ] Design reflexive testing architecture
- [ ] Create test generation templates
- [ ] Add tarpaulin configuration

Deliverables:
- /src/frontier/quality/reflexive_testing.rs stub
- Tarpaulin config
- Test generation design doc

Coordination:
- Daily standup: Progress updates
- Pair with Rust Coder for implementation
```

**Rust Coder:**
```
Tasks:
- [ ] Implement reflexive_testing.rs module
- [ ] Integrate with existing proptest
- [ ] Add auto-generation from RDF combinations
- [ ] Write unit tests (Chicago TDD)

Deliverables:
- Reflexive testing module (250+ LOC)
- Unit tests (AAA pattern)

Coordination:
- Pair with Test Engineer
- EOD: PR for code review
```

**Code Reviewer:**
```
Tasks:
- [ ] Review reflexive testing PR
- [ ] Check for type safety
- [ ] Verify error handling
- [ ] Run Andon signal checks

Deliverables:
- PR review with feedback
- Approval or change requests

Coordination:
- Review PR within 4 hours
- Pair with coder if major issues
```

---

#### Days 4-5: Fractal Patterns Foundation

**System Architect:**
```
Tasks:
- [ ] Review fractal patterns design
- [ ] Verify type-state approach
- [ ] Approve PhantomData usage
- [ ] Check for zero-cost properties

Deliverables:
- Design approval
- Type-level architecture validation

Coordination:
- Design review meeting
- Approve implementation plan
```

**Rust Coder:**
```
Tasks:
- [ ] Implement /src/frontier/foundation/fractal_patterns.rs
- [ ] Add LevelMarker traits (CliLevel, AgentLevel, EcosystemLevel)
- [ ] Implement FractalNoun and FractalVerb generics
- [ ] Add compile-time depth validation

Deliverables:
- Fractal patterns module (300+ LOC)
- Zero-cost abstractions verified

Coordination:
- Daily check-in with architect
- Submit PR for review
```

**Performance Benchmarker:**
```
Tasks:
- [ ] Create benchmarks for fractal patterns
- [ ] Verify zero-cost properties (PhantomData compiles away)
- [ ] Compare against hand-written code
- [ ] Measure compile-time impact

Deliverables:
- Benchmark results showing 0 runtime overhead
- Compile-time performance report

Coordination:
- Work with coder to identify hot paths
- Report findings to architect
```

---

### Week 2: Meta-Framework Integration

#### Days 6-7: Meta-Framework Implementation

**Backend Developer:**
```
Tasks:
- [ ] Add dependencies: erased-serde, typetag, oxrdf
- [ ] Create /src/frontier/foundation/meta_framework.rs
- [ ] Implement MetaIntrospectable trait
- [ ] Migrate RDF generation to oxrdf::Triple
- [ ] Integrate typetag trait registry

Deliverables:
- Meta-framework module (400+ LOC)
- Adapter for oxrdf
- Integration tests

Coordination:
- Morning: Review dependencies with architect
- Pair with Rust Coder for RDF migration
- Submit PR EOD Day 7
```

**Test Engineer:**
```
Tasks:
- [ ] Create integration tests for meta-framework
- [ ] Test RDF introspection performance
- [ ] Verify compile-time validation checks
- [ ] Benchmark against custom implementation

Deliverables:
- Integration test suite (15+ tests)
- Performance comparison report

Coordination:
- Work with Backend Dev for test coverage
- Report to Performance Benchmarker
```

**Performance Benchmarker:**
```
Tasks:
- [ ] Benchmark RDF introspection (target: 420ns)
- [ ] Compare against baseline (850ns)
- [ ] Verify 51% improvement
- [ ] Identify optimization opportunities

Deliverables:
- Benchmark results (should show 51% improvement)
- Performance report

Coordination:
- Coordinate with Test Engineer
- Report to System Architect
```

---

#### Days 8-10: RDF/SPARQL Stack Integration

**Backend Developer:**
```
Tasks:
- [ ] Add oxigraph, json-ld dependencies
- [ ] Create /src/frontier/foundation/rdf_composition.rs
- [ ] Implement RdfTripleStore trait for oxigraph
- [ ] Add SPARQL 1.1 query support
- [ ] Integrate JSON-LD for MCP

Deliverables:
- RDF composition module (500+ LOC)
- SPARQL 1.1 support
- JSON-LD integration

Coordination:
- Daily: Progress updates to architect
- Pair with Code Analyzer for security review
```

**Code Analyzer:**
```
Tasks:
- [ ] Review oxigraph integration for security
- [ ] Check for SPARQL injection vulnerabilities
- [ ] Verify input validation
- [ ] Audit dependencies (cargo audit)

Deliverables:
- Security review report
- Vulnerability assessment
- Dependency audit results

Coordination:
- Review code with Backend Dev
- Report findings to architect
```

**Test Engineer:**
```
Tasks:
- [ ] Create SPARQL 1.1 compliance test suite
- [ ] Test JSON-LD roundtrip conversions
- [ ] Verify federation query support
- [ ] Performance test with 1M+ triples

Deliverables:
- SPARQL test suite (50+ tests)
- JSON-LD test suite
- Large-scale performance tests

Coordination:
- Validate against W3C SPARQL 1.1 spec
- Coordinate with Performance Benchmarker
```

**Performance Benchmarker:**
```
Tasks:
- [ ] Benchmark SPARQL queries (simple and complex)
- [ ] Verify 10x improvement on complex queries
- [ ] Test federation query overhead (target: <10ms)
- [ ] Measure triple store scalability

Deliverables:
- SPARQL performance report
- Scalability analysis (1M+ triples)

Coordination:
- Work with Test Engineer for test data
- Report to System Architect
```

---

## Phase 2-5: Detailed Workflows

### Workflow Pattern: Feature Implementation

**Standard 5-Day Feature Implementation Cycle:**

#### Day 1: Design & Setup
- **System Architect:** Review feature design, approve approach
- **Backend Developer:** Add dependencies, create module stub
- **Test Engineer:** Design test strategy

#### Days 2-3: Implementation
- **Backend Developer:** Implement core functionality
- **Rust Coder:** Assist with complex Rust patterns
- **Test Engineer:** Create tests in parallel

#### Day 4: Review & Optimization
- **Code Reviewer:** Review PR, provide feedback
- **Performance Benchmarker:** Run benchmarks, identify bottlenecks
- **Code Analyzer:** Security and quality analysis

#### Day 5: Integration & Validation
- **Test Engineer:** Run integration tests
- **DevOps Engineer:** Add to CI pipeline
- **Production Validator:** Validate against checklist

---

## Communication Protocols

### Daily Standup (15 minutes, 9:00 AM)

**Format:**
```
Each agent reports:
1. Completed yesterday
2. Plan for today
3. Blockers/help needed

System Architect facilitates and resolves blockers
```

### Weekly Architecture Review (1 hour, Friday 2:00 PM)

**Agenda:**
```
1. System Architect: Week recap, key decisions
2. Backend Developer: Integration status
3. Performance Benchmarker: Performance metrics
4. Code Reviewer: Code quality trends
5. DevOps Engineer: CI/CD status
6. Next week planning
```

### Phase Gate Review (2 hours, End of each phase)

**Participants:** All agents + stakeholders

**Deliverables Review:**
```
1. System Architect: Architecture conformance
2. Test Engineer: Test coverage report
3. Performance Benchmarker: SLO validation
4. Production Validator: Readiness assessment
5. DevOps Engineer: CI status
6. Decision: Proceed / Adjust / Rollback
```

---

## Escalation Paths

### Issue Severity Levels

**P0 - Critical (Immediate)**
- Circular dependencies detected
- Security vulnerability found
- SLO failure (>50% degradation)
- **Escalation:** System Architect → Tech Lead → CTO

**P1 - High (Same day)**
- Test failures blocking progress
- Integration conflicts between features
- Performance regression (>20%)
- **Escalation:** System Architect → Tech Lead

**P2 - Medium (Next day)**
- Non-critical bugs
- API design questions
- Documentation gaps
- **Escalation:** Responsible agent → System Architect

**P3 - Low (Next sprint)**
- Nice-to-have improvements
- Refactoring opportunities
- Technical debt
- **Escalation:** Track in backlog, discuss in weekly review

---

## Success Metrics & Tracking

### Weekly Metrics Dashboard

**System Architect maintains:**
```
┌─────────────────────────────────────────────────────┐
│ Frontier Integration Progress - Week N              │
├─────────────────────────────────────────────────────┤
│ Phase Completion:        [▓▓▓▓▓░░░░░] 50%          │
│ Test Coverage:           85% (target: 80%)          │
│ Performance SLOs:        12/15 met (80%)            │
│ Blockers:                2 (P1: 1, P2: 1)           │
│ PRs Merged:              8/10                       │
│ CI Success Rate:         95%                        │
│ Security Issues:         0 critical, 2 low          │
└─────────────────────────────────────────────────────┘
```

### Agent-Specific KPIs

**Backend Developer:**
- Features integrated: N/week
- Integration tests passing: >95%
- Code review feedback cycles: <3

**Test Engineer:**
- Test coverage: >80%
- Chicago TDD compliance: 100%
- Regression bugs caught: Track

**Performance Benchmarker:**
- SLOs validated: N/week
- Performance regressions: 0
- Optimization recommendations: N/week

**Code Reviewer:**
- PRs reviewed: Within 4 hours
- Code quality score: >8/10
- Architectural violations: 0

---

## Tools & Infrastructure

### Required Tools

**All Agents:**
- Git (version control)
- Rust toolchain (stable)
- cargo-make (build automation)
- IDE with Rust analyzer

**Backend Developer:**
- cargo-audit (security scanning)
- cargo-deny (dependency validation)

**Test Engineer:**
- cargo-tarpaulin (code coverage)
- proptest (property-based testing)
- cucumber (BDD specifications)

**Performance Benchmarker:**
- criterion (benchmarking)
- flamegraph (profiling)
- perf (Linux performance tools)

**DevOps Engineer:**
- GitHub Actions (CI/CD)
- Docker (containerization)

### Shared Resources

**Documentation:**
- /docs/FRONTIER_MASTER_ARCHITECTURE.md
- /docs/FRONTIER_INTEGRATION_PATTERNS.md
- /docs/ADR-NNN-*.md (Architecture Decision Records)

**Code:**
- /src/frontier/* (feature modules)
- /tests/integration/* (integration tests)
- /benches/* (benchmark suites)

**Communication:**
- Daily standup (video call)
- #frontier-integration (Slack channel)
- GitHub Issues (bug tracking)
- GitHub PRs (code review)

---

## Best Practices

### For All Agents

1. **Always run Andon signal checks before committing**
   ```bash
   cargo make check
   cargo make test
   cargo make lint
   ```

2. **Follow Chicago TDD** - State-based tests, AAA pattern, real collaborators

3. **Batch operations** - Multiple file ops in single message

4. **Use cargo make** - NEVER use direct cargo commands

5. **Document decisions** - Create ADRs for architectural choices

6. **Type-first thinking** - Encode invariants in types

### For Backend Developers

1. **Zero-cost adapters** - Verify with benchmarks
2. **Feature gates** - All integrations behind features
3. **Error handling** - Use Result<T,E>, no unwrap/expect
4. **Documentation** - Document all public APIs

### For Test Engineers

1. **Chicago TDD** - State-based, not implementation-based
2. **Property-based** - Use proptest for invariants
3. **Coverage** - Aim for 80%+ on new code
4. **Integration tests** - Test feature combinations

### For Code Reviewers

1. **Type safety** - Verify compile-time guarantees
2. **Zero-cost** - Check for unnecessary allocations
3. **API ergonomics** - Easy to use correctly, hard to misuse
4. **Documentation** - Complete and accurate

---

**End of Agent Coordination Guide**

This guide ensures all agents work in harmony toward the successful 5-phase frontier integration, with clear responsibilities, communication protocols, and success metrics.
