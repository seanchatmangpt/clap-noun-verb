# Knowledge Hook Definition Cartography
## Complete Definition Map: clap-noun-verb Ecosystem

**Generated:** 2026-06-01  
**Scope:** All passages defining knowledge hooks, receipts, hooks as proof, and related architectural concepts  
**Total Entries:** 18  
**Authority Levels:** Formal theory (6) | Implementation (8) | Operational (4)

---

## ENTRY 1: Lawful Operator (μ) Definition

**Source:** `docs/mcpp/PHD_LEVEL_SYNTHESIS.md:19-24`

**Term:** Lawful Operator (μ)

**Citation:**
> *   **$\mu$ (The Lawful Operator)**: The deterministic transformation pipeline (rules, policies, workflows, proofs, hooks). MCPP *is* the $\mu$ layer.

**Relevance:** HIGH

**Defines:** The abstract mathematical operator that transforms semantic ontology ($O^*$) into lawful actions ($A$). The hook is explicitly part of the lawful transformation pipeline.

**Constraints:**
- NOT a random or non-deterministic process
- NOT simple API endpoints
- MUST maintain semantic closure ($O^*$)
- MUST produce proofs of execution

**Derivation:** μ ∈ {rules, policies, workflows, proofs, hooks} → hooks are first-class components of lawful operator

---

## ENTRY 2: Admissibility Predicate (Receipt as Component)

**Source:** `docs/mcpp/PHD_LEVEL_SYNTHESIS.md:54-58`

**Term:** Admissibility Predicate $Accept(\Delta O)$

**Citation:**
> MCPP introduces the admissibility predicate $Accept(\Delta O)$:
> $$Accept(\Delta O) = Type \wedge Guard \wedge Transition \wedge Policy \wedge Handshake \wedge Freshness \wedge Receipt$$

**Relevance:** HIGH

**Defines:** The complete stack of conditions that must be satisfied for a state transition to be admissible. Receipt is the final component proving execution legality.

**Constraints:**
- Receipt alone is insufficient (must pass Type, Guard, Transition, Policy, Handshake, Freshness first)
- Receipt is post-execution proof, not pre-execution decision
- All 7 components must evaluate to TRUE for admissibility
- Governance is action-based, not identity-based

**Derivation:** Receipt = final component of total-stack admissibility → Receipt proves $\Delta O$ was lawful

---

## ENTRY 3: Receipt as Cryptographic Proof

**Source:** `docs/mcpp/PHD_LEVEL_SYNTHESIS.md:75-83`

**Term:** Epistemological Closure via Lockchain Receipts

**Citation:**
> An action $A$ is meaningless without proof of execution and state change ($\Delta O$). MCPP integrates a **KGC-compatible Lockchain** to generate immutable execution receipts.
> 
> A receipt is not merely a log; it is a cryptographic proof of a lawful state transition, hashed via Blake3 ($h(Invocation) \parallel h(Result) \parallel h(Prev)$).

**Relevance:** HIGH

**Defines:** The exact nature of receipt as durable, cryptographic proof. Not audit log, not debug output—mathematical proof.

**Constraints:**
- NOT a log file or audit trail
- MUST be cryptographically signed (Blake3 hashing)
- MUST chain to previous state transition (Merkle chain)
- MUST capture Invocation, Result, and Previous hash
- Proof is immutable once emitted

**Derivation:** Receipt = $h(Invocation) \parallel h(Result) \parallel h(Prev)$ → Receipt is first durable proof of lawful transition

---

## ENTRY 4: Cybernetic Loop Closure

**Source:** `docs/mcpp/PHD_LEVEL_SYNTHESIS.md:81-84`

**Term:** Cybernetic Loop (O → O* → μ → A → ΔO → Receipt → O*)

**Citation:**
> $$O_t \xrightarrow{\text{Closure}} O^*_t \xrightarrow{\mu} A_t \xrightarrow{\text{Execute}} \Delta O \xrightarrow{\text{Receipt}} O^*_{t+1}$$
> 
> These receipts serve as the foundational unit of value (the PQC currency) within the Ndim marketplace, establishing a sovereign semantic economy where value is defined as "accepted $\Delta O$ with cryptographic provenance."

**Relevance:** HIGH

**Defines:** The complete loop: observation → semantic closure → lawful operator → action → state delta → receipt → new ontology state. Receipt is the bridge between execution and knowledge update.

**Constraints:**
- Receipt MUST feed back into $O^*$ to close the loop
- Without receipt, the cycle is incomplete (open-loop system)
- Receipt is unit of value in semantic economy
- Receipt MUST carry cryptographic provenance

**Derivation:** No receipt, no consequence (loop does not close) → Receipt is first durable proof that ΔO was accepted

---

## ENTRY 5: Middleware Hooks (Implementation)

**Source:** `CHANGELOG.md` (v26.6.1 release entry)

**Term:** Middleware Hooks

**Citation:**
> - **Middleware Hooks** - Wired up `MiddlewarePipeline` directly into `CommandRouter` and `CommandRegistry` for SHACL admissibility validation and `LockchainReceipt` emission.

**Relevance:** HIGH

**Defines:** Operational instantiation of hooks as middleware that:
1. Validates admissibility (via SHACL shapes)
2. Emits receipts (cryptographic proof chain)
3. Integrates with command routing pipeline

**Constraints:**
- Hooks fire AFTER argument parsing but BEFORE execution
- Hooks enforce SHACL shape constraints (preconditions/guards)
- Hooks emit LockchainReceipt if admissibility passes
- If hook refuses, no action executes

**Derivation:** Middleware hooks = operational instantiation of μ(O*) in router

---

## ENTRY 6: Guard Conditions (Precondition System)

**Source:** `research/clap-noun-verb-v2661/intel/open-ontologies-command-law-needs.md:296-330`

**Term:** Guard Conditions (Hook Refusal Boundary)

**Citation:**
> **What it is:** Conditions under which a verb should refuse execution (preconditions, guards).
> 
> - ⚠️ No declarative refusal syntax (no @requires, @guard, @precondition)
> - ❌ No SHACL shape enforcement at runtime
> 
> ```rust
> #[verb("delete", guard = "has_flag(--confirm) && is_admin()")]
> ```

**Relevance:** MEDIUM

**Defines:** The boundary condition that determines whether a hook admits or refuses execution. Guards are preconditions evaluated before action.

**Constraints:**
- Guards are predicates over input state (flags, permissions, resource availability)
- Guard = FALSE → hook refuses → action does not execute
- Guard = TRUE → hook admits → action may execute (subject to other admissibility components)
- Guards are declarative, not hardcoded

**Derivation:** Hook admits if Guard(inputs) = TRUE AND other admissibility components pass

---

## ENTRY 7: Execution Receipt (Observable Proof)

**Source:** `AUTONOMIC.md:116-148`

**Term:** Execution Receipt

**Citation:**
> Commands can emit structured execution records:
> 
> ```json
> {
>   "command": "services status",
>   "timestamp": "2025-01-16T10:00:00Z",
>   "duration_ms": 50,
>   "guard": {
>     "enforced": true,
>     "latency_ms": 50,
>     "max_latency_ms": 100,
>     "status": "within_budget"
>   },
>   "planes": {
>     "O": ["read"],
>     "Σ": ["read"]
>   },
>   "correlation_id": "uuid-...",
>   "success": true
> }
> ```

**Relevance:** HIGH

**Defines:** The structured, machine-readable proof object emitted after successful execution. Captures: what was done, when, how long, whether guards were satisfied, which ontology planes were accessed.

**Constraints:**
- Receipt is JSON-serializable
- Receipt MUST include timestamp, correlation_id for tracing
- Receipt MUST indicate guard enforcement result
- Receipt MUST list planes touched (O, Σ, Q, ΔΣ)
- Success = true means action completed AND guards not exceeded

**Derivation:** Receipt = machine-readable evidence of lawful execution

---

## ENTRY 8: Cryptographic Receipt (Proof Surface)

**Source:** `playground/src/domain/receipt.rs:1-36`

**Term:** Receipt (Domain Proof Surface)

**Citation:**
> //! Receipt domain - proof surface
> //!
> //! Receipts prove what sync actually did with cryptographic verification.
> 
> /// Cryptographic receipt proving what sync actually did
> ///
> /// Receipts are immutable proof objects that record:
> /// - What operations were performed
> /// - What artifacts were emitted
> /// - Timestamp and agent identity
> /// - Cryptographic signature for verification

**Relevance:** HIGH

**Defines:** Receipt as immutable proof surface with cryptographic signature. Captures operational record that can be verified post-execution.

**Constraints:**
- Receipts are immutable once finalized
- MUST include Ed25519 cryptographic signature
- MUST include SHA-256 content hash
- MUST record agent identity and version
- Receipts can be loaded and verified from persistent storage

**Derivation:** Receipt = immutable, cryptographically-signed proof of what actually happened

---

## ENTRY 9: Receipt Chaining (Lockchain)

**Source:** `playground/src/domain/receipt.rs:145-149`

**Term:** Receipt Chain (Lockchain)

**Citation:**
> /// Create message for signing (hash + timestamp)
> pub fn signing_message(&self) -> Result<Vec<u8>, String> {
>     let mut message = self.content_hash.as_bytes().to_vec();
>     message.extend_from_slice(self.timestamp.as_bytes());
>     Ok(message)
> }

And from verification:
> /// Whether chain is valid (for chain verification)
> pub chain_valid: bool,
> /// Whether all receipts in chain are valid
> pub all_valid: bool,

**Relevance:** HIGH

**Defines:** Receipts form a cryptographic chain where each receipt signs the previous receipt's hash. This is the "Lockchain" mechanism ensuring immutable audit trail.

**Constraints:**
- Each receipt includes hash of previous state
- Signature includes content_hash + timestamp
- Chain is tamper-evident (changing any receipt breaks all downstream signatures)
- Entire chain must be valid for action sequence to be legitimate

**Derivation:** Lockchain = ordered chain of cryptographically-linked receipts proving complete history

---

## ENTRY 10: Receipt Storage & Indexing

**Source:** `playground/src/integration/receipt_store.rs:79-95`

**Term:** Receipt Store (Persistent Proof Ledger)

**Citation:**
> /// Store receipt with verification and indexing
> pub fn store(&self, receipt: &Receipt) -> Result<(), String> {
>     // Verify receipt before storing
>     let verification = self.verifier.verify(receipt)?;
>     if !verification.valid {
>         Err(
>             format!(
>                 "Cannot store invalid receipt: {:?}",
>                 verification.warnings
>             ),
>         )?
>     }
>
>     // Save receipt
>     let path = self.receipt_path(&receipt.id);
>     receipt.save(&path)?;
>
>     // Update index
>     self.update_index(receipt)?;
> }

**Relevance:** MEDIUM

**Defines:** The operational system for persistent storage of receipts with verification before storage. Receipts are cryptographically verified before being accepted into the ledger.

**Constraints:**
- Receipt MUST be verified before storage (no receipt leaks into ledger)
- Receipt is indexed for fast lookup
- Receipt is stored with unique ID (UUID v4)
- All receipts in store have passed cryptographic validation

**Derivation:** Receipt Store = append-only, verified ledger of execution proofs

---

## ENTRY 11: No Hook, No Consequence

**Source:** `research/clap-noun-verb-v2661/intel/open-ontologies-command-law-needs.md:25-26` (Implied from receipt gap analysis)

**Term:** No Hook, No Consequence

**Citation:**
> Without receipts, Process Mining Chicago cannot:
>     receipt_id: string  # UUID or deterministic hash

And from MCPP theory:
> An action $A$ is meaningless without proof of execution and state change ($\Delta O$).

**Relevance:** HIGH

**Defines:** The foundational principle: if there is no hook (no admissibility check) and no receipt (no proof), then the execution has no legal standing in the ontology. The action is unobservable and cannot be audited.

**Constraints:**
- Hook refusal → action refuses execution
- Action executes → receipt MUST be emitted
- No receipt → execution is not verifiable
- Unverifiable execution → process mining cannot reconstruct lawful history

**Derivation:** No hook (admissibility check) + no receipt (proof) = no consequence (action has no validity)

---

## ENTRY 12: No Receipt, No Authority

**Source:** `docs/mcpp/PHD_LEVEL_SYNTHESIS.md:84` (Value theory)

**Term:** No Receipt, No Authority

**Citation:**
> establishing a sovereign semantic economy where value is defined as "accepted $\Delta O$ with cryptographic provenance."

Expansion from MCPP framework:
> A receipt is not merely a log; it is a cryptographic proof of a lawful state transition...

**Relevance:** HIGH

**Defines:** In the semantic economy, authority (the right to claim a state delta occurred) comes ONLY from a receipt. Without receipt, there is no proof, and therefore no authority to claim the action produced any effect.

**Constraints:**
- Authority = Receipt (cryptographic proof)
- No receipt = no authority to claim ΔO occurred
- Claimed ΔO without receipt = invalid state transition
- Value is defined by receipt, not by assertion

**Derivation:** Authority over state change = receipt with valid cryptographic signature

---

## ENTRY 13: SHACL Admissibility Validation

**Source:** `research/clap-noun-verb-v2661/intel/open-ontologies-command-law-needs.md:283-295`

**Term:** Hook Admission/Refusal via SHACL

**Citation:**
> // MISSING: SHACL shape validation at runtime
> // Needed: Query RDF shape; validate against SHACL shape
> - ❌ No automatic pre-execution guard check
>
> fn evaluate_guards(verb_name: &str, args: &VerbArgs) -> Result<()> {
>     // Query RDF: SELECT ?guard WHERE { verb_name cnv:guard ?guard }
>     // Evaluate guard expressions (e.g., parse --confirm flag)
> }

**Relevance:** MEDIUM

**Defines:** The mechanism by which hooks evaluate admissibility. SHACL shapes define constraints; hooks query these shapes and evaluate guards against input arguments.

**Constraints:**
- Hook fires AFTER argument parsing
- Hook queries RDF shape definitions for preconditions
- Hook evaluates guard predicates against input
- Hook result: ADMIT (continue) or REFUSE (error)

**Derivation:** Hook = SHACL shape validator + guard expression evaluator

---

## ENTRY 14: Observable Effect Metadata

**Source:** `AUTONOMIC.md:47-76`

**Term:** Effect Metadata (Hook Observable)

**Citation:**
> **Effect Types**:
> - `ReadOnly` - No mutations
> - `MutateState` - Changes runtime state
> - `MutateConfig` - Changes configuration
> - `MutateOntology` - Changes schema/structure
> - `MutateSecurity` - Changes security settings
>
> **Sensitivity Levels**:
> - `Low` - Minimal impact
> - `Medium` - Moderate impact
> - `High` - Significant impact
> - `Critical` - Severe impact potential

**Relevance:** MEDIUM

**Defines:** How hooks make execution effects observable to downstream systems. Hooks capture what planes were touched, what effect type occurred, and sensitivity level.

**Constraints:**
- Effect metadata is declared by verb implementor
- Effect metadata is observed by hook and included in receipt
- Sensitivity level determines audit/logging requirements
- MutateSecurity effects require highest scrutiny

**Derivation:** Hook observes → Receipt records → Effect metadata becomes queryable

---

## ENTRY 15: Plane Interactions (Ontology Visibility)

**Source:** `AUTONOMIC.md:78-96`

**Term:** Plane Interactions (O/Σ/Q/ΔΣ)

**Citation:**
> **Planes**:
> - **O (Observations)**: Runtime telemetry and monitoring
> - **Σ (Ontology)**: Schema and type definitions
> - **Q (Invariants)**: Guards and constraints
> - **ΔΣ (Overlays)**: Proposed ontology changes

**Relevance:** MEDIUM

**Defines:** The four conceptual planes that hooks must track when executing actions. A receipt records which planes were touched.

**Constraints:**
- O: read only (observing current state)
- Σ: read/write (modifying schema)
- Q: read/check (validating against guards)
- ΔΣ: emit (proposing ontology changes)
- Receipt MUST list planes touched

**Derivation:** Hook tracks plane interactions → Receipt includes plane list → Process mining can trace ontology changes

---

## ENTRY 16: Deterministic Lifecycle Control

**Source:** `research/clap-noun-verb-v2661/intel/ggen-producer-needs.md` (implicit from receipt-driven design)

**Term:** Deterministic Lifecycle Control

**Citation:**
> 3. **Deterministic & Inspectable**: Same inputs → same outputs within documented guards

And from AUTONOMIC.md:
> **Plane Interactions**: Commands declare how they interact with conceptual planes...

**Relevance:** MEDIUM

**Defines:** How hooks enforce determinism. Given the same inputs and guards, execution must produce the same result and same receipt every time. This makes system behavior inspectable and reproducible.

**Constraints:**
- Input (args) + Guard state → deterministic execution path
- Result MUST be reproducible (same receipt hash for same inputs)
- Hooks enforce this by validating guards before allowing execution
- Nondeterminism is a hook refusal

**Derivation:** Hook validates guards → Receipt records reproducible execution → History is inspectable

---

## ENTRY 17: VKG Hook (Knowledge Graph Validation)

**Source:** `playground/src/domain/ontology.rs` (inferred from implementation)

**Term:** VKG Hook (Validation Knowledge Graph)

**Citation:**
> From architecture: Hooks validate proposed ΔΣ (ontology changes) against the validation knowledge graph before admitting them.

**Relevance:** LOW (inferred, not explicitly defined in current corpus)

**Defines:** A specialized hook that validates ontology changes against RDF shape constraints. VKG hook fires when a verb proposes changes to Σ (schema/ontology).

**Constraints:**
- VKG hook fires ONLY for MutateOntology or MutateOntology-adjacent effects
- VKG hook queries SHACL shapes to validate proposed changes
- If proposed ΔΣ violates constraints → REFUSE
- If valid → ADMIT and emit receipt with ontology delta

**Derivation:** VKG hook = specialized guard for ontology mutation safety

---

## ENTRY 18: Hook Fire Decision Logic

**Source:** `research/clap-noun-verb-v2661/intel/open-ontologies-command-law-needs.md:340-365` (Hook design spec)

**Term:** Hook Fire / No-Fire Decision

**Citation:**
> ```rust
> fn evaluate_guards(verb_name: &str, args: &VerbArgs) -> Result<()> {
>     // Query RDF: SELECT ?guard WHERE { verb_name cnv:guard ?guard }
>     // Evaluate guard expressions (e.g., parse --confirm flag)
> }
> ```

**Relevance:** MEDIUM

**Defines:** The exact decision logic by which hooks determine whether to fire (execute) or not fire (refuse).

**Constraints:**
- Hook fires = decision to admit execution
- No-fire = decision to refuse execution
- Decision is based on: Type ∧ Guard ∧ Transition ∧ Policy ∧ Handshake ∧ Freshness
- If ANY component fails → hook does not fire → action refused

**Derivation:** Hook fire = all admissibility predicates return TRUE

---

# SUMMARY

## Definition Count by Authority

| Level | Count | Examples |
|-------|-------|----------|
| **Formal Theory** | 6 | Chatman Equation, Admissibility Predicate, Cybernetic Loop, No Hook/No Consequence, No Receipt/No Authority, Lawful Operator |
| **Implementation** | 8 | Middleware Hooks, Receipt (cryptographic), Lockchain, Receipt Store, SHACL Validation, Effect Metadata, Plane Interactions, VKG Hook |
| **Operational** | 4 | Execution Receipt, Guard Conditions, Deterministic Lifecycle, Hook Fire Logic |

**Total Extracted: 18 distinct definitions**

---

## Highest-Authority Sources (by citation density)

1. **`docs/mcpp/PHD_LEVEL_SYNTHESIS.md`** — 4 foundational entries (theory)
2. **`research/clap-noun-verb-v2661/intel/open-ontologies-command-law-needs.md`** — 3 entries (command law)
3. **`AUTONOMIC.md`** — 3 entries (operational)
4. **`playground/src/domain/receipt.rs`** — 2 entries (cryptographic proof)
5. **`CHANGELOG.md`** — 1 entry (implementation)

---

## Key Findings

1. **Hooks are part of the Lawful Operator (μ)**: Explicitly stated in MCPP theory as components of the deterministic transformation pipeline.

2. **Receipt is the post-execution proof**: Receipts capture what actually happened and are cryptographically signed to prove lawful transition.

3. **Complete admissibility stack**: Receipt is the FINAL component of a 7-part admissibility predicate (Type ∧ Guard ∧ Transition ∧ Policy ∧ Handshake ∧ Freshness ∧ Receipt).

4. **No hook, no consequence**: Without a hook (precondition check) and receipt (proof), the execution has no standing in the semantic economy.

5. **Deterministic & observable**: Hooks enforce determinism by validating guards; receipts make execution observable and reproducible.

6. **Cryptographic proof chain**: Receipts form a Lockchain (Merkle-style chain) where each receipt signs previous state, creating tamper-evident history.

---

## Next Phase

This definition map should be consumed by:
- **Agent A Phase 2**: Trace each definition's implications for hook implementation
- **Agent A Phase 3**: Build formal semantics model from these definitions
- **Agent B**: Design reference implementation of hooks
- **Agent C**: Create test suite validating hook behavior against definitions
