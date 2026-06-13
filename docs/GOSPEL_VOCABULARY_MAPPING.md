# Gospel RDF Vocabulary Mapping Table

**Purpose:** Quick reference for mapping Composite Gospel Index (CGI) concepts to standard public RDF vocabularies.

**Status:** ALIVE ✓  
**Generated:** 2026-06-01  
**Source:** W3C Public Vocabularies (DCTERMS, BIBO, OWL, RDFS, PROV-O)

---

## Core Concepts → RDF Terms

| CGI Concept | Recommended Vocabulary | RDF Class/Property | Alternative | Usage Example |
|-------------|------------------------|-------------------|-------------|---|
| **Gospel (Book)** | BIBO | `bibo:Book` | `bible:Gospel` | `:GospelOfMark a bibo:Book` |
| **Chapter** | BIBO | `bibo:Chapter` | `bible:Chapter` | `:Mark_1 a bibo:Chapter ; dct:isPartOf :GospelOfMark` |
| **Verse** | BIBO | `bibo:Verse` | `bible:Verse` | `:Mark_1_1 a bibo:Verse ; dct:isPartOf :Mark_1` |
| **Verse Number (start)** | BIBO | `bibo:numericVerseStart` | — | `:Mark_1_1 bibo:numericVerseStart 1` |
| **Verse Number (end)** | BIBO | `bibo:numericVerseEnd` | — | `:Mark_1_1 bibo:numericVerseEnd 1` |
| **Pericope** | Custom (OWL-based) | `bible:Pericope` (ObjectProperty union) | DCTERMS `dct:hasPart` | `:Pericopa a bible:Pericope ; bible:startVerse :Mark_3_13 ; bible:endVerse :Mark_3_19` |
| **Passage / Range** | DCTERMS | `dct:isPartOf` | `dct:hasPart` | `:Verse dct:isPartOf :Chapter` |
| **Text Content** | Custom | `bible:content` | RDFS `rdfs:comment` | `:Mark_1_1 bible:content "Greek text"@el` |
| **Title / Label** | RDFS | `rdfs:label` | DCTERMS `dct:title` | `:Mark_1_1 rdfs:label "Mark 1:1"@en` |
| **Description** | DCTERMS | `dct:description` | RDFS `rdfs:comment` | `:Pericope dct:description "The Sermon on the Mount"@en` |
| **Creator / Author** | DCTERMS | `dct:creator` | PROV-O `prov:wasAttributedTo` | `:GospelOfMark dct:creator :MarkCommunity` |
| **Source / Tradition** | DCTERMS | `dct:source` | Custom `bible:source` | `:GospelOfMark dct:source :MarkanTradition` |
| **Date / Composition** | DCTERMS | `dct:date` | DCTERMS `dct:issued` | `:GospelOfMark dct:date "ca. 65-70 CE"@en` |
| **Language** | DCTERMS | `dct:language` | RDFS `rdfs:comment` | `:GospelOfMark dct:language "Koine Greek"@en` |
| **Synoptic Parallel** | OWL | `owl:sameAs` | DCTERMS `dct:relation` | `:Mark_1_1 owl:sameAs :Matthew_1_1 ; owl:sameAs :Luke_3_1` |
| **Related / Cross-Reference** | DCTERMS | `dct:relation` | OWL `owl:seeAlso` | `:Pericope dct:relation :ParallelPericope` |
| **Part-Of Hierarchy** | DCTERMS | `dct:isPartOf` | RDF/RDFS hierarchy | `:Verse dct:isPartOf :Chapter dct:isPartOf :Gospel` |
| **Has Part (Inverse)** | DCTERMS | `dct:hasPart` | — | `:Gospel dct:hasPart :Chapter dct:hasPart :Verse` |
| **Derivation / Source Chain** | PROV-O | `prov:wasDerivedFrom` | — | `:WrittenGospel prov:wasDerivedFrom :OralTradition` |
| **License** | DCTERMS | `dct:license` | — | `:GospelPattern dct:license <https://creativecommons.org/licenses/by/4.0/>` |
| **Ontology** | OWL | `owl:Ontology` | — | `:GospelOntology a owl:Ontology` |

---

## Vocabulary Namespace Declarations

```ttl
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix owl: <http://www.w3.org/2002/07/owl#> .
@prefix dct: <http://purl.org/dc/terms/> .
@prefix bibo: <http://purl.org/ontology/bibo/> .
@prefix prov: <http://www.w3.org/ns/prov#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix bible: <http://purl.org/ontology/bible/> .
```

---

## Usage Pattern Summary

### Hierarchical Structure (Parent-Child)
```ttl
:Gospel dct:hasPart :Chapter .
:Chapter dct:hasPart :Verse .

:Verse dct:isPartOf :Chapter .
:Chapter dct:isPartOf :Gospel .
```

### Pericope (Multi-Verse Thematic Unit)
```ttl
:Pericope a bible:Pericope ;
    bible:startVerse :Verse1 ;
    bible:endVerse :Verse5 ;
    dct:description "Thematic description"@en .
```

### Synoptic Relationships
```ttl
:Mark_1_1 owl:sameAs :Matthew_1_1 ;
         owl:sameAs :Luke_3_1 .

:MarkPassage dct:relation :MatthewPassage ;
            dct:relation :LukePassage .
```

### Authorial Attribution
```ttl
:Gospel dct:creator :Tradition ;
       prov:wasDerivedFrom :OralSource ;
       dct:date "ca. 65-70 CE"@en ;
       dct:license <CC-BY-4.0> .
```

---

## W3C Vocabulary Standards Referenced

| Vocabulary | Namespace | Use in Gospel Model |
|-----------|-----------|-------------------|
| **DCTERMS** | http://purl.org/dc/terms/ | Hierarchy, creator, date, language, description, license |
| **BIBO** | http://purl.org/ontology/bibo/ | Book, chapter, verse, numbering |
| **OWL** | http://www.w3.org/2002/07/owl# | Class definitions, objectProperties, sameAs equivalence |
| **RDFS** | http://www.w3.org/2000/01/rdf-schema# | Labels, comments, class hierarchy |
| **RDF** | http://www.w3.org/1999/02/22-rdf-syntax-ns# | Type declarations, basic RDF constructs |
| **PROV-O** | http://www.w3.org/ns/prov# | Derivation chains, agent attribution |
| **XSD** | http://www.w3.org/2001/XMLSchema# | Typed literals (xsd:date, xsd:gYear, xsd:string) |
| **Custom Bible** | http://purl.org/ontology/bible/ | Gospel, pericope (non-standard, OWL-based) |

---

## Property Type Quick Reference

### ObjectProperties (Link Resources)
- `dct:creator` → Agent
- `dct:source` → Source
- `dct:isPartOf` → Parent resource
- `dct:hasPart` → Child resource
- `dct:relation` → Related resource
- `owl:sameAs` → Equivalent resource
- `prov:wasDerivedFrom` → Source/origin
- `bible:startVerse` → Verse
- `bible:endVerse` → Verse

### DatatypeProperties (Literals)
- `rdfs:label` → String (language-tagged: `@en`, `@el`)
- `rdfs:comment` → String
- `dct:description` → String
- `dct:date` → String or Date
- `dct:language` → String
- `bibo:numericVerseStart` → Integer
- `bibo:numericVerseEnd` → Integer
- `bibo:numChapters` → Integer
- `bible:content` → String (multi-lingual via `@lang`)

---

## Examples

### Gospel Record
```ttl
:GospelOfMark
    a bibo:Book ;
    rdfs:label "Gospel of Mark"@en ;
    dct:creator :MarkCommunity ;
    dct:date "ca. 65-70 CE"@en ;
    dct:language "Koine Greek"@en ;
    dct:source :MarkanTradition ;
    prov:wasDerivedFrom :OralTradition ;
    bibo:numChapters 16 ;
    dct:license <https://creativecommons.org/licenses/by/4.0/> .
```

### Verse with Synoptic Parallel
```ttl
:Mark_1_1
    a bibo:Verse ;
    rdfs:label "Mark 1:1"@en ;
    dct:isPartOf :Mark_1 ;
    bibo:numericVerseStart 1 ;
    bibo:numericVerseEnd 1 ;
    bible:content "Ἀρχὴ τοῦ εὐαγγελίου Ἰησοῦ Χριστοῦ"@el ;
    owl:sameAs :Matthew_1_1 ;
    owl:sameAs :Luke_3_1 .
```

### Pericope
```ttl
:Mark_3_13_19
    a bible:Pericope ;
    rdfs:label "Calling of the Twelve"@en ;
    dct:description "Jesus appoints twelve apostles for his ministry"@en ;
    bible:startVerse :Mark_3_13 ;
    bible:endVerse :Mark_3_19 ;
    dct:subject "Apostolic Commission"@en ;
    dct:relation :Matthew_10_1 ;
    dct:relation :Luke_6_12 .
```

---

## Notes

- **Language Tags:** Always use `@en`, `@el` (Greek), `@he` (Hebrew) for multi-lingual content
- **Synoptic Equivalence:** Use `owl:sameAs` for verses/passages that are functionally equivalent across gospels
- **Pericope Semantics:** Pericope boundaries are thematic, not structural; reflect narrative or theological coherence
- **Attribution Chain:** Use `dct:creator` for immediate author, `prov:wasDerivedFrom` for source tradition
- **Licensing:** Always include `dct:license` when defining ontologies or models
- **Dereferenceable URIs:** Use `http://purl.org/` for vocabularies (resolves to W3C standards)

---

## See Also

- **Full Pattern Ontology:** `/ontology/gospel-passage-pattern.ttl`
- **Public Source Ledger:** `/docs/PUBLIC_SOURCE_LEDGER.md`
- **Gospel RDF Patterns Guide:** `/docs/GOSPEL_RDF_PATTERNS.md`
