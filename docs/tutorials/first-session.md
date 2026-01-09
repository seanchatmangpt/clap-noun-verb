# Your First Interactive Wizard Session

Learn how to create an interactive, multi-turn wizard conversation.

## Goal

Build an interactive CLI that:
- Maintains conversation history
- Handles user input
- Generates contextual responses
- Provides graceful error handling

## Complete Code

Create `src/main.rs`:

```rust
use clap_noun_verb::wizard::{
    WizardConfig, WizardBuilder, Prompt, Message, Role
};
use std::io::{self, Write};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧙 Wizard Interactive Session");
    println!("Type 'exit' to quit, 'help' for help\n");

    // Load configuration
    let config = WizardConfig::from_env()?;

    // Create wizard
    let mut wizard = WizardBuilder::new()
        .with_config(config.clone())
        .build()
        .await?;

    // System message for context
    let system = "You are a helpful wizard. \
                 Respond concisely and helpfully to questions.";

    // Conversation history
    let mut history = vec![];

    // Main loop
    loop {
        // Print prompt
        print!("\n> ");
        io::stdout().flush()?;

        // Read user input
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        // Handle special commands
        match input {
            "exit" | "quit" => {
                println!("\n👋 Goodbye!");
                break;
            }
            "help" => {
                println!("\nCommands:");
                println!("  exit, quit  - Leave the wizard");
                println!("  help        - Show this help");
                println!("  clear       - Clear history");
                println!("  history     - Show conversation history");
                continue;
            }
            "clear" => {
                history.clear();
                println!("✨ History cleared");
                continue;
            }
            "history" => {
                if history.is_empty() {
                    println!("📝 No history yet");
                } else {
                    println!("\n📜 Conversation History:");
                    for (i, msg) in history.iter().enumerate() {
                        let role = match msg.role {
                            Role::User => "👤 You",
                            Role::Assistant => "🧙 Wizard",
                        };
                        println!("{}:\n{}\n", role, msg.content);
                    }
                }
                continue;
            }
            "" => continue,
            _ => {}
        }

        // Add user message to history
        history.push(Message {
            role: Role::User,
            content: input.to_string(),
        });

        // Create prompt with history
        let mut prompt = Prompt::new(input)
            .with_system(system);

        // Add conversation history to prompt
        if !history.is_empty() {
            prompt = prompt.with_history(history.clone());
        }

        // Generate response
        print!("\n🧙 ");
        io::stdout().flush()?;

        match wizard.generate(prompt).await {
            Ok(response) => {
                println!("{}\n", response.text);

                // Add wizard response to history
                history.push(Message {
                    role: Role::Assistant,
                    content: response.text.clone(),
                });

                // Show token usage
                if let Some(usage) = response.usage {
                    println!("📊 [{} tokens]", usage.prompt + usage.completion);
                }
            }
            Err(e) => {
                println!("❌ Error: {}\n", e);
            }
        }
    }

    Ok(())
}
```

## Running It

```bash
cargo run
```

## Example Interaction

```
🧙 Wizard Interactive Session
Type 'exit' to quit, 'help' for help

> What is Rust?

🧙 Rust is a modern systems programming language...

> Why should I learn it?

🧙 Rust offers memory safety without garbage collection...

> help

Commands:
  exit, quit  - Leave the wizard
  help        - Show this help
  clear       - Clear history
  history     - Show conversation history

> history

📜 Conversation History:
👤 You:
What is Rust?

🧙 Wizard:
Rust is a modern systems programming language...

> exit

👋 Goodbye!
```

## Key Concepts

### Conversation History

The `history` vector maintains previous messages:

```rust
history.push(Message {
    role: Role::User,
    content: input.to_string(),
});
```

### Multi-Turn Context

History is passed to each prompt for context:

```rust
let prompt = prompt.with_history(history.clone());
```

### Error Handling

Gracefully handle errors without panicking:

```rust
match wizard.generate(prompt).await {
    Ok(response) => { /* handle success */ }
    Err(e) => println!("❌ Error: {}", e),
}
```

### Interactive Loop

Main loop reads, processes, and responds:

```rust
loop {
    // Read input
    // Process commands
    // Generate response
    // Store in history
}
```

## Exercises

Try modifying the code to:

1. **Add temperature control** - Let users adjust response creativity
2. **Save history to file** - Persist conversations
3. **Multiple wizards** - Support different AI models
4. **Custom system prompts** - Let users set the wizard's role
5. **Streaming responses** - Show tokens as they arrive (see [Streaming Guide](../how-to/streaming.md))

## What You've Learned

- ✅ Building interactive CLI applications
- ✅ Maintaining conversation history
- ✅ Handling user input and commands
- ✅ Error handling and recovery
- ✅ Multi-turn AI conversations

---

**Next:** Learn how to add advanced features like [Streaming](../how-to/streaming.md) and [Caching](../how-to/caching.md)

[Back to Tutorials →](./README.md)
