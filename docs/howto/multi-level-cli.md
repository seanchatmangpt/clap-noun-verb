# How-to: Build Multi-Level CLIs

**Problem**: You need a CLI with multiple command categories (nouns), each with multiple operations (verbs)

**Solution**: Define multiple nouns in your RDF ontology, and the code generator will create a hierarchical CLI structure

## Ontology Design Pattern

Create `ontology/complete-system.ttl`:

```turtle
@prefix cnv: <https://cnv.dev/ontology#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

# ============================================================================
# Services Management Commands
# ============================================================================

cnv:Services a cnv:Noun ;
    cnv:name "services" ;
    rdfs:comment "Manage system services" .

cnv:ServiceStatus a cnv:Verb ;
    cnv:name "status" ;
    cnv:hasNoun cnv:Services ;
    cnv:description "Show service status" ;
    cnv:handler "service_status" .

cnv:ServiceStart a cnv:Verb ;
    cnv:name "start" ;
    cnv:hasNoun cnv:Services ;
    cnv:description "Start a service" ;
    cnv:handler "service_start" .

# ============================================================================
# Configuration Commands
# ============================================================================

cnv:Config a cnv:Noun ;
    cnv:name "config" ;
    rdfs:comment "Manage system configuration" .

cnv:ConfigShow a cnv:Verb ;
    cnv:name "show" ;
    cnv:hasNoun cnv:Config ;
    cnv:description "Show current configuration" ;
    cnv:handler "config_show" .

cnv:ConfigSet a cnv:Verb ;
    cnv:name "set" ;
    cnv:hasNoun cnv:Config ;
    cnv:description "Set configuration value" ;
    cnv:handler "config_set" .

# ============================================================================
# Database Commands
# ============================================================================

cnv:Database a cnv:Noun ;
    cnv:name "database" ;
    rdfs:comment "Manage database operations" .

cnv:DatabaseMigrate a cnv:Verb ;
    cnv:name "migrate" ;
    cnv:hasNoun cnv:Database ;
    cnv:description "Run database migrations" ;
    cnv:handler "database_migrate" .

cnv:DatabaseBackup a cnv:Verb ;
    cnv:name "backup" ;
    cnv:hasNoun cnv:Database ;
    cnv:description "Create database backup" ;
    cnv:handler "database_backup" .
```

## Generated CLI Structure

This ontology generates a CLI with this usage:

```bash
# Services commands
mycli services status
mycli services start

# Config commands
mycli config show
mycli config set --key database.url --value postgres://localhost

# Database commands
mycli database migrate --version latest
mycli database backup --output backup.sql
```

## Implementation Pattern

```rust
use clap_noun_verb::rdf::{TurtleParser, CliCodeGenerator};
use std::fs;

pub fn generate_multi_level_cli() -> Result<(), Box<dyn std::error::Error>> {
    // Load ontology
    let turtle = fs::read_to_string("ontology/complete-system.ttl")?;
    let parser = TurtleParser::new();
    let ontology = parser.parse(&turtle)?;

    // Generate code
    let generator = CliCodeGenerator::new()?;
    let generated = generator.generate_from_ontology(&ontology)?;

    println!("Generated {} nouns", generated.noun_count());
    println!("Generated {} verbs", generated.verb_count());

    // Save generated code
    fs::write("src/generated_cli.rs", generated.rust_code())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multi_level_generation() {
        let result = generate_multi_level_cli();
        assert!(result.is_ok());
    }
}
```

## Adding Custom Logic

After generation, implement actual command logic:

```rust
// Implement generated handler stubs
pub async fn service_status(name: &str) -> Result<String> {
    // Your implementation
    Ok(format!("Service {} is running", name))
}

pub async fn config_show() -> Result<String> {
    // Your implementation
    Ok("Configuration displayed".to_string())
}

pub async fn database_migrate(version: &str) -> Result<String> {
    // Your implementation
    Ok(format!("Migrated to version {}", version))
}
```

## Best Practices

### 1. Organize Ontology by Function
- Group related verbs under one noun
- Keep nouns to 3-7 verbs each
- Use clear, memorable noun names

### 2. Noun Naming
- Use plural for collections: `services`, `users`, `databases`
- Use singular for singletons: `config`, `status`
- Keep names lowercase

### 3. Verb Naming
- Use action verbs: `start`, `stop`, `create`, `delete`
- Consistent naming across nouns: `show`, `list`, `get`
- Keep names short (1-2 words)

### 4. Documentation
- Add `rdfs:comment` to each noun
- Add `rdfs:comment` to important verbs
- Use `rdfs:label` for display names

## Example: Real-World Multi-Level System

```turtle
@prefix cnv: <https://cnv.dev/ontology#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

# User Management
cnv:Users a cnv:Noun ;
    cnv:name "users" ;
    rdfs:comment "User account management" .

cnv:UserList a cnv:Verb ; cnv:name "list" ; cnv:hasNoun cnv:Users .
cnv:UserCreate a cnv:Verb ; cnv:name "create" ; cnv:hasNoun cnv:Users .
cnv:UserDelete a cnv:Verb ; cnv:name "delete" ; cnv:hasNoun cnv:Users .
cnv:UserGrant a cnv:Verb ; cnv:name "grant-role" ; cnv:hasNoun cnv:Users .

# Role Management
cnv:Roles a cnv:Noun ;
    cnv:name "roles" ;
    rdfs:comment "Role-based access control" .

cnv:RoleList a cnv:Verb ; cnv:name "list" ; cnv:hasNoun cnv:Roles .
cnv:RoleCreate a cnv:Verb ; cnv:name "create" ; cnv:hasNoun cnv:Roles .
cnv:RoleDelete a cnv:Verb ; cnv:name "delete" ; cnv:hasNoun cnv:Roles .

# System Administration
cnv:Admin a cnv:Noun ;
    cnv:name "admin" ;
    rdfs:comment "System administration" .

cnv:AdminHealth a cnv:Verb ; cnv:name "health" ; cnv:hasNoun cnv:Admin .
cnv:AdminMetrics a cnv:Verb ; cnv:name "metrics" ; cnv:hasNoun cnv:Admin .
```

Generates CLI:

```bash
$ mycli users list
$ mycli users create --name john --role admin
$ mycli roles list
$ mycli admin health
$ mycli admin metrics --format json
```

## Validation Checklist

- ✅ Each verb has exactly one `cnv:hasNoun`
- ✅ All noun references exist
- ✅ No duplicate verb names within a noun
- ✅ All names are valid Rust identifiers
- ✅ Ontology validates with `parser.validate_ontology()`

---

**Related**:
- [Tutorial 2: Create Your First RDF Ontology](../tutorials/tutorial-2-first-rdf.md)
- [How-to: Validate Ontologies](validation.md)
- [Reference: RDF Vocabulary](../reference/vocabulary.md)
