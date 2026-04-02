# Wizard Package Quick Reference

## TL;DR - Wizard in 30 Seconds

```rust
use clap_noun_verb::{verb, AppContext, WizardBuilder};

#[verb(name = "init", noun = "project")]
async fn init_project(ctx: AppContext) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Build wizard with prompts
    let wizard = WizardBuilder::new()
        .context(ctx)
        .prompt_text("Project name:")
        .prompt_choice("Type:", &["library", "binary"])
        .build()?;

    // 2. Run wizard interactively
    let session = wizard.start();
    let completed = run_wizard_session(session).await?;

    // 3. Use responses
    let responses = completed.responses();
    initialize_project_from_responses(&responses)?;

    Ok(())
}
```

## Core Concepts (80/20 Rule)

### 1. Type-Safe State Machine (20% - Most Important)

```rust
WizardBuilder::new()
    .build()?          // → WizardSession<New>
    .start()           // → WizardSession<Prompting>
    .submit_response() // → WizardSession<Processing>
    .complete()        // → WizardSession<Complete>
    .receipt()         // → WizardReceipt
```

Each state has specific methods. Compiler prevents invalid transitions.

### 2. Zero-Cost Abstractions (20%)

```rust
// PhantomData - zero runtime cost
struct WizardSession<S: SessionState> {
    state: PhantomData<S>,  // Optimized away at compile time
    // ... rest of fields
}

// Transparent wrappers - zero overhead
#[repr(transparent)]
struct SessionId(uuid::Uuid);  // Same size as Uuid
```

### 3. Feature Flags (20%)

```toml
# In Cargo.toml
[dependencies]
clap-noun-verb = { version = "5.5", features = ["wizard"] }
```

Without `wizard` feature: Zero overhead, module not compiled.

### 4. Error Handling (20%)

```rust
pub enum WizardError {
    PromptValidation { prompt_id, message },
    ResponseValidation { prompt_id, message, constraints },
    AIService(rust_genai::Error),  // Feature-gated
    Timeout { session_id, duration },
    UserAborted { session_id },
}

// Converts to NounVerbError
error.into_noun_verb_error()
```

### 5. Deterministic Receipts (20%)

```rust
let completed: WizardSession<Complete> = ...;
let receipt = completed.receipt();

// Receipt contains:
// - session_id
// - all exchanges (prompt + response pairs)
// - metadata (timestamps, version)
// - cryptographic hash (if crypto feature enabled)

// Replay session from receipt
let replayed = receipt.replay()?;
```

## API Cheat Sheet

### WizardBuilder

```rust
WizardBuilder::new()
    .prompt_text("Question?")                    // Text input
    .prompt_choice("Pick:", &["A", "B", "C"])    // Multiple choice
    .prompt(custom_prompt)                       // Custom prompt
    .config(WizardConfig::default())             // Configuration
    .context(app_context)                        // CLI context
    .recovery(custom_recovery)                   // Error recovery
    .build()?                                    // → WizardSession<New>
```

### WizardConfig

```rust
WizardConfig {
    max_prompts: 20,
    enable_ai: true,                    // Feature-gated
    ai_config: AIConfig { ... },        // Feature-gated
    output_format: OutputFormat::Json,
    timeout: Some(Duration::from_secs(300)),
}
```

### State Transitions

```rust
// New → Prompting
let prompting = session_new.start();

// Prompting → Processing
let processing = session_prompting.submit_response(response)?;

// Prompting → Aborted
let aborted = session_prompting.abort();

// Processing → Prompting (loop back)
let prompting = session_processing.process().await?;

// Processing → Complete (done)
let complete = session_processing.complete();
```

### WizardPrompt Builders

```rust
// Text prompt
WizardPrompt::text("Your name:");

// Integer with range
WizardPrompt::integer("Age:", Some(0), Some(120));

// Choice
WizardPrompt::choice("Color:", &["Red", "Green", "Blue"]);

// Boolean
WizardPrompt::boolean("Continue?");

// File path
WizardPrompt::file_path("Config file:", true /* must_exist */);

// Custom validator
WizardPrompt::custom("Email:", |input| {
    if input.contains('@') {
        Ok(())
    } else {
        Err(ValidationError::new("Invalid email"))
    }
});
```

### Error Recovery

```rust
struct MyRecovery;

impl RecoveryStrategy for MyRecovery {
    fn recover(&self, error: &WizardError) -> WizardResult<RecoveryAction> {
        match error {
            WizardError::ResponseValidation { .. } => Ok(RecoveryAction::Retry),
            WizardError::Timeout { .. } => Ok(RecoveryAction::UseDefault(default_value)),
            _ => Ok(RecoveryAction::Abort),
        }
    }
}
```

## Type System Quick Reference

```rust
// Core Types
WizardSession<S: SessionState>  // Generic session with state machine
WizardPrompt                    // Prompt definition
WizardResponse                  // User response
WizardConfig                    // Configuration
WizardReceipt                   // Deterministic output
WizardError                     // Error type
WizardResult<T>                 // Result<T, WizardError>

// States (zero-cost)
New                             // Initial state
Prompting                       // Showing prompts
Processing                      // Validating response
Complete                        // All prompts done
Aborted                         // User cancelled

// Zero-cost wrappers
SessionId(uuid::Uuid)           // Session identifier
PromptId(u64)                   // Prompt identifier
Timestamp(i64)                  // Unix timestamp

// Response types
ResponseType::Text
ResponseType::Integer { min, max }
ResponseType::Choice { options }
ResponseType::Boolean
ResponseType::FilePath { must_exist }
ResponseType::Custom { validator }
```

## Integration Patterns

### Pattern 1: Simple Wizard

```rust
#[verb(name = "setup")]
async fn setup(ctx: AppContext) -> Result<()> {
    let wizard = WizardBuilder::new()
        .context(ctx)
        .prompt_text("Name:")
        .prompt_text("Email:")
        .build()?;

    let session = run_wizard_session(wizard.start()).await?;
    process_responses(session.responses())?;
    Ok(())
}
```

### Pattern 2: Conditional Prompts

```rust
let mut builder = WizardBuilder::new().context(ctx);

if needs_advanced {
    builder = builder
        .prompt_text("Advanced option 1:")
        .prompt_text("Advanced option 2:");
}

let wizard = builder.build()?;
```

### Pattern 3: AI-Powered Suggestions

```rust
let wizard = WizardBuilder::new()
    .context(ctx)
    .config(WizardConfig {
        enable_ai: true,
        ai_config: AIConfig {
            model: "gpt-4".to_string(),
            temperature: 0.7,
            max_tokens: 100,
            system_prompt: "You are a helpful CLI assistant".to_string(),
        },
        ..Default::default()
    })
    .prompt_text("Describe your project:")
    .build()?;
```

### Pattern 4: Custom Validation

```rust
fn validate_email(input: &str) -> Result<(), ValidationError> {
    if input.contains('@') && input.contains('.') {
        Ok(())
    } else {
        Err(ValidationError::new("Invalid email format"))
    }
}

let wizard = WizardBuilder::new()
    .prompt(WizardPrompt::custom("Email:", validate_email))
    .build()?;
```

## Performance Guidelines

### Do's
- Use zero-cost wrappers (SessionId, PromptId)
- Leverage type-level state machine (compile-time checks)
- Reuse existing AppContext from parent verb
- Batch prompts in single wizard session

### Don'ts
- Don't create new wizard for each prompt (use single session)
- Don't bypass validation (use typed ResponseType)
- Don't ignore receipts (they enable replay and audit)

## Testing Quick Start

```rust
#[test]
fn test_wizard_happy_path() {
    // Arrange
    let wizard = WizardBuilder::new()
        .prompt_text("Name:")
        .build()
        .unwrap();

    // Act
    let session = wizard.start();
    let response = WizardResponse::new(PromptId(0), "Alice".to_string());
    let processing = session.submit_response(response).unwrap();
    let complete = processing.complete();

    // Assert
    let receipt = complete.receipt();
    assert_eq!(receipt.exchanges.len(), 1);
    assert_eq!(receipt.exchanges[0].response.raw_input, "Alice");
}
```

## Common Patterns

### Pattern: Multi-Step Form
```rust
WizardBuilder::new()
    .prompt_text("Step 1: Name")
    .prompt_text("Step 2: Email")
    .prompt_choice("Step 3: Plan", &["Free", "Pro", "Enterprise"])
    .build()?
```

### Pattern: Conditional Logic
```rust
let responses = completed.responses();
if responses["plan"] == "Enterprise" {
    // Show enterprise-specific wizard
}
```

### Pattern: Receipt Storage
```rust
let receipt = completed.receipt();
let json = serde_json::to_string(&receipt)?;
std::fs::write("wizard-session.json", json)?;

// Later: Replay
let json = std::fs::read_to_string("wizard-session.json")?;
let receipt: WizardReceipt = serde_json::from_str(&json)?;
let replayed = receipt.replay()?;
```

## File Locations

```
src/wizard/
├── mod.rs              # Public API
├── builder.rs          # WizardBuilder
├── session.rs          # WizardSession<S>
├── prompt.rs           # WizardPrompt
├── response.rs         # WizardResponse
├── error.rs            # WizardError
├── config.rs           # WizardConfig
├── validation.rs       # Validators
├── receipt.rs          # WizardReceipt
└── ai/
    ├── mod.rs
    ├── suggestions.rs
    └── context.rs
```

## Next Steps for Implementation

1. Implement error.rs (error types and recovery)
2. Implement prompt.rs (prompt types and builders)
3. Implement response.rs (response validation)
4. Implement config.rs (configuration types)
5. Implement session.rs (state machine)
6. Implement builder.rs (WizardBuilder API)
7. Implement validation.rs (constraint checking)
8. Implement receipt.rs (deterministic receipts)
9. Implement ai/ (feature-gated AI integration)
10. Write tests (Chicago TDD - AAA pattern)

## Memory Aid

**W**izard **I**s **Z**ero-cost **A**synchronous **R**eusable **D**eterministic

- **W**izard: Interactive CLI component
- **I**s: Type-safe state machine
- **Z**ero-cost: PhantomData, repr(transparent)
- **A**synchronous: Tokio-based for AI calls
- **R**eusable: Builder pattern, hooks, traits
- **D**eterministic: Receipts with cryptographic hash
