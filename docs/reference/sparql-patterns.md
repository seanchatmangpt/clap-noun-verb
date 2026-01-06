# Reference: SPARQL Query Patterns

Complete reference of SPARQL patterns used in clap-noun-verb CLI development.

## Basic SELECT Patterns

### Pattern: List All Resources of Type

```sparql
SELECT ?resource WHERE {
    ?resource a cnv:Noun .
}
```

Returns all nouns in the ontology.

### Pattern: Get Resource Properties

```sparql
SELECT ?property ?value WHERE {
    cnv:Services ?property ?value .
}
```

Returns all properties of the Services noun.

### Pattern: Filter by Property Value

```sparql
SELECT ?name WHERE {
    ?verb a cnv:Verb ;
          cnv:name ?name ;
          cnv:hasNoun cnv:Services .
}
```

Returns names of all verbs under Services.

## JOIN Patterns

### Pattern: Join Nouns and Verbs

```sparql
SELECT ?nounName ?verbName WHERE {
    ?noun a cnv:Noun ;
          cnv:name ?nounName .

    ?verb a cnv:Verb ;
          cnv:hasNoun ?noun ;
          cnv:name ?verbName .
}
ORDER BY ?nounName ?verbName
```

Classic two-table join showing noun-verb relationships.

### Pattern: Multi-level JOIN

```sparql
SELECT ?nounName ?verbName ?handlerName WHERE {
    ?noun a cnv:Noun ; cnv:name ?nounName .
    ?verb a cnv:Verb ;
          cnv:hasNoun ?noun ;
          cnv:name ?verbName ;
          cnv:handler ?handlerName .
}
```

Three-way relationship: Noun → Verb → Handler.

## OPTIONAL Patterns

### Pattern: Optional Properties

```sparql
SELECT ?name ?description WHERE {
    ?verb a cnv:Verb ;
          cnv:name ?name .
    OPTIONAL { ?verb cnv:description ?description }
}
```

Description is optional; returns all verbs with or without descriptions.

### Pattern: Coalesce Missing Values

```sparql
SELECT ?name (COALESCE(?description, "No description") as ?desc) WHERE {
    ?verb a cnv:Verb ;
          cnv:name ?name .
    OPTIONAL { ?verb cnv:description ?description }
}
```

Provide default value when property is missing.

## FILTER Patterns

### Pattern: String Matching

```sparql
SELECT ?name WHERE {
    ?verb a cnv:Verb ; cnv:name ?name .
    FILTER (STRSTARTS(?name, "st"))
}
```

Find verbs starting with "st" (case-sensitive).

### Pattern: Case-Insensitive Regex

```sparql
SELECT ?name WHERE {
    ?verb a cnv:Verb ; cnv:name ?name .
    FILTER (REGEX(?name, "^st", "i"))
}
```

Find verbs starting with "st" (case-insensitive).

### Pattern: Contain Check

```sparql
SELECT ?name WHERE {
    ?verb a cnv:Verb ; cnv:name ?name .
    FILTER (CONTAINS(?name, "serve"))
}
```

Find verbs containing "serve".

### Pattern: Length Check

```sparql
SELECT ?name WHERE {
    ?verb a cnv:Verb ; cnv:name ?name .
    FILTER (STRLEN(?name) > 3 && STRLEN(?name) < 10)
}
```

Find verbs with names between 4-9 characters.

### Pattern: Logical Operators

```sparql
SELECT ?name WHERE {
    ?verb a cnv:Verb ; cnv:name ?name .
    FILTER (?name = "status" || ?name = "start" || ?name = "stop")
}
```

OR condition for multiple values.

## EXISTS Patterns

### Pattern: Check Property Existence

```sparql
SELECT ?verb WHERE {
    ?verb a cnv:Verb .
    FILTER EXISTS { ?verb cnv:description ?d }
}
```

Find verbs that have descriptions.

### Pattern: Negative Existence

```sparql
SELECT ?verb WHERE {
    ?verb a cnv:Verb .
    FILTER NOT EXISTS { ?verb cnv:description ?d }
}
```

Find verbs without descriptions.

### Pattern: Cross-Check with Other Resources

```sparql
SELECT ?verb WHERE {
    ?verb a cnv:Verb ;
          cnv:hasNoun ?noun .
    FILTER NOT EXISTS { ?noun a cnv:Noun }
}
```

Find verbs with invalid noun references.

## AGGREGATION Patterns

### Pattern: COUNT

```sparql
SELECT (COUNT(?verb) as ?count) WHERE {
    ?noun a cnv:Noun .
    ?verb a cnv:Verb ;
          cnv:hasNoun ?noun .
}
```

Count total number of verbs.

### Pattern: GROUP BY with COUNT

```sparql
SELECT ?nounName (COUNT(?verb) as ?verbCount) WHERE {
    ?noun a cnv:Noun ;
          cnv:name ?nounName .

    ?verb a cnv:Verb ;
          cnv:hasNoun ?noun .
}
GROUP BY ?nounName
ORDER BY DESC(?verbCount)
```

Count verbs per noun (sorted by count descending).

### Pattern: GROUP_CONCAT

```sparql
SELECT ?nounName
       (GROUP_CONCAT(?verbName; separator=", ") as ?verbs) WHERE {
    ?noun a cnv:Noun ;
          cnv:name ?nounName .

    ?verb a cnv:Verb ;
          cnv:hasNoun ?noun ;
          cnv:name ?verbName .
}
GROUP BY ?nounName
```

List all verbs for each noun as comma-separated string.

### Pattern: MIN/MAX

```sparql
SELECT ?nounName
       (MIN(?priority) as ?minPriority)
       (MAX(?priority) as ?maxPriority) WHERE {
    ?noun a cnv:Noun ;
          cnv:name ?nounName .

    ?verb a cnv:Verb ;
          cnv:hasNoun ?noun ;
          cnv:priority ?priority .
}
GROUP BY ?nounName
```

Find priority range per noun.

## UNION Patterns

### Pattern: Alternative Patterns

```sparql
SELECT ?resource ?type WHERE {
    {
        ?resource a cnv:Noun .
        BIND("Noun" as ?type)
    }
    UNION
    {
        ?resource a cnv:Verb .
        BIND("Verb" as ?type)
    }
}
```

Get both nouns and verbs with their types.

### Pattern: Multi-condition UNION

```sparql
SELECT ?name WHERE {
    {
        ?resource a cnv:Noun ; cnv:name ?name .
    }
    UNION
    {
        ?resource a cnv:Verb ; cnv:name ?name .
    }
    UNION
    {
        ?resource rdfs:label ?name .
    }
}
```

Search across multiple property types.

## BIND Patterns

### Pattern: Derived Values

```sparql
SELECT ?nounName ?verbName ?fullCommand WHERE {
    ?noun a cnv:Noun ; cnv:name ?nounName .
    ?verb a cnv:Verb ;
          cnv:hasNoun ?noun ;
          cnv:name ?verbName .
    BIND(CONCAT(?nounName, " ", ?verbName) as ?fullCommand)
}
```

Create derived column combining noun and verb names.

### Pattern: Type Indicators

```sparql
SELECT ?resource ?type WHERE {
    ?resource a cnv:Noun .
    BIND("command category" as ?type)
}
```

Add semantic labels to results.

## MINUS Pattern

### Pattern: Set Difference

```sparql
SELECT ?verb WHERE {
    ?verb a cnv:Verb .
    MINUS {
        ?verb cnv:hasNoun cnv:Services .
    }
}
```

Find verbs not under the Services noun.

## DISTINCT Pattern

### Pattern: Remove Duplicates

```sparql
SELECT DISTINCT ?nounName WHERE {
    ?noun a cnv:Noun ; cnv:name ?nounName .
    ?verb a cnv:Verb ; cnv:hasNoun ?noun .
}
```

Get unique noun names even if referenced by multiple verbs.

## ORDER BY and LIMIT

### Pattern: Sorting and Pagination

```sparql
SELECT ?name WHERE {
    ?verb a cnv:Verb ; cnv:name ?name .
}
ORDER BY ?name
LIMIT 10
OFFSET 0
```

Get first 10 verbs sorted alphabetically.

## Complex Real-World Examples

### Query 1: Validate Ontology Completeness

```sparql
# Find all nouns that have no verbs
SELECT ?noun WHERE {
    ?noun a cnv:Noun .
    FILTER NOT EXISTS {
        ?verb a cnv:Verb ; cnv:hasNoun ?noun .
    }
}
```

### Query 2: Generate Command Documentation

```sparql
SELECT ?noun ?nounDesc ?verb ?verbDesc WHERE {
    ?noun a cnv:Noun ;
          cnv:name ?noun .
    OPTIONAL { ?noun rdfs:comment ?nounDesc }

    ?verb a cnv:Verb ;
          cnv:hasNoun ?noun ;
          cnv:name ?verb .
    OPTIONAL { ?verb rdfs:comment ?verbDesc }
}
ORDER BY ?noun ?verb
```

### Query 3: Agent Capability Discovery

```sparql
SELECT ?capability (COUNT(?verb) as ?commandCount) WHERE {
    ?noun a cnv:Noun ;
          cnv:name ?capability .

    ?verb a cnv:Verb ;
          cnv:hasNoun ?noun .
}
GROUP BY ?capability
ORDER BY DESC(?commandCount)
```

---

**Related**:
- [Tutorial 4: Query Ontologies with SPARQL](../tutorials/tutorial-4-sparql.md)
- [How-to: Query with SPARQL](../howto/sparql-queries.md)
- [Explanation: Semantic Web Fundamentals](../explanation/semantic-web.md)
