# ggen Template Generation - Practical Examples

Real-world examples of using ggen to generate code, commands, and configurations from templates.

## Generate Commands Quick Start

### 1. Simple "Hello World" Generator
```bash
# Generate a hello world program
ggen template generate \
  --template hello.tmpl \
  --vars name=NounVerb \
  --output ./examples/generated

# Output creates: examples/generated/hello.rs
# Content: fn main() { println!("Hello, NounVerb!"); }
```

### 2. Basic Rust Service Template
```bash
# Generate a basic Rust service
ggen template generate \
  --template rust.tmpl \
  --vars name=MyService \
  --output ./examples/generated

# Output creates: examples/generated/src/main.rs
```

### 3. Production Rust Service with Error Handling
```bash
# Generate a production-ready service
ggen template generate \
  --template rust-service-with-placeholders.tmpl \
  --vars name=BookService,description="Book catalog service" \
  --output ./examples/generated/services
```

### 4. Safe Error Handling Patterns
```bash
# Generate error handling patterns
ggen template generate \
  --template safe-error-handling.tmpl \
  --vars project=NounVerbCli,errors="ParseError,ValidationError,CommandError" \
  --output ./examples/generated/errors
```

## Generate Commands for clap-noun-verb

### Generate Noun Command Structure
```bash
# Create template for noun commands
cat > /tmp/noun-command.tmpl << 'EOF'
---
to: "src/commands/{{ noun | snake_case }}.rs"
vars:
  noun: "User"
  description: "User noun command"
---

use clap::Parser;

#[derive(Parser)]
#[command(name = "{{ noun | snake_case }}")]
#[command(about = "{{ description }}")]
pub struct {{ noun }}Command {
    /// The action to perform
    #[arg(value_name = "ACTION")]
    pub action: String,
}

impl {{ noun }}Command {
    pub fn execute(&self) -> Result<(), String> {
        match self.action.as_str() {
            "list" => self.list(),
            "create" => self.create(),
            "delete" => self.delete(),
            _ => Err(format!("Unknown action: {}", self.action)),
        }
    }

    fn list(&self) -> Result<(), String> {
        println!("Listing {{ noun | snake_case }}s");
        Ok(())
    }

    fn create(&self) -> Result<(), String> {
        println!("Creating {{ noun | snake_case }}");
        Ok(())
    }

    fn delete(&self) -> Result<(), String> {
        println!("Deleting {{ noun | snake_case }}");
        Ok(())
    }
}
EOF

# Generate noun commands
for NOUN in User Product Order Service; do
  ggen template generate \
    --template /tmp/noun-command.tmpl \
    --vars noun=$NOUN \
    --output ./src/commands
done
```

### Generate Verb Implementation
```bash
# Create template for verb commands
cat > /tmp/verb-command.tmpl << 'EOF'
---
to: "src/verbs/{{ verb | snake_case }}.rs"
vars:
  verb: "Create"
  description: "Create operation verb"
---

pub struct {{ verb }}Verb;

impl {{ verb }}Verb {
    pub fn new() -> Self {
        Self
    }

    pub fn execute(&self, target: &str) -> Result<(), String> {
        println!("Executing {{ verb | snake_case }} on: {}", target);
        Ok(())
    }
}
EOF

# Generate verb implementations
for VERB in Create Read Update Delete; do
  ggen template generate \
    --template /tmp/verb-command.tmpl \
    --vars verb=$VERB \
    --output ./src/verbs
done
```

## Batch Generation Scripts

### Generate Multiple Services
```bash
#!/bin/bash
# batch_generate_services.sh

SERVICES=(
  "User Service:UserService:User management"
  "Product Service:ProductService:Product catalog"
  "Order Service:OrderService:Order processing"
  "Auth Service:AuthService:Authentication"
)

mkdir -p src/services

for service_spec in "${SERVICES[@]}"; do
  IFS=':' read -r name var_name desc <<< "$service_spec"

  ggen template generate \
    --template ai-generated.tmpl \
    --vars name=$var_name,description="$desc",framework=axum \
    --output ./src/services

  echo "Generated $name"
done
```

Usage:
```bash
chmod +x batch_generate_services.sh
./batch_generate_services.sh
```

### Generate Error Types
```bash
#!/bin/bash
# batch_generate_errors.sh

ERRORS=(
  "ParseError"
  "ValidationError"
  "CommandError"
  "ExecutionError"
)

mkdir -p src/errors

for error_type in "${ERRORS[@]}"; do
  cat > /tmp/error-type.tmpl << EOF
---
to: "src/errors/${error_type,,}.rs"
vars:
  error_type: "$error_type"
---

#[derive(Debug, Clone)]
pub struct $error_type {
    pub message: String,
    pub code: u32,
}

impl $error_type {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            code: 500,
        }
    }
}

impl std::fmt::Display for $error_type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", stringify!($error_type), self.message)
    }
}

impl std::error::Error for $error_type {}
EOF

  ggen template generate \
    --template /tmp/error-type.tmpl \
    --output .
done
```

## Template Workflow Examples

### 1. Preview Before Generation
```bash
# Step 1: Check what files will be created
ggen template generate_tree --template ai-generated.tmpl

# Step 2: Validate template syntax
ggen template lint --template ai-generated.tmpl

# Step 3: Show template metadata
ggen template show --template ai-generated.tmpl

# Step 4: Actually generate
ggen template generate \
  --template ai-generated.tmpl \
  --vars name=MyService \
  --output ./src
```

### 2. Custom Template Creation Workflow
```bash
# Step 1: Create your custom template
mkdir -p ~/ggen/templates/custom

cat > ~/ggen/templates/custom/noun-verb-cli.tmpl << 'EOF'
---
to: "src/cli/{{ name | snake_case }}_cli.rs"
vars:
  name: "NounVerbCli"
  nouns:
    - "User"
    - "Product"
  verbs:
    - "create"
    - "read"
    - "update"
    - "delete"
---

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "{{ name | snake_case }}")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    // Noun commands
    User {
        #[command(subcommand)]
        action: UserAction,
    },
    Product {
        #[command(subcommand)]
        action: ProductAction,
    },
}

#[derive(Subcommand)]
pub enum UserAction {
    Create { name: String },
    Read { id: u32 },
    Update { id: u32, name: String },
    Delete { id: u32 },
}

#[derive(Subcommand)]
pub enum ProductAction {
    Create { name: String },
    Read { id: u32 },
    Update { id: u32, name: String },
    Delete { id: u32 },
}
EOF

# Step 2: Lint the template
ggen template lint --template custom/noun-verb-cli.tmpl

# Step 3: Show template details
ggen template show --template custom/noun-verb-cli.tmpl

# Step 4: Generate from custom template
ggen template generate \
  --template custom/noun-verb-cli.tmpl \
  --vars name=MyNounVerbCli \
  --output ./src
```

### 3. Integrated Development Workflow
```bash
# Create a complete project using templates

PROJECT_DIR="./my-noun-verb-project"
mkdir -p $PROJECT_DIR/{src/commands,src/verbs,tests,docs}

# Generate core structures from templates
ggen template generate \
  --template hello.tmpl \
  --vars name=$PROJECT_DIR \
  --output $PROJECT_DIR/src

# Generate error handling
ggen template generate \
  --template safe-error-handling.tmpl \
  --output $PROJECT_DIR/src/errors

# Generate CRUD command examples
ggen template generate \
  --template ai-generated.tmpl \
  --vars name=CommandProcessor \
  --output $PROJECT_DIR/src/commands

# List generated files
echo "Generated project structure:"
find $PROJECT_DIR -type f -name "*.rs" | head -20
```

## Advanced Template Examples

### Multi-File Generation
```bash
# Create template that generates multiple related files
cat > /tmp/service-module.tmpl << 'EOF'
---
to: "src/modules/{{ name | snake_case }}/mod.rs"
vars:
  name: "UserModule"
---

pub mod model;
pub mod service;
pub mod handler;
pub mod error;

pub use model::*;
pub use service::*;
pub use handler::*;
pub use error::*;
EOF

ggen template generate \
  --template /tmp/service-module.tmpl \
  --vars name=UserModule \
  --output .
```

### Template with Complex Variables
```bash
# Template with multiple nested variables
ggen template generate \
  --template ai-generated.tmpl \
  --vars \
    name=OrderService,\
    description="Order management service",\
    framework=axum,\
    database=postgres,\
    cache=redis \
  --output ./src/services
```

## Common Patterns

### Pattern 1: Noun-Verb Command Matrix
```bash
# Generate all noun-verb combinations
NOUNS=("User" "Product" "Order")
VERBS=("Create" "Read" "Update" "Delete")

for NOUN in "${NOUNS[@]}"; do
  for VERB in "${VERBS[@]}"; do
    # Generate command for each noun-verb pair
    ggen template generate \
      --template noun-verb-command.tmpl \
      --vars noun=$NOUN,verb=$VERB \
      --output ./src/commands
  done
done
```

### Pattern 2: Service Layer Stack
```bash
# Generate complete service layer
ggen template generate --template ai-generated.tmpl \
  --vars name=Repository --output ./src/data

ggen template generate --template ai-generated.tmpl \
  --vars name=Service --output ./src/domain

ggen template generate --template ai-generated.tmpl \
  --vars name=Handler --output ./src/handlers
```

### Pattern 3: Error Type Hierarchy
```bash
# Generate error types from template
BASE_ERRORS=("ParseError" "ValidationError" "ExecutionError")

for ERROR in "${BASE_ERRORS[@]}"; do
  ggen template generate \
    --template error-type.tmpl \
    --vars error_name=$ERROR \
    --output ./src/errors
done
```

## Tips and Tricks

### Validate All Templates Before Batch Generation
```bash
# Check all templates are valid
for tmpl in ~/ggen/templates/*.tmpl; do
  echo "Validating $(basename $tmpl)..."
  ggen template lint --template "$(basename $tmpl)" || echo "FAILED"
done
```

### Generate and Track Changes
```bash
# Generate code and commit
ggen template generate --template hello.tmpl --output ./generated

git add generated/
git commit -m "chore: Generate code from templates"
git log --oneline -n 5
```

### List and Categorize Generated Files
```bash
# After generation, list by type
ggen template generate --template ai-generated.tmpl \
  --vars name=TestService --output ./generated

echo "=== Generated Rust files ==="
find ./generated -name "*.rs"

echo "=== Generated tests ==="
find ./generated -name "*_test.rs"

echo "=== Total files generated ==="
find ./generated -type f | wc -l
```

---

**Pro Tips**:
1. Always validate templates before large batch generation
2. Use meaningful variable names for clarity
3. Organize generated code into subdirectories
4. Version control the template definitions but review generated code
5. Create custom templates for your specific patterns
