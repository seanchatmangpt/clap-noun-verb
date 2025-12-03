# Tutorial 02: Domain Separation - The Golden Rule

**Learning Path:** Understanding Architecture → Building Maintainable CLIs
**Time:** 15 minutes
**Prerequisites:** [Tutorial 01: Your First CLI](01-your-first-cli.md)

---

## The Golden Rule

**CLI validates, domain computes, integration connects.**

This tutorial teaches you why domain separation is critical for production CLIs.

---

## Why Separate Domain Logic?

### ❌ Bad: Logic in CLI Layer

```rust
use clap_noun_verb_macros::verb;
use serde::Serialize;

#[derive(Serialize)]
struct UserStats {
    total_users: usize,
    active_users: usize,
}

#[verb]
fn show_stats(database_url: String) -> Result<UserStats, Box<dyn std::error::Error>> {
    // ❌ BAD: Database logic in CLI function
    let conn = establish_connection(&database_url)?;
    let users = load_all_users(&conn)?;

    let total = users.len();
    let active = users.iter().filter(|u| u.is_active).count();

    Ok(UserStats {
        total_users: total,
        active_users: active,
    })
}
```

**Problems:**
- ❌ Can't test without database
- ❌ Can't reuse logic outside CLI
- ❌ Hard to mock for testing
- ❌ Violates Single Responsibility Principle

---

### ✅ Good: Domain-Separated

```rust
// domain/user_stats.rs - Pure business logic
pub struct UserRepository {
    conn: DatabaseConnection,
}

impl UserRepository {
    pub fn calculate_stats(&self) -> Result<UserStats, DomainError> {
        let users = self.load_all_users()?;

        Ok(UserStats {
            total_users: users.len(),
            active_users: users.iter().filter(|u| u.is_active).count(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_stats_with_mock_data() {
        // ✅ GOOD: Can test without database
        let mock_repo = MockUserRepository::new();
        let stats = mock_repo.calculate_stats().unwrap();
        assert_eq!(stats.total_users, 10);
    }
}

// commands/user_commands.rs - Thin CLI wrapper
use clap_noun_verb_macros::verb;

#[verb(help = "Show user statistics")]
fn show_stats(
    #[arg(env = "DATABASE_URL", help = "Database connection URL")]
    database_url: String,
) -> Result<UserStatsOutput, Box<dyn std::error::Error>> {
    // ✅ GOOD: CLI just validates and delegates
    let repo = UserRepository::connect(&database_url)?;
    let stats = repo.calculate_stats()?;

    Ok(UserStatsOutput::from(stats))
}
```

**Benefits:**
- ✅ Domain logic testable independently
- ✅ Reusable in web API, cron jobs, etc.
- ✅ Clear separation of concerns
- ✅ Easy to mock for testing

---

## The Three Layers

### Layer 1: CLI (Thin Validation)

**Responsibility:** Parse, validate, route

```rust
#[verb(help = "Process payment")]
fn process_payment(
    #[arg(help = "Amount in cents", value_hint = "number")] amount: u64,
    #[arg(help = "Currency code", default = "USD")] currency: String,
) -> Result<PaymentReceipt, Box<dyn std::error::Error>> {
    // Validate inputs
    if amount == 0 {
        return Err("Amount must be greater than zero".into());
    }

    // Delegate to domain
    let payment = domain::payments::process(amount, &currency)?;

    Ok(PaymentReceipt::from(payment))
}
```

**Rules:**
- Parse CLI arguments
- Basic validation (nulls, ranges)
- Delegate to domain immediately
- Return JSON-serializable results

---

### Layer 2: Domain (Pure Logic)

**Responsibility:** Business rules, calculations, algorithms

```rust
// domain/payments.rs
pub struct Payment {
    pub amount: u64,
    pub currency: String,
    pub status: PaymentStatus,
}

pub fn process(amount: u64, currency: &str) -> Result<Payment, PaymentError> {
    // ✅ GOOD: Pure business logic
    validate_currency(currency)?;
    calculate_fees(amount)?;

    Ok(Payment {
        amount,
        currency: currency.to_string(),
        status: PaymentStatus::Pending,
    })
}

fn validate_currency(currency: &str) -> Result<(), PaymentError> {
    const VALID_CURRENCIES: &[&str] = &["USD", "EUR", "GBP"];

    if !VALID_CURRENCIES.contains(&currency) {
        return Err(PaymentError::InvalidCurrency);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_payment_valid() {
        let payment = process(1000, "USD").unwrap();
        assert_eq!(payment.amount, 1000);
        assert_eq!(payment.status, PaymentStatus::Pending);
    }

    #[test]
    fn test_validate_currency_invalid() {
        let result = validate_currency("JPY");
        assert!(result.is_err());
    }
}
```

**Rules:**
- No CLI dependencies
- Pure Rust functions
- Comprehensive tests
- Clear error types

---

### Layer 3: Integration (Glue Code)

**Responsibility:** Connect CLI to domain, handle I/O

```rust
// integration/payment_gateway.rs
use crate::domain;

pub struct PaymentGatewayClient {
    api_key: String,
}

impl PaymentGatewayClient {
    pub fn submit(&self, payment: &domain::Payment) -> Result<String, IntegrationError> {
        // ✅ GOOD: External I/O in integration layer
        let response = reqwest::blocking::Client::new()
            .post("https://api.payments.example/v1/charge")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(payment)
            .send()?;

        Ok(response.json()?)
    }
}
```

**Rules:**
- External API calls
- Database connections
- File I/O
- Network requests

---

## Exercise: Refactor Monolithic CLI

**Goal:** Separate domain logic from CLI layer

**Arrange:** Start with monolithic code

```rust
// ❌ BAD: Everything in one function
#[verb]
fn calculate_tax(income: f64, state: String) -> Result<TaxResult, Box<dyn std::error::Error>> {
    // Tax calculation logic embedded in CLI
    let tax_rate = match state.as_str() {
        "CA" => 0.093,
        "NY" => 0.085,
        "TX" => 0.0,
        _ => return Err("Unknown state".into()),
    };

    let tax = income * tax_rate;

    Ok(TaxResult {
        income,
        state,
        tax,
        tax_rate,
    })
}
```

**Act:** Separate into layers

```rust
// Step 1: Extract domain logic
// domain/tax.rs
pub fn calculate_tax(income: f64, state: &str) -> Result<TaxCalculation, TaxError> {
    let tax_rate = get_tax_rate(state)?;
    let tax = income * tax_rate;

    Ok(TaxCalculation {
        income,
        state: state.to_string(),
        tax,
        tax_rate,
    })
}

fn get_tax_rate(state: &str) -> Result<f64, TaxError> {
    match state {
        "CA" => Ok(0.093),
        "NY" => Ok(0.085),
        "TX" => Ok(0.0),
        _ => Err(TaxError::UnknownState(state.to_string())),
    }
}

// Step 2: Create thin CLI wrapper
// commands/tax_commands.rs
#[verb(help = "Calculate state income tax")]
fn calculate_tax(
    #[arg(help = "Annual income", value_hint = "number")] income: f64,
    #[arg(help = "State code (CA, NY, TX)")] state: String,
) -> Result<TaxResult, Box<dyn std::error::Error>> {
    // ✅ GOOD: Thin CLI wrapper
    let calculation = crate::domain::tax::calculate_tax(income, &state)?;
    Ok(TaxResult::from(calculation))
}
```

**Assert:** Test domain independently

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_california_tax() {
        let calc = calculate_tax(100_000.0, "CA").unwrap();
        assert_eq!(calc.tax_rate, 0.093);
        assert_eq!(calc.tax, 9_300.0);
    }

    #[test]
    fn test_unknown_state() {
        let result = calculate_tax(50_000.0, "ZZ");
        assert!(result.is_err());
    }
}
```

---

## Directory Structure Best Practices

```
my-cli/
├── src/
│   ├── main.rs              # Entry point
│   ├── domain/              # Pure business logic
│   │   ├── mod.rs
│   │   ├── payments.rs
│   │   ├── users.rs
│   │   └── tax.rs
│   ├── integration/         # External I/O
│   │   ├── mod.rs
│   │   ├── database.rs
│   │   └── payment_gateway.rs
│   └── commands/            # CLI layer
│       ├── mod.rs
│       ├── payment_commands.rs
│       └── user_commands.rs
├── tests/                   # Integration tests
└── Cargo.toml
```

**File organization:**
- **`domain/`** - No external dependencies, pure Rust
- **`integration/`** - External APIs, databases, network
- **`commands/`** - CLI wrappers using `#[verb]`

---

## Anti-Patterns to Avoid

### ❌ Anti-Pattern 1: CLI Logic Leak

```rust
#[verb]
fn process_order(
    #[arg] order_id: String,
    #[arg] verbose: bool, // ❌ BAD: CLI concern in domain
) -> Result<OrderStatus, Box<dyn std::error::Error>> {
    if verbose {
        println!("Processing order {}", order_id); // ❌ BAD: Side effect
    }
    // ...
}
```

**Fix:** Keep CLI concerns in CLI layer

```rust
#[verb]
fn process_order(
    #[arg] order_id: String,
    #[arg] verbose: bool,
) -> Result<OrderStatus, Box<dyn std::error::Error>> {
    if verbose {
        eprintln!("Processing order {}", order_id); // ✅ GOOD: CLI handles output
    }

    let status = domain::orders::process(&order_id)?; // ✅ GOOD: Domain is pure
    Ok(OrderStatus::from(status))
}
```

---

### ❌ Anti-Pattern 2: Domain Depends on CLI

```rust
// domain/orders.rs
use clap_noun_verb::Result; // ❌ BAD: Domain imports CLI framework

pub fn process(order_id: &str) -> Result<Order> { // ❌ BAD: CLI error type
    // ...
}
```

**Fix:** Domain has its own error types

```rust
// domain/orders.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum OrderError {
    #[error("Order not found: {0}")]
    NotFound(String),
    #[error("Invalid order state")]
    InvalidState,
}

pub fn process(order_id: &str) -> Result<Order, OrderError> { // ✅ GOOD: Domain error
    // ...
}

// commands/order_commands.rs
#[verb]
fn process_order(
    #[arg] order_id: String,
) -> Result<OrderOutput, Box<dyn std::error::Error>> {
    let order = domain::orders::process(&order_id)?; // ✅ GOOD: Error conversion
    Ok(OrderOutput::from(order))
}
```

---

## Key Takeaways

✅ **CLI validates and routes** - Thin layer, no business logic
✅ **Domain computes** - Pure functions, comprehensive tests
✅ **Integration connects** - External I/O isolated
✅ **Test domain independently** - No CLI framework needed
✅ **Reuse domain logic** - Same code in CLI, web API, cron jobs

---

## Next Steps

- **[Tutorial 03: Adding Commands](03-adding-commands.md)** - Command patterns and organization
- **[Tutorial 04: Testing Basics](04-testing-basics.md)** - Chicago TDD for domain logic
- **[Explanation: Domain Separation](../explanation/architecture/domain-separation.md)** - Philosophy deep dive

**Estimated time to next tutorial:** 15 minutes

---

*Part of the [clap-noun-verb Tutorial Series](README.md) - Learning-oriented documentation*
