//! Example: Async Handler Support
//!
//! Demonstrates how to use `run_async()` to execute async operations from sync handlers.
//! This enables database queries, HTTP calls, and other I/O operations in CLI handlers.

use clap_noun_verb::async_verb::run_async;
use clap_noun_verb::Result;
use clap_noun_verb_macros::{noun, verb};
use serde::Serialize;
use std::time::Duration;

// Business logic (can be async)
async fn fetch_user_data(id: u32) -> Result<UserData> {
    // Simulate async operation (would be real database/HTTP call)
    tokio::time::sleep(Duration::from_millis(100)).await;

    Ok(UserData { id, name: format!("User {}", id), email: format!("user{}@example.com", id) })
}

async fn fetch_orders(user_id: u32) -> Result<Vec<Order>> {
    tokio::time::sleep(Duration::from_millis(50)).await;

    Ok(vec![
        Order { id: 1, amount: 99.99, status: "completed".to_string() },
        Order { id: 2, amount: 49.99, status: "pending".to_string() },
    ])
}

#[derive(Serialize)]
struct UserData {
    id: u32,
    name: String,
    email: String,
}

#[derive(Serialize)]
struct UserProfile {
    user: UserData,
    orders: Vec<Order>,
    order_count: usize,
}

#[derive(Serialize)]
struct Order {
    id: u32,
    amount: f64,
    status: String,
}

/// Fetch user profile from database
#[noun("users", "User management")]
#[verb("profile")]
fn get_user_profile(user_id: u32) -> Result<UserProfile> {
    run_async(async {
        // Fetch user and orders concurrently
        let user = fetch_user_data(user_id).await?;
        let orders = fetch_orders(user_id).await?;
        let order_count = orders.len();

        Ok(UserProfile { user, orders, order_count })
    })
}

/// Create a new user
#[noun("users", "User management")]
#[verb("create")]
fn create_user(name: String, email: String) -> Result<UserData> {
    run_async(async {
        // Simulate async user creation
        tokio::time::sleep(Duration::from_millis(50)).await;

        Ok(UserData { id: 1, name, email })
    })
}

/// List all users
#[noun("users", "User management")]
#[verb("list")]
fn list_users() -> Result<Vec<UserData>> {
    run_async(async {
        // Simulate fetching users from database
        tokio::time::sleep(Duration::from_millis(100)).await;

        Ok(vec![
            UserData { id: 1, name: "Alice".to_string(), email: "alice@example.com".to_string() },
            UserData { id: 2, name: "Bob".to_string(), email: "bob@example.com".to_string() },
        ])
    })
}

fn main() -> Result<()> {
    clap_noun_verb::run()
}
