# How-To: Capability Registry & Packing

**Goal**: Register, manage, and organize capabilities into reusable packs.

## Overview

Capability packing allows you to:
- **Register** new capabilities with metadata
- **Organize** capabilities into named packs
- **Discover** available capabilities
- **Remove** outdated or unused capabilities
- **Compose** capabilities for complex operations

This is useful for:
- Managing feature flags and optional capabilities
- Building modular CLI applications
- Distributing capability bundles
- Dynamic capability loading

## Adding Capabilities

### Basic Registration

```bash
# Register a simple capability
myapp pack add --name "file-operations" --version 1.0.0

# Register with description
myapp pack add --name "file-operations" --version 1.0.0 --description "File I/O operations"

# Register with metadata
myapp pack add --name "file-operations" --version 1.0.0 --metadata "tags=io,files"
```

### Output

The `pack add` command returns a `PackAddedOutput`:
```json
{
  "pack_name": "file-operations",
  "version": "1.0.0",
  "status": "registered",
  "timestamp": "2026-06-01T10:30:00Z",
  "capabilities": 1
}
```

## Listing Registered Packs

### View All Packs

```bash
# List all registered capability packs
myapp pack list

# List with detailed information
myapp pack list --verbose

# List in JSON format
myapp pack list --format json
```

### Output Format

Table view:
```
Pack Name              Version    Description                    Status
file-operations       1.0.0      File I/O operations           active
network-operations    2.1.0      Network capabilities          active
security-filters      1.5.2      Security and validation       active
```

JSON view:
```json
{
  "packs": [
    {
      "name": "file-operations",
      "version": "1.0.0",
      "description": "File I/O operations",
      "status": "active",
      "registered_at": "2026-05-20T10:00:00Z"
    }
  ],
  "total": 1
}
```

## Removing Capabilities

### Unregister a Pack

```bash
# Remove a capability pack
myapp pack remove --name "file-operations"

# Force removal (with confirmation bypass)
myapp pack remove --name "file-operations" --force

# Verify before removal
myapp pack remove --name "file-operations" --dry-run
```

### Output

```json
{
  "pack_name": "file-operations",
  "status": "removed",
  "timestamp": "2026-06-01T10:31:00Z",
  "message": "Capability pack removed successfully"
}
```

## Practical Examples

### Example 1: Building a Modular CLI

```bash
# Start with core capabilities
myapp pack add --name "core" --version 1.0.0 --description "Core operations"

# Add optional feature packs
myapp pack add --name "advanced-search" --version 2.0.0
myapp pack add --name "export-formats" --version 1.5.0
myapp pack add --name "analytics" --version 3.1.0

# View all capabilities
myapp pack list

# Disable analytics when not needed
myapp pack remove --name "analytics"
```

### Example 2: Version Management

```bash
# Upgrade a capability pack
# First check current version
myapp pack list | grep database

# Add new version (automatically replaces old)
myapp pack add --name "database" --version 2.0.0

# Verify upgrade
myapp pack list | grep database
```

### Example 3: Capability Discovery

```bash
# List all available packs
myapp pack list --format json > capabilities.json

# Parse capabilities for display
cat capabilities.json | jq '.packs[].name'

# Count total capabilities
myapp pack list --format json | jq '.packs | length'
```

## Integration with Graph Module

### Storing Graph-Based Capabilities

```bash
# Load a capability ontology
myapp graph load --file capability-ontology.ttl

# Register as a capability pack
myapp pack add --name "semantic-capabilities" --file capability-ontology.ttl

# Query capability relationships
myapp graph query --sparql "SELECT ?cap ?requires WHERE { ?cap rdf:type Capability . ?cap requires ?requires }"
```

## Integration with Diagnostics

### Health Checks for Capability Packs

```bash
# Check capability health
myapp doctor check --component packs

# Get detailed pack status
myapp doctor check --format json | jq '.components.packs'

# Verify all packs are accessible
myapp pack list --verify
```

## Best Practices

### 1. Semantic Versioning

```bash
# Use semantic versioning: MAJOR.MINOR.PATCH
myapp pack add --name "core-service" --version 1.0.0  # Initial release
myapp pack add --name "core-service" --version 1.1.0  # Minor feature
myapp pack add --name "core-service" --version 2.0.0  # Breaking change
```

### 2. Descriptive Naming

```bash
# Good: Clear, specific names
myapp pack add --name "json-export" --version 1.0.0
myapp pack add --name "csv-export" --version 1.0.0
myapp pack add --name "postgresql-backend" --version 1.0.0

# Avoid: Generic, unclear names
myapp pack add --name "tools" --version 1.0.0
myapp pack add --name "stuff" --version 1.0.0
```

### 3. Metadata Usage

```bash
# Use metadata for categorization
myapp pack add --name "json-export" \
  --version 1.0.0 \
  --metadata "category=serialization,format=json"

myapp pack add --name "network-http" \
  --version 2.0.0 \
  --metadata "category=network,protocol=http,stability=stable"
```

### 4. Regular Cleanup

```bash
# Periodically review installed packs
myapp pack list

# Remove unused capabilities
myapp pack remove --name "deprecated-feature"

# Archive old versions
# (Use pack metadata to mark as deprecated)
```

## Troubleshooting

### "Pack already exists" Error

```bash
# A pack with that name is already registered
# Either:
# 1. Remove it first
myapp pack remove --name "file-operations"
myapp pack add --name "file-operations" --version 2.0.0

# 2. Or change the name
myapp pack add --name "file-operations-v2" --version 1.0.0
```

### "Pack not found" Error

```bash
# Verify the pack is registered
myapp pack list | grep "pack-name"

# Check exact spelling
myapp pack list --format json | jq '.packs[] | select(.name == "pack-name")'
```

### Pack Registration Fails

```bash
# Check for system errors
myapp doctor check --component packs

# Verify metadata format
# Metadata should be key=value pairs separated by commas
myapp pack add --name "test" --version 1.0.0 --metadata "key1=value1,key2=value2"
```

## Advanced Usage

### Capability Dependencies

```bash
# Create packs that depend on each other
myapp pack add --name "database-core" --version 1.0.0

# Add a pack that depends on database-core
myapp pack add --name "database-migrations" --version 1.0.0 --metadata "requires=database-core"

# Verify dependencies
myapp doctor check --verify-dependencies
```

### Programmatic Access

```bash
# Export capabilities for programmatic use
myapp pack list --format json > /tmp/capabilities.json

# Use in scripts
jq '.packs[] | select(.status == "active") | .name' /tmp/capabilities.json
```

## See Also

- [Graph Operations](graph-operations.md) - Working with RDF graphs as capabilities
- [System Diagnostics](diagnostics.md) - Health checks for packs
- [Reference: Capability API](../reference/api/capability.md) - Complete API documentation
