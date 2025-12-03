# Tutorial 04: Testing Basics - Chicago TDD

**Learning Path:** Understanding Testing → Production-Ready CLIs
**Time:** 25 minutes
**Prerequisites:** [Tutorial 03: Adding Commands](03-adding-commands.md)

---

## What You'll Learn

How to test clap-noun-verb applications using Chicago TDD:
- Test domain logic independently of CLI
- State-based testing with real collaborators
- AAA pattern (Arrange-Act-Assert)
- Integration testing for CLI commands

---

## Chicago TDD: The Philosophy

**Chicago TDD** (also called "Classical TDD"):
- **State-based testing** - Verify outputs and state changes
- **Real collaborators** - Use real objects, minimize mocks
- **Behavior verification** - Test what code does, not how

### ❌ London TDD (Mock-Heavy)

```rust
#[test]
fn test_process_payment_london_style() {
    let mut mock_gateway = MockPaymentGateway::new();
    mock_gateway.expect_charge()
        .with(eq(100))
        .times(1)
        .returning(|_| Ok("txn_123".to_string()));

    // ❌ Tests implementation details (mocking every interaction)
    let service = PaymentService::new(mock_gateway);
    service.process(100).unwrap();
}
```

### ✅ Chicago TDD (State-Based)

```rust
#[test]
fn test_process_payment_chicago_style() {
    // Arrange: Set up real objects
    let service = PaymentService::new_with_test_gateway();

    // Act: Perform operation
    let result = service.process(100).unwrap();

    // Assert: Verify observable outputs
    assert_eq!(result.amount, 100);
    assert_eq!(result.status, PaymentStatus::Completed);
}
```

---

## Testing Domain Logic (No CLI)

### Step 1: Write Domain Tests First

```rust
// domain/calculator.rs
pub fn calculate_compound_interest(
    principal: f64,
    rate: f64,
    years: u32,
) -> Result<f64, CalculationError> {
    if principal <= 0.0 {
        return Err(CalculationError::InvalidPrincipal);
    }
    if rate < 0.0 || rate > 1.0 {
        return Err(CalculationError::InvalidRate);
    }

    let amount = principal * (1.0 + rate).powi(years as i32);
    Ok(amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_compound_interest_valid() {
        // Arrange
        let principal = 1000.0;
        let rate = 0.05;
        let years = 10;

        // Act
        let result = calculate_compound_interest(principal, rate, years).unwrap();

        // Assert
        assert!((result - 1628.89).abs() < 0.01);
    }

    #[test]
    fn test_calculate_compound_interest_invalid_principal() {
        // Arrange
        let principal = -100.0;

        // Act
        let result = calculate_compound_interest(principal, 0.05, 10);

        // Assert
        assert!(matches!(result, Err(CalculationError::InvalidPrincipal)));
    }

    #[test]
    fn test_calculate_compound_interest_invalid_rate() {
        // Arrange
        let rate = 1.5; // > 100%

        // Act
        let result = calculate_compound_interest(1000.0, rate, 10);

        // Assert
        assert!(matches!(result, Err(CalculationError::InvalidRate)));
    }
}
```

**Key Points:**
- ✅ AAA pattern (Arrange-Act-Assert)
- ✅ Test edge cases (negative, zero, boundary values)
- ✅ No CLI dependencies
- ✅ Observable outputs verified

---

### Step 2: Thin CLI Wrapper (Minimal Testing)

```rust
// commands/finance.rs
use clap_noun_verb_macros::verb;
use serde::Serialize;

#[derive(Serialize)]
pub struct InterestResult {
    principal: f64,
    rate: f64,
    years: u32,
    final_amount: f64,
}

#[verb(help = "Calculate compound interest")]
pub fn calculate_interest(
    #[arg(help = "Initial principal", value_hint = "number")] principal: f64,
    #[arg(help = "Annual interest rate (0-1)", value_hint = "number")] rate: f64,
    #[arg(help = "Number of years", value_hint = "number")] years: u32,
) -> Result<InterestResult, Box<dyn std::error::Error>> {
    // Thin wrapper - just delegates to domain
    let final_amount = crate::domain::calculator::calculate_compound_interest(
        principal, rate, years
    )?;

    Ok(InterestResult {
        principal,
        rate,
        years,
        final_amount,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_delegates_to_domain() {
        // Arrange
        let principal = 1000.0;
        let rate = 0.05;
        let years = 10;

        // Act
        let result = calculate_interest(principal, rate, years).unwrap();

        // Assert: Verify delegation worked
        assert_eq!(result.principal, principal);
        assert_eq!(result.rate, rate);
        assert_eq!(result.years, years);
        assert!((result.final_amount - 1628.89).abs() < 0.01);
    }
}
```

**Chicago Principle:** Most testing happens in domain layer, CLI tests verify integration.

---

## Testing Patterns

### Pattern 1: Test Error Paths

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_divide_by_zero() {
        // Arrange
        let numerator = 10.0;
        let denominator = 0.0;

        // Act
        let result = divide(numerator, denominator);

        // Assert: Error case
        assert!(result.is_err());
        assert!(matches!(result, Err(MathError::DivisionByZero)));
    }
}
```

---

### Pattern 2: Test State Changes

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_activation_changes_state() {
        // Arrange
        let mut user = User::new("alice");
        assert!(!user.is_active()); // Initial state

        // Act
        user.activate();

        // Assert: State changed
        assert!(user.is_active());
    }
}
```

---

### Pattern 3: Test Return Values

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sum_returns_correct_total() {
        // Arrange
        let numbers = vec![1, 2, 3, 4, 5];

        // Act
        let total = sum(&numbers);

        // Assert: Return value
        assert_eq!(total, 15);
    }
}
```

---

## Integration Testing

### Test CLI Commands End-to-End

```rust
// tests/integration_test.rs
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_finance_calculate_interest_command() {
    // Arrange
    let mut cmd = Command::cargo_bin("my-cli").unwrap();

    // Act
    cmd.arg("finance")
        .arg("calculate-interest")
        .arg("--principal").arg("1000")
        .arg("--rate").arg("0.05")
        .arg("--years").arg("10");

    // Assert
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("final_amount"))
        .stdout(predicate::str::contains("1628.89"));
}

#[test]
fn test_invalid_principal_returns_error() {
    // Arrange
    let mut cmd = Command::cargo_bin("my-cli").unwrap();

    // Act
    cmd.arg("finance")
        .arg("calculate-interest")
        .arg("--principal").arg("-100")
        .arg("--rate").arg("0.05")
        .arg("--years").arg("10");

    // Assert
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("InvalidPrincipal"));
}
```

**Dependencies:**
```toml
[dev-dependencies]
assert_cmd = "2.0"
predicates = "3.0"
```

---

## Exercise: Test-Driven Development

**Goal:** Build a password validator using TDD

### Step 1: Write Failing Tests First

```rust
// domain/password.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_password() {
        // Arrange
        let password = "Secure123!";

        // Act
        let result = validate_password(password);

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn test_password_too_short() {
        // Arrange
        let password = "Sec1!";

        // Act
        let result = validate_password(password);

        // Assert
        assert!(matches!(result, Err(PasswordError::TooShort)));
    }

    #[test]
    fn test_password_no_uppercase() {
        // Arrange
        let password = "secure123!";

        // Act
        let result = validate_password(password);

        // Assert
        assert!(matches!(result, Err(PasswordError::MissingUppercase)));
    }

    #[test]
    fn test_password_no_number() {
        // Arrange
        let password = "SecurePass!";

        // Act
        let result = validate_password(password);

        // Assert
        assert!(matches!(result, Err(PasswordError::MissingNumber)));
    }

    #[test]
    fn test_password_no_special_char() {
        // Arrange
        let password = "SecurePass123";

        // Act
        let result = validate_password(password);

        // Assert
        assert!(matches!(result, Err(PasswordError::MissingSpecialChar)));
    }
}
```

### Step 2: Implement Domain Logic

```rust
// domain/password.rs
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum PasswordError {
    #[error("Password too short (minimum 8 characters)")]
    TooShort,
    #[error("Password must contain uppercase letter")]
    MissingUppercase,
    #[error("Password must contain number")]
    MissingNumber,
    #[error("Password must contain special character")]
    MissingSpecialChar,
}

pub fn validate_password(password: &str) -> Result<(), PasswordError> {
    if password.len() < 8 {
        return Err(PasswordError::TooShort);
    }

    if !password.chars().any(|c| c.is_uppercase()) {
        return Err(PasswordError::MissingUppercase);
    }

    if !password.chars().any(|c| c.is_numeric()) {
        return Err(PasswordError::MissingNumber);
    }

    let special_chars = "!@#$%^&*()_+-=[]{}|;:,.<>?";
    if !password.chars().any(|c| special_chars.contains(c)) {
        return Err(PasswordError::MissingSpecialChar);
    }

    Ok(())
}
```

### Step 3: Run Tests

```bash
cargo test
```

**Expected:**
```
running 5 tests
test domain::password::tests::test_valid_password ... ok
test domain::password::tests::test_password_too_short ... ok
test domain::password::tests::test_password_no_uppercase ... ok
test domain::password::tests::test_password_no_number ... ok
test domain::password::tests::test_password_no_special_char ... ok
```

### Step 4: Create CLI Wrapper

```rust
// commands/security.rs
#[verb(help = "Validate password strength")]
pub fn validate_password(
    #[arg(help = "Password to validate")] password: String,
) -> Result<ValidationResult, Box<dyn std::error::Error>> {
    crate::domain::password::validate_password(&password)?;

    Ok(ValidationResult {
        valid: true,
        message: "Password meets requirements".to_string(),
    })
}
```

---

## Testing Best Practices

### ✅ Do:
1. **Test domain logic exhaustively** - All edge cases, errors, boundary values
2. **Use AAA pattern** - Arrange-Act-Assert structure
3. **One assertion per test** - Clear failure messages
4. **Test observable outputs** - Return values, state changes, side effects
5. **Use descriptive test names** - `test_calculate_interest_with_negative_rate_returns_error`

### ❌ Don't:
1. **Test implementation details** - Avoid testing private functions
2. **Use excessive mocking** - Use real objects when possible
3. **Test CLI parsing** - clap handles this, trust the framework
4. **Ignore error cases** - Test failures as much as success
5. **Write meaningless tests** - Every test should verify behavior

---

## Key Takeaways

✅ **Chicago TDD** - State-based testing with real collaborators
✅ **AAA pattern** - Arrange-Act-Assert for clear tests
✅ **Domain-first testing** - 80% of tests in domain layer
✅ **Integration tests** - Verify CLI commands end-to-end
✅ **Test behavior, not implementation** - Observable outputs matter

---

## Next Steps

- **[Tutorial 05: Output Formats](05-output-formats.md)** - JSON, YAML, custom serialization
- **[Tutorial 06: Autonomic Features](06-autonomic-features.md)** - Machine-grade introspection
- **[How-To: Testing Strategies](../howto/testing/strategies.md)** - Advanced testing patterns

**Estimated time to next tutorial:** 20 minutes

---

## Quick Reference: Test Organization

```
my-cli/
├── src/
│   ├── domain/           # ← Most tests here
│   │   ├── calculator.rs #    #[cfg(test)] mod tests
│   │   └── password.rs   #    #[cfg(test)] mod tests
│   └── commands/         # ← Minimal tests
│       └── finance.rs    #    Integration verification
└── tests/
    └── integration_test.rs # ← End-to-end CLI tests
```

---

*Part of the [clap-noun-verb Tutorial Series](README.md) - Learning-oriented documentation*
