# Reference: Error Codes and Troubleshooting

Complete reference of all error types and how to resolve them.

## TurtleError Variants

### ParseError

**Message**: `Parse error at line {line}: {message}`

**Causes**:
- Missing period at end of statement
- Invalid Turtle syntax
- Undefined prefix
- Malformed IRI

**Example**:
```
Parse error at line 15: Unexpected token ';'
```

**Fix**:
```turtle
# ❌ WRONG - missing period
cnv:Services a cnv:Noun

# ✅ CORRECT
cnv:Services a cnv:Noun .
```

### ValidationError

**Message**: `Validation failed: {reason}`

**Causes**:
- RDF semantics violated
- Invalid object type
- Constraint violation

**Example**:
```
Validation failed: Range constraint violated for cnv:name
```

**Fix**: Check data types match expected ranges in ontology schema.

### InvalidPrefix

**Message**: `Invalid prefix: {prefix}`

**Causes**:
- Prefix used but not declared
- Invalid prefix syntax

**Example**:
```
Invalid prefix: foo
```

**Fix**:
```turtle
# Declare prefix first
@prefix foo: <https://example.com/foo#> .

# Then use it
foo:MyResource a cnv:Noun .
```

### IoError

**Message**: `IO error: {system error}`

**Causes**:
- File not found
- Permission denied
- Disk full
- Invalid path

**Example**:
```
IO error: No such file or directory
```

**Fix**:
```bash
# Check file exists
ls -l ontology/services-cli.ttl

# Check permissions
chmod 644 ontology/services-cli.ttl
```

## CodeGenError Variants

### CodeGenError::ParseError

**Message**: `Parse error: {source}`

**Cause**: Underlying Turtle parsing failed

**Fix**: See TurtleError causes above.

### CodeGenError::InvalidIdentifier

**Message**: `Invalid identifier: {identifier}`

**Causes**:
- Name contains spaces
- Name starts with number
- Name contains special characters

**Example**:
```
Invalid identifier: "My Service"
```

**Fix**:
```turtle
# ❌ WRONG - spaces in name
cnv:MyService a cnv:Noun ; cnv:name "My Service" .

# ✅ CORRECT - use camelCase or snake_case
cnv:MyService a cnv:Noun ; cnv:name "my_service" .
```

### CodeGenError::MissingNounReference

**Message**: `Missing noun reference: {verb}`

**Causes**:
- Verb has no cnv:hasNoun property
- Referenced noun doesn't exist

**Example**:
```
Missing noun reference: cnv:StatusVerb
```

**Fix**:
```turtle
# ❌ WRONG - verb without noun reference
cnv:StatusVerb a cnv:Verb ;
    cnv:name "status" .

# ✅ CORRECT - add cnv:hasNoun
cnv:StatusVerb a cnv:Verb ;
    cnv:name "status" ;
    cnv:hasNoun cnv:Services .

# ✅ AND make sure Services noun exists
cnv:Services a cnv:Noun ;
    cnv:name "services" .
```

### CodeGenError::GenerationFailed

**Message**: `Code generation failed: {reason}`

**Causes**:
- Macro expansion failed
- Invalid Rust code generation
- Internal generator error

**Example**:
```
Code generation failed: Failed to generate verb macro for status
```

**Fix**:
1. Verify ontology is valid: `cargo make test validate_ontology`
2. Check for unusual characters in names
3. Simplify ontology to minimal example
4. Report issue with minimal reproduction

## SparqlError Variants

### QueryParseError

**Message**: `Query parse error: {message}`

**Causes**:
- Invalid SPARQL syntax
- Unknown function
- Malformed pattern

**Example**:
```
Query parse error: Unexpected token in WHERE clause
```

**Fix**:
1. Validate query at https://www.w3.org/2001/sw/wiki/SparqlQueryValidator
2. Check variable names (case-sensitive)
3. Verify prefix declarations

```sparql
# ❌ WRONG - undefined prefix
SELECT ?v WHERE { ?v a foo:Verb }

# ✅ CORRECT - declare prefix
PREFIX cnv: <https://cnv.dev/ontology#>
SELECT ?v WHERE { ?v a cnv:Verb }
```

### ExecutionError

**Message**: `Execution error: {message}`

**Causes**:
- Runtime evaluation failure
- Type mismatch
- Graph evaluation error

**Example**:
```
Execution error: Type error in filter expression
```

**Fix**:
1. Verify data types in ontology
2. Check FILTER conditions are type-correct
3. Use explicit type casting if needed

### BindingError

**Message**: `Binding error: {message}`

**Causes**:
- Variable not bound in results
- Result set has inconsistent bindings

**Example**:
```
Binding error: Variable ?name not bound
```

**Fix**:
```sparql
# ❌ WRONG - ?name only bound in OPTIONAL
SELECT ?verb ?name WHERE {
    ?verb a cnv:Verb .
    OPTIONAL { ?verb cnv:name ?name }
}

# ✅ CORRECT - name is in base pattern
SELECT ?verb ?name WHERE {
    ?verb a cnv:Verb ;
          cnv:name ?name .
}
```

## HTTP/API Errors (MCP Tools)

### 400 Bad Request

**Cause**: Invalid input to MCP tool

**Common issues**:
- Empty turtle_definition
- Malformed JSON input
- Missing required fields

**Fix**:
```json
// ❌ WRONG
{
  "turtle_definition": ""
}

// ✅ CORRECT
{
  "turtle_definition": "@prefix cnv: <...> . cnv:Services a cnv:Noun ...",
  "ontology_iri": "https://myapp.dev/ontology"
}
```

### 422 Unprocessable Entity

**Cause**: Input validation failed

**Common issues**:
- Invalid Turtle syntax
- Ontology validation failed
- Constraint violations

**Fix**: Check validation errors in response and fix root cause.

### 500 Internal Server Error

**Cause**: Unexpected error in code generator or executor

**Fix**:
1. Enable debug logging
2. Create minimal reproduction
3. Check server logs for stack trace
4. Report issue with reproduction steps

## Performance-Related Errors

### Timeout Error

**Message**: Operation exceeded time limit

**Causes**:
- Ontology too large
- Complex SPARQL query
- Code generation for many verbs

**Fix**:
1. Split large ontologies into modules
2. Use simpler SPARQL queries
3. Increase timeout limits in configuration
4. Profile to find bottlenecks

### Out of Memory

**Message**: Allocation failed

**Causes**:
- Very large ontologies
- Unbounded query results
- Memory leak

**Fix**:
1. Use streaming queries instead of loading all results
2. Split ontology into smaller files
3. Add LIMIT clause to queries
4. Check for circular references

## Diagnostic Output Format

All errors include diagnostic information:

```json
{
  "error": "ParseError",
  "message": "Parse error at line 15: Unexpected token",
  "context": {
    "line": 15,
    "column": 42,
    "text": "cnv:Services a cnv:Noun"
  }
}
```

## Debugging Checklist

- ✅ Error message clearly identifies root cause
- ✅ Line numbers point to problem in Turtle file
- ✅ Error type suggests solution
- ✅ Context provides surrounding code
- ✅ Suggestions for fixing included in message

## Getting Help

If you encounter an error not listed here:

1. **Check the logs**: Enable `RUST_LOG=debug`
2. **Validate input**: Use our validation tools
3. **Simplify reproduction**: Create minimal example
4. **Search issues**: https://github.com/seanchatmangpt/clap-noun-verb/issues
5. **Report new error**: Include full error message, reproduction steps, and expected behavior

---

**Related**:
- [How-to: Debug RDF Issues](../howto/debugging.md)
- [How-to: Validate Ontologies](../howto/validation.md)
- [API Reference](api-reference.md)
