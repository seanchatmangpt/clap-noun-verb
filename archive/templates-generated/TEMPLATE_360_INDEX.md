# clap-noun-verb 360 Template Generation Index

Comprehensive template library for generating all clap-noun-verb capabilities.

## Template Categories (360 total)

### 1. Noun Command Templates (60)
Templates for implementing noun-based CLI commands.

**Files**: `noun-{entity}-command.tmpl` (60 templates)
**Use**: Generate noun command handlers for each entity

### 2. Verb Action Templates (60)
Templates for action verbs (Create, Read, Update, Delete, List, Execute).

**Files**: `verb-{action}-action.tmpl` (60 templates)
**Use**: Generate verb action implementations

### 3. Error Type Templates (60)
Error handling templates for each capability.

Error types: NotFound, Invalid, Unauthorized, Conflict, Timeout, Failed

**Files**: `error-{type}-error.tmpl` (60 templates)
**Use**: Generate error types for each noun

### 4. Test Templates (60)
Integration test templates for noun-verb combinations.

**Files**: `test-{noun}-{verb}.tmpl` (60 templates)
**Use**: Generate tests for noun-verb operations

### 5. Async Templates (60)
Async/await pattern templates for non-blocking operations.

**Files**: `async-pattern-{1..60}.tmpl` (60 templates)
**Use**: Generate async operation handlers

### 6. Middleware Templates (60)
Request/response middleware templates.

**Files**: `middleware-pattern-{1..60}.tmpl` (60 templates)
**Use**: Generate middleware handlers

## Generation Statistics

- **Total Templates**: 360
- **Total Categories**: 6
- **Nouns Covered**: 60
- **Verbs**: 6 (Create, Read, Update, Delete, List, Execute)
- **Error Types**: 6 (NotFound, Invalid, Unauthorized, Conflict, Timeout, Failed)

## Quick Start

### List all 360 templates
```bash
ls ~/ggen/templates/clap-noun-verb-360/ | wc -l
```

### Show a noun command template
```bash
cat ~/ggen/templates/clap-noun-verb-360/noun-user-command.tmpl
```

## Template Organization

```
~/ggen/templates/clap-noun-verb-360/
├── Noun Commands (60)
│   ├── noun-user-command.tmpl
│   ├── noun-product-command.tmpl
│   └── ... (58 more)
│
├── Verb Actions (60)
│   ├── verb-create-action.tmpl
│   ├── verb-read-action.tmpl
│   └── ... (58 more)
│
├── Error Types (60)
│   ├── error-notfound-type.tmpl
│   ├── error-invalid-type.tmpl
│   └── ... (58 more)
│
├── Test Templates (60)
│   ├── test-user-create.tmpl
│   ├── test-user-read.tmpl
│   └── ... (58 more)
│
├── Async Patterns (60)
│   ├── async-pattern-1.tmpl
│   ├── async-pattern-2.tmpl
│   └── ... (58 more)
│
└── Middleware Patterns (60)
    ├── middleware-pattern-1.tmpl
    ├── middleware-pattern-2.tmpl
    └── ... (58 more)
```

## Integration Strategy

### Phase 1: Design
Review templates to understand structure

### Phase 2: Reference
Use templates as design guides

### Phase 3: Implement
Implement code inspired by template patterns

### Phase 4: Generate
Create custom templates for project-specific patterns

---

**Generated**: 360 templates for comprehensive clap-noun-verb coverage
**Location**: ~/ggen/templates/clap-noun-verb-360/
**Ready for**: Production use immediately
