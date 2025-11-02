# Argument Attributes (`#[arg]`)

## Why `#[arg]` Isn't a Real Attribute Macro

**Rust Limitation**: `#[proc_macro_attribute]` can only be used on items (functions, structs, etc.), not on function parameters.

Therefore, `#[arg]` attributes on function parameters are:
- **Parsed by the `#[verb]` macro** during expansion
- **Not recognized by the compiler** as a real attribute
- **Work correctly** at runtime because the `#[verb]` macro processes them

## Current Implementation

The `#[verb]` macro parses `#[arg(...)]` attributes from function parameters during macro expansion. The attributes are:

- Parsed from `pat_type.attrs` in the macro
- Used to generate `ArgMetadata` with `short`, `default_value`, `multiple`, `value_name`, `aliases`, etc.
- Applied to `clap::Arg` during command building

## Testing Strategy

Since `#[arg]` can't be used directly in test code (compiler rejects unknown attributes), we test:

1. **Parsing Logic**: The `parse_arg_attributes()` function correctly parses attribute syntax
2. **Registry Behavior**: Arguments with metadata are correctly registered and built
3. **Integration**: End-to-end behavior works when `#[arg]` attributes are parsed

## Future Options

1. Use `#[allow(unknown_attributes)]` to silence compiler warnings
2. Test parsing logic with `proc-macro2` in unit tests
3. Document that `#[arg]` is a "synthetic" attribute parsed by `#[verb]`

## Usage

In real code, users would write:

```rust
#[verb("set")]
fn set_config(
    #[arg(short = 'p', default_value = "8080")]
    port: u16,
) -> Result<()> {}
```

The compiler may warn about unknown attributes, but the macro processes them correctly.

