# Wizard Package Architecture Design
## Type-First, Zero-Cost AI-Powered CLI Wizard Component

**Version:** 1.0
**Date:** 2026-01-09
**Status:** Architecture Design Review
**Feature Flag:** `wizard`

---

## Executive Summary

The wizard package introduces AI-powered interactive CLI wizards to clap-noun-verb using type-first design principles and zero-cost abstractions. It integrates rust-genai for AI capabilities while maintaining deterministic outputs and full backwards compatibility.

### Key Design Principles

1. **Type-First Thinking**: Encode wizard invariants at compile time
2. **Zero-Cost Abstractions**: Generic state machines with monomorphization
3. **Deterministic Outputs**: Reproducible wizard sessions with receipts
4. **Feature-Gated Integration**: Conditional compilation for minimal overhead
5. **Memory Safety**: Lifetime-aware session management
6. **API Ergonomics**: Builder pattern for easy wizard construction

---

## 1. Type System Design

### 1.1 Core Type Hierarchy

```rust
/// Generic wizard session with compile-time state machine
///
/// Zero-cost abstraction: State machine transitions happen at compile time
/// through const generics and type-level programming
pub struct WizardSession<S: SessionState> {
    /// Current session state (zero-cost - optimized away at compile time)
    state: PhantomData<S>,

    /// Session identifier (deterministic UUID for reproducibility)
    id: SessionId,

    /// Accumulated prompts and responses
    conversation: Vec<Exchange>,

    /// Context from parent CLI command
    context: AppContext,

    /// Configuration for wizard behavior
    config: WizardConfig,
}

/// Type-safe session states (zero-cost - only used at compile time)
pub trait SessionState: sealed::Sealed {}

/// Session states form a type-level state machine
pub struct New;
pub struct Prompting;
pub struct Processing;
pub struct Complete;
pub struct Aborted;

impl SessionState for New {}
impl SessionState for Prompting {}
impl SessionState for Processing {}
impl SessionState for Complete {}
impl SessionState for Aborted {}

/// Sealed trait pattern prevents external state implementations
mod sealed {
    pub trait Sealed {}
    impl Sealed for super::New {}
    impl Sealed for super::Prompting {}
    impl Sealed for super::Processing {}
    impl Sealed for super::Complete {}
    impl Sealed for super::Aborted {}
}
```

### 1.2 Prompt and Response Types

```rust
/// Strongly-typed wizard prompt
///
/// Encodes prompt metadata and validation rules at compile time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WizardPrompt {
    /// Unique prompt identifier
    id: PromptId,

    /// Human-readable prompt text
    text: String,

    /// Expected response type (used for validation)
    response_type: ResponseType,

    /// Validation constraints (encoded at type level where possible)
    constraints: Vec<Constraint>,

    /// Optional AI context for intelligent suggestions
    #[cfg(feature = "wizard")]
    ai_context: Option<AIContext>,
}

/// Response type discriminator (zero-cost enum)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ResponseType {
    /// Free-form text
    Text,

    /// Bounded integer
    Integer { min: Option<i64>, max: Option<i64> },

    /// Multiple choice (index into options)
    Choice { options: &'static [&'static str] },

    /// Boolean yes/no
    Boolean,

    /// File path (validated for existence)
    FilePath { must_exist: bool },

    /// Custom validator function (zero-cost function pointer)
    Custom { validator: fn(&str) -> Result<(), ValidationError> },
}

/// Deterministic wizard response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WizardResponse {
    /// Prompt this responds to
    prompt_id: PromptId,

    /// Raw user input
    raw_input: String,

    /// Validated and parsed value
    parsed_value: serde_json::Value,

    /// Timestamp for audit trail
    timestamp: Timestamp,

    /// Optional AI-generated suggestions that were presented
    #[cfg(feature = "wizard")]
    ai_suggestions: Option<Vec<String>>,
}

/// Prompt-Response exchange for conversation history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Exchange {
    pub prompt: WizardPrompt,
    pub response: WizardResponse,
}
```

### 1.3 Configuration Types

```rust
/// Zero-cost wizard configuration
///
/// Uses const generics where possible for compile-time optimization
#[derive(Debug, Clone)]
pub struct WizardConfig {
    /// Maximum prompts before auto-abort
    max_prompts: usize,

    /// Enable AI-powered suggestions
    #[cfg(feature = "wizard")]
    enable_ai: bool,

    /// AI model configuration (only compiled when wizard feature enabled)
    #[cfg(feature = "wizard")]
    ai_config: AIConfig,

    /// Output format for wizard results
    output_format: OutputFormat,

    /// Session timeout (None = no timeout)
    timeout: Option<Duration>,
}

/// AI configuration (feature-gated)
#[cfg(feature = "wizard")]
#[derive(Debug, Clone)]
pub struct AIConfig {
    /// rust-genai model identifier
    model: String,

    /// Temperature for response generation
    temperature: f32,

    /// Maximum tokens per AI response
    max_tokens: usize,

    /// System prompt template
    system_prompt: String,
}
```

### 1.4 Identity Types (Zero-Cost Wrappers)

```rust
/// Session identifier (zero-cost wrapper around UUID)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(transparent)]
pub struct SessionId(uuid::Uuid);

/// Prompt identifier (zero-cost wrapper around u64)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(transparent)]
pub struct PromptId(u64);

/// Timestamp (zero-cost wrapper with deterministic serialization)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[repr(transparent)]
pub struct Timestamp(i64);
```

---

## 2. Error Handling Strategy

### 2.1 Error Type Hierarchy

```rust
/// Wizard-specific error types
///
/// Integrates with existing NounVerbError through conversion traits
#[derive(Error, Debug)]
pub enum WizardError {
    /// Prompt validation failed
    #[error("Prompt validation failed: {message}")]
    PromptValidation {
        prompt_id: PromptId,
        message: String,
    },

    /// Response validation failed
    #[error("Response validation failed for prompt {prompt_id}: {message}")]
    ResponseValidation {
        prompt_id: PromptId,
        message: String,
        constraints: String,
    },

    /// AI service error (only when wizard feature enabled)
    #[cfg(feature = "wizard")]
    #[error("AI service error: {0}")]
    AIService(#[from] rust_genai::Error),

    /// Session state transition error
    #[error("Invalid state transition from {from} to {to}")]
    InvalidStateTransition {
        from: &'static str,
        to: &'static str,
    },

    /// Session timeout
    #[error("Wizard session timed out after {duration:?}")]
    Timeout {
        session_id: SessionId,
        duration: Duration,
    },

    /// Session aborted by user
    #[error("Wizard session aborted by user")]
    UserAborted {
        session_id: SessionId,
    },

    /// Context error
    #[error("Context error: {0}")]
    Context(#[from] crate::context::ContextError),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

impl WizardError {
    /// Convert to NounVerbError for framework integration
    pub fn into_noun_verb_error(self) -> NounVerbError {
        NounVerbError::ExecutionError {
            message: self.to_string(),
        }
    }

    /// Create prompt validation error with constraints
    pub fn prompt_validation(
        prompt_id: PromptId,
        message: impl Into<String>,
    ) -> Self {
        Self::PromptValidation {
            prompt_id,
            message: message.into(),
        }
    }

    /// Create response validation error with constraints
    pub fn response_validation(
        prompt_id: PromptId,
        message: impl Into<String>,
        constraints: impl Into<String>,
    ) -> Self {
        Self::ResponseValidation {
            prompt_id,
            message: message.into(),
            constraints: constraints.into(),
        }
    }
}

/// Result type alias for wizard operations
pub type WizardResult<T> = Result<T, WizardError>;
```

### 2.2 Error Recovery Strategies

```rust
/// Error recovery strategy trait
pub trait RecoveryStrategy {
    /// Attempt to recover from error and continue wizard
    fn recover(&self, error: &WizardError) -> WizardResult<RecoveryAction>;
}

/// Recovery action after error
#[derive(Debug, Clone)]
pub enum RecoveryAction {
    /// Retry the current prompt
    Retry,

    /// Skip to next prompt
    Skip,

    /// Abort the wizard session
    Abort,

    /// Use default value and continue
    UseDefault(serde_json::Value),
}

/// Default recovery strategy (retry validation errors, abort on system errors)
#[derive(Debug, Clone, Copy)]
pub struct DefaultRecovery;

impl RecoveryStrategy for DefaultRecovery {
    fn recover(&self, error: &WizardError) -> WizardResult<RecoveryAction> {
        match error {
            WizardError::ResponseValidation { .. } => Ok(RecoveryAction::Retry),
            WizardError::UserAborted { .. } => Ok(RecoveryAction::Abort),
            WizardError::Timeout { .. } => Ok(RecoveryAction::Abort),
            _ => Ok(RecoveryAction::Abort),
        }
    }
}
```

---

## 3. Feature Flag Design

### 3.1 Cargo.toml Integration

```toml
# Wizard feature (AI-powered CLI wizards)
wizard = ["async", "dep:rust-genai", "dep:tokio"]

# Optional: Include in frontier features
frontier-wizard = ["wizard", "agent2028", "rdf"]
```

### 3.2 Conditional Compilation Strategy

```rust
// In src/lib.rs
#[cfg(feature = "wizard")]
pub mod wizard;

// Conditional re-exports
#[cfg(feature = "wizard")]
pub use wizard::{
    WizardBuilder, WizardConfig, WizardError, WizardPrompt,
    WizardResponse, WizardResult, WizardSession,
};
```

### 3.3 Feature-Gated Types

```rust
// AI-specific types only compiled when wizard feature enabled
#[cfg(feature = "wizard")]
mod ai {
    use rust_genai::Client;

    /// AI-powered suggestion engine
    pub struct SuggestionEngine {
        client: Client,
        config: super::AIConfig,
    }

    impl SuggestionEngine {
        /// Generate suggestions for current prompt
        pub async fn suggest(
            &self,
            prompt: &WizardPrompt,
            context: &[Exchange],
        ) -> WizardResult<Vec<String>> {
            // Implementation uses rust-genai
            todo!("AI suggestion generation")
        }
    }
}

// Stub implementation when wizard feature disabled
#[cfg(not(feature = "wizard"))]
mod ai {
    // Empty module - AI types not available without wizard feature
}
```

---

## 4. Module Organization

### 4.1 Directory Structure

```
src/wizard/
├── mod.rs              # Public API and re-exports
├── session.rs          # WizardSession state machine
├── prompt.rs           # Prompt types and builders
├── response.rs         # Response validation and parsing
├── error.rs            # Error types and recovery
├── config.rs           # Configuration types
├── builder.rs          # WizardBuilder (ergonomic API)
├── validation.rs       # Constraint validation logic
├── receipt.rs          # Deterministic session receipts
└── ai/
    ├── mod.rs          # AI integration (feature-gated)
    ├── suggestions.rs  # Suggestion engine
    └── context.rs      # AI context management
```

### 4.2 Module Dependencies

```
wizard (public API)
├── session (state machine)
│   ├── prompt
│   ├── response
│   └── config
├── builder (ergonomic construction)
│   └── config
├── error (error handling)
├── validation (constraint checking)
│   └── response
├── receipt (deterministic outputs)
│   └── session
└── ai (feature-gated)
    ├── suggestions
    └── context
```

---

## 5. API Surface Design

### 5.1 Builder Pattern (Primary API)

```rust
/// Ergonomic wizard builder
///
/// Follows CliBuilder pattern for consistency
pub struct WizardBuilder {
    prompts: Vec<WizardPrompt>,
    config: WizardConfig,
    context: Option<AppContext>,
    recovery: Box<dyn RecoveryStrategy>,
}

impl WizardBuilder {
    /// Create a new wizard builder
    pub fn new() -> Self {
        Self {
            prompts: Vec::new(),
            config: WizardConfig::default(),
            context: None,
            recovery: Box::new(DefaultRecovery),
        }
    }

    /// Add a text prompt
    pub fn prompt_text(mut self, text: impl Into<String>) -> Self {
        self.prompts.push(WizardPrompt::text(text));
        self
    }

    /// Add a choice prompt
    pub fn prompt_choice(
        mut self,
        text: impl Into<String>,
        options: &'static [&'static str],
    ) -> Self {
        self.prompts.push(WizardPrompt::choice(text, options));
        self
    }

    /// Add a custom prompt
    pub fn prompt(mut self, prompt: WizardPrompt) -> Self {
        self.prompts.push(prompt);
        self
    }

    /// Set wizard configuration
    pub fn config(mut self, config: WizardConfig) -> Self {
        self.config = config;
        self
    }

    /// Set application context
    pub fn context(mut self, context: AppContext) -> Self {
        self.context = Some(context);
        self
    }

    /// Set recovery strategy
    pub fn recovery<R: RecoveryStrategy + 'static>(mut self, recovery: R) -> Self {
        self.recovery = Box::new(recovery);
        self
    }

    /// Build the wizard session (starts in New state)
    pub fn build(self) -> WizardResult<WizardSession<New>> {
        WizardSession::new(
            self.prompts,
            self.config,
            self.context.unwrap_or_default(),
        )
    }
}
```

### 5.2 State Machine Transitions (Type-Safe)

```rust
impl WizardSession<New> {
    /// Start the wizard (New -> Prompting)
    pub fn start(self) -> WizardSession<Prompting> {
        WizardSession {
            state: PhantomData,
            id: self.id,
            conversation: self.conversation,
            context: self.context,
            config: self.config,
        }
    }
}

impl WizardSession<Prompting> {
    /// Present next prompt to user
    pub async fn next_prompt(&self) -> WizardResult<Option<&WizardPrompt>> {
        // Returns None when all prompts completed
        todo!()
    }

    /// Submit response and validate (Prompting -> Processing)
    pub fn submit_response(
        self,
        response: WizardResponse,
    ) -> WizardResult<WizardSession<Processing>> {
        // Validate response before transitioning
        todo!()
    }

    /// Abort wizard (Prompting -> Aborted)
    pub fn abort(self) -> WizardSession<Aborted> {
        WizardSession {
            state: PhantomData,
            id: self.id,
            conversation: self.conversation,
            context: self.context,
            config: self.config,
        }
    }
}

impl WizardSession<Processing> {
    /// Process response and move to next prompt (Processing -> Prompting)
    pub async fn process(self) -> WizardResult<WizardSession<Prompting>> {
        todo!()
    }

    /// Complete wizard if all prompts finished (Processing -> Complete)
    pub fn complete(self) -> WizardSession<Complete> {
        WizardSession {
            state: PhantomData,
            id: self.id,
            conversation: self.conversation,
            context: self.context,
            config: self.config,
        }
    }
}

impl WizardSession<Complete> {
    /// Generate deterministic receipt
    pub fn receipt(&self) -> WizardReceipt {
        WizardReceipt::from_session(self)
    }

    /// Extract all responses as JSON
    pub fn responses(&self) -> serde_json::Value {
        serde_json::to_value(&self.conversation)
            .unwrap_or(serde_json::Value::Null)
    }
}

impl WizardSession<Aborted> {
    /// Get partial responses before abort
    pub fn partial_responses(&self) -> &[Exchange] {
        &self.conversation
    }
}
```

### 5.3 Trait-Based Extension Points

```rust
/// Trait for wizard lifecycle hooks
pub trait WizardHook {
    /// Called before each prompt
    fn before_prompt(&mut self, prompt: &WizardPrompt) -> WizardResult<()> {
        Ok(())
    }

    /// Called after each response
    fn after_response(&mut self, response: &WizardResponse) -> WizardResult<()> {
        Ok(())
    }

    /// Called on wizard completion
    fn on_complete(&mut self, session: &WizardSession<Complete>) -> WizardResult<()> {
        Ok(())
    }

    /// Called on wizard abort
    fn on_abort(&mut self, session: &WizardSession<Aborted>) -> WizardResult<()> {
        Ok(())
    }
}
```

---

## 6. Integration Points

### 6.1 Integration with clap-noun-verb Framework

```rust
// In user's CLI code
use clap_noun_verb::{verb, AppContext, WizardBuilder};

#[verb(name = "init", noun = "project")]
async fn init_project(ctx: AppContext) -> Result<(), Box<dyn std::error::Error>> {
    let wizard = WizardBuilder::new()
        .context(ctx)
        .prompt_text("Project name:")
        .prompt_choice("Project type:", &["library", "binary", "workspace"])
        .prompt_text("Author name:")
        .prompt_text("License:")
        .config(WizardConfig::default())
        .build()?;

    let session = wizard.start();

    // Run wizard interactively
    let completed = run_wizard_session(session).await?;

    // Extract responses
    let responses = completed.responses();

    // Generate receipt for reproducibility
    let receipt = completed.receipt();

    // Use responses to initialize project
    initialize_project_from_responses(&responses)?;

    Ok(())
}
```

### 6.2 Context Passing

```rust
impl WizardSession<Prompting> {
    /// Access parent CLI context
    pub fn context(&self) -> &AppContext {
        &self.context
    }

    /// Store value in context for later use
    pub fn store_in_context<T: Send + Sync + 'static>(
        &self,
        value: T,
    ) -> WizardResult<()> {
        self.context.insert(value)
            .map_err(WizardError::from)
    }
}
```

### 6.3 Output Formatting Integration

```rust
impl WizardSession<Complete> {
    /// Format wizard results using existing OutputFormat
    pub fn format_output(&self, format: OutputFormat) -> WizardResult<String> {
        match format {
            OutputFormat::Json => {
                serde_json::to_string_pretty(&self.conversation)
                    .map_err(WizardError::from)
            }
            OutputFormat::Yaml => {
                #[cfg(feature = "config-formats")]
                serde_yaml::to_string(&self.conversation)
                    .map_err(|e| WizardError::Serialization(
                        serde_json::Error::custom(e.to_string())
                    ))

                #[cfg(not(feature = "config-formats"))]
                Err(WizardError::Serialization(
                    serde_json::Error::custom("YAML format requires config-formats feature")
                ))
            }
            OutputFormat::Text => {
                Ok(self.conversation
                    .iter()
                    .map(|ex| format!("{}: {}", ex.prompt.text, ex.response.raw_input))
                    .collect::<Vec<_>>()
                    .join("\n"))
            }
        }
    }
}
```

### 6.4 Async Runtime Integration

```rust
// Wizard operations are async to support AI calls
#[cfg(feature = "wizard")]
pub async fn run_wizard_session(
    mut session: WizardSession<Prompting>,
) -> WizardResult<WizardSession<Complete>> {
    while let Some(prompt) = session.next_prompt().await? {
        // Get AI suggestions if enabled
        #[cfg(feature = "wizard")]
        let suggestions = if session.config.enable_ai {
            session.generate_suggestions(prompt).await?
        } else {
            Vec::new()
        };

        // Present prompt to user
        let response = prompt_user(prompt, suggestions).await?;

        // Submit and validate
        session = session.submit_response(response)?;

        // Process response
        session = session.process().await?;

        // Check if complete
        if session.is_complete() {
            return Ok(session.complete());
        }
    }

    Ok(session.complete())
}
```

---

## 7. Performance Characteristics

### 7.1 Zero-Cost Abstractions

| Component | Abstraction | Runtime Cost |
|-----------|-------------|--------------|
| `SessionState` | Type-level state machine | **Zero** - PhantomData optimized away |
| `SessionId` | Wrapper around UUID | **Zero** - #[repr(transparent)] |
| `PromptId` | Wrapper around u64 | **Zero** - #[repr(transparent)] |
| State transitions | Type-safe APIs | **Zero** - Monomorphization |
| Generic config | `WizardConfig` | **Zero** - Const propagation |

### 7.2 Memory Layout

```rust
// Session memory layout (no overhead from state machine)
assert_eq!(
    std::mem::size_of::<WizardSession<New>>(),
    std::mem::size_of::<WizardSession<Complete>>()
);

// Zero-cost wrappers
assert_eq!(
    std::mem::size_of::<SessionId>(),
    std::mem::size_of::<uuid::Uuid>()
);
```

### 7.3 Performance SLOs

- Session creation: ≤ 1ms
- Prompt validation: ≤ 100μs per prompt
- Response validation: ≤ 500μs per response
- Receipt generation: ≤ 10ms
- AI suggestion (when enabled): ≤ 2s (network-dependent)

---

## 8. Deterministic Outputs

### 8.1 Wizard Receipt

```rust
/// Deterministic wizard session receipt
///
/// Can be replayed to reproduce exact same session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WizardReceipt {
    /// Session identifier
    session_id: SessionId,

    /// Conversation history
    exchanges: Vec<Exchange>,

    /// Session metadata
    metadata: ReceiptMetadata,

    /// Cryptographic hash of session
    #[cfg(feature = "crypto")]
    hash: ReceiptHash,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiptMetadata {
    /// Start timestamp
    started_at: Timestamp,

    /// Completion timestamp
    completed_at: Timestamp,

    /// Wizard version
    version: &'static str,

    /// Configuration used
    config: WizardConfig,
}

impl WizardReceipt {
    /// Replay receipt to reconstruct session
    pub fn replay(&self) -> WizardResult<WizardSession<Complete>> {
        todo!("Replay wizard session from receipt")
    }

    /// Verify receipt integrity
    #[cfg(feature = "crypto")]
    pub fn verify(&self) -> WizardResult<bool> {
        let computed_hash = self.compute_hash()?;
        Ok(computed_hash == self.hash)
    }
}
```

---

## 9. Testing Strategy

### 9.1 Unit Tests (Chicago TDD)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_state_transitions() {
        // Arrange
        let session = WizardSession::<New>::default();

        // Act - type-safe transitions
        let prompting = session.start();
        let aborted = prompting.abort();

        // Assert - state is encoded in type
        assert!(matches!(aborted, WizardSession::<Aborted> { .. }));
    }

    #[test]
    fn test_response_validation() {
        // Arrange
        let prompt = WizardPrompt::integer("Age:", Some(0), Some(120));

        // Act
        let valid = prompt.validate_response("25");
        let invalid = prompt.validate_response("150");

        // Assert
        assert!(valid.is_ok());
        assert!(invalid.is_err());
    }

    #[test]
    fn test_zero_cost_wrappers() {
        // Arrange & Assert - verify zero-cost abstractions
        assert_eq!(
            std::mem::size_of::<SessionId>(),
            std::mem::size_of::<uuid::Uuid>()
        );

        assert_eq!(
            std::mem::size_of::<WizardSession<New>>(),
            std::mem::size_of::<WizardSession<Complete>>()
        );
    }
}
```

### 9.2 Property Tests

```rust
#[cfg(test)]
mod proptests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_session_id_roundtrip(id in any::<u128>()) {
            let session_id = SessionId::from_u128(id);
            let serialized = serde_json::to_string(&session_id)?;
            let deserialized: SessionId = serde_json::from_str(&serialized)?;
            prop_assert_eq!(session_id, deserialized);
        }

        #[test]
        fn test_receipt_determinism(exchanges in prop::collection::vec(any::<Exchange>(), 1..10)) {
            let receipt1 = WizardReceipt::from_exchanges(exchanges.clone());
            let receipt2 = WizardReceipt::from_exchanges(exchanges);
            prop_assert_eq!(receipt1.hash, receipt2.hash);
        }
    }
}
```

### 9.3 Integration Tests

```rust
#[tokio::test]
#[cfg(feature = "wizard")]
async fn test_wizard_integration_with_cli() {
    // Arrange
    let ctx = AppContext::new();
    let wizard = WizardBuilder::new()
        .context(ctx)
        .prompt_text("Name:")
        .build()
        .unwrap();

    // Act
    let session = wizard.start();
    let response = WizardResponse::new(PromptId(0), "Test Name".to_string());
    let processing = session.submit_response(response).unwrap();
    let completed = processing.complete();

    // Assert
    let receipt = completed.receipt();
    assert_eq!(receipt.exchanges.len(), 1);
    assert_eq!(receipt.exchanges[0].response.raw_input, "Test Name");
}
```

---

## 10. Migration Path and Backwards Compatibility

### 10.1 Feature Flag Guarantee

- **Without `wizard` feature**: Zero compilation overhead, no rust-genai dependency
- **With `wizard` feature**: Full wizard capabilities available
- **Existing code**: Completely unaffected, no breaking changes

### 10.2 Deprecation Strategy

- No deprecations required (net new feature)
- Future wizard v2.0 changes will use standard deprecation process

---

## 11. Security Considerations

### 11.1 Input Validation

- All user inputs validated before processing
- Type-safe constraints prevent injection attacks
- File path validation prevents directory traversal

### 11.2 AI Safety

- AI suggestions are optional and user-controlled
- No automatic execution of AI-generated code
- All AI interactions logged in receipt

### 11.3 Receipt Integrity

```rust
#[cfg(feature = "crypto")]
impl WizardReceipt {
    fn compute_hash(&self) -> WizardResult<ReceiptHash> {
        use sha3::{Sha3_256, Digest};

        let mut hasher = Sha3_256::new();
        let serialized = serde_json::to_vec(&self.exchanges)?;
        hasher.update(&serialized);

        Ok(ReceiptHash(hasher.finalize().into()))
    }
}
```

---

## 12. Dependencies

### 12.1 Required Dependencies (when `wizard` feature enabled)

```toml
[dependencies]
# AI capabilities
rust-genai = { version = "0.1", optional = true }

# Async runtime (already optional)
tokio = { version = "1.40", features = ["rt", "time"], optional = true }

# Existing dependencies (no new required deps)
uuid = { version = "1.0", features = ["v4"], optional = true }  # Already in agent2028
serde = { version = "1.0", features = ["derive"] }  # Already required
serde_json = "1.0"  # Already required
```

### 12.2 Dependency Graph Impact

- **Without wizard**: No impact (0 new dependencies)
- **With wizard**: +1 dependency (rust-genai), reuses existing async infrastructure

---

## 13. Future Extensions

### 13.1 Planned v2.0 Features

- Multi-step wizards with branching logic
- Wizard templates for common patterns
- Integration with RDF ontology for semantic wizards
- LLM-powered wizard generation from specifications

### 13.2 Extension Points

```rust
/// Trait for custom wizard behaviors
pub trait WizardBehavior {
    fn customize_prompt(&self, prompt: &mut WizardPrompt);
    fn customize_response(&self, response: &mut WizardResponse);
}
```

---

## 14. Architecture Decision Records (ADRs)

### ADR-001: Type-Level State Machine

**Decision**: Use PhantomData-based type-level state machine for session states.

**Rationale**:
- Zero runtime overhead (PhantomData optimized away)
- Compile-time enforcement of valid state transitions
- Prevents invalid API usage (e.g., can't call `complete()` on `New` state)

**Consequences**:
- More complex type signatures
- Better safety and performance
- API is self-documenting through types

### ADR-002: Feature-Gated AI Integration

**Decision**: AI capabilities only available with `wizard` feature flag.

**Rationale**:
- Avoids forcing rust-genai dependency on all users
- Minimal compile time impact for non-AI use cases
- Allows wizard to work offline without AI

**Consequences**:
- Some duplication in AI-gated vs non-AI code paths
- Clear performance boundary (AI is optional overhead)

### ADR-003: Deterministic Receipts

**Decision**: Generate cryptographic receipts for all wizard sessions.

**Rationale**:
- Enables session replay for debugging
- Audit trail for compliance
- Reproducible builds and testing

**Consequences**:
- Requires crypto feature for hashing
- Small overhead for hash computation
- Excellent for debugging and verification

---

## 15. Conclusion

This architecture provides a **type-first, zero-cost, AI-powered wizard system** that:

✅ Encodes invariants in types (compile-time safety)
✅ Uses zero-cost abstractions (PhantomData, repr(transparent))
✅ Integrates seamlessly with existing clap-noun-verb patterns
✅ Maintains backwards compatibility (feature-gated)
✅ Provides deterministic outputs (receipts)
✅ Follows Rust best practices (ownership, lifetimes, error handling)
✅ Supports Chicago TDD testing methodology

**Next Steps**:
1. Review and approve architecture
2. Implement core types (session, prompt, response, error)
3. Implement builder and state machine
4. Add AI integration (feature-gated)
5. Write comprehensive tests (Chicago TDD)
6. Benchmark against SLOs
7. Document public APIs

---

**Architecture Review Status**: ✅ **Ready for Implementation**
