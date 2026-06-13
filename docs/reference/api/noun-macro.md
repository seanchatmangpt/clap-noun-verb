# Reference: #[noun] Macro (DEPRECATED)

> [!WARNING]
> The `#[noun]` macro has been deprecated since version 5.6.0 and is now a **no-op**. It will be removed in a future major release.

## Overview

In previous versions of `clap-noun-verb`, the `#[noun]` macro was used to explicitly define a noun subcommand container. In modern versions of the framework, nouns are **automatically inferred** from file names and module structures, eliminating the boilerplate macro.

## Migration Guide

If you have legacy code using `#[noun]`, follow these steps to migrate:

### 1. Remove the Macro Attribute

Simply remove the `#[noun]` attribute from your functions. 

#### Before (Legacy v4/v5.5):

```rust
#[noun(name = "config", about = "Manage application configuration")]
pub fn config_noun() {
    // Legacy setup
}

#[verb("set")]
fn set_config(key: String, value: String) -> Result<()> {
    // ...
}
```

#### After (Modern v5.6.0+):

In your file `config.rs`, the noun is automatically detected as `config` based on the file name. Place your verbs directly in that file:

```rust
//! Manage application configuration

use clap_noun_verb::Result;

#[verb("set")]
fn set_config(key: String, value: String) -> Result<()> {
    // ...
}
```

### 2. Custom Noun Naming

If you need a noun that does not match the file name, or if you are grouping multiple verbs under different nouns within the same file, use the explicit noun parameter on the `#[verb]` macro instead:

```rust
#[verb("set", "settings")] // Verb "set" under the noun "settings"
fn set_setting(key: String, value: String) -> Result<()> {
    // ...
}
```

## See Also

- [#[verb] Macro Reference](./verb-macro.md)
- [API Catalog](../api-catalog.md)
