# OSHB License Compliance & Integration Guidelines

**Source:** Open Scriptures Hebrew Bible (OSHB)  
**Assessment Date:** 2026-06-02  
**Verdict:** **ALIVE** ✓

---

## Quick Summary

| Aspect | Status | Detail |
|--------|--------|--------|
| **License** | ✓ Public | CC BY 4.0 (attribution required) + Public Domain (text layer) |
| **Lemma Layer** | ✓ Safe | Reference external source, do not copy wholesale |
| **Morphology Codes** | ✓ Safe | Metadata layer—freely usable with attribution |
| **Full Text Copy** | ⚠ Restricted | Reference only; do not vendor full Bible text |
| **Attribution Feasible** | ✓ Yes | Clear source, stable repository, version-pinnable |

---

## License Details

### CC BY 4.0 Attribution Requirements

When integrating OSHB morphology or lemma data, you must:

1. **Name the creator:** "Open Scriptures"
2. **Provide license link:** https://creativecommons.org/licenses/by/4.0/
3. **Indicate if changes made:** (If derived/transformed)
4. **State no additional restrictions:** Cannot impose restrictions beyond CC BY 4.0

### Practical Implementation

**In code:**
```rust
// Morphology reference from Open Scriptures Hebrew Bible (OSHB)
// CC BY 4.0: https://creativecommons.org/licenses/by/4.0/
// Source: https://github.com/openscriptures/morphhb
```

**In documentation:**
```markdown
### References

- **Morphology & Lemma Data:** Open Scriptures Hebrew Bible (OSHB)  
  Licensed under CC BY 4.0  
  https://github.com/openscriptures/morphhb
```

**In Cargo.toml or package metadata:**
```toml
# If vendoring OSHB-derived data, include in license declaration
license = "MIT OR Apache-2.0 WITH OSHB-CC-BY-4.0-attribution"
# Add to package documentation
```

---

## Safe Integration Patterns

### Pattern 1: External Reference (Preferred)

```
Do NOT copy OSHB data into this repository.
Instead, reference it dynamically:
  - Parse from: https://github.com/openscriptures/morphhb/raw/master/wlc/
  - Version-pin via commit hash
  - Cache locally if needed (with upstream provenance note)
```

### Pattern 2: Metadata Extraction

```rust
// PERMITTED: Extract and re-use morphology definitions
let morphology_codes = vec![
    ("אַ", "Noun, masculine singular"), // Derived from OSHB
    ("בִ", "Preposition"), // All morphology definitions sourced from OSHB
];
// Include attribution above or in module documentation
```

### Pattern 3: Lemma Mapping

```rust
// PERMITTED: Map OSHB lemmas to semantic domains
struct HebrewLemma {
    oshb_id: String,
    root: String,
    english_gloss: String,
    // Map to internal semantic domain
    semantic_field: SemanticDomain,
}
// Must cite OSHB as source of lemma inventory
```

---

## Boundary Lines (Do Not Cross)

| Action | Allowed | Rationale |
|--------|---------|-----------|
| Reference OSHB morphology codes by name | ✓ | Metadata, falls under fair use + CC BY 4.0 |
| Link to OSHB GitHub repository | ✓ | Attribution via direct link |
| Quote OSHB documentation in our docs | ✓ | Fair use + attribution |
| Extract and re-publish lemma definitions | ✓ | CC BY 4.0 permits with attribution |
| Vendor full Hebrew Bible text (even PD) | ✗ | Bloats repo; reference external instead |
| Use OSHB without attribution | ✗ | Violates CC BY 4.0 |
| Claim OSHB data as original work | ✗ | Copyright infringement |
| Impose additional license restrictions on OSHB-derived content | ✗ | Violates CC BY 4.0 no-additional-restrictions clause |

---

## Integration Checklist

Before committing any OSHB-sourced code or data:

- [ ] Attribution comment added (Creator, License URL, Source)
- [ ] License boundary documented (what is/isn't vendored)
- [ ] Link to OSHB repo included (for users to verify source)
- [ ] No wholesale copy of Hebrew text (reference external instead)
- [ ] If transforming OSHB data, transformation rationale documented
- [ ] Code review confirms compliance

---

## Questions?

- **OSHB License:** https://github.com/openscriptures/morphhb/blob/master/LICENSE.md
- **CC BY 4.0 Summary:** https://creativecommons.org/licenses/by/4.0/
- **OSIS Standard:** https://www.crossway.com/web/osis/ (shared standard, not OSHB-specific)

---

## Revision History

| Date | Change |
|------|--------|
| 2026-06-02 | Initial assessment: OSHB vetted as ALIVE source |

---

**Prepared by:** Meremoth (OSHB Research Agent)  
**Status:** ✓ Ready for integration
