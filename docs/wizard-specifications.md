# Wizard Package - System Requirements Specification

**Project**: clap-noun-verb
**Component**: wizard package
**Version**: 0.1.0
**Date**: 2026-01-09
**Methodology**: SPARC + Chicago TDD + DfLSS
**Branch**: claude/wizard-package-launch-Qumas

---

## 1. Introduction

### 1.1 Purpose
The wizard package provides AI-powered interactive CLI capabilities for the clap-noun-verb framework, integrating rust-genai to enable intelligent command generation, guided workflows, and context-aware assistance.

### 1.2 Scope

**In Scope**:
- Wizard CLI commands and subcommands
- AI model integration via rust-genai
- Interactive, script, and API interaction modes
- Session lifecycle management
- Output formatting with determinism guarantees
- Feature flag support (conditional compilation)
- Integration with existing clap noun-verb macros

**Out of Scope**:
- Custom AI model training
- Persistent session storage (v0.1.0)
- Web UI interface
- Multi-user collaboration features

### 1.3 Definitions

- **Wizard**: Interactive AI-powered CLI assistant for command generation
- **Session**: Stateful conversation context with the AI model
- **Interaction Mode**: Method of engaging with the wizard (interactive/script/API)
- **Determinism**: Guarantee of reproducible outputs given identical inputs
- **Feature Flag**: Compile-time conditional compilation based on `wizard` feature

---

## 2. Functional Requirements

### 2.1 Wizard CLI Commands (FR-2.1)

#### FR-2.1.1: Primary Command Structure
```rust
// Type-first command structure using clap derives
#[derive(Parser)]
#[noun(wizard)]
struct WizardNoun {
    #[verb(start)]
    start: StartVerb,

    #[verb(chat)]
    chat: ChatVerb,

    #[verb(generate)]
    generate: GenerateVerb,

    #[verb(configure)]
    configure: ConfigureVerb,
}
```

**Acceptance Criteria**:
- [ ] `wizard start` initializes new wizard session
- [ ] `wizard chat` enters interactive conversation mode
- [ ] `wizard generate` produces command suggestions
- [ ] `wizard configure` manages AI model settings
- [ ] All commands compile with `--features wizard`
- [ ] Stub implementations compile without feature flag

#### FR-2.1.2: Start Command
```yaml
command: wizard start
purpose: Initialize wizard session
arguments:
  - name: model
    type: String
    required: false
    default: "gpt-4"
    description: "AI model identifier"
  - name: temperature
    type: f32
    required: false
    default: 0.7
    range: [0.0, 2.0]
    description: "Model temperature for response randomness"
  - name: max-tokens
    type: u32
    required: false
    default: 2048
    description: "Maximum tokens per response"

outputs:
  - session_id: UUID
  - model_config: ModelConfiguration
  - status: SessionStatus

error_cases:
  - InvalidModelError: Unknown model identifier
  - ConfigurationError: Invalid parameter values
  - ApiKeyMissingError: No API key configured
```

**Acceptance Criteria**:
- [ ] Session initializes in ≤ 100ms (P95)
- [ ] Returns unique session UUID
- [ ] Validates all parameters before API calls
- [ ] Fails fast with clear error messages
- [ ] Idempotent: same inputs = same session config

#### FR-2.1.3: Chat Command
```yaml
command: wizard chat
purpose: Interactive conversation with AI assistant
arguments:
  - name: session-id
    type: UUID
    required: false
    description: "Resume existing session"
  - name: prompt
    type: String
    required: false
    description: "Initial prompt (for script mode)"
  - name: mode
    type: InteractionMode
    required: false
    default: interactive
    values: [interactive, script, one-shot]

behavior:
  interactive:
    - Display REPL prompt
    - Accept multi-line input (Ctrl+D to submit)
    - Stream responses in real-time
    - Maintain conversation history
    - Support commands: /exit, /clear, /history, /save

  script:
    - Accept prompt from --prompt argument or stdin
    - Generate single response
    - Exit after response
    - Return exit code 0 on success

  one-shot:
    - Single prompt-response cycle
    - No session persistence
    - Stateless operation

outputs:
  - response: String
  - token_usage: TokenUsage
  - latency: Duration
```

**Acceptance Criteria**:
- [ ] Interactive mode supports REPL with history
- [ ] Script mode completes in single invocation
- [ ] Responses stream with ≤ 200ms first token latency
- [ ] Conversation history maintained per session
- [ ] All modes handle Ctrl+C gracefully

#### FR-2.1.4: Generate Command
```yaml
command: wizard generate
purpose: Generate command suggestions based on natural language
arguments:
  - name: description
    type: String
    required: true
    description: "Natural language description of desired command"
  - name: format
    type: OutputFormat
    required: false
    default: text
    values: [text, json, yaml, shell]
  - name: validate
    type: bool
    required: false
    default: true
    description: "Validate generated commands"

outputs:
  - commands: Vec<GeneratedCommand>
  - confidence: f32
  - alternatives: Vec<GeneratedCommand>

generated_command_structure:
  - command: String
  - explanation: String
  - confidence: f32
  - safety_level: SafetyLevel [safe, moderate, dangerous]
  - dry_run_suggested: bool
```

**Acceptance Criteria**:
- [ ] Generates 1-3 command suggestions per request
- [ ] Response time ≤ 5s (P95)
- [ ] Confidence scores in range [0.0, 1.0]
- [ ] Safety levels accurately categorized
- [ ] Validation catches common errors

#### FR-2.1.5: Configure Command
```yaml
command: wizard configure
purpose: Manage wizard configuration
subcommands:
  - set:
      arguments:
        - key: String (required)
        - value: String (required)
      examples:
        - "wizard configure set model gpt-4"
        - "wizard configure set temperature 0.5"

  - get:
      arguments:
        - key: String (optional)
      behavior: "Show single key or all config"

  - reset:
      behavior: "Reset to default configuration"

  - list-models:
      behavior: "Show available AI models"

configuration_keys:
  - model: String
  - temperature: f32
  - max_tokens: u32
  - api_key: String (sensitive)
  - api_endpoint: Url
  - timeout: Duration
```

**Acceptance Criteria**:
- [ ] Configuration persists across sessions
- [ ] API keys stored securely (not in plaintext)
- [ ] Invalid configurations rejected with clear errors
- [ ] `list-models` queries available models from API
- [ ] All operations complete in ≤ 500ms

### 2.2 Session Lifecycle Management (FR-2.2)

#### FR-2.2.1: Session State Machine
```rust
// Type-safe state machine using sealed traits
pub enum SessionState {
    Uninitialized,
    Initializing,
    Active { context: ConversationContext },
    Paused,
    Terminated,
    Error { reason: ErrorReason },
}

// State transitions encoded in types
impl Session {
    pub fn initialize(config: Config) -> Result<Session<Active>, InitError>;
    pub fn pause(self: Session<Active>) -> Session<Paused>;
    pub fn resume(self: Session<Paused>) -> Result<Session<Active>, ResumeError>;
    pub fn terminate(self) -> ();
}
```

**State Transition Invariants**:
- `Uninitialized → Initializing → Active`: Successful initialization
- `Active → Paused → Active`: Pause/resume cycle
- `Active → Terminated`: Normal termination
- `Any → Error`: Error state from any state
- `Error → Terminated`: Error recovery path

**Acceptance Criteria**:
- [ ] State transitions are type-safe (invalid transitions impossible)
- [ ] Each state has well-defined entry/exit conditions
- [ ] State machine is deterministic
- [ ] All states implement Drop for cleanup
- [ ] Transitions validated by tests

#### FR-2.2.2: Conversation Context
```rust
pub struct ConversationContext {
    session_id: SessionId,
    messages: Vec<Message>,
    token_count: TokenCount,
    created_at: Timestamp,
    last_activity: Timestamp,
}

pub struct Message {
    role: Role,  // User | Assistant | System
    content: String,
    tokens: u32,
    timestamp: Timestamp,
}
```

**Context Management Rules**:
- Maximum 100 messages per session (token limit enforcement)
- Automatic context pruning when approaching token limits
- System messages preserved during pruning
- Recent messages prioritized over older ones

**Acceptance Criteria**:
- [ ] Context size bounded by token limits
- [ ] Pruning maintains conversation coherence
- [ ] System messages never pruned
- [ ] Context serializable to/from JSON

### 2.3 Interaction Modes (FR-2.3)

#### FR-2.3.1: Interactive Mode
```yaml
mode: interactive
behavior:
  - REPL-style interface
  - Multi-line input support
  - Real-time response streaming
  - Command history (up/down arrows)
  - Auto-completion for commands
  - Syntax highlighting for generated code

commands:
  /exit: "Terminate session"
  /clear: "Clear conversation history"
  /history: "Show message history"
  /save <file>: "Export conversation to file"
  /load <file>: "Import conversation from file"
  /config: "Show current configuration"
  /help: "Display help message"

ux_requirements:
  - Response streaming with progress indicator
  - Color-coded output (prompts, responses, errors)
  - Graceful terminal resize handling
  - Unicode support
```

**Acceptance Criteria**:
- [ ] REPL accepts multi-line input
- [ ] Command history persists across prompts
- [ ] Streaming responses visible in real-time
- [ ] All slash commands functional
- [ ] Terminal resizing doesn't break UI

#### FR-2.3.2: Script Mode
```yaml
mode: script
behavior:
  - Single prompt → single response
  - Accept input from CLI args or stdin
  - Output to stdout (machine-readable)
  - Exit code indicates success/failure

input_sources:
  - --prompt "text": Inline prompt
  - stdin: Pipe or redirect input
  - --file path: Read from file

output_formats:
  - text: Human-readable (default)
  - json: Structured data
  - yaml: Structured data (alternative)

exit_codes:
  0: Success
  1: General error
  2: Configuration error
  3: API error
  4: Timeout
```

**Acceptance Criteria**:
- [ ] Accepts prompts from multiple sources
- [ ] Output format selectable via --format flag
- [ ] Exit codes match specification
- [ ] Pipeable in shell scripts
- [ ] No interactive prompts in script mode

#### FR-2.3.3: API Mode
```rust
// Programmatic API for embedding wizard in Rust applications
pub struct WizardClient {
    config: ClientConfig,
    runtime: Runtime,
}

impl WizardClient {
    pub async fn new(config: ClientConfig) -> Result<Self, ClientError>;

    pub async fn chat(
        &self,
        prompt: &str,
        options: ChatOptions,
    ) -> Result<ChatResponse, ChatError>;

    pub async fn generate_command(
        &self,
        description: &str,
    ) -> Result<Vec<GeneratedCommand>, GenerateError>;

    pub async fn stream_chat(
        &self,
        prompt: &str,
    ) -> Result<impl Stream<Item = ResponseChunk>, ChatError>;
}
```

**Acceptance Criteria**:
- [ ] Client supports async/await
- [ ] Streaming responses via async streams
- [ ] Thread-safe (Send + Sync)
- [ ] Graceful shutdown
- [ ] Resource cleanup on drop

### 2.4 AI Model Configuration (FR-2.4)

#### FR-2.4.1: rust-genai Integration
```rust
use genai::{Client, ModelIden, ChatRequest, ChatOptions};

pub struct WizardModelConfig {
    pub model: ModelIden,
    pub temperature: f32,
    pub max_tokens: u32,
    pub top_p: f32,
    pub frequency_penalty: f32,
    pub presence_penalty: f32,
}

impl Default for WizardModelConfig {
    fn default() -> Self {
        Self {
            model: ModelIden::from("gpt-4"),
            temperature: 0.7,
            max_tokens: 2048,
            top_p: 1.0,
            frequency_penalty: 0.0,
            presence_penalty: 0.0,
        }
    }
}
```

**Supported Models**:
- OpenAI: gpt-4, gpt-4-turbo, gpt-3.5-turbo
- Anthropic: claude-3-opus, claude-3-sonnet
- Local: ollama/llama2, ollama/mistral
- Configurable via rust-genai adapter pattern

**Acceptance Criteria**:
- [ ] All rust-genai supported models available
- [ ] Model switching without recompilation
- [ ] Configuration validation before API calls
- [ ] Fallback to default on invalid config
- [ ] Model capabilities queried at runtime

#### FR-2.4.2: Determinism Guarantees
```yaml
determinism_modes:
  deterministic:
    temperature: 0.0
    top_p: 1.0
    seed: fixed
    guarantee: "Identical inputs → identical outputs"
    use_case: "Testing, reproducibility"

  balanced:
    temperature: 0.7
    top_p: 0.9
    seed: random
    guarantee: "Varied but coherent outputs"
    use_case: "General usage"

  creative:
    temperature: 1.2
    top_p: 0.95
    seed: random
    guarantee: "Maximum diversity"
    use_case: "Brainstorming, alternatives"
```

**Acceptance Criteria**:
- [ ] Deterministic mode produces identical outputs
- [ ] Seed configurable for reproducibility
- [ ] Mode selectable via CLI flag
- [ ] Default mode documented clearly
- [ ] Property tests verify determinism

### 2.5 Output Formatting (FR-2.5)

#### FR-2.5.1: Format Types
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Text,      // Human-readable plain text
    Json,      // Structured JSON
    Yaml,      // Structured YAML
    Markdown,  // Formatted markdown
    Shell,     // Shell-executable script
}

pub trait FormatOutput {
    fn format(&self, format: OutputFormat) -> Result<String, FormatError>;
}
```

**Format Specifications**:
```yaml
text:
  description: "Human-readable output"
  features:
    - Word wrapping at 80 chars
    - ANSI color codes (when TTY detected)
    - Section headers

json:
  description: "Machine-readable structured data"
  schema:
    type: object
    required: [response, metadata]
    properties:
      response: {type: string}
      metadata:
        type: object
        properties:
          tokens: {type: integer}
          latency_ms: {type: integer}
          model: {type: string}

yaml:
  description: "Human and machine readable"
  features:
    - Readable structure
    - Comments preserved
    - Multi-line string support

markdown:
  description: "Formatted text with markup"
  features:
    - Code fences for commands
    - Bullet points for lists
    - Headers for sections

shell:
  description: "Executable shell script"
  features:
    - Shebang line included
    - Comments explain commands
    - Error checking (set -e)
    - Shellcheck compatible
```

**Acceptance Criteria**:
- [ ] All formats implement FormatOutput trait
- [ ] JSON output is valid JSON
- [ ] YAML output is valid YAML
- [ ] Shell output is executable
- [ ] Format selection via --format flag
- [ ] Default format: text for TTY, json for pipes

---

## 3. Non-Functional Requirements

### 3.1 Performance (NFR-3.1)

#### NFR-3.1.1: Latency Targets
```yaml
operation: session_initialization
target: ≤ 100ms
percentile: P95
measurement: "Time from wizard start to ready state"
validation: "Benchmark with criterion"

operation: first_token_latency
target: ≤ 200ms
percentile: P50
measurement: "Time from prompt submission to first response token"
validation: "Integration test with real API calls"

operation: full_response_generation
target: ≤ 5s
percentile: P95
measurement: "Time from prompt to complete response"
validation: "End-to-end test with typical prompts"

operation: command_generation
target: ≤ 3s
percentile: P95
measurement: "wizard generate response time"
validation: "Performance test suite"
```

**Acceptance Criteria**:
- [ ] All latency targets met in benchmarks
- [ ] Performance regression tests in CI
- [ ] SLO dashboard shows current metrics
- [ ] `cargo make slo-check` validates targets

#### NFR-3.1.2: Throughput
```yaml
metric: concurrent_sessions
target: 100+ sessions
constraint: "Single process"
measurement: "Number of active sessions without degradation"

metric: requests_per_second
target: 10 RPS per session
constraint: "API rate limits respected"
measurement: "Sustained request rate"

metric: token_throughput
target: 1000 tokens/second
measurement: "Aggregate token processing rate"
```

**Acceptance Criteria**:
- [ ] Load tests verify 100+ concurrent sessions
- [ ] Rate limiting prevents API quota exhaustion
- [ ] Throughput degrades gracefully under load
- [ ] Resource usage scales linearly

### 3.2 Resource Constraints (NFR-3.2)

#### NFR-3.2.1: Memory Usage
```yaml
metric: memory_per_session
target: ≤ 50MB
percentile: P95
measurement: "Heap allocation per active session"
validation: "Memory profiler (valgrind, heaptrack)"

metric: total_memory_footprint
target: ≤ 500MB
constraint: "100 concurrent sessions"
calculation: "Base + (sessions × per_session_memory)"

optimization_strategies:
  - String interning for repeated content
  - Message history pruning
  - Streaming responses (no buffering)
  - Lazy loading of model metadata
```

**Acceptance Criteria**:
- [ ] Memory usage stays within targets
- [ ] No memory leaks detected
- [ ] Memory profiling in CI
- [ ] Graceful OOM handling

#### NFR-3.2.2: Binary Size
```yaml
metric: binary_size
target: ≤ 10MB
measurement: "Stripped release binary"
constraint: "With wizard feature enabled"

metric: binary_size_without_feature
target: ≤ 2MB
measurement: "Stripped release binary without wizard"
```

**Acceptance Criteria**:
- [ ] Release binary meets size targets
- [ ] Feature flag compilation verified
- [ ] Size regression tests in CI

### 3.3 Reliability (NFR-3.3)

#### NFR-3.3.1: Error Handling
```rust
// Comprehensive error types using thiserror
#[derive(Debug, thiserror::Error)]
pub enum WizardError {
    #[error("Session initialization failed: {0}")]
    InitializationError(String),

    #[error("API error: {0}")]
    ApiError(#[from] genai::Error),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Invalid input: {0}")]
    ValidationError(String),

    #[error("Timeout after {0:?}")]
    TimeoutError(Duration),

    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),
}

pub type WizardResult<T> = Result<T, WizardError>;
```

**Error Handling Requirements**:
- All public APIs return `Result<T, E>`
- No `unwrap()` or `expect()` in production code
- Errors include context and recovery suggestions
- Error messages user-friendly and actionable
- Errors logged with appropriate levels

**Acceptance Criteria**:
- [ ] All error paths tested
- [ ] Error messages reviewed for clarity
- [ ] No panics in production code
- [ ] Error recovery tested

#### NFR-3.3.2: Idempotency
```yaml
requirement: "Operations with same inputs produce same outputs"
scope: "Session initialization, command generation (deterministic mode)"

guarantees:
  - "wizard start with same config → identical session state"
  - "wizard generate with deterministic mode → identical commands"
  - "wizard configure set → same configuration"

exceptions:
  - "Session IDs always unique (even with same config)"
  - "Timestamps reflect actual execution time"
  - "Non-deterministic mode intentionally varied"
```

**Acceptance Criteria**:
- [ ] Idempotency tested with property tests
- [ ] Deterministic mode verified
- [ ] Session state reproducible
- [ ] Side effects documented

#### NFR-3.3.3: Data Integrity
```yaml
requirement: "Zero data loss during normal operations"

guarantees:
  - "Conversation history preserved across pauses"
  - "Configuration changes atomic"
  - "API responses fully captured"
  - "Error states don't corrupt data"

validation:
  - "Serialization round-trip tests"
  - "Crash recovery tests"
  - "Concurrent modification tests"
```

**Acceptance Criteria**:
- [ ] No data loss in crash scenarios
- [ ] Atomic operations tested
- [ ] Serialization verified
- [ ] Concurrent access safe

### 3.4 Observability (NFR-3.4)

#### NFR-3.4.1: Tracing Integration
```rust
use tracing::{info, warn, error, debug, trace};

// Instrument all public APIs
#[tracing::instrument(skip(self))]
pub async fn chat(&self, prompt: &str) -> WizardResult<Response> {
    info!("Starting chat with prompt length: {}", prompt.len());

    // Span tracks entire operation
    let _span = tracing::span!(
        tracing::Level::INFO,
        "wizard_chat",
        session_id = %self.session_id,
        model = %self.config.model
    );

    // ... implementation
}
```

**Tracing Requirements**:
- All public APIs instrumented
- Spans for async operations
- Fields include: session_id, model, operation
- Log levels: ERROR (failures), WARN (recoverable), INFO (operations), DEBUG (details), TRACE (verbose)

**Acceptance Criteria**:
- [ ] Tracing enabled in all modules
- [ ] Spans properly nested
- [ ] Performance overhead ≤ 5%
- [ ] JSON output for structured logging

#### NFR-3.4.2: Metrics
```yaml
metrics:
  - name: wizard_sessions_active
    type: gauge
    description: "Number of active wizard sessions"

  - name: wizard_requests_total
    type: counter
    labels: [operation, status]
    description: "Total requests by operation and status"

  - name: wizard_request_duration_seconds
    type: histogram
    labels: [operation]
    buckets: [0.1, 0.5, 1.0, 2.0, 5.0, 10.0]
    description: "Request duration distribution"

  - name: wizard_tokens_processed
    type: counter
    labels: [model]
    description: "Total tokens processed by model"
```

**Acceptance Criteria**:
- [ ] Metrics exported in Prometheus format
- [ ] Metrics updated in real-time
- [ ] Dashboard queries documented
- [ ] SLO metrics included

---

## 4. Feature Flag Specifications

### 4.1 Feature Flag Behavior (FF-4.1)

#### FF-4.1.1: Enabled State (`--features wizard`)
```yaml
behavior: "Full AI capabilities"
dependencies:
  - rust-genai = "0.1"
  - tokio = { features = ["full"] }
  - reqwest = { features = ["json", "rustls-tls"] }
  - serde = { features = ["derive"] }
  - tracing = "0.1"

compiled_modules:
  - wizard::cli
  - wizard::session
  - wizard::client
  - wizard::ai_model
  - wizard::format

binary_size: "~10MB"
startup_time: "~100ms"
```

**Acceptance Criteria**:
- [ ] All wizard commands available
- [ ] rust-genai integration functional
- [ ] Full feature set accessible
- [ ] Tests pass with feature enabled

#### FF-4.1.2: Disabled State (default, no feature flag)
```yaml
behavior: "Stub implementations"
dependencies:
  - Minimal dependencies only
  - No AI libraries

compiled_modules:
  - wizard::stub (provides no-op implementations)

implementation:
  ```rust
  #[cfg(not(feature = "wizard"))]
  pub mod wizard {
      pub fn start() -> Result<(), String> {
          Err("Wizard feature not enabled. \
               Recompile with --features wizard".to_string())
      }
  }
  ```

binary_size: "~2MB (no increase)"
error_message: "Clear instruction to enable feature"
```

**Acceptance Criteria**:
- [ ] Compilation succeeds without feature
- [ ] Binary size minimal without feature
- [ ] Error messages helpful
- [ ] No AI dependencies included

#### FF-4.1.3: Feature Transition Matrix
```yaml
transition_from_disabled_to_enabled:
  - Recompile with --features wizard
  - No data migration needed (no data yet)
  - Configuration created on first use

transition_from_enabled_to_disabled:
  - Existing sessions terminated
  - Configuration preserved (ignored)
  - No runtime errors (compile-time only)
```

---

## 5. Integration Specifications

### 5.1 Clap Integration (INT-5.1)

#### INT-5.1.1: Noun-Verb Macro Integration
```rust
// Integration with existing clap-noun-verb infrastructure
use clap_noun_verb::{noun, verb, Parser};

#[derive(Parser)]
#[noun(wizard)]
#[cfg(feature = "wizard")]
pub struct WizardNoun {
    /// AI model to use
    #[arg(long, default_value = "gpt-4")]
    pub model: String,

    /// Temperature for response generation
    #[arg(long, default_value = "0.7")]
    pub temperature: f32,

    #[verb(start)]
    pub start: StartVerb,

    #[verb(chat)]
    pub chat: ChatVerb,

    #[verb(generate)]
    pub generate: GenerateVerb,
}

// Stub when disabled
#[derive(Parser)]
#[noun(wizard)]
#[cfg(not(feature = "wizard"))]
pub struct WizardNoun {
    // Minimal stub that shows helpful error
}
```

**Acceptance Criteria**:
- [ ] Macro expansion works with feature flag
- [ ] Help text generated correctly
- [ ] Subcommands registered properly
- [ ] Type safety preserved

#### INT-5.1.2: Argument Parsing
```yaml
integration_points:
  - Global args propagate to subcommands
  - Environment variables respected (WIZARD_MODEL, WIZARD_API_KEY)
  - Config file support (.wizard.toml)
  - Precedence: CLI args > env vars > config file > defaults

configuration_hierarchy:
  1. Command-line arguments (highest priority)
  2. Environment variables
  3. Configuration file (~/.wizard.toml)
  4. Built-in defaults (lowest priority)
```

**Acceptance Criteria**:
- [ ] Argument precedence tested
- [ ] Environment variables work
- [ ] Config file parsing tested
- [ ] Defaults documented

### 5.2 Output Module Integration (INT-5.2)

#### INT-5.2.1: Format Module Usage
```rust
// Use existing format module from clap-noun-verb
use clap_noun_verb::format::{FormatOutput, OutputFormat};

impl FormatOutput for WizardResponse {
    fn format(&self, format: OutputFormat) -> Result<String, FormatError> {
        match format {
            OutputFormat::Json => serde_json::to_string_pretty(self)
                .map_err(|e| FormatError::SerializationError(e.to_string())),
            OutputFormat::Yaml => serde_yaml::to_string(self)
                .map_err(|e| FormatError::SerializationError(e.to_string())),
            OutputFormat::Text => Ok(self.to_text()),
        }
    }
}
```

**Acceptance Criteria**:
- [ ] All output formats supported
- [ ] Format module integration tested
- [ ] Error handling consistent
- [ ] Output validated

### 5.3 Error Handling Integration (INT-5.3)

```rust
// Integrate with existing error types
use clap_noun_verb::error::{AppError, ErrorContext};

impl From<WizardError> for AppError {
    fn from(err: WizardError) -> Self {
        match err {
            WizardError::ApiError(e) =>
                AppError::ExternalServiceError(e.to_string()),
            WizardError::ConfigError(e) =>
                AppError::ConfigurationError(e),
            // ... other conversions
        }
    }
}
```

**Acceptance Criteria**:
- [ ] Error conversion tested
- [ ] Error context preserved
- [ ] User-facing messages clear
- [ ] Exit codes consistent

---

## 6. Security & Privacy Specifications

### 6.1 Input Sanitization (SEC-6.1)

#### SEC-6.1.1: Prompt Injection Prevention
```rust
pub struct PromptSanitizer {
    max_length: usize,
    forbidden_patterns: Vec<Regex>,
}

impl PromptSanitizer {
    pub fn sanitize(&self, input: &str) -> Result<String, ValidationError> {
        // Length check
        if input.len() > self.max_length {
            return Err(ValidationError::InputTooLong {
                max: self.max_length,
                actual: input.len(),
            });
        }

        // Pattern matching for prompt injection
        for pattern in &self.forbidden_patterns {
            if pattern.is_match(input) {
                return Err(ValidationError::ForbiddenPattern);
            }
        }

        // Unicode normalization
        let normalized = input.nfc().collect::<String>();

        Ok(normalized)
    }
}
```

**Sanitization Rules**:
- Maximum prompt length: 10,000 characters
- Forbidden patterns: Prompt injection attempts, system prompt overrides
- Unicode normalization: NFC form
- Special character escaping for shell output

**Acceptance Criteria**:
- [ ] Input validation tested with malicious inputs
- [ ] Length limits enforced
- [ ] Injection patterns detected
- [ ] Unicode handling verified

### 6.2 API Key Management (SEC-6.2)

#### SEC-6.2.1: Secure Storage
```yaml
storage_mechanism: "OS keyring integration"
libraries:
  - keyring = "2.0"

storage_locations:
  linux: "Secret Service API / gnome-keyring"
  macos: "macOS Keychain"
  windows: "Windows Credential Manager"

fallback: "Environment variable (WIZARD_API_KEY)"

security_requirements:
  - Never store API keys in plaintext files
  - Never log API keys
  - Redact keys in error messages
  - Clear keys from memory after use
```

**Key Management API**:
```rust
pub trait ApiKeyStorage {
    fn store(&self, key: &str) -> Result<(), KeyStorageError>;
    fn retrieve(&self) -> Result<String, KeyStorageError>;
    fn delete(&self) -> Result<(), KeyStorageError>;
}

#[cfg(feature = "secure-storage")]
impl ApiKeyStorage for KeyringStorage {
    // OS keyring implementation
}

#[cfg(not(feature = "secure-storage"))]
impl ApiKeyStorage for EnvVarStorage {
    // Fallback to environment variables
}
```

**Acceptance Criteria**:
- [ ] Keys stored securely on all platforms
- [ ] Keys never in plaintext files
- [ ] Keys never logged
- [ ] Memory cleared after use

### 6.3 Rate Limiting (SEC-6.3)

#### SEC-6.3.1: API Rate Limiting
```rust
pub struct RateLimiter {
    requests_per_minute: u32,
    tokens_per_minute: u32,
    bucket: TokenBucket,
}

impl RateLimiter {
    pub async fn acquire(&self) -> Result<Permit, RateLimitError> {
        self.bucket.acquire().await
    }
}
```

**Rate Limiting Configuration**:
```yaml
default_limits:
  requests_per_minute: 60
  tokens_per_minute: 100000
  burst_size: 10

behavior:
  - Block when limit exceeded
  - Return clear error message
  - Suggest wait time
  - Exponential backoff for retries
```

**Acceptance Criteria**:
- [ ] Rate limits enforced
- [ ] Burst handling tested
- [ ] Error messages clear
- [ ] Backoff strategy verified

### 6.4 Data Retention (SEC-6.4)

#### SEC-6.4.1: Privacy Policy
```yaml
data_retention:
  session_data:
    duration: "In-memory only (not persisted in v0.1.0)"
    cleanup: "On session termination"

  configuration:
    duration: "Until user deletion"
    location: "~/.wizard.toml"

  logs:
    duration: "Based on log rotation policy"
    sensitive_data: "Redacted"

user_controls:
  - /clear: Clear conversation history
  - wizard configure reset: Delete configuration
  - Session termination: Clear all session data
```

**Acceptance Criteria**:
- [ ] Session data cleared on exit
- [ ] No persistent conversation storage (v0.1.0)
- [ ] User controls functional
- [ ] Privacy policy documented

---

## 7. Test Specifications (Chicago TDD)

### 7.1 Unit Test Coverage (TEST-7.1)

#### TEST-7.1.1: Coverage Targets
```yaml
overall_coverage: 85%+
critical_paths: 95%+

critical_modules:
  - wizard::session (state machine)
  - wizard::ai_model (API integration)
  - wizard::sanitizer (security)
  - wizard::rate_limiter (reliability)

coverage_measurement:
  - Tool: cargo-tllvm-cov
  - Command: cargo make coverage
  - Report: HTML coverage report
  - CI: Coverage tracked in pull requests
```

#### TEST-7.1.2: Chicago TDD Test Structure
```rust
// AAA Pattern: Arrange-Act-Assert
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_initialization_creates_active_session() {
        // Arrange: Set up test data
        let config = WizardConfig::default();

        // Act: Perform the operation
        let session = Session::initialize(config).expect("initialization failed");

        // Assert: Verify observable state
        assert_eq!(session.state(), SessionState::Active);
        assert!(session.session_id().is_some());
        assert_eq!(session.message_count(), 0);
    }

    #[test]
    fn test_session_chat_adds_message_to_history() {
        // Arrange
        let mut session = create_test_session();
        let prompt = "test prompt";

        // Act
        session.chat(prompt).expect("chat failed");

        // Assert: Verify state change
        assert_eq!(session.message_count(), 2); // User + Assistant
        assert_eq!(session.last_message().role, Role::Assistant);
    }
}
```

**Test Requirements**:
- [ ] All public APIs have unit tests
- [ ] AAA pattern consistently applied
- [ ] Tests verify observable outputs/state changes
- [ ] No meaningless "smoke tests"
- [ ] Real collaborators, minimal mocks

### 7.2 Integration Tests (TEST-7.2)

#### TEST-7.2.1: End-to-End Scenarios
```rust
// Integration test with real AI API (using test API key)
#[tokio::test]
#[ignore] // Run with --ignored flag (requires API key)
async fn test_wizard_chat_with_real_api() {
    // Arrange: Set up real environment
    let config = WizardConfig {
        model: "gpt-3.5-turbo".into(),
        temperature: 0.0, // Deterministic
        ..Default::default()
    };

    // Act: Perform real operation
    let client = WizardClient::new(config).await.unwrap();
    let response = client.chat("Say 'hello'", Default::default())
        .await
        .unwrap();

    // Assert: Verify real behavior
    assert!(response.content.to_lowercase().contains("hello"));
    assert!(response.token_usage.total > 0);
}
```

**Integration Test Scenarios**:
- [ ] Session lifecycle: init → chat → terminate
- [ ] Command generation: description → commands
- [ ] Configuration: set → get → verify
- [ ] Error handling: invalid input → clear error
- [ ] Output formatting: response → all formats
- [ ] Feature flag: disabled → helpful error

### 7.3 Property-Based Tests (TEST-7.3)

#### TEST-7.3.1: Property Test Strategy
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_prompt_sanitizer_preserves_length_bounds(
        input in "\\PC{0,10000}"
    ) {
        let sanitizer = PromptSanitizer::default();
        let result = sanitizer.sanitize(&input);

        // Property: Output never exceeds input (after sanitization)
        if let Ok(sanitized) = result {
            assert!(sanitized.len() <= input.len());
        }
    }

    #[test]
    fn test_session_state_transitions_are_valid(
        operations in prop::collection::vec(operation_strategy(), 1..100)
    ) {
        let mut session = Session::new();

        for op in operations {
            let _ = apply_operation(&mut session, op);

            // Property: Session always in valid state
            assert!(session.is_valid_state());
        }
    }
}
```

**Property Test Coverage**:
- [ ] Input sanitization properties
- [ ] State machine invariants
- [ ] Serialization round-trips
- [ ] Rate limiter fairness
- [ ] Configuration validation

### 7.4 Performance Benchmarks (TEST-7.4)

#### TEST-7.4.1: Benchmark Suite
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_session_initialization(c: &mut Criterion) {
    c.bench_function("session_init", |b| {
        b.iter(|| {
            let config = black_box(WizardConfig::default());
            Session::initialize(config).unwrap()
        })
    });
}

fn benchmark_prompt_sanitization(c: &mut Criterion) {
    let sanitizer = PromptSanitizer::default();
    let prompt = "test prompt ".repeat(100);

    c.bench_function("sanitize_prompt", |b| {
        b.iter(|| {
            sanitizer.sanitize(black_box(&prompt)).unwrap()
        })
    });
}

criterion_group!(benches,
    benchmark_session_initialization,
    benchmark_prompt_sanitization
);
criterion_main!(benches);
```

**Benchmark Targets**:
```yaml
benchmarks:
  - name: session_initialization
    target: ≤ 100ms (P95)

  - name: prompt_sanitization
    target: ≤ 1ms

  - name: output_formatting
    target: ≤ 10ms

  - name: configuration_loading
    target: ≤ 50ms

ci_integration:
  - Run on every PR
  - Compare against baseline
  - Fail on >10% regression
```

**Acceptance Criteria**:
- [ ] All benchmarks meet targets
- [ ] Benchmarks run in CI
- [ ] Regression detection active
- [ ] Results tracked over time

### 7.5 Snapshot Tests (TEST-7.5)

#### TEST-7.5.1: Output Determinism
```rust
use insta::assert_snapshot;

#[test]
fn test_command_generation_output_format() {
    let command = GeneratedCommand {
        command: "ls -la".into(),
        explanation: "List all files with details".into(),
        confidence: 0.95,
        safety_level: SafetyLevel::Safe,
    };

    // Snapshot test for JSON output
    let json_output = command.format(OutputFormat::Json).unwrap();
    assert_snapshot!("command_json", json_output);

    // Snapshot test for text output
    let text_output = command.format(OutputFormat::Text).unwrap();
    assert_snapshot!("command_text", text_output);
}
```

**Snapshot Test Coverage**:
- [ ] All output formats
- [ ] Error messages
- [ ] Help text
- [ ] Configuration display

---

## 8. Acceptance Criteria & Validation

### 8.1 Specification Validation Checklist

**Requirements Quality**:
- [ ] All requirements are testable
- [ ] Acceptance criteria are clear and measurable
- [ ] Edge cases documented
- [ ] Performance metrics defined with measurement strategy
- [ ] Security requirements specified with validation approach
- [ ] Dependencies identified with version constraints
- [ ] Constraints documented (technical, business, regulatory)

**SPARC Compliance**:
- [ ] Functional requirements complete (FR-2.x)
- [ ] Non-functional requirements complete (NFR-3.x)
- [ ] Feature flag specifications complete (FF-4.x)
- [ ] Integration specifications complete (INT-5.x)
- [ ] Security specifications complete (SEC-6.x)
- [ ] Test specifications complete (TEST-7.x)

**Chicago TDD Compliance**:
- [ ] State-based testing approach documented
- [ ] Real collaborators preferred over mocks
- [ ] Behavior verification specified
- [ ] AAA pattern required for all tests
- [ ] Observable outputs identified for each component

**Type-First Thinking**:
- [ ] Core types defined with invariants
- [ ] State machine encoded in types
- [ ] Error types comprehensive
- [ ] Zero-cost abstractions identified
- [ ] API design prevents misuse through types

### 8.2 Definition of Done for Specifications

**Documentation Completeness**:
- [ ] All sections completed
- [ ] Examples provided for complex features
- [ ] Diagrams included where helpful
- [ ] Glossary of terms provided

**Stakeholder Review**:
- [ ] Technical review completed
- [ ] Security review completed
- [ ] Performance targets validated
- [ ] User experience reviewed

**Traceability**:
- [ ] Requirements numbered (FR-X.Y.Z, NFR-X.Y.Z, etc.)
- [ ] Acceptance criteria linked to requirements
- [ ] Test specifications linked to requirements
- [ ] Implementation tasks derivable from specs

**Implementation Readiness**:
- [ ] Clear enough for implementation without ambiguity
- [ ] All dependencies identified
- [ ] Integration points defined
- [ ] Error cases specified

---

## 9. Appendix

### 9.1 Data Model Summary

```rust
// Core domain types
pub struct Session {
    id: SessionId,
    state: SessionState,
    context: ConversationContext,
    config: WizardConfig,
}

pub struct ConversationContext {
    messages: Vec<Message>,
    token_count: TokenCount,
    created_at: Timestamp,
    last_activity: Timestamp,
}

pub struct Message {
    role: Role,
    content: String,
    tokens: u32,
    timestamp: Timestamp,
}

pub enum SessionState {
    Uninitialized,
    Initializing,
    Active { context: ConversationContext },
    Paused,
    Terminated,
    Error { reason: ErrorReason },
}

pub struct WizardConfig {
    model: ModelIden,
    temperature: f32,
    max_tokens: u32,
    api_key: SecretString,
}

pub struct GeneratedCommand {
    command: String,
    explanation: String,
    confidence: f32,
    safety_level: SafetyLevel,
    dry_run_suggested: bool,
}
```

### 9.2 API Surface Summary

```rust
// Public API for CLI
pub async fn wizard_start(config: WizardConfig) -> WizardResult<Session>;
pub async fn wizard_chat(session: &mut Session, prompt: &str) -> WizardResult<Response>;
pub async fn wizard_generate(description: &str) -> WizardResult<Vec<GeneratedCommand>>;
pub async fn wizard_configure(operation: ConfigOperation) -> WizardResult<()>;

// Public API for library usage
pub struct WizardClient { /* ... */ }
impl WizardClient {
    pub async fn new(config: ClientConfig) -> Result<Self, ClientError>;
    pub async fn chat(&self, prompt: &str, options: ChatOptions) -> Result<ChatResponse, ChatError>;
    pub async fn generate_command(&self, description: &str) -> Result<Vec<GeneratedCommand>, GenerateError>;
    pub async fn stream_chat(&self, prompt: &str) -> Result<impl Stream<Item = ResponseChunk>, ChatError>;
}
```

### 9.3 Performance SLO Summary

| Operation | Target | Percentile | Measurement |
|-----------|--------|------------|-------------|
| Session init | ≤ 100ms | P95 | Time to ready state |
| First token | ≤ 200ms | P50 | Time to first response token |
| Full response | ≤ 5s | P95 | Complete response generation |
| Command gen | ≤ 3s | P95 | wizard generate response |
| Memory/session | ≤ 50MB | P95 | Heap allocation |
| Total memory | ≤ 500MB | - | 100 concurrent sessions |
| Binary size | ≤ 10MB | - | Release binary (with feature) |

### 9.4 Test Coverage Summary

| Category | Target | Measurement |
|----------|--------|-------------|
| Overall | 85%+ | Line coverage |
| Critical paths | 95%+ | Session, AI model, security |
| Unit tests | All public APIs | Chicago TDD with AAA |
| Integration tests | 6+ scenarios | End-to-end workflows |
| Property tests | 5+ properties | State invariants |
| Benchmarks | All SLOs | Criterion benchmarks |

---

## 10. Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 0.1.0 | 2026-01-09 | Specification Engineer | Initial specification document |

---

**End of System Requirements Specification**
