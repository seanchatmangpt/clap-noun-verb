// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Executable witness for the complete `AppContext` lifecycle.

use clap_noun_verb::AppContext;

#[derive(Clone, Debug, PartialEq, Eq)]
struct DatabaseConfig {
    url: String,
    pool_size: u32,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let context = AppContext::new();
    assert!(context.is_empty()?);

    let config = DatabaseConfig {
        url: "postgres://localhost/clap_noun_verb".to_string(),
        pool_size: 8,
    };
    context.insert(config.clone())?;
    context.insert(String::from("production"))?;

    assert_eq!(context.len()?, 2);
    assert!(context.contains::<DatabaseConfig>()?);
    assert_eq!(context.get::<DatabaseConfig>()?, config);
    assert_eq!(context.with::<DatabaseConfig, _, _>(|value| value.pool_size)?, 8);

    let removed = context.remove::<String>()?;
    assert_eq!(removed.as_deref(), Some("production"));
    assert!(context.get::<String>().is_err());

    context.clear()?;
    assert!(context.is_empty()?);
    println!("AppContext lifecycle: insert → get → with → remove → refusal → clear");
    Ok(())
}
