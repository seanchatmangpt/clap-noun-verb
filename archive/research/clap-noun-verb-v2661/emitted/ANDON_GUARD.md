# ANDON GUARD - CLAP NOUN VERB V2661 API TRUTH RESEARCH

**Workflow ID:** CLAP_NOUN_VERB_V2661_API_TRUTH_RESEARCH_001

**Date Issued:** 2026-06-01

**Effective Until:** EOL or next major version bump

---

## GUARD RULES (Non-Negotiable)

### Rule 1: No Stale API Assumptions
- Any agent claiming `#[noun]` or `#[verb]` macro capabilities without citing local **v26.6.1 proof** is marked **STALE_API_ASSUMPTION**
- Proof sources (in priority order):
  1. Local `/Users/sac/clap-noun-verb/Cargo.toml` version field
  2. Local `/Users/sac/clap-noun-verb/CHANGELOG.md` entry
  3. Local `/Users/sac/clap-noun-verb/src/lib.rs` or `/clap-noun-verb-macros/src/lib.rs`
  4. Integration tests in `/tests/`
  5. Examples in `/examples/`
  6. Public crates.io docs (only if version explicitly v26.6.1)

### Rule 2: No Implementation in Phase 0
- This research phase is **READ-ONLY**
- No commits, no code changes, no file writes
- Exception: This document itself (proof of research execution)

### Rule 3: Source-of-Truth Ordering (Immutable)
When API discrepancies occur, trust in this order:
1. **Local Cargo.toml** — Single source of package version truth
2. **Local Changelog** — Official history of changes
3. **Local Rustdoc comments** — Implementation intent
4. **Test files** — Actual behavior proof (AAA pattern required)
5. **Example files** — Documented usage patterns
6. **Public documentation** — Narrative aids only

### Rule 4: Mark All Assumptions
Any claim about clap-noun-verb capability that cannot be traced to v26.6.1 source code must be marked with flag:
```
⚠️ STALE_API_ASSUMPTION — Not verified in v26.6.1 source
```

---

## VERIFICATION PROTOCOL

### Phase 1: Version Truth (Completed)
- Extract version from Cargo.toml ✓
- Verify against Changelog ✓
- Confirm git commit hash ✓
- Map workspace structure ✓

### Phase 2: Macro API Mapping (Execute Next)
- Scan `#[verb]` macro definition in macros crate
- Document all syntactically valid forms
- Extract compile-time validation rules
- Capture return type requirements

### Phase 3: Trait Definitions (Execute Next)
- Map `VerbCommand`, `NounCommand`, `VerbContext`, `VerbArgs` types
- Document all required methods and their signatures
- Verify dyn-compatibility (sync only, no async in traits)

### Phase 4: Test Proof (Execute Next)
- Verify every claimed macro capability via test evidence
- Use integration tests as ground truth
- Flag any test marked `.disabled` or `.wip`

### Phase 5: Breaking Change Analysis (Execute Next)
- Compare v26.6.1 API against v26.6.0
- Document all breaking changes since last release
- Identify deprecations with migration paths

---

## INCIDENT FLAGS

If any agent claims one of these WITHOUT v26.6.1 proof, respond with the flag:

| Claim | Flag | Status |
|-------|------|--------|
| "#[noun] macro exists" | STALE_API_ASSUMPTION | VERIFY LOCALLY |
| "#[verb] macro exists and works" | STALE_API_ASSUMPTION | VERIFY LOCALLY |
| "Async functions in VerbCommand trait" | STALE_API_ASSUMPTION | CHECK `src/verb.rs` trait def |
| "RDF generation supported" | STALE_API_ASSUMPTION | CHECK feature gates |
| "Linkme distributed slices auto-discover" | STALE_API_ASSUMPTION | VERIFY `src/cli/registry.rs` |
| "v26.6.1 supports feature X" | STALE_API_ASSUMPTION | CHECK Cargo.toml features |

---

## RESEARCH BOUNDARIES

### IN SCOPE
- Macro definitions and syntax validation rules
- Public trait APIs and method signatures
- Test evidence of working functionality
- Feature flags and their dependencies
- Breaking changes since last version

### OUT OF SCOPE
- Performance benchmarks (may be stale)
- External crate comparisons (subjective)
- Future roadmap predictions (speculative)
- Unmerged branch contents (not v26.6.1)

---

## AUDITOR SIGN-OFF

**Research Phase:** Phase 0 (Andon Guard) ✓

**Executed By:** claude.ai/code (Haiku 4.5)

**Source Repo:** `/Users/sac/clap-noun-verb`

**Branch:** `minimalist-refactor-final` (ahead of main by 9 commits)

**Commit at research time:** `854735e` (feat(bin): add clap-noun-verb-gen CLI generator)

---

## NEXT STEPS

1. Read Phase 1 findings in `version-truth.yaml`
2. Begin Phase 2 macro API mapping
3. Cross-check all claims against local proofs
4. Flag any external docs that contradict local sources
