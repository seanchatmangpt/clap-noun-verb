# Federated Type System Specification

## Overview

The Federated Type System provides **type safety across CLI boundaries** by encoding types in RDF ontologies and verifying compatibility at both compile-time (via code generation) and runtime (via SHACL validation).

## Type Theory Foundation

### Type Lattice

```
                    ⊤ (Top/Any)
                    │
        ┌───────────┼───────────┐
        │           │           │
     Primitive  Composite   Function
        │           │           │
    ┌───┴───┐   ┌───┴───┐      │
    │       │   │       │      │
  Scalar  File Sum  Product   │
    │       │   │       │      │
   Int   Image │    Record    │
  String   │   │      │       │
   Bool   PNG  Union  Tuple   │
          JPEG │      │       │
               │      │       │
               └──────┴───────┘
                      │
                     ⊥ (Bottom/Never)
```

### Core Type Constructors

1. **Primitive Types**: `String`, `Int`, `Float`, `Bool`, `File`
2. **Sum Types**: `A | B` (union, disjunction)
3. **Product Types**: `{a: A, b: B}` (record, conjunction)
4. **Collection Types**: `List<A>`, `Set<A>`, `Map<K, V>`
5. **Function Types**: `A -> B`
6. **Parametric Types**: `Container<T>`

## Type Encoding in RDF

### Primitive Types

```turtle
@prefix clicap: <https://clicap.org/ontology#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

# Scalar types map to XSD datatypes
clicap:String rdfs:subClassOf xsd:string .
clicap:Integer rdfs:subClassOf xsd:integer .
clicap:Float rdfs:subClassOf xsd:float .
clicap:Boolean rdfs:subClassOf xsd:boolean .

# File types have custom properties
clicap:File a owl:Class ;
    clicap:hasProperty [
        clicap:propertyName "path" ;
        clicap:propertyType clicap:String
    ] ;
    clicap:hasProperty [
        clicap:propertyName "size_bytes" ;
        clicap:propertyType clicap:Integer
    ] ;
    clicap:hasProperty [
        clicap:propertyName "mime_type" ;
        clicap:propertyType clicap:String
    ] .
```

### Sum Types (Tagged Unions)

```turtle
# ImageFile = PNG | JPEG | WebP
clicap:ImageFile a clicap:SumType ;
    clicap:alternatives (clicap:PNGFile clicap:JPEGFile clicap:WebPFile) .

# Each alternative is a distinct type
clicap:PNGFile a owl:Class ;
    rdfs:subClassOf clicap:ImageFile ;
    clicap:mimeType "image/png" .

clicap:JPEGFile a owl:Class ;
    rdfs:subClassOf clicap:ImageFile ;
    clicap:mimeType "image/jpeg" .
```

### Product Types (Records)

```turtle
# ConversionResult = {output: ImageFile, metadata: Metadata}
clicap:ConversionResult a clicap:ProductType ;
    clicap:hasField [
        clicap:fieldName "output" ;
        clicap:fieldType clicap:ImageFile ;
        clicap:required true
    ] ;
    clicap:hasField [
        clicap:fieldName "metadata" ;
        clicap:fieldType clicap:Metadata ;
        clicap:required false
    ] .
```

### Collection Types

```turtle
# List<String>
clicap:StringList a clicap:CollectionType ;
    clicap:collectionKind clicap:List ;
    clicap:elementType clicap:String .

# Set<ImageFile>
clicap:ImageFileSet a clicap:CollectionType ;
    clicap:collectionKind clicap:Set ;
    clicap:elementType clicap:ImageFile .

# Map<String, Integer>
clicap:StringIntMap a clicap:CollectionType ;
    clicap:collectionKind clicap:Map ;
    clicap:keyType clicap:String ;
    clicap:valueType clicap:Integer .
```

### Function Types

```turtle
# ImageFile -> JSONData
:ConvertCommand clicap:typeSignature [
    a clicap:FunctionType ;
    clicap:input clicap:ImageFile ;
    clicap:output clicap:JSONData ;
    clicap:sideEffects "none"
] .

# (ImageFile, Format) -> ImageFile
:TransformCommand clicap:typeSignature [
    a clicap:FunctionType ;
    clicap:input [
        a clicap:ProductType ;
        clicap:hasField [clicap:fieldName "input"; clicap:fieldType clicap:ImageFile] ;
        clicap:hasField [clicap:fieldName "format"; clicap:fieldType clicap:Format]
    ] ;
    clicap:output clicap:ImageFile ;
    clicap:sideEffects "filesystem-write"
] .
```

## Subtyping Rules (Liskov Substitution Principle)

### Subtyping Relation: `A <: B` (A is a subtype of B)

**Rules**:

1. **Reflexivity**: `A <: A`
2. **Transitivity**: `A <: B ∧ B <: C ⇒ A <: C`
3. **Width Subtyping** (Records):
   ```
   {a: A, b: B, c: C} <: {a: A, b: B}
   ```
   (Extra fields are safe to ignore)

4. **Depth Subtyping** (Records):
   ```
   A <: A'  ∧  B <: B'
   ────────────────────
   {a: A, b: B} <: {a: A', b: B'}
   ```

5. **Sum Type Subtyping**:
   ```
   A <: C  ∧  B <: C
   ──────────────────
   (A | B) <: C
   ```

6. **Function Subtyping** (Contravariant in input, Covariant in output):
   ```
   A' <: A  ∧  B <: B'
   ────────────────────
   (A -> B) <: (A' -> B')
   ```

### Examples

```rust
// ✓ Valid: PNG <: ImageFile
let png: PNGFile = load_png("image.png");
let image: ImageFile = png; // Upcast

// ✓ Valid: Function subtyping
fn process_image(f: impl Fn(ImageFile) -> JSONData);
fn convert_png(png: PNGFile) -> StructuredData { ... }

process_image(convert_png); // ✓ OK
// Because: (PNGFile -> StructuredData) <: (ImageFile -> JSONData)
//          ^^^^^^^^^^^^^ contravariant  ^^^^^^^^^^^^^^ covariant

// ✗ Invalid: Cannot downcast without runtime check
let image: ImageFile = ...;
let png: PNGFile = image; // ✗ Type error: ImageFile </: PNGFile
```

## Type Compatibility Verification

### Compile-Time Verification (Code Generation)

```rust
// Generated from RDF ontology
#[derive(Debug, Clone)]
enum ImageFile {
    PNG(PNGFile),
    JPEG(JPEGFile),
    WebP(WebPFile),
}

impl From<PNGFile> for ImageFile {
    fn from(png: PNGFile) -> Self {
        ImageFile::PNG(png)
    }
}

// Type-safe function signature
fn convert_image(input: ImageFile) -> Result<ImageFile, Error> {
    // Compiler ensures type safety
    Ok(input)
}

// Usage
let png = PNGFile::load("image.png")?;
let result = convert_image(png.into()); // ✓ Type-checked at compile-time
```

### Runtime Verification (SHACL Validation)

```turtle
# Type constraint: argument must be subtype of ImageFile
clicap:ConvertCommandInputConstraint a sh:NodeShape ;
    sh:targetClass clicap:ConvertCommandInput ;
    sh:property [
        sh:path clicap:input ;
        sh:or (
            [ sh:class clicap:PNGFile ]
            [ sh:class clicap:JPEGFile ]
            [ sh:class clicap:WebPFile ]
        ) ;
        sh:message "Input must be a valid image file (PNG, JPEG, or WebP)"
    ] .
```

```rust
fn validate_type_runtime(
    value: &Value,
    expected_type: &str,
    ontology: &Graph,
) -> Result<(), TypeError> {
    // Load SHACL shape for expected type
    let shape = load_shacl_shape(ontology, expected_type)?;

    // Validate value against shape
    let report = shacl::validate(value, &shape)?;

    if !report.conforms {
        return Err(TypeError::ShaclValidationFailed {
            violations: report.results,
        });
    }

    Ok(())
}
```

## Type Inference

### Hindley-Milner Type Inference

For CLIs that don't provide explicit type annotations:

```rust
fn infer_type(value: &Value) -> Type {
    match value {
        Value::String(_) => Type::String,
        Value::Int(_) => Type::Int,
        Value::Float(_) => Type::Float,
        Value::Bool(_) => Type::Bool,

        Value::File(file) => {
            // Infer file type from MIME type
            match file.mime_type.as_str() {
                "image/png" => Type::Named("clicap:PNGFile"),
                "image/jpeg" => Type::Named("clicap:JPEGFile"),
                mime if mime.starts_with("image/") => Type::Named("clicap:ImageFile"),
                _ => Type::Named("clicap:File"),
            }
        }

        Value::List(elements) => {
            // Infer element type (must be homogeneous)
            let elem_types: HashSet<_> = elements.iter()
                .map(|e| infer_type(e))
                .collect();

            if elem_types.len() == 1 {
                Type::List(Box::new(elem_types.into_iter().next().unwrap()))
            } else {
                // Heterogeneous: find common supertype
                Type::List(Box::new(find_common_supertype(elem_types)))
            }
        }

        Value::Map(map) => {
            // Infer as product type (record)
            let fields = map.iter()
                .map(|(k, v)| (k.clone(), infer_type(v)))
                .collect();

            Type::Product(fields)
        }
    }
}

fn find_common_supertype(types: HashSet<Type>) -> Type {
    // Query ontology for least upper bound in type lattice
    let ontology = load_ontology();

    types.into_iter()
        .reduce(|a, b| least_upper_bound(&ontology, &a, &b))
        .unwrap_or(Type::Top)
}
```

## Schema Evolution

### Backward Compatibility Rules

A schema change is **backward compatible** if old clients can read new data.

**Safe Changes**:
- ✓ Add optional field to record
- ✓ Add new alternative to sum type
- ✓ Widen type (e.g., `Int -> Number`)
- ✓ Rename field (with alias)

**Unsafe Changes**:
- ✗ Remove field from record
- ✗ Remove alternative from sum type
- ✗ Narrow type (e.g., `Number -> Int`)
- ✗ Change field type incompatibly

### Forward Compatibility Rules

A schema change is **forward compatible** if new clients can read old data.

**Safe Changes**:
- ✓ Add required field (with default value)
- ✓ Remove optional field
- ✓ Narrow type (with fallback)

### Version Negotiation

```rust
#[derive(Debug, Clone)]
struct TypeVersion {
    major: u32,
    minor: u32,
    patch: u32,
}

fn is_compatible(client_version: &TypeVersion, server_version: &TypeVersion) -> bool {
    // Major version must match
    if client_version.major != server_version.major {
        return false;
    }

    // Client minor version must be <= server minor version
    // (Server can have new optional fields that client ignores)
    client_version.minor <= server_version.minor
}

async fn negotiate_version(
    cli_uri: &str,
    required_types: &[String],
) -> Result<TypeVersion, Error> {
    // Fetch available versions from CLI
    let ontology = fetch_ontology(cli_uri).await?;
    let available_versions = extract_type_versions(&ontology)?;

    // Find compatible version
    for required in required_types {
        let client_version = get_required_version(required)?;

        for available in &available_versions {
            if is_compatible(&client_version, available) {
                return Ok(available.clone());
            }
        }

        return Err(Error::NoCompatibleVersion {
            required: client_version,
            available: available_versions,
        });
    }

    Ok(available_versions[0].clone())
}
```

## Type Proof System

### Formal Type Safety Proof

**Theorem (Type Safety)**: If a command has type signature `A -> B`, and we invoke it with value `v: A`, then the result is `r: B`.

**Proof (Sketch)**:

1. **Static Type Checking** (Code Generation):
   - Rust compiler ensures argument type matches `A`
   - Rust compiler ensures return type matches `B`

2. **Network Serialization Soundness**:
   - Protobuf schema generated from RDF ensures wire format matches `A`
   - Deserialization validates payload conforms to protobuf schema

3. **Runtime Validation** (Defense-in-Depth):
   - SHACL validation ensures received payload satisfies RDF constraints for `A`
   - Provider validates response satisfies constraints for `B` before returning

4. **Signature Verification**:
   - Ed25519 signature ensures payload not tampered
   - If signature valid, payload originated from legitimate CLI

**∴ By construction, cross-CLI invocation is type-safe** (modulo Byzantine failures, handled by BFT consensus)

## Implementation in Rust

### Type System Core

```rust
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    // Primitives
    String,
    Int,
    Float,
    Bool,

    // Named types (reference to RDF URI)
    Named(String),

    // Sum types
    Sum(Vec<Type>),

    // Product types (records)
    Product(HashMap<String, Type>),

    // Collections
    List(Box<Type>),
    Set(Box<Type>),
    Map(Box<Type>, Box<Type>),

    // Functions
    Function(Box<Type>, Box<Type>),

    // Top and Bottom
    Top,
    Bottom,
}

impl Type {
    pub fn is_subtype_of(&self, other: &Type, ontology: &Graph) -> bool {
        match (self, other) {
            // Reflexivity
            (a, b) if a == b => true,

            // Bottom is subtype of everything
            (Type::Bottom, _) => true,

            // Everything is subtype of Top
            (_, Type::Top) => true,

            // Named types: query ontology for rdfs:subClassOf
            (Type::Named(a), Type::Named(b)) => {
                query_subclass_relation(ontology, a, b)
            }

            // Sum types
            (Type::Sum(alternatives), other) => {
                alternatives.iter().all(|alt| alt.is_subtype_of(other, ontology))
            }
            (this, Type::Sum(alternatives)) => {
                alternatives.iter().any(|alt| this.is_subtype_of(alt, ontology))
            }

            // Product types (width and depth subtyping)
            (Type::Product(fields_a), Type::Product(fields_b)) => {
                fields_b.iter().all(|(name, type_b)| {
                    fields_a.get(name)
                        .map(|type_a| type_a.is_subtype_of(type_b, ontology))
                        .unwrap_or(false)
                })
            }

            // Collection types
            (Type::List(elem_a), Type::List(elem_b)) => {
                elem_a.is_subtype_of(elem_b, ontology)
            }

            // Function types (contravariant in input, covariant in output)
            (Type::Function(in_a, out_a), Type::Function(in_b, out_b)) => {
                in_b.is_subtype_of(in_a, ontology) // contravariant
                    && out_a.is_subtype_of(out_b, ontology) // covariant
            }

            _ => false,
        }
    }
}

fn query_subclass_relation(ontology: &Graph, a: &str, b: &str) -> bool {
    let query = format!(
        "ASK {{ <{a}> rdfs:subClassOf* <{b}> }}",
        a = a,
        b = b
    );

    ontology.query(&query)
        .and_then(|result| result.as_bool())
        .unwrap_or(false)
}
```

## Example: End-to-End Type Safety

```rust
// 1. Define types in RDF
// (see clicap-ontology.ttl)

// 2. Generate Rust types from RDF
mod generated {
    #[derive(Debug, Clone)]
    pub enum ImageFile {
        PNG(PNGFile),
        JPEG(JPEGFile),
    }

    #[derive(Debug, Clone)]
    pub struct PNGFile {
        pub data: Vec<u8>,
        pub width: u32,
        pub height: u32,
    }

    // ...
}

// 3. Type-safe function signatures
async fn invoke_convert(
    cli: &str,
    input: generated::ImageFile,
    format: generated::ImageFormat,
) -> Result<generated::ImageFile, Error> {
    // Compile-time type safety: arguments match expected types

    // Runtime type validation
    let ontology = fetch_ontology(cli).await?;
    validate_type_runtime(&input, "clicap:ImageFile", &ontology)?;

    // Invoke with gRPC (type-safe serialization)
    let client = connect_grpc(cli).await?;
    let response = client.convert(input, format).await?;

    // Validate response type
    validate_type_runtime(&response, "clicap:ImageFile", &ontology)?;

    Ok(response)
}

// 4. Usage (fully type-checked)
#[tokio::main]
async fn main() -> Result<(), Error> {
    let png = generated::PNGFile::load("input.png")?;
    let result = invoke_convert(
        "cli://converter.example.com",
        generated::ImageFile::PNG(png),
        generated::ImageFormat::JPEG,
    ).await?;

    match result {
        generated::ImageFile::JPEG(jpeg) => jpeg.save("output.jpg")?,
        _ => {} // Compiler warns: non-exhaustive pattern
    }

    Ok(())
}
```

## References

- [Type Systems](http://lucacardelli.name/Papers/TypeSystems.pdf) - Luca Cardelli
- [Semantic Web Type System](https://www.w3.org/TR/owl2-overview/)
- [SHACL Specification](https://www.w3.org/TR/shacl/)
- [Hindley-Milner Type Inference](https://en.wikipedia.org/wiki/Hindley%E2%80%93Milner_type_system)
