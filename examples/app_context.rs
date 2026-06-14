// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! # AppContext Example
//!
//! Demonstrates `AppContext` — a type-erased shared context store backed by
//! `Arc<RwLock<HashMap<TypeId, Box<dyn Any + Send + Sync>>>>`.
//!
//! ## Capabilities witnessed
//!
//! - `AppContext::new()` + `insert<T>()` + `get<T>()` + `contains<T>()`
//! - `len()` + `is_empty()` + `remove<T>()` + `clear()`
//! - `with<T, F, R>()` — borrow a value and apply a closure
//! - `ContextError` on missing-type get
//!
//! **Doc**: docs/reference/api-catalog.md (AppContext section)

use clap_noun_verb::{AppContext, Result};

#[derive(Debug, Clone)]
struct DatabaseConfig {
    url: String,
    pool_size: u32,
}

#[derive(Debug, Clone)]
struct FeatureFlags {
    dark_mode: bool,
    beta_features: bool,
}

fn main() -> Result<()> {
    let ctx = AppContext::new();
    assert!(ctx.is_empty().unwrap_or(false), "new context must be empty");
    println!("empty: {}", ctx.is_empty().unwrap_or(false));

    ctx.insert(DatabaseConfig {
        url: "postgres://localhost/mydb".into(),
        pool_size: 10,
    })
    .map_err(|e| clap_noun_verb::NounVerbError::execution_error(e.to_string()))?;

    ctx.insert(FeatureFlags { dark_mode: true, beta_features: false })
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(e.to_string()))?;

    let len = ctx.len().map_err(|e| clap_noun_verb::NounVerbError::execution_error(e.to_string()))?;
    assert_eq!(len, 2, "context must hold 2 entries after two inserts");
    println!("len after 2 inserts: {len}");

    let has_db = ctx.contains::<DatabaseConfig>()
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(e.to_string()))?;
    assert!(has_db, "context must contain DatabaseConfig");
    println!("contains DatabaseConfig: {has_db}");

    let pool = ctx
        .with::<DatabaseConfig, _, _>(|cfg| cfg.pool_size)
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(e.to_string()))?;
    assert_eq!(pool, 10, "pool_size must be 10");
    println!("DatabaseConfig.pool_size via with(): {pool}");

    let flags: FeatureFlags = ctx.get()
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(e.to_string()))?;
    assert!(flags.dark_mode, "dark_mode must be true");
    assert!(!flags.beta_features, "beta_features must be false");
    println!("FeatureFlags: dark_mode={} beta_features={}", flags.dark_mode, flags.beta_features);

    ctx.remove::<FeatureFlags>()
        .map_err(|e| clap_noun_verb::NounVerbError::execution_error(e.to_string()))?;
    let len_after = ctx.len().map_err(|e| clap_noun_verb::NounVerbError::execution_error(e.to_string()))?;
    assert_eq!(len_after, 1, "context must hold 1 entry after remove");
    println!("len after remove: {len_after}");

    ctx.clear().map_err(|e| clap_noun_verb::NounVerbError::execution_error(e.to_string()))?;
    let empty = ctx.is_empty().unwrap_or(false);
    assert!(empty, "context must be empty after clear");
    println!("empty after clear: {empty}");

    Ok(())
}
