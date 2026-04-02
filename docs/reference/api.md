# API Reference

Complete documentation of Wizard's public API.

## Core Types

### WizardConfig

Configuration for wizard initialization.

```rust
pub struct WizardConfig {
    pub model_config: ModelConfig,
    pub temperature: f32,
    pub max_tokens: u32,
    pub timeout: Duration,
}
```

**Methods:**

- `from_env() -> Result<Self>` - Load from environment variables
- `with_model(model: String) -> Self` - Set AI model
- `with_temperature(temp: f32) -> Self` - Set temperature (0.0-1.0)
- `with_timeout(duration: Duration) -> Self` - Set request timeout

### Prompt

Represents a user prompt to the AI model.

```rust
pub struct Prompt {
    pub text: String,
    pub system: Option<String>,
    pub history: Vec<Message>,
}
```

**Methods:**

- `new(text: impl Into<String>) -> Self` - Create prompt
- `with_system(system: String) -> Self` - Add system message
- `with_history(history: Vec<Message>) -> Self` - Add conversation history

### WizardResponse

Response from the AI model.

```rust
pub struct WizardResponse {
    pub text: String,
    pub model: String,
    pub usage: Option<TokenUsage>,
}
```

### WizardError

Error type for wizard operations.

```rust
pub enum WizardError {
    Config(String),
    Request(String),
    Parse(String),
    Auth(String),
    RateLimit(String),
    Timeout(String),
    Network(String),
    Other(String),
}
```

## Client Types

### GenAiClient

Basic AI client for single requests.

```rust
pub struct GenAiClient {
    /* ... */
}
```

**Methods:**

- `new(config: WizardConfig) -> WizardResult<Self>` - Create client
- `async generate(&mut self, prompt: Prompt) -> WizardResult<WizardResponse>` - Generate response

### StreamingClient

Client for streaming responses.

```rust
pub struct StreamingClient {
    /* ... */
}
```

**Methods:**

- `new(config: WizardConfig) -> WizardResult<Self>` - Create client
- `async generate_stream(&self, prompt: Prompt) -> WizardResult<Stream>` - Stream response

### CachedClient

Client with response caching.

```rust
pub struct CachedClient {
    /* ... */
}
```

**Methods:**

- `new(config: WizardConfig, cache_config: CacheConfig) -> WizardResult<Self>`
- `async generate(&mut self, prompt: Prompt) -> WizardResult<WizardResponse>`

## Type Aliases

```rust
pub type WizardResult<T> = Result<T, WizardError>;
```

## Traits

### State

Marker trait for session state (compile-time use only).

```rust
pub trait State: Sealed {}
pub struct Init;
pub struct Active;
pub struct Complete;
// impl State for each...
```

## Full Documentation

For complete API documentation, run:

```bash
cargo doc --features wizard --open
```

---

[Back to Reference →](./README.md)
