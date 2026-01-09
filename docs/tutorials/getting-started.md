# Getting Started with Wizard

Get up and running with the Wizard package in 5 minutes.

## Prerequisites

- Rust 1.74+ ([Install Rust](https://rustup.rs/))
- An AI API key (OpenAI, Anthropic, or Gemini)
- Basic knowledge of Rust

## Step 1: Create a New Project

```bash
cargo new my-wizard-app
cd my-wizard-app
```

## Step 2: Add Wizard Dependency

Edit `Cargo.toml`:

```toml
[dependencies]
clap-noun-verb = { version = "5.5", features = ["wizard"] }
tokio = { version = "1.40", features = ["full"] }
```

## Step 3: Set Your API Key

```bash
# OpenAI
export OPENAI_API_KEY="your-key-here"

# OR Anthropic
export ANTHROPIC_API_KEY="your-key-here"

# OR Gemini
export GEMINI_API_KEY="your-key-here"
```

## Step 4: Write Your First Program

Edit `src/main.rs`:

```rust
use clap_noun_verb::wizard::{WizardConfig, WizardBuilder, Prompt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧙 Welcome to Wizard!\n");

    // Load configuration from environment
    let config = WizardConfig::from_env()?;
    println!("Using model: {}", config.model_config.model);
    println!();

    // Create a wizard
    let mut wizard = WizardBuilder::new()
        .with_config(config)
        .build()
        .await?;

    // Create a prompt
    let prompt = Prompt::new("What is Rust?")
        .with_system("You are a helpful programming assistant.");

    // Generate a response
    println!("📝 Generating response...\n");
    let response = wizard.generate(prompt).await?;

    println!("✨ Response:\n{}\n", response.text);
    println!("Model: {}", response.model);
    println!("Tokens used: {:?}", response.usage);

    Ok(())
}
```

## Step 5: Run It!

```bash
cargo run
```

You should see something like:

```
🧙 Welcome to Wizard!

Using model: Claude 3 Opus
📝 Generating response...

✨ Response:
Rust is a modern programming language...
Model: claude-3-opus-20240229
Tokens used: Some(TokenUsage { prompt: 15, completion: 150 })
```

## ✅ You're Done!

Congratulations! You've created your first Wizard application.

## Next Steps

Now that you have the basics working:

1. **[Your First Wizard Session](./first-session.md)** - Build an interactive session
2. **[How-To Guides](../how-to/README.md)** - Learn specific features
3. **[API Reference](../reference/api.md)** - Explore all APIs

## Troubleshooting

**API Key not found?**
```bash
# Check your environment variable
echo $OPENAI_API_KEY
# Should output your key
```

**Compilation errors?**
- Ensure you're on Rust 1.74+: `rustc --version`
- Ensure the `wizard` feature is enabled in Cargo.toml

**Connection errors?**
- Check your internet connection
- Verify your API key is correct
- Check the API provider's status page

## What You Learned

- ✅ Setting up Wizard dependency
- ✅ Configuring AI providers
- ✅ Creating and building wizards
- ✅ Generating responses from prompts
- ✅ Working with async/await in Rust

---

[Next: Your First Wizard Session →](./first-session.md)
