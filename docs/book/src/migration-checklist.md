# Migration Checklist

This chapter provides a comprehensive checklist for completing the migration from regular clap to clap-noun-verb, including common pitfalls, solutions, and best practices.

## Step-by-step checklist

### Phase 1: Preparation

- [ ] **Analyze current CLI structure**
  - [ ] Map all commands to noun-verb pattern
  - [ ] Identify command groupings
  - [ ] Document argument requirements
  - [ ] Note any special behaviors

- [ ] **Set up development environment**
  - [ ] Add `clap-noun-verb` to `Cargo.toml`
  - [ ] Verify clap version compatibility (4.x)
  - [ ] Create feature branch for migration
  - [ ] Set up test infrastructure

- [ ] **Plan the migration**
  - [ ] Decide on command grouping strategy
  - [ ] Plan module structure
  - [ ] Identify which commands to port first (start with simple ones)
  - [ ] Set migration timeline

### Phase 2: Implementation

- [ ] **Port base structure**
  - [ ] Create `CliBuilder` with basic setup
  - [ ] Add global arguments (--verbose, --config, etc.)
  - [ ] Test CLI builds without errors

- [ ] **Port simple commands first**
  - [ ] Port commands with no arguments
  - [ ] Port commands with simple arguments
  - [ ] Test each ported command individually
  - [ ] Verify help output matches

- [ ] **Port complex commands**
  - [ ] Port commands with multiple arguments
  - [ ] Port commands with optional arguments
  - [ ] Port commands with complex validation
  - [ ] Test all argument combinations

- [ ] **Port nested structures** (if applicable)
  - [ ] Port compound nouns
  - [ ] Test nested command routing
  - [ ] Verify nested help output

### Phase 3: Testing and Validation

- [ ] **Unit tests**
  - [ ] Test all handlers individually
  - [ ] Test argument extraction
  - [ ] Test error cases
  - [ ] Test edge cases

- [ ] **Integration tests**
  - [ ] Test complete command workflows
  - [ ] Test multiple commands in sequence
  - [ ] Test with various argument combinations
  - [ ] Test help output

- [ ] **Backward compatibility**
  - [ ] Verify all existing command invocations work
  - [ ] Verify argument parsing matches
  - [ ] Verify help text matches
  - [ ] Test with existing scripts/workflows

- [ ] **Validation**
  - [ ] Enable `auto_validate(true)`
  - [ ] Run manual validation
  - [ ] Fix any validation errors
  - [ ] Verify no duplicate names

### Phase 4: Code Quality

- [ ] **Code organization**
  - [ ] Organize commands into modules
  - [ ] Separate handlers from CLI structure
  - [ ] Follow consistent naming conventions
  - [ ] Add appropriate documentation

- [ ] **Error handling**
  - [ ] Replace all `unwrap()` with proper error handling
  - [ ] Use `NounVerbError` for all errors
  - [ ] Add context to error messages
  - [ ] Test error messages are helpful

- [ ] **Documentation**
  - [ ] Update command documentation
  - [ ] Update README with new structure
  - [ ] Add examples for each command
  - [ ] Document any breaking changes

### Phase 5: Final Verification

- [ ] **Smoke tests**
  - [ ] Run full test suite
  - [ ] Test with real-world scenarios
  - [ ] Test with various users/workflows
  - [ ] Performance testing

- [ ] **Review**
  - [ ] Code review
  - [ ] Verify all tests pass
  - [ ] Verify no regressions
  - [ ] Check for any remaining old clap code

- [ ] **Deployment**
  - [ ] Update version number
  - [ ] Write migration notes (if needed)
  - [ ] Update release notes
  - [ ] Deploy and verify in production

## Common pitfalls and solutions

### Pitfall 1: Forgetting Global Arguments

**Problem**: Global arguments not accessible in handlers.

**Solution**: Always pass `VerbArgs` to handlers, use `get_global_*` methods:

```rust,no_run
// ❌ Bad: Global args not accessible
verb!("project", ..., |args: &VerbArgs| {
    // Can't access --verbose, --config
})

// ✅ Good: Access global args
verb!("project", ..., |args: &VerbArgs| {
    let verbose = args.get_global_flag_count("verbose");
    let config = args.get_global_str("config");
})
```

### Pitfall 2: Not Handling Required Arguments

**Problem**: Panic when required argument is missing.

**Solution**: Use `get_one_str()` which returns `Result<String>`:

```rust,no_run
// ❌ Bad: Will panic if missing
let name = args.get_one_str("name").unwrap();

// ✅ Good: Proper error handling
let name = args.get_one_str("name")?;
```

### Pitfall 3: Ignoring Error Propagation

**Problem**: Errors not properly propagated.

**Solution**: Always use `?` operator or handle errors explicitly:

```rust,no_run
// ❌ Bad: Errors swallowed
verb!("project", ..., |args: &VerbArgs| {
    let name = args.get_one_str("name").unwrap_or_default();
    create_project(&name);  // Error ignored
    Ok(())
})

// ✅ Good: Errors propagated
verb!("project", ..., |args: &VerbArgs| -> Result<()> {
    let name = args.get_one_str("name")?;
    create_project(&name)?;  // Errors propagated
    Ok(())
})
```

### Pitfall 4: Duplicate Names

**Problem**: Validation fails due to duplicate command names.

**Solution**: Use unique names, enable auto-validation:

```rust,no_run
// ❌ Bad: Duplicate names
.noun(noun!("ai", ..., [...]))
.noun(noun!("ai", ..., [...]))  // Duplicate!

// ✅ Good: Unique names
.noun(noun!("ai", ..., [...]))
.noun(noun!("marketplace", ..., [...]))
```

### Pitfall 5: Incorrect Argument Definitions

**Problem**: Arguments not parsed correctly.

**Solution**: Match argument definitions exactly:

```rust,no_run
// ❌ Bad: Mismatched argument name
verb!("generate", ..., |args: &VerbArgs| {
    let desc = args.get_one_str("description")?;  // Looking for "description"
}, args: [
    Arg::new("desc"),  // But defined as "desc"
])

// ✅ Good: Matched names
verb!("generate", ..., |args: &VerbArgs| {
    let desc = args.get_one_str("description")?;
}, args: [
    Arg::new("description").short('d').long("description").required(true),
])
```

### Pitfall 6: Not Testing All Argument Combinations

**Problem**: Some argument combinations fail in production.

**Solution**: Test all combinations:

```rust,no_run
#[test]
fn test_all_argument_combinations() -> Result<()> {
    // Test with all flags
    test_command(vec!["ggen", "ai", "project", "name", "--rust"])?;
    
    // Test without flags
    test_command(vec!["ggen", "ai", "project", "name"])?;
    
    // Test with optional args
    test_command(vec!["ggen", "ai", "generate", "-d", "desc", "-o", "out"])?;
    
    // Test without optional args
    test_command(vec!["ggen", "ai", "generate", "-d", "desc"])?;
    
    Ok(())
}
```

### Pitfall 7: Breaking Backward Compatibility

**Problem**: Existing scripts/workflows break.

**Solution**: Maintain command structure and argument names:

```bash
# ✅ Good: Same command structure
ggen ai project my-project  # Works before and after

# ❌ Bad: Changed structure
ggen project my-project ai  # Breaks existing scripts
```

## Best practices

### 1. Start Simple, Build Up

Port simple commands first, then progressively add complexity:

1. Commands with no arguments
2. Commands with simple arguments
3. Commands with optional arguments
4. Commands with complex validation
5. Nested structures

### 2. Test Incrementally

Test each command as you port it:

```rust,no_run
// After porting each command
#[test]
fn test_ai_project_works() -> Result<()> {
    // Test immediately
}
```

### 3. Use Type-Safe Extraction

Always use type-safe methods:

```rust,no_run
// ✅ Good
let name = args.get_one_str("name")?;
let port = args.get_one::<u16>("port")?;

// ❌ Bad: Direct access without type safety
let name = args.matches.get_one::<String>("name").unwrap();
```

### 4. Organize Code Logically

Group related commands:

```
commands/
├── ai.rs          # All AI commands
├── marketplace.rs # All marketplace commands
└── template.rs    # All template commands
```

### 5. Document Command Structure

Add clear documentation:

```rust,no_run
/// Generate complete projects using AI.
///
/// # Examples
///
/// ```bash
/// ggen ai project my-app
/// ggen ai project my-app --rust
/// ```
verb!("project", "Generate complete projects", ...)
```

### 6. Enable Validation

Always enable structure validation:

```rust,no_run
cli.auto_validate(true)
```

### 7. Handle Errors Gracefully

Provide helpful error messages:

```rust,no_run
let name = args.get_one_str("name")
    .map_err(|_| NounVerbError::argument_error(
        "Project name is required. Usage: ggen ai project <name>"
    ))?;
```

### 8. Maintain Consistency

Use consistent patterns across commands:

```rust,no_run
// ✅ Good: Consistent argument patterns
Arg::new("description").short('d').long("description").required(true)
Arg::new("output").short('o').long("output")

// ❌ Bad: Inconsistent
Arg::new("desc").short('d').required(true)  // Different name
Arg::new("output").long("output-file")  // Different name
```

### 9. Test Real-World Scenarios

Test with actual usage patterns:

```rust,no_run
#[test]
fn test_real_world_workflow() -> Result<()> {
    // Search for package
    run_cli_with_args(vec!["ggen", "marketplace", "search", "rust"], build_cli)?;
    
    // Add package
    run_cli_with_args(vec!["ggen", "marketplace", "add", "io.ggen.rust.axum"], build_cli)?;
    
    // Generate project using package
    run_cli_with_args(vec!["ggen", "ai", "project", "my-app", "--rust"], build_cli)?;
    
    Ok(())
}
```

### 10. Review Help Output

Ensure help is clear and consistent:

```bash
# Check root help
ggen --help

# Check noun help
ggen ai --help

# Check verb help
ggen ai project --help
```

## Verification steps

Before considering the migration complete:

### 1. Command Coverage

- [ ] All commands ported
- [ ] All arguments supported
- [ ] All flags work
- [ ] All combinations tested

### 2. Functionality

- [ ] All commands produce same results
- [ ] All edge cases handled
- [ ] All error cases handled
- [ ] Performance acceptable

### 3. Compatibility

- [ ] Backward compatible (if required)
- [ ] Help text matches (if required)
- [ ] Scripts still work
- [ ] Workflows still work

### 4. Code Quality

- [ ] No `unwrap()` or `expect()` in production code
- [ ] Proper error handling everywhere
- [ ] All tests pass
- [ ] Code reviewed
- [ ] Documentation updated

### 5. Deployment

- [ ] Version updated
- [ ] Release notes written
- [ ] Migration guide written (if needed)
- [ ] Ready for deployment

## Final checklist

Before merging/deploying:

```
Migration Completion Checklist
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

□ All commands ported and tested
□ All arguments working correctly
□ Global arguments accessible everywhere
□ Error handling implemented properly
□ All tests passing (unit, integration, compatibility)
□ Help output verified
□ Code organized and documented
□ No breaking changes (or documented if intentional)
□ Performance acceptable
□ Code review completed
□ Ready for production

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

## Conclusion

Migrating from regular clap to clap-noun-verb provides:

- **Cleaner code structure** - Commands organized by functionality
- **Better maintainability** - Easier to add new commands
- **Type safety** - Compile-time verification
- **Less boilerplate** - Macros handle repetitive patterns
- **Better UX** - Intuitive command structure

Following this checklist ensures a smooth migration with minimal risk.

### Next Steps After Migration

1. **Monitor usage** - Watch for any issues in production
2. **Gather feedback** - Collect user feedback on new structure
3. **Iterate** - Make improvements based on feedback
4. **Document** - Keep documentation updated
5. **Share** - Share migration experience with community

Good luck with your migration!

