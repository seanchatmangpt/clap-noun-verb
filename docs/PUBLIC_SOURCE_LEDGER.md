# Public Source Ledger

Repository of verified, publicly-licensed language and morphology datasets safe for reference integration.

**Last Updated:** 2026-06-02  
**Purpose:** Track license boundaries, attribution requirements, and safe reference patterns for external datasets.

---

## Open Scriptures Hebrew Bible (OSHB)

### Source Identity
- **Full Name:** Open Scriptures Hebrew Bible (OSHB) / morphhb
- **Repository:** https://github.com/openscriptures/morphhb
- **Maintainers:** Open Scriptures team
- **Primary Format:** OSIS XML + TSV lemma/morphology tables
- **Content Layer:** Hebrew Bible (Tanakh) with annotated morphology and lemma data

### License Terms

| Component | License | Attribution Required |
|-----------|---------|----------------------|
| Lemma data (Hebrew word roots) | CC BY 4.0 | Yes |
| Morphology codes (POS, gender, number, binyan) | CC BY 4.0 | Yes |
| OSIS XML structure & markup | CC BY 4.0 | Yes |
| Heritage Text (Hebrew text base) | Public Domain | Optional but recommended |

**License Text:** CC BY 4.0 requires attribution to:
- Open Scriptures organization
- Original data contributors (listed in repository)
- Link to license: https://creativecommons.org/licenses/by/4.0/

### Safe Reference Patterns

**✓ PERMITTED (Reference, do not copy):**
- Lemma inventory and mapping (e.g., "root צדק maps to righteousness domain")
- Morphology tag definitions and their semantics
- OSIS verse reference URNs (e.g., `urn:bible:wlc:Genesis.1.1`)
- Metadata about source provenance (version, revision date)
- Statistical summaries (e.g., "OSHB contains X lemmas, Y morphological tags")

**✗ PROHIBITED (Without full copy + attribution):**
- Verbatim copy of full Hebrew Bible text into this repository
- Unattributed lemma/morphology data dumps in artifacts
- Derivative works claimed as original without OSHB attribution

### Recommended Integration Pattern

**Do NOT vendor the full OSHB dataset.** Instead:

1. **Reference externally:**
   ```
   Parse from: https://github.com/openscriptures/morphhb/raw/master/wlc/
   Cache locally if needed (with version pin and attribution note)
   ```

2. **Citation in code:**
   ```rust
   // Lemma mapping sourced from Open Scriptures Hebrew Bible (OSHB)
   // CC BY 4.0: https://creativecommons.org/licenses/by/4.0/
   // Repository: https://github.com/openscriptures/morphhb
   ```

3. **In documentation:**
   ```markdown
   Morphology and lemma data: Open Scriptures Hebrew Bible (OSHB) 
   licensed under CC BY 4.0.
   ```

### Assessment

| Criterion | Status | Notes |
|-----------|--------|-------|
| Public License | ✓ PASS | CC BY 4.0 + PD |
| Source Accessible | ✓ PASS | GitHub public repository |
| Lemma Layer Safe | ✓ PASS | Reference-friendly, well-scoped |
| Morphology Safe | ✓ PASS | Metadata layer, not full text |
| Attribution Feasible | ✓ PASS | Clear provenance chain |
| Derivative Works Clear | ✓ PASS | License terms unambiguous |

**VERDICT: ALIVE** ✓

This source is **safe and recommended** for reference integration with proper attribution.

### References for Consumers

- **OSIS Standard:** https://www.crossway.com/web/osis/
- **Westminster Morphology:** https://github.com/openscriptures/hebrew-morphology
- **OSHB Documentation:** In-repository README and schema comments
- **Citation Example:** Groves, A., et al. "Open Scriptures Hebrew Bible." 2010–2026. CC BY 4.0.

---

## Gospel & Biblical Text RDF Patterns (Public Linked Data)

### Source Identity
- **Pattern Name:** Gospel Passage & Pericope Ontology
- **Vocabulary Basis:** W3C Standard (DCTERMS, BIBO, OWL, RDFS, PROV-O)
- **Namespace:** http://purl.org/ontology/bible/ (public, dereferenceable)
- **Use Case:** Public RDF patterns for modeling biblical texts, Gospel structure, passages, and synoptic relationships
- **Published By:** Linked Data community (standard vocabularies, not proprietary)

### Content Layers

| Layer | Vocabulary | Public | Reusable | Notes |
|-------|-----------|--------|----------|-------|
| **Gospel/Book** | BIBO + custom | ✓ | ✓ | `bibo:Book`, `bible:Gospel` |
| **Chapter** | BIBO + custom | ✓ | ✓ | `bibo:Chapter`, `bible:Chapter` |
| **Verse** | BIBO + custom | ✓ | ✓ | `bibo:Verse`, `bible:Verse` with numbering |
| **Pericope** | Custom (OWL-based) | ✓ | ✓ | Multi-verse thematic units via `owl:ObjectProperty` |
| **Passage** | DCTERMS | ✓ | ✓ | `dct:isPartOf`, `dct:hasPart`, `dct:relation` |
| **Author/Source** | DCTERMS + PROV-O | ✓ | ✓ | `dct:creator`, `prov:wasDerivedFrom` |
| **Synoptic Relations** | OWL | ✓ | ✓ | `owl:sameAs` for parallel passages across gospels |

### Safe Reference Patterns

**✓ PERMITTED (Reference and reuse):**
- Ontology structure and class definitions for Gospel, Chapter, Verse, Pericope
- Property definitions for hierarchical relationships (`dct:isPartOf`, `bible:startVerse`, `bible:endVerse`)
- RDF patterns for synoptic parallels (`owl:sameAs`)
- Source/tradition attribution patterns (DCTERMS + PROV-O)
- Example instantiations of Gospel references (e.g., "Mark 1:1")
- Use in domain-specific semantic composition (e.g., biblical exegesis CLI)

**✗ PROHIBITED (Without explicit attribution):**
- Verbatim copy of full Gospel texts into this repository (use external reference + link)
- Unattributed theological interpretation beyond ontology structure
- Claiming pattern innovation where patterns are directly from W3C standards

### Recommended Integration Pattern

**Do NOT embed full Gospel texts; reference external canonical sources:**

1. **RDF Pattern Definition (keep in repo):**
   ```
   /ontology/gospel-passage-pattern.ttl
   Contains: Class/property definitions, not text data
   ```

2. **Gospel Text Reference (external):**
   ```
   Greek New Testament: https://www.biblicalhumanities.org/
   English Standard Version: https://www.crossway.com/
   (with proper attribution and license compliance)
   ```

3. **Citation in code:**
   ```rust
   // Gospel RDF patterns based on W3C vocabularies (DCTERMS, BIBO, OWL, PROV-O)
   // Pericope modeling: /ontology/gospel-passage-pattern.ttl
   // Reference: Composite Gospel Index RDF patterns (public domain structures)
   ```

### Assessment

| Criterion | Status | Notes |
|-----------|--------|-------|
| Public Vocabulary | ✓ PASS | W3C standards (DCTERMS, BIBO, OWL, PROV-O) |
| Pattern Source Accessible | ✓ PASS | All vocabularies published and dereferenceable |
| Pericope Pattern Safe | ✓ PASS | Defined using standard OWL ObjectProperties, no proprietary vocab |
| Synoptic Relations Safe | ✓ PASS | Uses standard `owl:sameAs` for equivalence |
| Attribution Feasible | ✓ PASS | Credit to W3C vocabularies + Gospel traditions |
| No Vendor Lock-in | ✓ PASS | Open standards, fully reusable |
| Semantic Composability | ✓ PASS | Compatible with clap-noun-verb semantic domain modules |

**VERDICT: ALIVE** ✓

Public RDF patterns for Gospel structure are **safe, recommended, and fully reusable** with standard attribution.

### References for Consumers

- **DCTERMS Specification:** https://www.dublincore.org/specifications/dublin-core/dcterms/
- **BIBO Ontology:** http://purl.org/ontology/bibo/
- **OWL 2 Standard:** https://www.w3.org/TR/owl2-overview/
- **PROV-O Specification:** https://www.w3.org/TR/prov-o/
- **W3C Linked Data Platform:** https://www.w3.org/TR/ldp/
- **Gospel RDF Pattern File:** /ontology/gospel-passage-pattern.ttl

---

## Appendix: License Summary Table

| Source | Basis | Pericope Safe | Passage Safe | Author/Source | Synoptic | Verdict |
|--------|-------|---------------|--------------|---------------|----------|---------|
| OSHB | CC BY 4.0 + PD | N/A | ✓ | ✓ | ✓ | **ALIVE** |
| Gospel RDF Patterns | W3C Public | ✓ | ✓ | ✓ | ✓ | **ALIVE** |

---

**Legend:**  
- **ALIVE:** Approved for safe reference integration.
- **PARTIAL:** Use permitted with restrictions (noted above).
- **BLOCKED:** Do not use without explicit legal review.
