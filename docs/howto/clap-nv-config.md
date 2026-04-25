# How-to: Manage CLI Defaults with `clap-nv.toml`

The `clap-noun-verb` framework supports first-class configuration via `clap-nv.toml`. This allows you to define standardized default arguments for your CLI, making it easier for users to run your application without repetitive flags.

## 1. Project Initialization

You can generate a starter configuration template using the built-in scaffolding utility:

```rust
use clap_noun_verb::cli::scaffold_config;

fn main() -> clap_noun_verb::Result<()> {
    // Generate a default clap-nv.toml if it doesn't exist
    scaffold_config(false)?;
    Ok(())
}
```

## 2. Configuration Structure

The configuration file supports standard TOML syntax. Structured data is automatically flattened into command-line arguments using dot notation.

### Basic Arguments
```toml
# Top-level keys become standard flags
host = "localhost"
port = 8080
verbose = true
```
*Translates to: `--host localhost --port 8080 --verbose`*

### Nested Modules
```toml
[database]
url = "postgres://localhost/db"
pool_size = 5
```
*Translates to: `--database.url postgres://localhost/db --database.pool_size 5`*

### Arrays
```toml
tags = ["api", "production"]
```
*Translates to: `--tags api --tags production`*

## 3. Loading Configuration

The framework automatically searches for `clap-nv.toml` in the current working directory. You can load it and merge it with your command execution logic:

```rust
use clap_noun_verb::config::ConfigLoader;
use clap_noun_verb::cli::CliBuilder;

fn main() -> clap_noun_verb::Result<()> {
    // 1. Auto-discover and load clap-nv.toml
    let config = ConfigLoader::new().load_optional()?;
    let config_args = config.to_cli_args();

    // 2. Combine with actual process arguments
    let mut args: Vec<String> = std::env::args().collect();
    args.extend(config_args);

    // 3. Run the CLI
    CliBuilder::new("my-app")
        .version("1.0.0")
        .run_with_args(args)?;

    Ok(())
}
```

## 4. Priority Order

`ConfigLoader` searches for files in the following order (first match wins):
1.  `clap-nv.toml`
2.  `clap-nv.yaml`
3.  `.env.yaml`
4.  `config.yaml`
5.  `config.yml`
6.  `.config/app.yaml`

This allows you to keep project-specific defaults in `clap-nv.toml` while still allowing environment-specific overrides in `.env.yaml`.
