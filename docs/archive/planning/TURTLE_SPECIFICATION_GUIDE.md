# Turtle/RDF Specification Guide for ggen-clap-noun-verb

**Complete reference for creating CLI specifications using Turtle/RDF syntax**

---

## Table of Contents

1. [Introduction](#introduction)
2. [Turtle Syntax Basics](#turtle-syntax-basics)
3. [Ontology Reference](#ontology-reference)
4. [Component Specifications](#component-specifications)
5. [Type System](#type-system)
6. [Validation Rules](#validation-rules)
7. [Complete Examples](#complete-examples)
8. [Best Practices](#best-practices)

---

## Introduction

This guide provides a complete reference for creating Turtle/RDF specifications that generate Rust CLI applications using ggen-clap-noun-verb.

### What is Turtle?

Turtle (Terse RDF Triple Language) is a textual syntax for expressing RDF (Resource Description Framework) data. It provides a compact, human-readable way to describe structured information.

### Why Turtle for CLIs?

- **Declarative**: Specify what, not how
- **Semantic**: Machine-readable with clear meaning
- **Composable**: Reuse and extend specifications
- **Validated**: Can be checked for correctness
- **Generated**: Automatic code generation

---

## Turtle Syntax Basics

### Prefixes

Declare namespaces for abbreviated URIs:

```turtle
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix clap: <http://clap-noun-verb.io/ontology#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix my: <http://example.org/my-cli#> .
```

### Triples

RDF consists of subject-predicate-object triples:

```turtle
my:MyApp a clap:CliApplication .
# Subject: my:MyApp
# Predicate: a (rdf:type)
# Object: clap:CliApplication
```

### Property Lists

Group properties for the same subject:

```turtle
my:MyNoun a clap:Noun ;
    clap:name "my-noun" ;
    clap:about "Noun description" ;
    clap:verbs (my:Verb1 my:Verb2) .
```

### Collections

Use parentheses for ordered lists:

```turtle
clap:nouns (my:Noun1 my:Noun2 my:Noun3) .
```

### Literals

Different types of literal values:

```turtle
clap:name "string-value" .           # String
clap:version "1.0.0" .                 # String
clap:defaultValue 42 .                 # Integer
clap:defaultValue 3.14 .               # Float
clap:required true .                   # Boolean
```

---

## Ontology Reference

### Core Classes

#### CliApplication

The top-level CLI application definition.

**Properties**:
- `clap:name` (required) - Application name
- `clap:version` (required) - Version string
- `clap:author` (optional) - Author name
- `clap:about` (required) - Application description
- `clap:nouns` (required) - List of nouns
- `clap:globalFlags` (optional) - List of global flags

**Example**:
```turtle
my:App a clap:CliApplication ;
    clap:name "my-app" ;
    clap:version "1.0.0" ;
    clap:author "Developer Name" ;
    clap:about "Application description" ;
    clap:nouns (my:Noun1 my:Noun2) ;
    clap:globalFlags (my:VerboseFlag) .
```

#### Noun

A domain entity or resource type.

**Properties**:
- `clap:name` (required) - Noun name (kebab-case)
- `clap:about` (required) - Noun description
- `clap:verbs` (required) - List of verbs

**Example**:
```turtle
my:UserNoun a clap:Noun ;
    clap:name "user" ;
    clap:about "User resource operations" ;
    clap:verbs (my:CreateVerb my:ReadVerb my:UpdateVerb my:DeleteVerb) .
```

#### Verb

An action performed on a noun.

**Properties**:
- `clap:name` (required) - Verb name (kebab-case)
- `clap:about` (required) - Verb description
- `clap:arguments` (optional) - List of arguments
- `clap:flags` (optional) - List of flags
- `clap:returnType` (optional) - Return type
- `clap:example` (optional) - Usage example
- `clap:validation` (optional) - Validation rules

**Example**:
```turtle
my:CreateVerb a clap:Verb ;
    clap:name "create" ;
    clap:about "Create a new resource" ;
    clap:arguments (my:NameArg my:EmailArg) ;
    clap:flags (my:ForceFlag) ;
    clap:returnType my:ResourceResponse ;
    clap:example "my-app user create --name 'John' --email 'john@example.com'" ;
    clap:validation my:ValidEmail .
```

#### Argument

A command-line argument (positional or named).

**Properties**:
- `clap:name` (required) - Argument name (kebab-case)
- `clap:shortName` (optional) - Short name (single character)
- `clap:about` (required) - Argument description
- `clap:valueType` (required) - Type (xsd:string, xsd:integer, etc.)
- `clap:required` (required) - Boolean indicating if required
- `clap:position` (optional) - Position for positional args
- `clap:defaultValue` (optional) - Default value

**Example**:
```turtle
my:NameArg a clap:Argument ;
    clap:name "name" ;
    clap:shortName "n" ;
    clap:about "User name" ;
    clap:valueType xsd:string ;
    clap:required true ;
    clap:position 1 .
```

#### Flag

A boolean or optional configuration flag.

**Properties**:
- `clap:name` (required) - Flag name (kebab-case)
- `clap:shortName` (optional) - Short name (single character)
- `clap:about` (required) - Flag description
- `clap:valueType` (required) - Type (usually xsd:boolean)
- `clap:defaultValue` (optional) - Default value
- `clap:global` (optional) - Boolean for global flags
- `clap:env` (optional) - Environment variable name

**Example**:
```turtle
my:VerboseFlag a clap:Flag ;
    clap:name "verbose" ;
    clap:shortName "v" ;
    clap:about "Enable verbose output" ;
    clap:valueType xsd:boolean ;
    clap:defaultValue false ;
    clap:global true ;
    clap:env "VERBOSE" .
```

#### Validation

Input validation rule.

**Properties**:
- `clap:field` (required) - Field name to validate
- `clap:constraint` (required) - Validation expression
- `clap:errorMessage` (required) - Error message

**Example**:
```turtle
my:ValidEmail a clap:Validation ;
    clap:field "email" ;
    clap:constraint "regex('^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}$')" ;
    clap:errorMessage "Invalid email format" .
```

#### ComplexType

Structured return type.

**Properties**:
- `clap:fields` - List of fields

**Example**:
```turtle
my:UserResponse a clap:ComplexType ;
    clap:fields (my:IdField my:NameField my:EmailField) .
```

---

## Component Specifications

### CLI Application Template

```turtle
@prefix clap: <http://clap-noun-verb.io/ontology#> .
@prefix my: <http://example.org/my-cli#> .

my:App a clap:CliApplication ;
    clap:name "app-name" ;
    clap:version "1.0.0" ;
    clap:author "Your Name" ;
    clap:about "Application description" ;
    clap:nouns (my:Noun1) ;
    clap:globalFlags (my:GlobalFlag1) .
```

### Noun Template

```turtle
my:MyNoun a clap:Noun ;
    clap:name "noun-name" ;
    clap:about "What this noun represents" ;
    clap:verbs (my:Verb1 my:Verb2) .
```

### Verb Template

```turtle
my:MyVerb a clap:Verb ;
    clap:name "verb-name" ;
    clap:about "What this verb does" ;
    clap:arguments (my:Arg1 my:Arg2) ;
    clap:flags (my:Flag1) ;
    clap:returnType xsd:boolean ;
    clap:example "app-name noun-name verb-name --arg1 value" ;
    clap:validation my:Validation1 .
```

### Required Argument

```turtle
my:RequiredArg a clap:Argument ;
    clap:name "arg-name" ;
    clap:shortName "a" ;
    clap:about "Argument description" ;
    clap:valueType xsd:string ;
    clap:required true .
```

### Optional Argument with Default

```turtle
my:OptionalArg a clap:Argument ;
    clap:name "arg-name" ;
    clap:shortName "a" ;
    clap:about "Argument description" ;
    clap:valueType xsd:integer ;
    clap:required false ;
    clap:defaultValue 10 .
```

### Positional Argument

```turtle
my:PositionalArg a clap:Argument ;
    clap:name "filename" ;
    clap:about "File to process" ;
    clap:valueType clap:Path ;
    clap:required true ;
    clap:position 1 .
```

### Boolean Flag

```turtle
my:BoolFlag a clap:Flag ;
    clap:name "force" ;
    clap:shortName "f" ;
    clap:about "Force operation" ;
    clap:valueType xsd:boolean ;
    clap:defaultValue false .
```

### Global Flag with Environment Variable

```turtle
my:GlobalFlag a clap:Flag ;
    clap:name "api-key" ;
    clap:about "API authentication key" ;
    clap:valueType xsd:string ;
    clap:global true ;
    clap:env "API_KEY" .
```

---

## Type System

### Primitive Types (XSD)

| Turtle Type | Rust Type | Description |
|------------|-----------|-------------|
| `xsd:string` | `String` | Text string |
| `xsd:integer` | `i32` | 32-bit signed integer |
| `xsd:long` | `i64` | 64-bit signed integer |
| `xsd:unsignedShort` | `u16` | 16-bit unsigned integer (ports) |
| `xsd:unsignedInt` | `u32` | 32-bit unsigned integer |
| `xsd:unsignedLong` | `u64` | 64-bit unsigned integer |
| `xsd:float` | `f32` | 32-bit float |
| `xsd:double` | `f64` | 64-bit float |
| `xsd:boolean` | `bool` | Boolean (true/false) |

### Custom Types (clap)

| Turtle Type | Rust Type | Description |
|------------|-----------|-------------|
| `clap:Path` | `std::path::PathBuf` | File system path |
| `clap:Url` | `url::Url` | URL/URI |
| `clap:IpAddr` | `std::net::IpAddr` | IP address |

### Complex Types

Define structured types:

```turtle
my:UserResponse a clap:ComplexType ;
    clap:fields (my:IdField my:NameField my:EmailField my:CreatedField) .

my:IdField a clap:Field ;
    clap:name "id" ;
    clap:valueType xsd:unsignedLong .

my:NameField a clap:Field ;
    clap:name "name" ;
    clap:valueType xsd:string .
```

### Optional Types

Use `clap:required false` for `Option<T>`:

```turtle
my:OptionalArg a clap:Argument ;
    clap:name "optional-field" ;
    clap:valueType xsd:string ;
    clap:required false .
# Generates: Option<String>
```

### Collection Types

For multiple values, use collection notation:

```turtle
my:MultiArg a clap:Argument ;
    clap:name "values" ;
    clap:valueType xsd:string ;
    clap:multiple true .
# Generates: Vec<String>
```

---

## Validation Rules

### Constraint Expressions

Validation constraints support various expressions:

#### Comparisons

```turtle
# Equality
clap:constraint "value == 'expected'" .

# Inequality
clap:constraint "value != 0" .

# Ranges
clap:constraint "value > 0" .
clap:constraint "value >= 1 && value <= 100" .
```

#### Regex Patterns

```turtle
# Email validation
clap:constraint "regex('^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}$')" .

# Username validation
clap:constraint "regex('^[a-zA-Z0-9_-]{3,20}$')" .

# URL validation
clap:constraint "regex('^https?://[a-zA-Z0-9.-]+(:\\d+)?(/.*)?$')" .
```

#### Method Calls

```turtle
# Path operations
clap:constraint "path.exists()" .
clap:constraint "path.is_file()" .
clap:constraint "path.is_dir()" .

# String operations
clap:constraint "value.len() > 0" .
clap:constraint "value.starts_with('prefix')" .
```

#### Set Operations

```turtle
# Membership
clap:constraint "value in ['low', 'medium', 'high']" .

# Exclusion
clap:constraint "value not in ['reserved1', 'reserved2']" .
```

### Common Validations

#### Email Validation

```turtle
my:ValidEmail a clap:Validation ;
    clap:field "email" ;
    clap:constraint "regex('^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}$')" ;
    clap:errorMessage "Invalid email format" .
```

#### Port Range Validation

```turtle
my:ValidPort a clap:Validation ;
    clap:field "port" ;
    clap:constraint "value >= 1 && value <= 65535" ;
    clap:errorMessage "Port must be between 1 and 65535" .
```

#### Non-Zero Validation

```turtle
my:NonZero a clap:Validation ;
    clap:field "divisor" ;
    clap:constraint "value != 0" ;
    clap:errorMessage "Value cannot be zero" .
```

#### Path Existence

```turtle
my:PathExists a clap:Validation ;
    clap:field "path" ;
    clap:constraint "path.exists()" ;
    clap:errorMessage "Path does not exist" .
```

#### Password Strength

```turtle
my:StrongPassword a clap:Validation ;
    clap:field "password" ;
    clap:constraint "value.len() >= 8 && regex('[A-Z]') && regex('[a-z]') && regex('[0-9]')" ;
    clap:errorMessage "Password must be at least 8 characters with uppercase, lowercase, and numbers" .
```

---

## Complete Examples

### Minimal CLI

```turtle
@prefix clap: <http://clap-noun-verb.io/ontology#> .
@prefix min: <http://example.org/minimal#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

min:App a clap:CliApplication ;
    clap:name "minimal" ;
    clap:version "1.0.0" ;
    clap:about "Minimal CLI example" ;
    clap:nouns (min:Hello) .

min:Hello a clap:Noun ;
    clap:name "hello" ;
    clap:about "Greetings" ;
    clap:verbs (min:SayHello) .

min:SayHello a clap:Verb ;
    clap:name "say" ;
    clap:about "Say hello" ;
    clap:arguments (min:NameArg) ;
    clap:returnType xsd:string .

min:NameArg a clap:Argument ;
    clap:name "name" ;
    clap:about "Name to greet" ;
    clap:valueType xsd:string ;
    clap:required true .
```

**Usage**:
```bash
minimal hello say --name "World"
# Output: Hello, World!
```

### CRUD CLI

```turtle
@prefix clap: <http://clap-noun-verb.io/ontology#> .
@prefix crud: <http://example.org/crud#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

crud:App a clap:CliApplication ;
    clap:name "crud-app" ;
    clap:version "1.0.0" ;
    clap:about "CRUD operations CLI" ;
    clap:nouns (crud:Item) .

crud:Item a clap:Noun ;
    clap:name "item" ;
    clap:about "Item operations" ;
    clap:verbs (crud:Create crud:Read crud:Update crud:Delete crud:List) .

crud:Create a clap:Verb ;
    clap:name "create" ;
    clap:about "Create new item" ;
    clap:arguments (crud:NameArg crud:ValueArg) ;
    clap:returnType crud:ItemResponse .

crud:Read a clap:Verb ;
    clap:name "read" ;
    clap:about "Read item by ID" ;
    clap:arguments (crud:IdArg) ;
    clap:returnType crud:ItemResponse .

crud:Update a clap:Verb ;
    clap:name "update" ;
    clap:about "Update existing item" ;
    clap:arguments (crud:IdArg crud:NameArg crud:ValueArg) ;
    clap:returnType crud:ItemResponse .

crud:Delete a clap:Verb ;
    clap:name "delete" ;
    clap:about "Delete item by ID" ;
    clap:arguments (crud:IdArg) ;
    clap:flags (crud:ConfirmFlag) ;
    clap:returnType xsd:boolean .

crud:List a clap:Verb ;
    clap:name "list" ;
    clap:about "List all items" ;
    clap:arguments (crud:LimitArg crud:OffsetArg) ;
    clap:returnType crud:ItemListResponse .

# Arguments
crud:IdArg a clap:Argument ;
    clap:name "id" ;
    clap:about "Item ID" ;
    clap:valueType xsd:unsignedLong ;
    clap:required true .

crud:NameArg a clap:Argument ;
    clap:name "name" ;
    clap:about "Item name" ;
    clap:valueType xsd:string ;
    clap:required false .

crud:ValueArg a clap:Argument ;
    clap:name "value" ;
    clap:about "Item value" ;
    clap:valueType xsd:string ;
    clap:required false .

crud:LimitArg a clap:Argument ;
    clap:name "limit" ;
    clap:about "Maximum number of results" ;
    clap:valueType xsd:unsignedInt ;
    clap:required false ;
    clap:defaultValue 10 .

crud:OffsetArg a clap:Argument ;
    clap:name "offset" ;
    clap:about "Offset for pagination" ;
    clap:valueType xsd:unsignedInt ;
    clap:required false ;
    clap:defaultValue 0 .

# Flags
crud:ConfirmFlag a clap:Flag ;
    clap:name "confirm" ;
    clap:about "Skip confirmation prompt" ;
    clap:valueType xsd:boolean ;
    clap:defaultValue false .

# Complex types
crud:ItemResponse a clap:ComplexType ;
    clap:fields (crud:ItemIdField crud:ItemNameField crud:ItemValueField) .

crud:ItemListResponse a clap:ComplexType ;
    clap:fields (crud:ItemsField crud:TotalField) .
```

---

## Best Practices

### Naming Conventions

1. **Use kebab-case** for names: `user-id`, `config-file`
2. **Use lowercase** for nouns and verbs: `user`, `create`
3. **Be descriptive**: `email-address` instead of `ea`
4. **Short names**: Single character for flags: `-v`, `-f`

### Documentation

1. **Always include** `clap:about` for all components
2. **Add examples** with `clap:example` for verbs
3. **Clear error messages** in validations
4. **Document defaults** explicitly

### Type Safety

1. **Use appropriate types**: `u16` for ports, `PathBuf` for paths
2. **Mark required fields**: Set `clap:required true` explicitly
3. **Provide defaults**: Use `clap:defaultValue` for optional args
4. **Validate inputs**: Add validation rules for all inputs

### Organization

1. **Group related items**: Keep noun definitions together
2. **Consistent ordering**: Application → Nouns → Verbs → Arguments → Flags → Validations
3. **Use prefixes**: Separate application prefix from standard prefixes

### Performance

1. **Minimize validation**: Only validate what's necessary
2. **Reuse validators**: Define once, reference multiple times
3. **Simple constraints**: Prefer built-in validators over complex regex

---

## Related Documentation

- [Turtle Specifications README](/home/user/clap-noun-verb/examples/turtle-specs/README.md)
- [USAGE_GUIDE.md](/home/user/clap-noun-verb/docs/USAGE_GUIDE.md)
- [EXAMPLES_SHOWCASE.md](/home/user/clap-noun-verb/docs/EXAMPLES_SHOWCASE.md)
- [W3C Turtle Specification](https://www.w3.org/TR/turtle/)
- [XML Schema Types](https://www.w3.org/TR/xmlschema11-2/)

---

**Ready to create your specification?** Start with the [USAGE_GUIDE.md](/home/user/clap-noun-verb/docs/USAGE_GUIDE.md)!
