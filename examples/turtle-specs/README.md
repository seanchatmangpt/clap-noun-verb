# Turtle Specifications for ggen-clap-noun-verb

This directory contains comprehensive Turtle (TTL) specification examples demonstrating the capabilities of ggen-clap-noun-verb for generating CLI applications from semantic specifications.

## Overview

These examples showcase how to use RDF/Turtle syntax to define complete CLI applications with noun-verb command structures, typed arguments, validation rules, and complex type definitions.

## Files

### 1. calculator.ttl
**Basic Arithmetic CLI**

A simple calculator demonstrating fundamental noun-verb patterns with typed integer arguments.

**Features:**
- Two nouns: `calc`, `math` (aliased)
- Four verbs: `add`, `subtract`, `multiply`, `divide`
- Typed arguments: `left` (Integer), `right` (Integer)
- Validation: Non-zero divisor check
- Return type: Integer

**Example Commands:**
```bash
calculator calc add --left 5 --right 3
calculator math multiply --left 6 --right 7
calculator calc divide --left 20 --right 4
```

**Key Concepts:**
- Simple noun-verb structure
- Required positional arguments
- Basic type validation
- Return type specifications

---

### 2. file-manager.ttl
**File System Operations CLI**

A comprehensive file manager demonstrating complex operations with flags and path handling.

**Features:**
- Two nouns: `file`, `dir`
- Verbs: `create`, `delete`, `list`, `move`, `copy`
- Path arguments with validation
- Multiple flags: `--recursive`, `--verbose`, `--force`
- Boolean return types

**Example Commands:**
```bash
fm file create --path /tmp/newfile.txt --verbose
fm file copy --path source.txt --destination dest.txt
fm dir create --path /tmp/newdir --recursive
fm dir delete --path /tmp/olddir --recursive --force
fm file list --path /tmp --recursive
```

**Key Concepts:**
- Path type handling
- Optional flags with defaults
- Path existence validation
- Destructive operation confirmations
- Recursive operations

---

### 3. user-api.ttl
**REST API Client CLI**

A sophisticated REST API client demonstrating CRUD operations with complex types and validation.

**Features:**
- Three nouns: `user`, `post`, `comment`
- Full CRUD verbs: `create`, `read`, `update`, `delete`, `list`
- Complex arguments: `name`, `email`, `age`, `bio`, `title`, `content`, `tags`
- Pagination: `limit`, `offset`, `filter`
- Global flags: `--base-url`, `--api-key`, `--verbose`
- Complex return types with structured responses
- Advanced validation: email format, positive IDs

**Example Commands:**
```bash
api-client user create --name "John Doe" --email "john@example.com" --age 30
api-client user read --id 123
api-client user list --limit 10 --offset 0
api-client post create --user-id 123 --title "My Post" --content "Content"
api-client comment create --post-id 456 --user-id 123 --content "Great!"
```

**Key Concepts:**
- CRUD pattern implementation
- Complex type definitions
- Email validation with regex
- Positive integer validation
- Pagination and filtering
- Global configuration flags
- Environment variable integration
- Confirmation flags for destructive operations

---

### 4. web-server.ttl
**Server Configuration and Management CLI**

An advanced server management CLI demonstrating service lifecycle operations and configuration management.

**Features:**
- Three nouns: `server`, `config`, `route`
- Server lifecycle verbs: `start`, `stop`, `restart`, `status`, `reload`
- Configuration verbs: `validate`, `show`, `set`, `reset`
- Route management verbs: `add`, `remove`, `list`, `test`
- Arguments: `port`, `host`, `workers`, `timeout`, `config-file`, `path`, `handler`, `method`
- Flags: `--daemon`, `--graceful`, `--dry-run`, `--force`, `--json`, `--strict`
- Global flags: `--config`, `--log-level`
- Complex return types: `ServerStatus`, `RouteList`, `RouteTestResult`
- Port validation (1-65535)
- File existence validation

**Example Commands:**
```bash
webserver server start --port 8080 --host 0.0.0.0 --daemon
webserver server stop --timeout 30
webserver server restart --graceful
webserver server status --verbose
webserver config validate --config-file /etc/webserver/config.toml
webserver config show --format json
webserver config set --key server.port --value 8080
webserver route add --path /api/users --handler users_handler --method GET
webserver route list --filter /api
```

**Key Concepts:**
- Service lifecycle management
- Configuration validation
- Daemon mode operation
- Graceful restart patterns
- Route management
- Multiple output formats (JSON, YAML, TOML)
- Dry-run mode
- Environment variable configuration
- Complex structured responses

---

## RDF/Turtle Ontology Structure

All specifications follow a consistent ontology structure:

### Prefixes
```turtle
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix clap: <http://clap-noun-verb.io/ontology#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
```

### Core Components

#### 1. CLI Application
```turtle
:App a clap:CliApplication ;
    clap:name "app-name" ;
    clap:version "1.0.0" ;
    clap:author "Developer Name" ;
    clap:about "Application description" ;
    clap:nouns (:Noun1 :Noun2) ;
    clap:globalFlags (:Flag1 :Flag2) .
```

#### 2. Nouns
```turtle
:NounName a clap:Noun ;
    clap:name "noun" ;
    clap:about "Noun description" ;
    clap:verbs (:Verb1 :Verb2) .
```

#### 3. Verbs
```turtle
:VerbName a clap:Verb ;
    clap:name "verb" ;
    clap:about "Verb description" ;
    clap:arguments (:Arg1 :Arg2) ;
    clap:flags (:Flag1) ;
    clap:returnType xsd:boolean ;
    clap:example "app noun verb --arg value" ;
    clap:validation :ValidationRule .
```

#### 4. Arguments
```turtle
:ArgName a clap:Argument ;
    clap:name "arg-name" ;
    clap:shortName "a" ;
    clap:about "Argument description" ;
    clap:valueType xsd:string ;
    clap:required true ;
    clap:position 1 ;
    clap:defaultValue "default" .
```

#### 5. Flags
```turtle
:FlagName a clap:Flag ;
    clap:name "flag-name" ;
    clap:shortName "f" ;
    clap:about "Flag description" ;
    clap:valueType xsd:boolean ;
    clap:defaultValue false ;
    clap:global false ;
    clap:env "ENV_VAR_NAME" .
```

#### 6. Validations
```turtle
:ValidationRule a clap:Validation ;
    clap:field "field-name" ;
    clap:constraint "validation expression" ;
    clap:errorMessage "Error message" .
```

#### 7. Complex Types
```turtle
:ComplexType a clap:ComplexType ;
    clap:fields (:Field1 :Field2) .
```

## Type System

### Primitive Types
- `xsd:string` - Text strings
- `xsd:integer` - Integer numbers
- `xsd:boolean` - Boolean values (true/false)
- `clap:Path` - File system paths

### Complex Types
Define structured return types with multiple fields for rich responses.

## Validation Rules

### Constraint Expressions
- Comparisons: `value > 0`, `value >= 1 && value <= 65535`
- Regex: `regex('^pattern$')`
- Method calls: `path.exists()`, `path.is_file()`

### Common Validations
- Positive integers: `value > 0`
- Port ranges: `value >= 1 && value <= 65535`
- Email format: `regex('^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$')`
- File existence: `path.exists() && path.is_file()`
- Non-zero divisor: `value != 0`

## Best Practices

### 1. Naming Conventions
- Use kebab-case for multi-word names: `user-id`, `config-file`
- Use lowercase for nouns and verbs: `user`, `create`
- Use descriptive names that reflect purpose

### 2. Arguments vs Flags
- **Arguments**: Required or optional positional/named values
- **Flags**: Boolean options or optional configuration switches

### 3. Global Flags
Use global flags for:
- Configuration files (`--config`)
- Logging levels (`--log-level`)
- API credentials (`--api-key`)
- Base URLs (`--base-url`)
- Verbosity (`--verbose`)

### 4. Validation
Always validate:
- Integer ranges (ports, IDs, limits)
- Email formats
- Path existence for file operations
- Required field presence
- Business logic constraints (non-zero divisors, etc.)

### 5. Examples
Include realistic `clap:example` values for each verb to demonstrate usage.

### 6. Documentation
Use clear, concise `clap:about` descriptions for all components.

### 7. Environment Variables
Use `clap:env` for sensitive or environment-specific configuration:
- API keys
- Configuration file paths
- Log levels

## Usage with ggen

To generate Rust CLI code from these specifications:

```bash
ggen parse --input calculator.ttl --output src/calculator.rs
ggen generate --spec user-api.ttl --template clap-derive
```

## Integration with clap-noun-verb

These specifications are designed to work with the clap-noun-verb Rust crate, which provides:
- Noun-verb command pattern parsing
- Type-safe argument handling
- Validation framework
- Complex return type mapping
- Global flag support

## Testing Specifications

Verify specification validity:

```bash
# Syntax validation
rapper -i turtle calculator.ttl

# Semantic validation
ggen validate calculator.ttl
```

## Extending the Ontology

To add custom properties:

```turtle
@prefix custom: <http://example.org/custom#> .

:CustomVerb a clap:Verb ;
    clap:name "custom" ;
    custom:rateLimit 100 ;
    custom:cacheable true .
```

## References

- **RDF Turtle Specification**: https://www.w3.org/TR/turtle/
- **XSD Types**: https://www.w3.org/TR/xmlschema11-2/
- **clap-noun-verb**: https://github.com/sac/clap-noun-verb
- **Rust clap**: https://docs.rs/clap/latest/clap/

## Contributing

To contribute new examples:

1. Follow the ontology structure
2. Include comprehensive documentation
3. Provide realistic use cases
4. Add validation rules
5. Include usage examples
6. Test with ggen parser

## License

These examples are provided as templates for ggen-clap-noun-verb integration and are released under the same license as the clap-noun-verb project.
