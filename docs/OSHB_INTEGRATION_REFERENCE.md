# OSHB Integration Reference

**Quick Integration Guide for Open Scriptures Hebrew Bible Morphology Layer**

---

## What is OSHB?

Open Scriptures Hebrew Bible (morphhb) is a public dataset containing:
- **Hebrew Bible text** in OSIS XML format
- **Lemma data** (Hebrew word roots with English glosses)
- **Morphology codes** (part-of-speech, gender, number, binyan—verb stems)
- **Westminster Hebrew Morphology** standard annotations

**License:** CC BY 4.0 (attribution required)  
**Repository:** https://github.com/openscriptures/morphhb  
**Status:** ✓ **ALIVE** — Safe for reference integration

---

## Do's and Don'ts

### ✓ DO (Safe & Encouraged)
- Reference OSHB morphology tags by name (e.g., "Noun masculine singular")
- Extract lemma definitions and map to semantic domains
- Link directly to OSHB GitHub repository
- Quote or cite OSHB documentation
- Use OSHB-derived morphology code definitions in your code

### ✗ DON'T (License Violation / Poor Practice)
- Copy full Hebrew Bible text into this repository
- Use OSHB data without attribution
- Claim OSHB data as original work
- Impose additional restrictions on OSHB-derived content
- Vendor entire OSHB dataset (reference externally instead)

---

## Attribution Template

**Use this in code comments:**

```rust
// Hebrew morphology reference:
// Open Scriptures Hebrew Bible (OSHB)
// CC BY 4.0: https://creativecommons.org/licenses/by/4.0/
// Repository: https://github.com/openscriptures/morphhb
// Westminster Morphology: https://github.com/openscriptures/hebrew-morphology
```

**Use this in documentation:**

```markdown
### Morphology Reference

Morphological definitions and lemma inventory sourced from:

**Open Scriptures Hebrew Bible (OSHB)**
- License: CC BY 4.0
- Repository: https://github.com/openscriptures/morphhb
- Attribution: https://creativecommons.org/licenses/by/4.0/
```

---

## Data Access Patterns

### Access Points

| Data | Format | Location |
|------|--------|----------|
| Hebrew text with morphology | OSIS XML | `https://github.com/openscriptures/morphhb/raw/master/wlc/` |
| Lemma inventory | TSV | `https://raw.githubusercontent.com/openscriptures/morphhb/master/morphhb/hebrew/lexicon.tsv` |
| Morphology codes | TTL/Documentation | `https://github.com/openscriptures/hebrew-morphology` |

### Recommended Approach

**Do NOT copy the full dataset into this repo.** Instead:

1. **Define an external resource reference:**
   ```rust
   const OSHB_REPO: &str = "https://github.com/openscriptures/morphhb";
   const OSHB_XML_URL: &str = "https://raw.githubusercontent.com/openscriptures/morphhb/master/wlc/";
   ```

2. **If caching locally, pin by commit hash:**
   ```rust
   // Cached from https://github.com/openscriptures/morphhb@abc123def456
   // as of 2026-06-02; see docs/OSHB_LICENSE_COMPLIANCE.md
   ```

3. **Parse only what you need:**
   ```rust
   // Extract morphology tags and lemma mappings from OSIS XML
   // Do not copy or transform the full Hebrew text
   ```

---

## Files in This Repository

| File | Purpose |
|------|---------|
| `docs/PUBLIC_SOURCE_LEDGER.md` | License ledger for all public sources (OSHB entry included) |
| `docs/OSHB_LICENSE_COMPLIANCE.md` | Detailed license compliance and integration rules |
| `docs/OSHB_INTEGRATION_REFERENCE.md` | This file—quick start guide |
| `ontology/oshb-reference.ttl` | Compact TTL reference (for ontology consumers) |
| `ontology/oshb-morphology-source.ttl` | Full RDF provenance graph |

---

## Examples

### Example 1: Referencing Morphology Codes

```rust
// Define morphology concept using OSHB source
#[derive(Clone)]
struct HebrewMorphology {
    // Source: Open Scriptures Hebrew Bible (OSHB)
    // CC BY 4.0: https://creativecommons.org/licenses/by/4.0/
    part_of_speech: HebrewPOS,
    gender: Option<Gender>,
    number: Number,
    binyan: Option<Binyan>, // For verbs
}

enum HebrewPOS {
    Noun,      // מִשְׁמָר (part of speech code: N)
    Verb,      // פָּקַד (POS: V)
    Adjective, // גָּדוֹל (POS: A)
    // ... mapped from OSHB codes
}
```

### Example 2: Mapping Lemmas to Semantic Domains

```rust
// Lemma inventory from OSHB
struct HebrewLemma {
    id: String,                // OSHB lemma ID
    root_form: String,         // e.g., "צדק"
    english_gloss: String,     // e.g., "to be righteous"
    semantic_domain: SemanticDomain, // Internal mapping
}

// Source attribution: All lemma data from Open Scriptures Hebrew Bible (OSHB)
// https://github.com/openscriptures/morphhb
// CC BY 4.0 Licensed
let lemmas = vec![
    HebrewLemma {
        id: "H6663".to_string(),
        root_form: "צדק".to_string(),
        english_gloss: "to be/act righteously".to_string(),
        semantic_domain: SemanticDomain::Justice,
    },
];
```

### Example 3: Parsing OSIS XML (External Reference)

```rust
// Do NOT copy OSHB's full OSIS XML into this repo.
// Instead, parse dynamically from:
// https://raw.githubusercontent.com/openscriptures/morphhb/master/wlc/

fn fetch_oshb_verse(book: &str, chapter: u32, verse: u32) -> Result<String> {
    // Parse from OSHB repository
    // Attribution: All text and morphology from OSHB
    // CC BY 4.0: https://creativecommons.org/licenses/by/4.0/
    let url = format!(
        "https://raw.githubusercontent.com/openscriptures/morphhb/master/wlc/{}.xml",
        book
    );
    // ... fetch and parse OSIS XML
    Ok(parsed_verse)
}
```

---

## License & Attribution

This guide is provided to ensure OSHB integration complies with CC BY 4.0.

**Summary:**
- ✓ Use OSHB data with proper attribution
- ✓ Link to https://github.com/openscriptures/morphhb
- ✓ Reference CC BY 4.0 license in code/docs
- ✓ Parse/cache externally, do not vendor full dataset
- ✗ Never use OSHB data without attribution
- ✗ Never claim OSHB work as original

---

## Resources

- **OSHB GitHub:** https://github.com/openscriptures/morphhb
- **CC BY 4.0 Summary:** https://creativecommons.org/licenses/by/4.0/
- **OSIS Standard:** https://www.crossway.com/web/osis/
- **Westminster Morphology:** https://github.com/openscriptures/hebrew-morphology

---

**Last Updated:** 2026-06-02  
**Prepared by:** Meremoth (OSHB Research Agent)
