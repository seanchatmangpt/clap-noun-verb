# How-To: System Health Checks

**Goal**: Run comprehensive diagnostics and interpret system health status.

## Overview

The diagnostics module (`doctor` command) provides:
- **Health checks** for all system components
- **Status reporting** with detailed diagnostics
- **Issue detection** and resolution guidance
- **Performance metrics** and resource usage
- **Compatibility verification**

This is useful for:
- Troubleshooting CLI issues
- Verifying system setup
- Performance analysis
- Pre-deployment checks

## Running Health Checks

### Basic Diagnostics

```bash
# Run comprehensive health check
myapp doctor check

# Run and display as JSON
myapp doctor check --format json

# Run specific component check
myapp doctor check --component graph

# Run verbose diagnostics with details
myapp doctor check --verbose
```

### Output

The `doctor check` command returns a `DoctorOutput`:

```json
{
  "timestamp": "2026-06-01T10:30:00Z",
  "status": "healthy",
  "overall_health": 95,
  "components": {
    "graph": {
      "status": "healthy",
      "health_score": 100,
      "details": "RDF loading and querying operational"
    },
    "packs": {
      "status": "healthy",
      "health_score": 90,
      "details": "5 capability packs registered"
    },
    "serialization": {
      "status": "healthy",
      "health_score": 100,
      "details": "JSON, YAML, Table, TSV formats available"
    }
  },
  "issues": [],
  "recommendations": []
}
```

## Component Checks

### Graph Module Check

```bash
# Check graph module health
myapp doctor check --component graph

# Detailed graph diagnostics
myapp doctor check --component graph --verbose
```

Expected output indicates:
- ✓ RDF file loading capability
- ✓ SPARQL query support
- ✓ Validation framework
- ✓ Format support (Turtle, N-Triples, RDF/XML)

### Capability Packs Check

```bash
# Check capability registry
myapp doctor check --component packs

# Verify pack integrity
myapp doctor check --component packs --verify
```

Checks include:
- ✓ Registry accessibility
- ✓ Installed capability count
- ✓ Version consistency
- ✓ Metadata validity

### Serialization Check

```bash
# Check output format support
myapp doctor check --component serialization

# Test all formats
myapp doctor check --component serialization --test-formats
```

Verifies:
- ✓ JSON serialization
- ✓ YAML serialization
- ✓ Table formatting
- ✓ TSV export

## Interpreting Health Status

### Health Scores

| Score | Status | Meaning |
|-------|--------|---------|
| 90-100 | ✓ Healthy | All systems operational |
| 70-89 | ⚠ Degraded | Some functionality limited |
| 50-69 | ⚠ Warning | Significant issues detected |
| <50 | ✗ Critical | System unusable |

### Status Indicators

```json
{
  "status": "healthy",        // ✓ All systems go
  "status": "degraded",       // ⚠ Some features limited
  "status": "warning",        // ⚠ Issues need attention
  "status": "critical"        // ✗ Immediate action needed
}
```

## Common Issues and Resolution

### Issue: Graph Module Not Available

**Symptom:**
```json
{
  "component": "graph",
  "status": "unavailable",
  "issue": "Graph module initialization failed"
}
```

**Resolution:**
```bash
# Verify graph dependencies
myapp doctor check --component graph --verbose

# Check file system permissions (if using file storage)
ls -la ~/.cache/clap-noun-verb/

# Rebuild module
myapp doctor check --repair --component graph
```

### Issue: Missing Capability Packs

**Symptom:**
```json
{
  "component": "packs",
  "issues": ["No capability packs registered"]
}
```

**Resolution:**
```bash
# Register default capabilities
myapp pack add --name "default-io" --version 1.0.0

# List registered packs
myapp pack list

# Verify pack integrity
myapp doctor check --component packs --verify
```

### Issue: Serialization Format Failures

**Symptom:**
```json
{
  "component": "serialization",
  "status": "degraded",
  "issues": ["YAML formatting unavailable"]
}
```

**Resolution:**
```bash
# Check available formats
myapp doctor check --component serialization --test-formats

# Force format reset
myapp doctor check --repair --component serialization

# Test specific format
myapp doctor check --test-format json
```

## Practical Examples

### Example 1: Pre-Deployment Verification

```bash
# Run full diagnostics before deployment
myapp doctor check --format json > health-report.json

# Verify critical components
jq '.overall_health' health-report.json

# Check for any issues
jq '.issues | length' health-report.json

# Only deploy if health >= 90
if [ $(jq '.overall_health' health-report.json) -ge 90 ]; then
    echo "System healthy, proceeding with deployment"
else
    echo "System issues detected, please review"
    jq '.issues' health-report.json
fi
```

### Example 2: Troubleshooting Failed Operations

```bash
# An operation failed, run diagnostics
myapp doctor check --verbose

# Focus on affected component
FAILING_COMPONENT="graph"
myapp doctor check --component $FAILING_COMPONENT --verbose

# Get detailed diagnostics
myapp doctor check --component $FAILING_COMPONENT --show-logs
```

### Example 3: Performance Analysis

```bash
# Get performance metrics
myapp doctor check --performance

# View detailed metrics
myapp doctor check --performance --format json

# Track metrics over time
myapp doctor check --performance > metrics-$(date +%s).json

# Compare metrics
diff metrics-1717329000.json metrics-1717315600.json
```

### Example 4: Regular Maintenance

```bash
# Schedule daily health checks
# Add to crontab:
# 0 2 * * * /usr/local/bin/myapp doctor check --format json > /var/log/myapp-health.json

# Review weekly
myapp doctor check --format json | jq '.overall_health'

# Archive old results
mv /var/log/myapp-health.json /var/log/archive/myapp-health-$(date +%Y%m%d).json
```

## Health Check Output Formats

### JSON Format (Programmatic)

```bash
myapp doctor check --format json
```

```json
{
  "timestamp": "2026-06-01T10:30:00Z",
  "status": "healthy",
  "overall_health": 95,
  "version": "26.6.1",
  "components": { ... }
}
```

### Table Format (Human-Readable)

```bash
myapp doctor check --format table
```

```
Component          Status    Health    Details
─────────────────────────────────────────────────
Graph Module       Healthy   100       RDF operational
Capability Packs   Healthy   90        5 packs
Serialization      Healthy   100       All formats OK
```

### YAML Format (Configuration)

```bash
myapp doctor check --format yaml
```

### TSV Format (Spreadsheet)

```bash
myapp doctor check --format tsv > health-report.tsv
```

## Automated Health Monitoring

### Continuous Monitoring

```bash
# Monitor health every 5 minutes
while true; do
    myapp doctor check --format json | jq '{
        timestamp: .timestamp,
        health: .overall_health,
        status: .status
    }'
    sleep 300
done
```

### Integration with Logging

```bash
# Send health status to log aggregation
myapp doctor check --format json | \
    jq --arg host $(hostname) \
    '. + {host: $host}' | \
    curl -X POST http://logs.example.com/api/health -d @-
```

### Alert on Issues

```bash
# Alert if health drops below threshold
HEALTH=$(myapp doctor check --format json | jq '.overall_health')
if [ $HEALTH -lt 75 ]; then
    # Send alert
    echo "System health degraded: $HEALTH" | mail -s "Health Alert" ops@example.com
fi
```

## Troubleshooting Doctor Command

### "Doctor command not found"

```bash
# Ensure diagnostics module is compiled
cargo make check --all

# Verify binary has doctor command
myapp --help | grep doctor

# Rebuild if needed
cargo make build
```

### "Permission denied" Error

```bash
# Doctor needs write access for diagnostics
# Check directory permissions
ls -la ~/.cache/clap-noun-verb/

# Fix permissions if needed
mkdir -p ~/.cache/clap-noun-verb/
chmod 755 ~/.cache/clap-noun-verb/
```

### "Component not found"

```bash
# List all available components
myapp doctor check --list-components

# Ensure component name is correct
myapp doctor check --component graph
```

## See Also

- [Graph Operations](graph-operations.md) - Troubleshooting graph module
- [Capability Packing](capability-packing.md) - Managing capability health
- [Reference: Diagnostics API](../reference/api/diagnostics.md) - Complete API documentation
- [Debugging Guide](debugging.md) - General debugging techniques
