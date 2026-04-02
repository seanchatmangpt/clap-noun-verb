# Anti-Patterns: What NOT To Do

This document shows **common mistakes** when separating domain logic from CLI code.

## Anti-Pattern 1: CLI Logic in Domain Layer

### ❌ WRONG - Domain knows about paths and println

```rust
// src/domain/processor.rs
use std::path::PathBuf;

pub fn process_file(input: PathBuf, output: PathBuf) -> Result<()> {
    println!("Processing file: {:?}", input);  // CLI concern!

    let data = std::fs::read_to_string(input)?;  // File I/O in domain!
    let result = transform(&data);

    std::fs::write(output, result)?;  // File I/O in domain!
    println!("Done!");  // CLI concern!

    Ok(())
}
```

**Problems:**
- Can't test without file system
- Coupled to specific I/O (files, not streams)
- User output mixed with logic
- Not reusable (what if data comes from network?)

### ✅ RIGHT - Domain takes data, returns data

```rust
// src/domain/processor.rs
use std::io::{BufRead, Write};

pub fn process_stream<R: BufRead, W: Write>(
    reader: R,
    writer: &mut W,
) -> Result<Stats> {
    // Pure logic, generic over I/O
    let mut stats = Stats::default();

    for line in reader.lines() {
        let line = line?;
        let transformed = transform(&line);
        writeln!(writer, "{}", transformed)?;
        stats.processed += 1;
    }

    Ok(stats)  // Return data, don't print!
}

// src/cli/commands.rs
pub fn process_file(input: PathBuf, output: PathBuf) -> Result<()> {
    let input_file = File::open(&input)?;
    let output_file = File::create(&output)?;

    let reader = BufReader::new(input_file);
    let mut writer = BufWriter::new(output_file);

    let stats = domain::process_stream(reader, &mut writer)?;

    println!("✓ Processed {} items", stats.processed);  // CLI formats!
    Ok(())
}
```

## Anti-Pattern 2: Domain Logic in CLI Layer

### ❌ WRONG - Business rules in CLI function

```rust
// src/cli/commands.rs
pub fn create_user(name: String, email: String, age: i32) -> Result<()> {
    // Validation in CLI!
    if name.is_empty() {
        anyhow::bail!("Name cannot be empty");
    }
    if !email.contains('@') {
        anyhow::bail!("Invalid email");
    }
    if age < 18 {
        anyhow::bail!("Must be 18 or older");
    }

    // Business logic in CLI!
    let username = name.to_lowercase().replace(' ', "_");
    let created_at = std::time::SystemTime::now();

    // Save to database
    save_user(username, email, age, created_at)?;

    println!("User created!");
    Ok(())
}
```

**Problems:**
- Can't test validation without CLI
- Can't reuse validation (API, tests, etc.)
- Business rules scattered across CLI functions

### ✅ RIGHT - Domain has rules, CLI calls domain

```rust
// src/domain/user.rs
pub struct User {
    pub name: String,
    pub email: String,
    pub age: i32,
}

pub fn create_user(name: String, email: String, age: i32) -> Result<User, ValidationError> {
    // Validation in domain
    validate_name(&name)?;
    validate_email(&email)?;
    validate_age(age)?;

    Ok(User { name, email, age })
}

fn validate_name(name: &str) -> Result<(), ValidationError> {
    if name.is_empty() {
        Err(ValidationError::EmptyName)
    } else {
        Ok(())
    }
}

fn validate_email(email: &str) -> Result<(), ValidationError> {
    if !email.contains('@') {
        Err(ValidationError::InvalidEmail)
    } else {
        Ok(())
    }
}

// src/cli/commands.rs
pub fn create_user_cmd(name: String, email: String, age: i32) -> Result<()> {
    let user = domain::create_user(name, email, age)
        .map_err(|e| anyhow::anyhow!("Validation failed: {}", e))?;

    save_user(&user)?;

    println!("✓ User created: {}", user.name);
    Ok(())
}
```

## Anti-Pattern 3: Untestable Code

### ❌ WRONG - Tight coupling prevents testing

```rust
pub fn sync_data() -> Result<()> {
    let client = reqwest::blocking::Client::new();  // Hard-coded!
    let db = Database::connect("postgres://...")?;   // Hard-coded!

    let data = client.get("https://api.example.com/data").send()?.json()?;

    for item in data {
        db.insert(item)?;
    }

    Ok(())
}
```

**Problems:**
- Can't test without real HTTP server
- Can't test without real database
- Slow tests
- Flaky tests

### ✅ RIGHT - Dependency injection enables testing

```rust
// Domain layer - generic over dependencies
pub fn sync_data<F, D>(fetcher: &F, db: &D) -> Result<Stats>
where
    F: DataFetcher,
    D: DataStore,
{
    let data = fetcher.fetch()?;

    let mut stats = Stats::default();
    for item in data {
        db.insert(item)?;
        stats.synced += 1;
    }

    Ok(stats)
}

// Test with mocks
#[cfg(test)]
mod tests {
    struct MockFetcher { data: Vec<Item> }
    impl DataFetcher for MockFetcher {
        fn fetch(&self) -> Result<Vec<Item>> { Ok(self.data.clone()) }
    }

    struct MockStore { items: Vec<Item> }
    impl DataStore for MockStore {
        fn insert(&mut self, item: Item) -> Result<()> {
            self.items.push(item);
            Ok(())
        }
    }

    #[test]
    fn test_sync_data() {
        let fetcher = MockFetcher { data: vec![Item::new(1)] };
        let mut store = MockStore { items: vec![] };

        let stats = sync_data(&fetcher, &mut store).unwrap();

        assert_eq!(stats.synced, 1);
        assert_eq!(store.items.len(), 1);
    }
}

// CLI layer - concrete implementations
pub fn sync_cmd() -> Result<()> {
    let client = HttpFetcher::new("https://api.example.com");
    let db = PostgresStore::connect("postgres://...")?;

    let stats = domain::sync_data(&client, &db)?;

    println!("✓ Synced {} items", stats.synced);
    Ok(())
}
```

## Anti-Pattern 4: Type Confusion

### ❌ WRONG - Primitive obsession

```rust
// Everything is String!
pub fn create_order(
    user_id: String,
    product_id: String,
    quantity: String,
    price: String,
) -> Result<String> {
    // Runtime validation nightmare
    let qty: i32 = quantity.parse()?;
    let price_val: f64 = price.parse()?;

    if qty <= 0 {
        anyhow::bail!("Invalid quantity");
    }

    // ... more validation
}
```

### ✅ RIGHT - Type-first design

```rust
// Domain types encode rules
#[derive(Debug)]
pub struct UserId(NonZeroU64);

#[derive(Debug)]
pub struct ProductId(NonZeroU64);

#[derive(Debug)]
pub struct Quantity(NonZeroU32);

#[derive(Debug)]
pub struct Price(f64);

impl Price {
    pub fn new(value: f64) -> Result<Self, ValidationError> {
        if value <= 0.0 {
            Err(ValidationError::InvalidPrice)
        } else {
            Ok(Self(value))
        }
    }
}

// Domain function - invalid states impossible
pub fn create_order(
    user_id: UserId,
    product_id: ProductId,
    quantity: Quantity,
    price: Price,
) -> Order {
    // No validation needed - types guarantee validity!
    Order {
        user_id,
        product_id,
        quantity,
        price,
    }
}

// CLI parses and validates
pub fn create_order_cmd(
    user_id: String,
    product_id: String,
    quantity: String,
    price: String,
) -> Result<()> {
    let user_id = UserId::parse(&user_id)?;
    let product_id = ProductId::parse(&product_id)?;
    let quantity = Quantity::parse(&quantity)?;
    let price = Price::new(price.parse()?)?;

    let order = domain::create_order(user_id, product_id, quantity, price);

    println!("✓ Order created: {:?}", order);
    Ok(())
}
```

## Anti-Pattern 5: God Functions

### ❌ WRONG - One function does everything

```rust
pub fn process(
    input: PathBuf,
    output: PathBuf,
    format: String,
    compress: bool,
    validate: bool,
) -> Result<()> {
    // 200 lines of mixed concerns
    // - File I/O
    // - Parsing
    // - Validation
    // - Transformation
    // - Formatting
    // - Compression
    // - More file I/O
    // - Error handling
    // - Logging
}
```

### ✅ RIGHT - Small, focused functions

```rust
// Domain layer - each function does one thing
pub fn parse_input(data: &str) -> Result<Input, ParseError>;
pub fn validate_input(input: &Input) -> Result<(), ValidationError>;
pub fn transform(input: Input) -> Result<Output, TransformError>;
pub fn format_output(output: &Output, format: Format) -> String;

// CLI layer - orchestrates domain functions
pub fn process_cmd(
    input: PathBuf,
    output: PathBuf,
    format: String,
    compress: bool,
    validate: bool,
) -> Result<()> {
    let data = std::fs::read_to_string(&input)?;
    let input = domain::parse_input(&data)?;

    if validate {
        domain::validate_input(&input)?;
    }

    let output_data = domain::transform(input)?;
    let formatted = domain::format_output(&output_data, Format::from_str(&format)?);

    let final_data = if compress {
        compress_data(&formatted)?
    } else {
        formatted
    };

    std::fs::write(&output, final_data)?;

    println!("✓ Processing complete");
    Ok(())
}
```

## Summary: Golden Rules

1. **Domain layer = pure logic**
   - Takes data, returns data
   - No I/O, no CLI dependencies
   - Generic over I/O types
   - Fully testable

2. **CLI layer = thin wrapper**
   - Parses arguments
   - Opens files, makes HTTP calls
   - Formats output for users
   - Delegates to domain

3. **Types encode invariants**
   - Use newtypes for IDs
   - Validation in constructors
   - Invalid states impossible

4. **Small, focused functions**
   - Single Responsibility Principle
   - Easy to test
   - Easy to reuse

5. **Dependency injection**
   - Generic over collaborators
   - Testable with mocks
   - Flexible composition
