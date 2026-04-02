# How To: Implement Streaming Responses

Stream AI-generated tokens as they arrive for faster, more responsive UX.

## Goal

Display tokens to the user as the AI generates them, rather than waiting for the complete response.

## Prerequisites

- Completed [Getting Started](../tutorials/getting-started.md)
- Understand Rust async/await
- Familiar with Prompts and Responses

## Complete Example

```rust
use clap_noun_verb::wizard::{WizardConfig, StreamingClient, Prompt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Streaming Example\n");

    // Load configuration
    let config = WizardConfig::from_env()?;

    // Create streaming client
    let client = StreamingClient::new(config).await?;

    // Create prompt
    let prompt = Prompt::new("Explain quantum computing in simple terms");

    // Stream response
    println!("🧙 Streaming response:\n");
    
    let mut stream = client.generate_stream(prompt).await?;
    
    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(chunk) => {
                // Print each token as it arrives
                print!("{}", chunk.text);
                std::io::stdout().flush()?;
                
                // Track token usage if available
                if let Some(usage) = chunk.usage {
                    println!("\n📊 Chunk tokens: {}", usage.completion);
                }
            }
            Err(e) => {
                eprintln!("\n❌ Stream error: {}", e);
                break;
            }
        }
    }

    println!("\n\n✨ Streaming complete!");
    Ok(())
}
```

## Key Concepts

### StreamingClient

Specialized client for token streaming:

```rust
let client = StreamingClient::new(config).await?;
```

### AsyncIterator

Stream returns an async iterator over chunks:

```rust
let mut stream = client.generate_stream(prompt).await?;

while let Some(chunk) = stream.next().await {
    // Process each chunk
}
```

### StreamChunk

Each chunk contains:

```rust
pub struct StreamChunk {
    pub text: String,           // Token text
    pub is_final: bool,         // Is this the final chunk?
    pub usage: Option<TokenUsage>, // Optional token usage
}
```

### Backpressure

Streaming automatically implements backpressure:

```rust
// If processing is slow, streaming pauses automatically
// If processing is fast, streaming tries to keep up
```

## Error Handling

Handle errors gracefully during streaming:

```rust
match chunk {
    Ok(chunk) => {
        // Process token
    }
    Err(e) => {
        // Handle stream error
        eprintln!("Stream error: {}", e);
        // Decide whether to continue or break
    }
}
```

## Performance Tips

1. **Flush output after each token** for immediate feedback
2. **Don't block** during streaming (use async all the way)
3. **Buffer large responses** to avoid slow I/O
4. **Monitor token usage** for billing/budgeting

## Configuration

Customize streaming behavior:

```rust
use clap_noun_verb::wizard::StreamingConfig;

let streaming_config = StreamingConfig::new(64) // 64-token buffer
    .with_backpressure(true);

let client = StreamingClient::new(config)
    .with_streaming_config(streaming_config)
    .await?;
```

## Cancellation

Cancel streaming if needed:

```rust
let mut stream = client.generate_stream(prompt).await?;

while let Some(chunk) = stream.next().await {
    // User pressed Ctrl+C or some other cancellation signal
    if should_cancel {
        drop(stream); // Cancels the stream
        break;
    }
    
    print!("{}", chunk.text);
}
```

## Common Patterns

### Display Response with Status

```rust
print!("🧙 Response: ");
std::io::stdout().flush()?;

while let Some(chunk) = stream.next().await {
    match chunk {
        Ok(chunk) => print!("{}", chunk.text),
        Err(e) => println!("\n❌ Error: {}", e),
    }
}
println!();
```

### Accumulate Full Response

```rust
let mut full_response = String::new();

while let Some(chunk) = stream.next().await {
    if let Ok(chunk) = chunk {
        full_response.push_str(&chunk.text);
    }
}

println!("Full response:\n{}", full_response);
```

### Stream with Status Updates

```rust
let mut token_count = 0;

while let Some(chunk) = stream.next().await {
    if let Ok(chunk) = chunk {
        print!("{}", chunk.text);
        token_count += 1;
        
        if token_count % 10 == 0 {
            eprint!(" [{}]", token_count);
        }
    }
}
println!("\n[Total: {} tokens]", token_count);
```

## Troubleshooting

**Stream never ends?**
- Check that your prompt is valid
- Verify API key is correct
- Check internet connection

**Missing tokens?**
- Ensure you're awaiting all chunks
- Don't break out of loop prematurely
- Check error messages

**Slow streaming?**
- Reduce buffer size for lower latency
- Check your internet bandwidth
- Check API provider status

## Next Steps

- Learn about [Caching](./caching.md) to complement streaming
- Explore [Error Handling](./error-handling.md)
- Read [How Streaming Works](../explanation/streaming-internals.md)

---

[Back to How-To Guides →](./README.md)
