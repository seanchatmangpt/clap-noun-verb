# Reading Guide: Navigate the Graph-Universe Documentation

This guide helps you navigate the comprehensive documentation of the graph-universe system based on your role, interests, and depth level.

---

## Quick Start (15 minutes)

**For anyone new to the system:**

1. Start here: **[PHILOSOPHY.md](PHILOSOPHY.md)** (read sections 1-2)
   - Understand: What is the graph-universe thesis?
   - Key idea: A = μ(Σ) — Application derives from Ontology via Kernel

2. Then: **[SYSTEMS.md](SYSTEMS.md)** (read "Overview" and "Tier 1-3" sections)
   - Understand: How do the organ systems fit together?
   - Key idea: 9 tiers working in harmony

3. Finally: This guide (READING_GUIDE.md)
   - Plan your deeper reading based on your role

**Time investment**: ~15 minutes
**Outcome**: Conceptual understanding of the thesis

---

## By Role

### 👨‍💼 Executive / Manager

**Goal**: Understand the value proposition and risks

**Reading path** (45 minutes):
1. PHILOSOPHY.md → "Why This Model?" section
2. SYSTEMS.md → "Scalability Limits & Estimates" section
3. DFLSS.md → "Phase Progression (Control)" section
4. SYSTEMS.md → "Security Threat Model" section

**Key takeaways**:
- Single source of truth eliminates inconsistency (saves engineering time)
- Autonomous optimization (DFLSS) continuously improves system
- Trillion-agent scale possible with nomrg overlay algebra
- Security is built-in, not bolted on

**Time**: ~45 minutes

---

### 🏗️ Architect / System Designer

**Goal**: Understand architecture and design principles

**Reading path** (2-3 hours):
1. PHILOSOPHY.md → Complete
2. SYSTEMS.md → Complete (all sections)
3. KNHK.md → Complete
4. MU_KERNEL.md → Complete
5. API_SPECIFICATION.md → Sections 1-9 (skim implementations, focus on contracts)
6. NOMRG.md → Complete
7. GGEN.md → Complete

**Key learning**:
- How KNHK (ontology) drives everything downstream
- How μ-kernel provides deterministic execution guarantees
- How nomrg enables conflict-free composition at scale
- How ggen turns ontology into working code

**Key questions to answer**:
- "What happens when an agent invokes a capability?" → See SYSTEMS.md "End-to-End Flow"
- "How do we prevent hand-edited code?" → See CODE_AS_PROJECTION.md + enforcement mechanism
- "What if DFLSS proposes a breaking change?" → See NOMRG.md + CTT verification
- "How do we verify correctness?" → See CTT (in CNV_PHASES_COMPLETE.md) + CLNRM.md

**Time**: ~2-3 hours

---

### 👨‍💻 Implementation Engineer

**Goal**: Understand how to build/extend the system

**Reading path** (4-5 hours):
1. CODE_AS_PROJECTION.md → Complete (understand the policy)
2. GGEN.md → Complete (you might implement this)
3. DFLSS.md → Complete
4. API_SPECIFICATION.md → Complete (read all, especially sections relevant to your component)
5. MU_KERNEL.md → "Current Implementation" section
6. CLNRM.md → Complete (write hermetic tests)
7. Source files: Read the codebase guided by comments and API spec

**Key tasks**:
- Implement ggen projection engine (1-2 months) — See GGEN.md Phase 1-6
- Extend clnrm with OpenTelemetry integration — See CLNRM.md Phase 1
- Add DFLSS into AHI optimization loop — See DFLSS.md + src/kernel/ahi_policy.rs

**Coding patterns**:
- See API_SPECIFICATION.md "Appendix: Common Patterns"
- All generated code: marked with @generated-from, links to source
- All tests: use HermeticContainer for isolation
- All operations: check quota pre-check, then execute, then record

**Time**: ~4-5 hours of reading + months of implementation

---

### 🔬 Security Analyst / Threat Modeler

**Goal**: Understand security properties and threat model

**Reading path** (2-3 hours):
1. MU_KERNEL.md → "Timing-Based Proofs" section
2. MU_KERNEL.md → "Guarantees Provided" section
3. CODE_AS_PROJECTION.md → Complete (immutability via enforcement)
4. CLNRM.md → Complete (isolation + determinism)
5. SYSTEMS.md → "Security Threat Model" section
6. API_SPECIFICATION.md → Section 8 (AHI governance and authorization)

**Key security properties**:

| Property | How Enforced | Proof |
|----------|--------------|-------|
| **Timing side-channel resistance** | τ ≤ 100ns (constant-time) | MU_KERNEL.md timing proofs |
| **Code integrity** | CODE_AS_PROJECTION enforcement + CI | CODE_AS_PROJECTION.md |
| **Isolation** | clnrm hermetic containers + μ-kernel | CLNRM.md, MU_KERNEL.md |
| **Determinism** | No dynamic allocation, no randomization | MU_KERNEL.md, CTT tests |
| **Authorization** | Type-state + AHI policy engine | API_SPECIFICATION.md §8.1 |
| **Auditability** | Every receipt in Γ, Ed25519 signed | SYSTEMS.md "End-to-End Flow" |
| **No hand-edits** | Pre-commit hook + CI checks | CODE_AS_PROJECTION.md enforcement |

**Threat scenarios**: See SYSTEMS.md "Security Threat Model"

**Time**: ~2-3 hours

---

### 📊 Operations / DevOps Engineer

**Goal**: Understand how to operate, monitor, and troubleshoot the system

**Reading path** (1.5-2 hours):
1. SYSTEMS.md → "Tier 6-9" sections (AHI, CNV, clnrm, DFLSS)
2. DFLSS.md → Complete (this is your tool for continuous improvement)
3. SYSTEMS.md → "Cross-System Dependencies" and "Failure Scenarios & Recovery"
4. MU_KERNEL.md → "Timing Bounds" section (understand constraints)
5. API_SPECIFICATION.md → Section 7 (DFLSS API)

**Key operational tasks**:
- Monitor receipt graph Γ for anomalies (DFLSS inputs)
- Approve ΔΣ (delta) proposals from AHI (NOMRG.md)
- Monitor phase progression during canary deployments (DFLSS.md)
- Check timing bounds are maintained (MU_KERNEL.md)

**Runbooks**:
- "System is slow" → Run DFLSS optimization loop, see DFLSS.md "Example: Complete Loop"
- "Quota exceeded" → See SYSTEMS.md "Failure Scenarios: Quota Exceeded"
- "Code drifted from schema" → Run `cargo run --bin ggen regenerate`
- "Canary deployment failed" → Automatic rollback, see DFLSS.md phase progression

**Time**: ~1.5-2 hours

---

### 🧪 QA / Test Engineer

**Goal**: Understand how to test the system comprehensively

**Reading path** (1.5-2 hours):
1. CLNRM.md → Complete (your main tool)
2. CNV_PHASES_COMPLETE.md → Complete (understand what we test)
3. API_SPECIFICATION.md → Section 6 (clnrm API)
4. API_SPECIFICATION.md → Appendix "Common Patterns" (test patterns)

**Key testing principles**:
- All tests must be hermetic (CLNRM.md)
- All tests must verify determinism (CLNRM.md)
- Tests organized in CTT 13-phase pipeline (CNV_PHASES_COMPLETE.md)
- Property-based testing with proptest (mandatory)

**Testing patterns** (copy-paste ready):
- See API_SPECIFICATION.md "Appendix: Common Patterns"
- Pattern 1: Check-then-Act (quota pre-check)
- Pattern 2: Hermetic Testing (HermeticContainer)
- Pattern 3: Determinism Verification (verify_determinism)

**Benchmark targets**:
- μ-kernel: < 100ns latency (p99), 0% violations
- ggen: deterministic output, <10ns per frame
- CTT: 191 tests, 100% pass rate, >95% code coverage
- DFLSS: optimization cycles complete in <1 hour

**Time**: ~1.5-2 hours

---

### 📚 Documentation / Technical Writer

**Goal**: Understand what to document and how systems interact

**Reading path** (2-3 hours):
1. SYSTEMS.md → Complete
2. evidence_graph_extended.json → Browse to understand interconnections
3. API_SPECIFICATION.md → Complete (use as template for API docs)
4. PHILOSOPHY.md → Complete (reference for "why" behind designs)

**Documentation strategy**:
- Every new feature: add to API_SPECIFICATION.md
- Every new system: add to SYSTEMS.md tier diagram
- Every implementation: update evidence_graph_extended.json
- Every change: update KNHK.md or relevant tier doc

**Key docs to maintain**:
- API_SPECIFICATION.md (source of truth for APIs)
- SYSTEMS.md (system architecture diagram)
- evidence_graph_extended.json (knowledge graph of evidence)
- README.md → Point to READING_GUIDE.md for beginners

**Time**: ~2-3 hours to learn structure, then ongoing maintenance

---

## By Topic

### 🎯 Topic: "Code-as-Projection" (How code is generated)

**Essential reading**:
1. CODE_AS_PROJECTION.md → Complete
2. GGEN.md → Complete
3. PHILOSOPHY.md → "Code-as-Projection" section

**Key questions answered**:
- Q: "How do I edit code?" → A: Edit the ontology (Σ), not the code. See CODE_AS_PROJECTION.md workflow
- Q: "What prevents hand-edits?" → A: Pre-commit hook + CI checks. See CODE_AS_PROJECTION.md enforcement
- Q: "How are code, tests, docs kept in sync?" → A: All generated from same Σ. See GGEN.md
- Q: "Can I regenerate code?" → A: Yes, `cargo run --bin ggen regenerate`

**Time**: ~1 hour

---

### ⏱️ Topic: "Timing Physics & Determinism" (How timing guarantees work)

**Essential reading**:
1. MU_KERNEL.md → Complete
2. CLNRM.md → "Features" section
3. API_SPECIFICATION.md → Section 2.4 (TimingBound, TimingProof)

**Key questions answered**:
- Q: "What is Chatman constant?" → A: τ ≤ 100 nanoseconds. See MU_KERNEL.md
- Q: "Why is predictable timing important?" → A: Enables security proofs, see MU_KERNEL.md "Timing-Based Proofs"
- Q: "How is determinism verified?" → A: Property-based tests + timing measurements, see CLNRM.md
- Q: "What's the safety margin?" → A: <100ns limit, actual ~47ns observed, see MU_KERNEL.md benchmarks

**Time**: ~1.5 hours

---

### 🧬 Topic: "Ontology & Knowledge (KNHK)" (How the schema works)

**Essential reading**:
1. KNHK.md → Complete
2. PHILOSOPHY.md → "O/Σ/Q/ΔΣ Plane Model" section
3. API_SPECIFICATION.md → Section 1 (KNHK API)

**Key questions answered**:
- Q: "What is KNHK?" → A: Kinetic Knowledge Hypergraph (ontology as source of truth). See KNHK.md
- Q: "What's the difference between graph and hypergraph?" → A: Hypergraphs support n-ary relationships. See KNHK.md "Hypergraph semantics"
- Q: "How is versioning handled?" → A: Via ΔΣ (delta) overlays. See NOMRG.md + KNHK.md "Kinetic"
- Q: "How do workflows differ from capabilities?" → A: Workflows are projections of ontology. See PHILOSOPHY.md

**Time**: ~1.5 hours

---

### 🔄 Topic: "Conflict-Free Composition (nomrg)" (How parallel development works)

**Essential reading**:
1. NOMRG.md → Complete
2. SYSTEMS.md → "Cross-System Dependencies" and "nomrg Updates KNHK" section
3. API_SPECIFICATION.md → Section 4 (nomrg API)

**Key questions answered**:
- Q: "Why can't we just use git merges?" → A: Textual merges fail at trillion-agent scale. See NOMRG.md "The Problem"
- Q: "How do overlays avoid conflicts?" → A: Graph overlay algebra with formal proofs. See NOMRG.md "Overlay Algebra"
- Q: "What does ΔΣ ⊕ ΔΣ mean?" → A: Composition of ontology changes (overlay merging). See NOMRG.md
- Q: "Is composition commutative?" → A: Yes, by design. See NOMRG.md "Axioms"

**Time**: ~1.5 hours

---

### ✅ Topic: "Verification & Testing (CTT + clnrm)" (How quality is assured)

**Essential reading**:
1. CNV_PHASES_COMPLETE.md → Complete
2. CLNRM.md → Complete
3. VALIDATION_REPORT.md → Skim (proof that system works)
4. API_SPECIFICATION.md → Sections 5-6 (CTT and clnrm APIs)

**Key questions answered**:
- Q: "What are the 13 phases?" → A: 6 feature phases + 7 lifecycle phases. See CNV_PHASES_COMPLETE.md
- Q: "What is hermetic testing?" → A: Isolated tests with no external dependencies. See CLNRM.md
- Q: "How do we prove determinism?" → A: Property-based tests with proptest. See CLNRM.md
- Q: "What's the test pass rate?" → A: 191 tests, 100% pass. See VALIDATION_REPORT.md

**Time**: ~1.5-2 hours

---

### 🤖 Topic: "Autonomous Optimization (DFLSS + AHI)" (How system self-improves)

**Essential reading**:
1. DFLSS.md → Complete
2. SYSTEMS.md → "Tier 8-9" sections
3. API_SPECIFICATION.md → Sections 7-8 (DFLSS and AHI APIs)

**Key questions answered**:
- Q: "What is DFLSS?" → A: Design for Lean Six Sigma (autonomous optimization). See DFLSS.md
- Q: "What are the 5 phases?" → A: Define→Measure→Explore→Design→Implement. See DFLSS.md
- Q: "How does canary deployment work?" → A: 1% → 10% → 50% → 100% traffic. See DFLSS.md "Phase Progression"
- Q: "What triggers an optimization?" → A: AHI detects anomalies in Γ (receipt graph). See DFLSS.md workflow

**Time**: ~1.5 hours

---

## Document Map

```
Quick Start
├── PHILOSOPHY.md (5 min) ← START HERE
├── SYSTEMS.md (10 min)
└── This guide (5 min)

Theory & Design
├── PHILOSOPHY.md (complete)
├── KNHK.md (complete)
├── MU_KERNEL.md (complete)
├── NOMRG.md (complete)
└── evidence_graph_extended.json (reference)

Implementation & Specs
├── CODE_AS_PROJECTION.md
├── GGEN.md
├── API_SPECIFICATION.md
├── DFLSS.md
└── CLNRM.md

Operations & Verification
├── SYSTEMS.md (complete)
├── CNV_PHASES_COMPLETE.md
├── VALIDATION_REPORT.md
└── READING_GUIDE.md (you are here)

Supporting Files
├── concept_coverage.json (gap analysis)
├── concept_gaps.json (remediation roadmap)
├── concept_ruleset.yaml (evidence matching)
└── evidence_graph.json (original graph)
```

---

## Learning Paths by Experience Level

### 🟢 Beginner (No prior knowledge)

1. **Day 1** (1 hour):
   - PHILOSOPHY.md (complete)
   - SYSTEMS.md "Overview" section
   - READING_GUIDE.md "Quick Start"

2. **Day 2** (2 hours):
   - Pick your role above (Manager, Architect, Engineer, etc.)
   - Follow role-specific reading path

3. **Day 3+** (ongoing):
   - Deep-dive into topics of interest
   - Join team discussions with conceptual grounding

### 🟡 Intermediate (Familiar with some concepts)

1. **Day 1** (30 minutes):
   - Skim PHILOSOPHY.md and SYSTEMS.md
   - Check evidence_graph_extended.json for connections you care about

2. **Day 2** (1-2 hours):
   - Read API_SPECIFICATION.md for your component
   - Follow topic-specific paths for areas of confusion

3. **Day 3+**:
   - Read source code guided by spec
   - Implement or modify components

### 🔴 Advanced (Domain expert)

1. Start with:
   - API_SPECIFICATION.md (complete, all sections)
   - evidence_graph_extended.json (detailed interconnections)

2. Then focus on:
   - Implementation details in source code
   - GGEN.md (if implementing projection engine)
   - DFLSS.md (if integrating optimization)

---

## Recommended Reading Order

**Option A: Linear (Safest)**
1. PHILOSOPHY.md
2. SYSTEMS.md
3. KNHK.md
4. MU_KERNEL.md
5. NOMRG.md
6. CODE_AS_PROJECTION.md
7. GGEN.md
8. CLNRM.md
9. DFLSS.md
10. API_SPECIFICATION.md
11. Your role-specific path

**Option B: Depth-First (By role)**
1. PHILOSOPHY.md (10 min)
2. Your role reading path (1-4 hours)
3. API_SPECIFICATION.md (relevant sections)
4. Source code deep-dive

**Option C: Topic-Based (By interest)**
1. PHILOSOPHY.md (foundation)
2. Pick a topic above
3. Follow topic reading path
4. Jump to other topics as connected

---

## Cross-References Quick Index

| Question | Answer | Document |
|----------|--------|----------|
| What is the thesis? | A = μ(Σ) | PHILOSOPHY.md § 1 |
| How do systems interact? | 9-tier model | SYSTEMS.md overview |
| What is KNHK? | Kinetic Knowledge Hypergraph | KNHK.md § 1 |
| How fast is μ-kernel? | <100ns per operation | MU_KERNEL.md § "Timing Bounds" |
| How is code generated? | From ontology via ggen | GGEN.md § 1 |
| How are tests hermetic? | HermeticContainer isolation | CLNRM.md § 1 |
| How is optimization autonomous? | DFLSS 5-phase workflow | DFLSS.md § 1 |
| How are merges conflict-free? | nomrg overlay algebra | NOMRG.md § "Overlay Algebra" |
| API for executing code? | CNVRuntime.execute() | API_SPECIFICATION.md § 9 |
| Security guarantees? | See threat model | SYSTEMS.md § "Security Threat Model" |

---

## Feedback & Navigation Tips

- **Stuck?** Check the cross-reference index above
- **Need details?** Jump to API_SPECIFICATION.md
- **Want examples?** See evidence_graph_extended.json for concrete evidence
- **Implementing?** Follow API_SPECIFICATION.md § "Appendix: Common Patterns"
- **Teaching others?** Start with PHILOSOPHY.md + READING_GUIDE.md

---

## Next Steps After Reading

1. **If Manager**: Present "Value Proposition" section to leadership
2. **If Architect**: Design your component (see API_SPECIFICATION.md)
3. **If Engineer**: Pick a task from concept_gaps.json remediation roadmap
4. **If QA**: Write hermetic tests using CLNRM.md patterns
5. **If DevOps**: Monitor Γ (receipt graph) for DFLSS opportunities

---

## Glossary of Key Terms

| Term | Definition | Document |
|------|-----------|----------|
| **Σ (Sigma)** | Ontology (schema, types, capabilities, policies) | PHILOSOPHY.md |
| **μ (Mu)** | Kernel (deterministic execution engine) | MU_KERNEL.md |
| **O** | Observations (runtime events, metrics, receipts) | PHILOSOPHY.md |
| **Q** | Invariants (constraints, guards, rules) | PHILOSOPHY.md |
| **ΔΣ (Delta-Sigma)** | Ontology changes (overlays) | NOMRG.md |
| **Γ (Gamma)** | Receipt graph (audit trail, causal chain) | SYSTEMS.md |
| **A** | Application (running system, behavior) | PHILOSOPHY.md |
| **KNHK** | Kinetic Knowledge Hypergraph | KNHK.md |
| **DFLSS** | Design for Lean Six Sigma | DFLSS.md |
| **AHI** | Autonomic Hyper Intelligence | DFLSS.md, SYSTEMS.md |
| **CTT** | Chicago TDD Tools | CNV_PHASES_COMPLETE.md |
| **CNV** | clap-noun-verb (agent CLI) | SYSTEMS.md § Tier 7 |
| **ggen** | Graph generator (projection engine) | GGEN.md |
| **nomrg** | No-merge (conflict-free composition) | NOMRG.md |
| **clnrm** | Cleanroom (hermetic testing) | CLNRM.md |
| **τ (Tau)** | Chatman constant (timing bound, ≤100ns) | MU_KERNEL.md |

Good luck, and welcome to the graph-universe!
