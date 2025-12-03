# Template Modular Architecture: One Template Per File Type

## Principle: Separation of Concerns at Template Level

**Problem**: Monolithic templates (like `api-stack.tmpl`) generate entire projects in one file, making them:
- ❌ Hard to maintain (1000+ lines)
- ❌ Hard to reuse (can't reuse just the Pydantic part)
- ❌ Hard to test (all or nothing)
- ❌ Hard to extend (must modify huge file)

**Solution**: Modular templates - one template per file type/concern.

---

## Architecture Pattern

### Current Pattern (Monolithic)

```
fastapi-from-rdf/
├── domain.ttl                    # RDF ontology
└── api-stack.tmpl                # ❌ ONE template for everything (800+ lines)
    ├── requirements.txt
    ├── main.py
    ├── core/config.py
    ├── core/database.py
    ├── models/*.py
    ├── schemas/*.py
    ├── api/routes/*.py
    ├── tests/*.py
    └── Dockerfile, docker-compose.yml
```

### Proposed Pattern (Modular)

```
fastapi-from-rdf/
├── domain.ttl                    # RDF ontology
└── templates/
    ├── pydantic.tmpl             # ✅ Generates schemas/*.py
    ├── route.tmpl                 # ✅ Generates api/routes/*.py
    ├── model.tmpl                 # ✅ Generates models/*.py
    ├── test.tmpl                  # ✅ Generates tests/*.py
    ├── main.tmpl                  # ✅ Generates main.py
    ├── config.tmpl                # ✅ Generates core/config.py
    ├── database.tmpl              # ✅ Generates core/database.py
    ├── docker.tmpl                # ✅ Generates Dockerfile, docker-compose.yml
    └── requirements.tmpl          # ✅ Generates requirements.txt
```

---

## Benefits of Modular Templates

### 1. **Separation of Concerns**
Each template has a single, clear responsibility:
- `pydantic.tmpl` - Only generates Pydantic schemas
- `route.tmpl` - Only generates FastAPI routes
- `model.tmpl` - Only generates SQLAlchemy models

### 2. **Reusability**
Mix and match templates across projects:
```bash
# Use Pydantic template from FastAPI project
ggen template generate pydantic.tmpl --rdf my-domain.ttl

# Use model template from different project
ggen template generate model.tmpl --rdf my-domain.ttl
```

### 3. **Maintainability**
Small, focused templates are easier to:
- Understand (50-100 lines vs 800+ lines)
- Modify (change Pydantic without affecting routes)
- Test (test each template independently)
- Debug (isolated failures)

### 4. **Composability**
Compose complete projects from modular templates:
```bash
# Generate complete FastAPI project
ggen project generate \
  --template pydantic.tmpl \
  --template route.tmpl \
  --template model.tmpl \
  --template test.tmpl \
  --rdf domain.ttl
```

### 5. **Extensibility**
Easy to add new file types:
```bash
# Add GraphQL support
ggen template generate graphql.tmpl --rdf domain.ttl

# Add TypeScript types
ggen template generate typescript.tmpl --rdf domain.ttl
```

---

## Template Structure

### Example: `pydantic.tmpl`

```yaml
---
# Pydantic Schema Generator
# Generates: schemas/{model_name}.py

rdf: domain.ttl

sparql:
  models: |
    PREFIX api: <http://api.example.org/schema#>
    PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
    SELECT ?model ?label
    WHERE {
      ?model a api:Model ;
             rdfs:label ?label .
    }

  fields: |
    PREFIX api: <http://api.example.org/schema#>
    SELECT ?model ?fieldName ?pythonType ?isRequired
    WHERE {
      ?model api:hasField ?field .
      ?field api:fieldName ?fieldName ;
             api:pythonType ?pythonType ;
             api:isRequired ?isRequired .
    }

vars:
  output_dir: "schemas"
---
{% for model in query('models') %}
{# FILE: schemas/{{ model.label | lower }}.py #}
"""{{ model.label }} Pydantic schemas"""
from pydantic import BaseModel, EmailStr, Field
from typing import Optional
from uuid import UUID

class {{ model.label }}Base(BaseModel):
    """Base {{ model.label }} schema"""
{% for field in query('fields') if field.model == model.model %}
    {{ field.fieldName }}: {{ field.pythonType }}{% if not field.isRequired %} = None{% endif %}
{% endfor %}

class {{ model.label }}Create({{ model.label }}Base):
    """Schema for creating {{ model.label }}"""
    pass

class {{ model.label }}Update(BaseModel):
    """Schema for updating {{ model.label }}"""
{% for field in query('fields') if field.model == model.model %}
    {{ field.fieldName }}: Optional[{{ field.pythonType }}] = None
{% endfor %}

class {{ model.label }}Response({{ model.label }}Base):
    """Schema for {{ model.label }} API response"""
    id: UUID
{% endfor %}
```

### Example: `route.tmpl`

```yaml
---
# FastAPI Route Generator
# Generates: api/routes/{model_name}s.py

rdf: domain.ttl

sparql:
  models: |
    PREFIX api: <http://api.example.org/schema#>
    PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
    SELECT ?model ?label ?routePrefix
    WHERE {
      ?model a api:Model ;
             rdfs:label ?label ;
             api:routePrefix ?routePrefix .
    }

vars:
  output_dir: "api/routes"
---
{% for model in query('models') %}
{# FILE: api/routes/{{ model.label | lower }}s.py #}
"""{{ model.label }} API routes"""
from fastapi import APIRouter, Depends, HTTPException
from sqlalchemy.ext.asyncio import AsyncSession

from core.database import get_db
from schemas.{{ model.label | lower }} import (
    {{ model.label }}Create,
    {{ model.label }}Update,
    {{ model.label }}Response,
)

router = APIRouter()

@router.get("/", response_model=List[{{ model.label }}Response])
async def list_{{ model.label | lower }}s(db: AsyncSession = Depends(get_db)):
    """List all {{ model.label | lower }}s"""
    # ... implementation ...

@router.post("/", response_model={{ model.label }}Response)
async def create_{{ model.label | lower }}(
    {{ model.label | lower }}_in: {{ model.label }}Create,
    db: AsyncSession = Depends(get_db),
):
    """Create new {{ model.label | lower }}"""
    # ... implementation ...

# ... more routes ...
{% endfor %}
```

---

## Template Composition

### Option 1: Template Manifest (Recommended)

Create a manifest file that defines the project structure:

```yaml
# fastapi-project.yaml
project_name: my_api
rdf: domain.ttl

templates:
  - template: templates/pydantic.tmpl
    output_dir: schemas
  
  - template: templates/model.tmpl
    output_dir: models
  
  - template: templates/route.tmpl
    output_dir: api/routes
  
  - template: templates/test.tmpl
    output_dir: tests
  
  - template: templates/main.tmpl
    output_file: main.py
  
  - template: templates/config.tmpl
    output_file: core/config.py
  
  - template: templates/docker.tmpl
    output_dir: .
```

Usage:
```bash
ggen project generate --manifest fastapi-project.yaml
```

### Option 2: Template Directory

Automatically discover templates in a directory:

```bash
# Generate all templates in directory
ggen template generate-tree templates/ --rdf domain.ttl
```

---

## Migration Plan: Refactor FastAPI Example

### Phase 1: Split Monolithic Template (Week 1)

1. **Extract Pydantic Template**
   - Create `templates/pydantic.tmpl`
   - Extract all schema generation logic
   - Test independently

2. **Extract Route Template**
   - Create `templates/route.tmpl`
   - Extract all route generation logic
   - Test independently

3. **Extract Model Template**
   - Create `templates/model.tmpl`
   - Extract all SQLAlchemy model generation
   - Test independently

4. **Extract Test Template**
   - Create `templates/test.tmpl`
   - Extract all test generation logic
   - Test independently

5. **Extract Infrastructure Templates**
   - `templates/main.tmpl` - main.py
   - `templates/config.tmpl` - core/config.py
   - `templates/database.tmpl` - core/database.py
   - `templates/docker.tmpl` - Docker files

### Phase 2: Create Template Manifest (Week 1)

1. **Create `fastapi-project.yaml`**
   - Define all templates
   - Define output structure
   - Define dependencies

2. **Update Documentation**
   - Show modular approach
   - Show composition examples
   - Show reuse examples

### Phase 3: Add Template Composition CLI (Week 2)

1. **Add `template compose` command**
   ```bash
   ggen template compose \
     --manifest fastapi-project.yaml \
     --rdf domain.ttl
   ```

2. **Add template discovery**
   - Auto-discover templates in directory
   - Generate all templates at once

---

## Template Guidelines

### 1. Single Responsibility
Each template should generate ONE type of file:
- ✅ `pydantic.tmpl` → schemas only
- ✅ `route.tmpl` → routes only
- ❌ `api-stack.tmpl` → everything

### 2. Clear Input/Output
Each template should clearly document:
- **Input**: What RDF data it needs
- **Output**: What files it generates
- **Dependencies**: What other templates it depends on

### 3. Reusable SPARQL Queries
Extract common SPARQL queries to shared files:
```yaml
# shared-queries.yaml
models_query: |
  PREFIX api: <http://api.example.org/schema#>
  SELECT ?model ?label WHERE {
    ?model a api:Model ;
           rdfs:label ?label .
  }
```

### 4. Template Parameters
Use template variables for customization:
```yaml
vars:
  project_name: "my_api"
  python_version: "3.11"
  fastapi_version: "0.104.1"
  output_dir: "schemas"
```

---

## Example: Refactored FastAPI Project Structure

```
fastapi-from-rdf/
├── domain.ttl                    # RDF ontology
├── fastapi-project.yaml          # Template manifest
└── templates/
    ├── pydantic.tmpl             # ~100 lines - schemas only
    ├── route.tmpl                # ~150 lines - routes only
    ├── model.tmpl                # ~100 lines - models only
    ├── test.tmpl                 # ~200 lines - tests only
    ├── main.tmpl                  # ~50 lines - main.py
    ├── config.tmpl                # ~30 lines - config.py
    ├── database.tmpl              # ~40 lines - database.py
    ├── docker.tmpl                # ~50 lines - Docker files
    └── requirements.tmpl          # ~20 lines - requirements.txt

Total: ~740 lines split across 9 focused templates
vs
Old: 800+ lines in one monolithic file
```

---

## Benefits Summary

| Aspect | Monolithic | Modular |
|--------|-----------|---------|
| **Maintainability** | ❌ Hard (800+ lines) | ✅ Easy (50-200 lines each) |
| **Reusability** | ❌ All or nothing | ✅ Mix and match |
| **Testability** | ❌ Hard to test parts | ✅ Test each independently |
| **Extensibility** | ❌ Modify huge file | ✅ Add new templates |
| **Composability** | ❌ Fixed structure | ✅ Flexible composition |
| **Clarity** | ❌ Mixed concerns | ✅ Single responsibility |

---

## Integration with Project-Embedded Templates

Modular templates work perfectly with **project-embedded templates** (see `PROJECT_EMBEDDED_TEMPLATES.md`):

```
clap-noun-verb/
├── templates/                    # ✅ Project-specific templates
│   ├── verb.tmpl                 # Generate new verb commands
│   ├── noun.tmpl                 # Generate new noun commands
│   ├── test.tmpl                 # Generate tests
│   └── doc.tmpl                  # Generate documentation
├── .ggen/
│   ├── project.yaml              # Template manifest
│   └── config.yaml               # Project config
└── ... (project code)
```

**Benefits**:
- ✅ Templates versioned with project
- ✅ Templates match project conventions
- ✅ New contributors can generate starter code
- ✅ Templates evolve with project

**Usage**:
```bash
# Generate from project templates
cd clap-noun-verb
ggen template generate templates/verb.tmpl \
  --var noun=utils \
  --var verb=create
```

## Next Steps

1. ✅ Document this pattern (this file)
2. ✅ Document project-embedded templates (`PROJECT_EMBEDDED_TEMPLATES.md`)
3. ⏳ Refactor `fastapi-from-rdf` example
4. ⏳ Create template manifest system
5. ⏳ Add `template compose` command
6. ⏳ Add project template discovery
7. ⏳ Update clap-noun-verb to use project templates
8. ⏳ Update all examples to use modular templates
9. ⏳ Update documentation with modular examples

---

**Last Updated**: Template modular architecture pattern documented. Project-embedded templates integration added.

