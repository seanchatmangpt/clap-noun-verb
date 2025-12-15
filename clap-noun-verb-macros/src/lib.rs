//! Procedural macros for clap-noun-verb
//!
//! This crate provides attribute macros `#[noun]` and `#[verb]` for
//! declarative CLI command registration.
//!
//! # I/O Support (v4.0)
//!
//! The #[verb] macro now auto-detects clio::Input and clio::Output types
//! in function parameters and automatically wires them with appropriate
//! clap configuration.
//!
//! # Compile-Time Error Proofing (Poka-Yoke)
//!
//! The macro system includes advanced compile-time validation:
//! - Gap 1: Forgotten #[verb] detection
//! - Gap 2: Duplicate verb detection
//! - Gap 3: Return type must implement Serialize
//! - Gap 4: Enhanced attribute syntax validation

mod io_detection;
mod rdf_generation;
mod telemetry_validation;
mod validation;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse::Parser, parse_macro_input, ItemFn};

// Note: #[arg(...)] attributes on function parameters cannot be a real proc_macro_attribute
// because Rust doesn't allow proc_macro_attribute on parameters - only on items.
// The #[verb] macro parses #[arg(...)] attributes directly from pat_type.attrs.
// However, we provide a pass-through #[arg] macro so the compiler accepts the attribute.
// Users may still need #[allow(unknown_attributes)] in some contexts.

/// Pass-through attribute macro for #[arg(...)] on function parameters
///
/// This attribute is parsed by the #[verb] macro but needs to exist as a real
/// proc_macro_attribute so the compiler accepts it. It does nothing itself -
/// the #[verb] macro processes these attributes during expansion.
///
/// Usage:
/// ```rust,ignore
/// #[verb("set")]
/// fn set_config(
///     #[arg(env = "SERVER_PORT", default_value = "8080")]
///     port: u16,
/// ) -> Result<()> {}
/// ```
#[proc_macro_attribute]
pub fn arg(_args: TokenStream, input: TokenStream) -> TokenStream {
    // Just pass through - the #[verb] macro will parse #[arg(...)] attributes
    input
}

/// Declare a telemetry span for compile-time validation
///
/// This macro creates a span constant and registers it in the distributed slice
/// for compile-time validation. If the span is never used in a `span!` macro,
/// compilation will fail.
///
/// # Example
///
/// ```rust,ignore
/// use clap_noun_verb_macros::declare_span;
///
/// // Declare span
/// declare_span!(PROCESS_REQUEST, "process_request");
///
/// // Use it (required or compilation fails)
/// fn process() {
///     span!(PROCESS_REQUEST, {
///         // ... work ...
///     })
/// }
/// ```
#[proc_macro]
pub fn declare_span(input: TokenStream) -> TokenStream {
    let input = proc_macro2::TokenStream::from(input);

    // Parse: IDENT, "name"
    let parser = syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated;
    let args = match Parser::parse2(parser, input) {
        Ok(args) => args,
        Err(e) => return e.to_compile_error().into(),
    };

    if args.len() != 2 {
        return syn::Error::new_spanned(
            quote::quote! { #args },
            "declare_span! requires exactly 2 arguments: IDENT, \"name\"",
        )
        .to_compile_error()
        .into();
    }

    let ident = match &args[0] {
        syn::Expr::Path(path) => {
            if let Some(ident) = path.path.get_ident() {
                ident.clone()
            } else {
                return syn::Error::new_spanned(
                    &args[0],
                    "First argument must be an identifier (e.g., PROCESS_REQUEST)",
                )
                .to_compile_error()
                .into();
            }
        }
        _ => {
            return syn::Error::new_spanned(
                &args[0],
                "First argument must be an identifier (e.g., PROCESS_REQUEST)",
            )
            .to_compile_error()
            .into()
        }
    };

    let name = match &args[1] {
        syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(s), .. }) => s.value(),
        _ => {
            return syn::Error::new_spanned(
                &args[1],
                "Second argument must be a string literal (e.g., \"process_request\")",
            )
            .to_compile_error()
            .into()
        }
    };

    let output = telemetry_validation::generate_span_declaration(&ident, &name);
    output.into()
}

/// Instrument a code block with a telemetry span
///
/// This macro wraps a block of code with span instrumentation and registers
/// usage for compile-time validation.
///
/// # Example
///
/// ```rust,ignore
/// use clap_noun_verb_macros::{declare_span, span};
///
/// declare_span!(PROCESS_DATA, "process_data");
///
/// fn process() -> Result<Data> {
///     span!(PROCESS_DATA, {
///         // Work happens here
///         Ok(Data::new())
///     })
/// }
/// ```
#[proc_macro]
pub fn span(input: TokenStream) -> TokenStream {
    let input = proc_macro2::TokenStream::from(input);

    // Parse: IDENT, { block }
    let parser = syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated;
    let args = match Parser::parse2(parser, input.clone()) {
        Ok(args) => args,
        Err(e) => return e.to_compile_error().into(),
    };

    if args.len() != 2 {
        return syn::Error::new_spanned(
            quote::quote! { #input },
            "span! requires exactly 2 arguments: SPAN_CONST, { block }",
        )
        .to_compile_error()
        .into();
    }

    let span_ident = match &args[0] {
        syn::Expr::Path(path) => {
            if let Some(ident) = path.path.get_ident() {
                ident.clone()
            } else {
                return syn::Error::new_spanned(
                    &args[0],
                    "First argument must be a span constant (e.g., PROCESS_REQUEST)",
                )
                .to_compile_error()
                .into();
            }
        }
        _ => {
            return syn::Error::new_spanned(
                &args[0],
                "First argument must be a span constant (e.g., PROCESS_REQUEST)",
            )
            .to_compile_error()
            .into()
        }
    };

    let block = &args[1];

    // Register usage (only when autonomic feature enabled)
    let usage = telemetry_validation::generate_span_usage(&span_ident);

    // Generate instrumented code
    // When autonomic feature is disabled, just execute the block without telemetry
    let expanded = quote::quote! {
        {
            // Span usage registration (only when autonomic feature enabled)
            #[cfg(feature = "autonomic")]
            #usage

            // Create span (only when autonomic feature enabled)
            #[cfg(feature = "autonomic")]
            let mut _span = ::clap_noun_verb::autonomic::telemetry::TraceSpan::new_root(#span_ident);

            // Execute block
            let _result = #block;

            // Finish span (only when autonomic feature enabled)
            #[cfg(feature = "autonomic")]
            _span.finish();

            _result
        }
    };

    expanded.into()
}

/// Attribute macro for registering a noun command
///
/// Usage:
/// ```rust,ignore
/// #[noun("services", "Manage services")]
/// fn my_function() {}
/// ```
#[proc_macro_attribute]
pub fn noun(args: TokenStream, input: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(input as ItemFn);

    // Parse arguments: name and about
    let parser = syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated;
    let args_vec: syn::punctuated::Punctuated<_, _> =
        match Parser::parse2(parser, proc_macro2::TokenStream::from(args)) {
            Ok(args) => args,
            Err(e) => return e.to_compile_error().into(),
        };

    if args_vec.len() != 2 {
        return syn::Error::new_spanned(
            quote! { #args_vec },
            "Expected exactly 2 arguments: name and about",
        )
        .to_compile_error()
        .into();
    }

    let name_expr = &args_vec[0];
    let about_expr = &args_vec[1];

    let name_str = match name_expr {
        syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(s), .. }) => s.value(),
        _ => {
            return syn::Error::new_spanned(name_expr, "First argument must be a string literal")
                .to_compile_error()
                .into()
        }
    };

    let about_str = match about_expr {
        syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(s), .. }) => s.value(),
        _ => {
            return syn::Error::new_spanned(about_expr, "Second argument must be a string literal")
                .to_compile_error()
                .into()
        }
    };

    let fn_name = &input_fn.sig.ident;
    let init_fn_name = quote::format_ident!("__init_noun_{}", fn_name);

    // Check if #[verb] is also present - if so, emit helper attribute for #[verb] to detect
    let has_verb_attr = input_fn.attrs.iter().any(|attr| {
        attr.path().is_ident("verb")
            || attr.path().segments.last().map(|seg| seg.ident == "verb").unwrap_or(false)
    });

    // Generate registration code
    // Core team approach: Emit helper attribute in generated code when #[verb] is present
    // This ensures #[verb] can always detect the noun name, regardless of processing order
    let mut output_fn = input_fn.clone();

    // Remove #[noun] attribute from output (it's been processed)
    output_fn.attrs.retain(|attr| {
        !attr.path().is_ident("noun")
            && attr.path().segments.last().map(|seg| seg.ident != "noun").unwrap_or(true)
    });

    // If #[verb] is present, emit helper doc comment in generated code
    // Core team approach: Use doc comment as hidden storage - Rust won't try to process it
    let helper_doc = if has_verb_attr {
        let doc_attr_value = format!("__noun_name_internal:{}", name_str);
        quote! {
            #[doc = #doc_attr_value]
        }
    } else {
        quote! {}
    };

    let expanded = quote! {
        #helper_doc
        #output_fn

        // Auto-generated registration
        // CRITICAL FIX: Use named function instead of closure to satisfy fn() type requirement
        #[allow(non_upper_case_globals)]
        #[linkme::distributed_slice(::clap_noun_verb::cli::registry::__NOUN_REGISTRY)]
        static #init_fn_name: fn() = {
            fn __register_impl() {
                ::clap_noun_verb::cli::registry::CommandRegistry::register_noun(
                    #name_str,
                    #about_str,
                );
            }
            __register_impl  // Return function pointer (not a call!)
        };
    };

    expanded.into()
}

/// Attribute macro for registering a verb command
///
/// Usage:
/// ```rust,ignore
/// #[verb("status")]
/// fn show_status() -> Result<Status> {}
/// ```
///
/// # Compile-Time Validation
///
/// This macro performs extensive compile-time validation:
/// - Return type must exist and implement Serialize
/// - Attribute syntax must be correct (helpful error messages)
/// - Duplicate verb registration detection
/// - Parameter attributes (#[arg]) must be valid
#[proc_macro_attribute]
pub fn verb(args: TokenStream, input: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(input as ItemFn);
    let args_tokens = proc_macro2::TokenStream::from(args.clone());

    // GAP 3: Validate return type implements Serialize
    if let Err(e) = validation::validate_return_type(&input_fn.sig.output, &input_fn.sig.ident) {
        return e.to_compile_error().into();
    }

    // GAP 4: Validate attribute syntax with helpful errors
    if let Err(e) = validation::validate_verb_attribute_syntax(&args_tokens, &input_fn) {
        return e.to_compile_error().into();
    }

    // 🛡️ POKA-YOKE FM-1.1: Validate verb function complexity (CLI layer purity)
    // Prevents business logic from leaking into verb functions
    if let Err(e) = validation::validate_verb_complexity(&input_fn) {
        return e.to_compile_error().into();
    }

    // 🛡️ POKA-YOKE FM-1.2: Validate no CLI types in parameters (domain independence)
    // Prevents domain functions from depending on CLI types
    if let Err(e) = validation::validate_no_cli_types_in_params(&input_fn.sig) {
        return e.to_compile_error().into();
    }

    // Validate #[arg] attributes on parameters
    for input in &input_fn.sig.inputs {
        if let syn::FnArg::Typed(pat_type) = input {
            if let Err(e) = validation::validate_arg_attribute_syntax(&pat_type.attrs) {
                return e.to_compile_error().into();
            }
        }
    }

    // Parse verb name from args
    let parser = syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated;
    let args_vec: syn::punctuated::Punctuated<_, _> =
        match Parser::parse2(parser, args_tokens.clone()) {
            Ok(args) => args,
            Err(_) => {
                // If parsing fails, extract verb name from function name
                let verb_name = extract_verb_name_from_fn_name(&input_fn);
                let docstring = extract_docstring(&input_fn);
                let arg_relationships = parse_argument_descriptions_with_relationships(&docstring);
                return generate_verb_registration(
                    input_fn,
                    verb_name,
                    None,
                    None,
                    arg_relationships,
                );
            }
        };

    let verb_name = if args_vec.is_empty() {
        extract_verb_name_from_fn_name(&input_fn)
    } else {
        match &args_vec[0] {
            syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(s), .. }) => s.value(),
            _ => {
                return syn::Error::new_spanned(
                    &args_vec[0],
                    "First argument must be a string literal",
                )
                .to_compile_error()
                .into()
            }
        }
    };

    // Extract noun name if provided as second arg, or auto-detect from #[noun] attribute or file context
    let noun_name = if args_vec.len() > 1 {
        match &args_vec[1] {
            syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(s), .. }) => Some(s.value()),
            _ => None,
        }
    } else {
        // Try to auto-detect noun name:
        // 1. First check for #[noun] attribute on same function
        // 2. Then try to infer from filename using file!() macro
        extract_noun_name_from_attributes(&input_fn)
            .or_else(|| extract_noun_name_from_file_context(&input_fn))
    };

    // If verb name was auto-inferred and noun name was auto-detected,
    // strip the noun name from the verb name if it appears in the function name
    // Example: show_collector_status() with noun="collector" -> verb="status" (not "collector_status")
    let verb_name = if args_vec.is_empty() {
        if let Some(noun) = noun_name.as_ref() {
            // Check if verb_name starts with noun_name (e.g., "collector_status" starts with "collector")
            if verb_name.starts_with(noun) && verb_name.len() > noun.len() {
                // Check if there's a separator (underscore) after the noun
                if verb_name.as_bytes()[noun.len()] == b'_' {
                    // Strip noun_ prefix (e.g., "collector_status" -> "status")
                    verb_name[noun.len() + 1..].to_string()
                } else {
                    verb_name
                }
            } else {
                verb_name
            }
        } else {
            verb_name
        }
    } else {
        verb_name
    };

    // Extract docstring for help text
    let docstring = extract_docstring(&input_fn);

    // Parse argument descriptions with relationship metadata from docstring
    let arg_relationships = parse_argument_descriptions_with_relationships(&docstring);

    // Clean docstring for about - remove # Arguments section and relationship tags
    let clean_about = clean_docstring_for_about(&docstring);
    generate_verb_registration(input_fn, verb_name, noun_name, Some(clean_about), arg_relationships)
}

/// Extract verb name from function name (remove common prefixes)
fn extract_verb_name_from_fn_name(input_fn: &ItemFn) -> String {
    let fn_name = input_fn.sig.ident.to_string();

    // List of prefixes to strip in order of priority
    let prefixes = [
        "show_", "get_", "list_", "create_", "delete_", "update_", "fetch_", "display_", "print_",
        "run_", "execute_", "check_", "verify_", "start_", "stop_", "restart_", "add_", "remove_",
        "set_", "unset_",
    ];

    // Try each prefix
    for prefix in &prefixes {
        if let Some(stripped) = fn_name.strip_prefix(prefix) {
            return stripped.to_string();
        }
    }

    // If no prefix matches, return the function name as-is
    fn_name
}

/// Extract noun name from filename using file!() macro
///
/// Core team approach: Infer noun name from source filename.
/// Example: `services.rs` -> `"services"`, `user_management.rs` -> `"user_management"`
fn extract_noun_name_from_file_context(_input_fn: &ItemFn) -> Option<String> {
    // Core team approach: Infer noun name from filename
    // We can't access filename at compile time in stable Rust, so we'll extract it
    // at runtime using file!() macro in the generated code.
    // Return None here to signal that we should extract from file!() at runtime.
    None // Will be extracted at runtime using file!() in generated code
}

// Note: Module doc extraction from `//!` comments is complex in proc macros
// because we need to parse the entire file. For now, we use function doc as fallback.
// Future enhancement: Use span information or file parsing to extract module docs.

/// Extract noun name from attributes on same function
///
/// Core team approach: Check for helper doc comment first (emitted by #[noun] when #[verb] is present),
/// then fall back to original #[noun] attribute. This works regardless of macro processing order.
///
/// The helper doc comment `#[doc = "__noun_name_internal:name"]` is emitted by #[noun] when it detects
/// #[verb] is also present, ensuring reliable noun name detection without Rust trying to process it.
fn extract_noun_name_from_attributes(input_fn: &ItemFn) -> Option<String> {
    // First, check for helper doc comment emitted by #[noun]
    // Format: #[doc = "__noun_name_internal:name"]
    for attr in &input_fn.attrs {
        if attr.path().is_ident("doc") {
            if let syn::Meta::NameValue(nv) = &attr.meta {
                if let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(s), .. }) = &nv.value {
                    let doc_value = s.value();
                    if let Some(noun_name) = doc_value.strip_prefix("__noun_name_internal:") {
                        return Some(noun_name.to_string());
                    }
                }
            }
        }
    }

    // Fallback: Check for original #[noun] attribute (when #[verb] processes first)
    for attr in &input_fn.attrs {
        let is_noun_attr = {
            if attr.path().is_ident("noun") {
                true
            } else {
                let segments: Vec<_> = attr.path().segments.iter().collect();
                segments.last().map(|seg| seg.ident == "noun").unwrap_or(false)
            }
        };

        if is_noun_attr {
            if let syn::Meta::List(meta_list) = &attr.meta {
                let parser =
                    syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated;
                if let Ok(args_vec) = parser.parse2(meta_list.tokens.clone()) {
                    if !args_vec.is_empty() {
                        if let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(s), .. }) =
                            &args_vec[0]
                        {
                            return Some(s.value());
                        }
                    }
                }
            }
        }
    }

    None
}

/// Extract docstring from function attributes
///
/// Doc comments in syn are stored as Meta::List with "doc" as the path.
/// Each doc comment line is a separate attribute.
fn extract_docstring(input_fn: &ItemFn) -> String {
    input_fn
        .attrs
        .iter()
        .filter_map(|attr| {
            if attr.path().is_ident("doc") {
                // Doc comments in syn 2.0 are stored as Meta::NameValue
                // Format: #[doc = "text"]
                let meta = &attr.meta;
                match meta {
                    syn::Meta::NameValue(nv) => {
                        if let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(s), .. }) =
                            &nv.value
                        {
                            Some(s.value().trim().to_string())
                        } else {
                            None
                        }
                    }
                    // Some doc comments might be in List format
                    syn::Meta::List(list) => {
                        // Extract tokens from list
                        let tokens = list.tokens.to_string();
                        // Remove quotes and extra formatting
                        Some(tokens.trim_matches('"').trim().to_string())
                    }
                    _ => None,
                }
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

/// Argument relationship metadata extracted from doc comments
///
/// Typer-like syntax for relationships in doc comments:
/// - `[group: name]` - Argument belongs to exclusive group "name"
/// - `[requires: arg]` - Argument requires "arg" to be present
/// - `[conflicts: arg]` - Argument conflicts with "arg"
///
/// Example:
/// ```text
/// # Arguments
/// * `json` - Export as JSON [group: format]
/// * `yaml` - Export as YAML [group: format]
/// * `filename` - Output filename [requires: format]
/// * `raw` - Raw output mode [conflicts: format]
/// * `config` - Config file [env: APP_CONFIG] [default: config.toml]
/// * `verbose` - Verbose output [hide]
/// * `format` - Output format [value_hint: file_path]
/// * `debug` - Debug mode [global]
/// * `force` - Force operation [exclusive]
/// * `output` - Output options [help_heading: Output]
/// ```
#[derive(Default, Clone)]
struct DocArgRelationships {
    /// Group name (for exclusive groups)
    group: Option<String>,
    /// Arguments this one requires
    requires: Vec<String>,
    /// Arguments this one conflicts with
    conflicts_with: Vec<String>,
    /// Environment variable to read value from
    env: Option<String>,
    /// Whether this argument should be hidden from help
    hide: bool,
    /// Default value for the argument
    default_value: Option<String>,
    /// Value hint for shell completion (file_path, dir_path, url, etc.)
    value_hint: Option<String>,
    /// Whether this argument is global (propagates to subcommands)
    global: bool,
    /// Whether this argument is exclusive (cannot be used with any other args)
    exclusive: bool,
    /// Help heading to group this argument under
    help_heading: Option<String>,
    /// Clean description without relationship tags
    description: String,
}

/// Remove markdown code fences from text
///
/// Removes triple-backtick code blocks (```rust, ```bash, ``` etc.) while preserving
/// the text content. This prevents code examples from appearing literally in help output.
///
/// Examples:
/// - Input: "Example:\n```rust\nlet x = 5;\n```\nDone."
/// - Output: "Example:\nlet x = 5;\nDone."
fn strip_markdown_code_fences(text: &str) -> String {
    let mut result = String::new();
    let lines: Vec<&str> = text.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim();

        // Check if this line starts a code fence
        if trimmed.starts_with("```") {
            // Skip the opening fence line
            i += 1;

            // Skip lines until we find the closing fence
            while i < lines.len() {
                let closing_line = lines[i].trim();
                if closing_line.starts_with("```") {
                    // Found closing fence, skip it and move past
                    i += 1;
                    break;
                } else {
                    // Include content lines (but only if they're meaningful)
                    let content = lines[i];
                    if !content.trim().is_empty() {
                        if !result.is_empty() && !result.ends_with('\n') {
                            result.push('\n');
                        }
                        result.push_str(content);
                    }
                    i += 1;
                }
            }
        } else {
            // Regular line, include it
            if !result.is_empty() && !result.ends_with('\n') {
                result.push('\n');
            }
            result.push_str(line);
            i += 1;
        }
    }

    result.trim().to_string()
}

/// Clean docstring for command about text
///
/// Removes the `# Arguments` section entirely (clap generates its own help for arguments)
/// and strips any relationship tags from the remaining text.
fn clean_docstring_for_about(docstring: &str) -> String {
    let mut result_lines = Vec::new();
    let mut in_arguments_section = false;

    for line in docstring.lines() {
        let trimmed = line.trim();

        // Check if we've entered the Arguments section
        if trimmed == "# Arguments" || trimmed.starts_with("# Arguments") {
            in_arguments_section = true;
            continue;
        }

        // If we hit another section heading, we're done with Arguments
        if in_arguments_section && trimmed.starts_with('#') && !trimmed.starts_with("# Arguments") {
            in_arguments_section = false;
        }

        // Skip lines in the Arguments section
        if in_arguments_section {
            continue;
        }

        // Strip any relationship tags from the line
        let clean_line = strip_relationship_tags(line);
        if !clean_line.is_empty() || line.is_empty() {
            result_lines.push(clean_line);
        }
    }

    // Trim trailing empty lines
    while result_lines.last().map(|s| s.trim().is_empty()).unwrap_or(false) {
        result_lines.pop();
    }

    let joined = result_lines.join("\n").trim().to_string();

    // Remove markdown code fences from the final result
    strip_markdown_code_fences(&joined)
}

/// Strip relationship tags from a line of text
///
/// Removes [group: xxx], [requires: xxx], [conflicts: xxx], [env: xxx], [hide],
/// [default: xxx], [value_hint: xxx], [global], [exclusive], [help_heading: xxx] tags.
fn strip_relationship_tags(text: &str) -> String {
    let mut result = String::new();
    let mut remaining = text;

    while !remaining.is_empty() {
        if let Some(tag_start) = remaining.find('[') {
            // Add text before the tag
            result.push_str(&remaining[..tag_start]);

            // Find the end of the tag
            if let Some(tag_end) = remaining[tag_start..].find(']') {
                let tag_content = &remaining[tag_start + 1..tag_start + tag_end];

                // Check if this is a relationship tag we should strip
                let is_relationship_tag = if let Some(colon_pos) = tag_content.find(':') {
                    let key = tag_content[..colon_pos].trim().to_lowercase();
                    matches!(
                        key.as_str(),
                        "group"
                            | "requires"
                            | "require"
                            | "conflicts"
                            | "conflicts_with"
                            | "conflict"
                            | "env"
                            | "default"
                            | "value_hint"
                            | "help_heading"
                    )
                } else {
                    // Tags without colon
                    let key = tag_content.trim().to_lowercase();
                    matches!(key.as_str(), "hide" | "global" | "exclusive")
                };

                if !is_relationship_tag {
                    // Keep unknown tags
                    result.push('[');
                    result.push_str(tag_content);
                    result.push(']');
                }

                // Move past the tag
                remaining = &remaining[tag_start + tag_end + 1..];
            } else {
                // No closing bracket, add rest as-is
                result.push_str(remaining);
                break;
            }
        } else {
            // No more tags, add the rest
            result.push_str(remaining);
            break;
        }
    }

    result
}

/// Parse relationship metadata from a description string
///
/// Extracts all Typer-like tags from argument descriptions:
/// - `[group: name]` - Argument belongs to exclusive group
/// - `[requires: arg]` - Argument requires another argument
/// - `[conflicts: arg]` - Argument conflicts with another argument
/// - `[env: VAR]` - Read value from environment variable
/// - `[hide]` - Hide this argument from help
/// - `[default: value]` - Default value for the argument
/// - `[value_hint: type]` - Value hint for shell completion
/// - `[global]` - Propagate to subcommands
/// - `[exclusive]` - Cannot be used with any other args
/// - `[help_heading: name]` - Group under help heading
fn parse_doc_relationships(description: &str) -> DocArgRelationships {
    let mut result = DocArgRelationships::default();
    let mut clean_parts = Vec::new();

    // Parse description, extracting relationship tags
    let mut remaining = description;

    while !remaining.is_empty() {
        // Look for a tag
        if let Some(tag_start) = remaining.find('[') {
            // Add text before the tag
            let before = remaining[..tag_start].trim();
            if !before.is_empty() {
                clean_parts.push(before.to_string());
            }

            // Find the end of the tag
            if let Some(tag_end) = remaining[tag_start..].find(']') {
                let tag_content = &remaining[tag_start + 1..tag_start + tag_end];

                // Parse tag content: "key: value" or "key" (for boolean tags)
                if let Some(colon_pos) = tag_content.find(':') {
                    let key = tag_content[..colon_pos].trim().to_lowercase();
                    let value = tag_content[colon_pos + 1..].trim().to_string();

                    match key.as_str() {
                        "group" => result.group = Some(value),
                        "requires" | "require" => result.requires.push(value),
                        "conflicts" | "conflicts_with" | "conflict" => {
                            result.conflicts_with.push(value)
                        }
                        "env" => result.env = Some(value),
                        "default" | "default_value" => result.default_value = Some(value),
                        "value_hint" | "hint" => result.value_hint = Some(value),
                        "help_heading" | "heading" => result.help_heading = Some(value),
                        _ => {
                            // Unknown tag, keep it in the description
                            clean_parts.push(format!("[{}]", tag_content));
                        }
                    }
                } else {
                    // Boolean tags without colon (e.g., [hide], [global], [exclusive])
                    let key = tag_content.trim().to_lowercase();
                    match key.as_str() {
                        "hide" | "hidden" => result.hide = true,
                        "global" => result.global = true,
                        "exclusive" => result.exclusive = true,
                        _ => {
                            // Unknown tag, keep it in the description
                            clean_parts.push(format!("[{}]", tag_content));
                        }
                    }
                }

                // Move past the tag
                remaining = &remaining[tag_start + tag_end + 1..];
            } else {
                // No closing bracket, add rest as-is
                clean_parts.push(remaining.to_string());
                break;
            }
        } else {
            // No more tags, add the rest
            let rest = remaining.trim();
            if !rest.is_empty() {
                clean_parts.push(rest.to_string());
            }
            break;
        }
    }

    let description = clean_parts.join(" ").trim().to_string();

    // Remove markdown code fences from the description
    result.description = strip_markdown_code_fences(&description);
    result
}

/// Parse argument descriptions WITH relationship metadata from docstring
///
/// Similar to parse_argument_descriptions but also extracts Typer-like
/// relationship tags from the description text.
fn parse_argument_descriptions_with_relationships(
    docstring: &str,
) -> std::collections::HashMap<String, DocArgRelationships> {
    let mut results = std::collections::HashMap::new();

    // Split docstring into lines
    let lines: Vec<&str> = docstring.lines().collect();
    let mut in_arguments_section = false;

    for line in lines {
        let trimmed = line.trim();

        // Check if we've entered the Arguments section
        if trimmed == "# Arguments" || trimmed.starts_with("# Arguments") {
            in_arguments_section = true;
            continue;
        }

        // If we hit another section heading, stop parsing
        if in_arguments_section && trimmed.starts_with('#') && !trimmed.starts_with("# Arguments") {
            break;
        }

        // Parse argument description line
        if in_arguments_section && trimmed.starts_with('*') {
            let rest = trimmed[1..].trim();

            if let Some(dash_pos) = rest.find('-') {
                let before_dash = rest[..dash_pos].trim();
                let description = rest[dash_pos + 1..].trim();

                // Extract argument name (remove backticks and asterisks)
                let arg_name =
                    before_dash.trim_start_matches('*').trim().trim_matches('`').trim().to_string();

                if !arg_name.is_empty() && !description.is_empty() {
                    // Parse relationships from description
                    let relationships = parse_doc_relationships(description);
                    results.insert(arg_name, relationships);
                }
            }
        }
    }

    results
}

/// Generate verb registration code with full type inference
fn generate_verb_registration(
    input_fn: ItemFn,
    verb_name: String,
    noun_name: Option<String>,
    about: Option<String>,
    arg_relationships: std::collections::HashMap<String, DocArgRelationships>,
) -> TokenStream {
    let fn_name = &input_fn.sig.ident;
    let wrapper_name = quote::format_ident!("__{}_wrapper", fn_name);
    let init_fn_name = quote::format_ident!("__init_{}", fn_name);

    // Analyze function signature for arguments
    let mut arg_extractions = Vec::new();
    let mut arg_calls = Vec::new();

    for input in &input_fn.sig.inputs {
        if let syn::FnArg::Typed(pat_type) = input {
            let arg_name = match &*pat_type.pat {
                syn::Pat::Ident(ident) => &ident.ident,
                _ => continue,
            };

            let arg_name_str = arg_name.to_string();

            // Determine if optional (Option<T>) or required
            let is_option = is_option_type(&pat_type.ty);
            let inner_type = extract_inner_type(&pat_type.ty);
            let is_flag = is_bool_type(&pat_type.ty);
            let is_usize_type = if let syn::Type::Path(type_path) = &*pat_type.ty {
                type_path.path.segments.last().map(|s| s.ident == "usize").unwrap_or(false)
            } else {
                false
            };
            let is_vec = is_vec_type(&pat_type.ty);
            let vec_inner_type =
                if is_vec { extract_inner_type(&pat_type.ty) } else { (*pat_type.ty).clone() };

            // Parse arg config to check for Count action
            let arg_config = parse_arg_attributes(&pat_type.attrs);
            let is_count_action = if let Some(config) = &arg_config {
                config.action.as_ref().map(|a| a == "count").unwrap_or(false)
                    || (is_usize_type && !is_option)
            } else {
                is_usize_type && !is_option
            };

            if is_count_action {
                // Count action - extract count from __handler_input
                arg_extractions.push(quote! {
                    let #arg_name: usize = __handler_input.args.get(#arg_name_str)
                        .and_then(|v| v.parse::<usize>().ok())
                        .unwrap_or(0);
                });
                arg_calls.push(quote! { #arg_name });
            } else if is_flag {
                // Boolean flags
                arg_extractions.push(quote! {
                    let #arg_name = __handler_input.opts.get(#arg_name_str)
                        .map(|v| v.parse::<bool>().unwrap_or(false))
                        .unwrap_or(false);
                });
                arg_calls.push(quote! { #arg_name });
            } else if is_vec {
                // Vec<T> types - extract from __handler_input.args as comma-separated string, then parse
                // The registry extracts multiple values and joins them
                arg_extractions.push(quote! {
                    let #arg_name: #pat_type.ty = if let Some(value_str) = __handler_input.args.get(#arg_name_str) {
                        // Parse comma-separated values
                        value_str.split(',')
                            .map(|s| s.trim().parse::<#vec_inner_type>())
                            .collect::<Result<Vec<_>, _>>()
                            .map_err(|_| ::clap_noun_verb::error::NounVerbError::argument_error(
                                format!("Invalid value for argument '{}'", #arg_name_str)
                            ))?
                    } else {
                        Vec::new()
                    };
                });
                arg_calls.push(quote! { #arg_name });
            } else if is_option {
                // Optional arguments
                arg_extractions.push(quote! {
                    let #arg_name = __handler_input.args.get(#arg_name_str)
                        .and_then(|v| v.parse::<#inner_type>().ok());
                });
                arg_calls.push(quote! { #arg_name });
            } else {
                // Required arguments
                arg_extractions.push(quote! {
                    let #arg_name = __handler_input.args.get(#arg_name_str)
                        .ok_or_else(|| ::clap_noun_verb::error::NounVerbError::missing_argument(#arg_name_str))?
                        .parse::<#inner_type>()
                        .map_err(|_| ::clap_noun_verb::error::NounVerbError::argument_error(
                            format!("Invalid value for argument '{}'", #arg_name_str)
                        ))?;
                });
                arg_calls.push(quote! { #arg_name });
            }
        }
    }

    // Generate argument metadata for registration
    let mut arg_metadata = Vec::new();
    for input in &input_fn.sig.inputs {
        if let syn::FnArg::Typed(pat_type) = input {
            let arg_name = match &*pat_type.pat {
                syn::Pat::Ident(ident) => ident.ident.to_string(),
                _ => continue,
            };

            let is_option = is_option_type(&pat_type.ty);
            let is_flag = is_bool_type(&pat_type.ty);

            // Check if it's usize (can be used with Count action for flags like -v, -vv, -vvv)
            let is_usize_type = if let syn::Type::Path(type_path) = &*pat_type.ty {
                type_path.path.segments.last().map(|s| s.ident == "usize").unwrap_or(false)
            } else {
                false
            };

            // Parse arg config to check for explicit Count action
            let arg_config = parse_arg_attributes(&pat_type.attrs);
            let has_count_action = if let Some(config) = &arg_config {
                config.action.as_ref().map(|a| a == "count").unwrap_or(false)
            } else {
                false
            };

            // usize type without Option is treated as a flag (Count action)
            let is_flag_type = is_flag || (is_usize_type && !is_option) || has_count_action;

            // Get inner type for validation (unwrap Option if needed)
            let inner_ty =
                if is_option { extract_inner_type(&pat_type.ty) } else { (*pat_type.ty).clone() };

            // Auto-infer validation from type
            let (mut min_val, mut max_val, mut min_len, mut max_len) =
                get_type_validation(&inner_ty);

            // Parse validation attributes from parameter (e.g., #[validate(min = 0, max = 100)])
            if let Some(validation) = parse_validation_attributes(&pat_type.attrs) {
                // Override type-inferred validation with explicit attributes
                if let Some(min) = validation.min_value {
                    min_val = Some(min);
                }
                if let Some(max) = validation.max_value {
                    max_val = Some(max);
                }
                if let Some(min) = validation.min_length {
                    min_len = Some(min);
                }
                if let Some(max) = validation.max_length {
                    max_len = Some(max);
                }
            }

            // Parse argument attributes (e.g., #[arg(short = 'v', default_value = "50")])
            // Note: arg_config already parsed above for is_flag_type check

            // Auto-detect multiple values from Vec<T> type
            let is_vec_type = is_vec_type(&inner_ty);
            let multiple_values =
                arg_config.as_ref().map(|c| c.multiple).unwrap_or(false) || is_vec_type;

            // Auto-infer action: usize type → Count (for flags like -v, -vv, -vvv),
            // bool flags → SetTrue (unless overridden)
            let inferred_action = if (is_usize_type && !is_option) || has_count_action {
                // usize type without Option is inferred as Count (for -v, -vv, -vvv patterns)
                Some("count".to_string())
            } else if is_flag && arg_config.as_ref().and_then(|c| c.action.as_ref()).is_none() {
                Some("set_true".to_string()) // Default for bool flags
            } else {
                None
            };

            let min_value_token = if let Some(min) = min_val {
                quote! { Some(#min.to_string()) }
            } else {
                quote! { None }
            };

            let max_value_token = if let Some(max) = max_val {
                quote! { Some(#max.to_string()) }
            } else {
                quote! { None }
            };

            let min_length_token = if let Some(min) = min_len {
                quote! { Some(#min) }
            } else {
                quote! { None }
            };

            let max_length_token = if let Some(max) = max_len {
                quote! { Some(#max) }
            } else {
                quote! { None }
            };

            // Get help text - priority: #[arg(help = "...")] > docstring > default
            // Note: For docstring, use the clean description (without relationship tags)
            let help_token = if let Some(config) = &arg_config {
                if let Some(ref explicit_help) = config.help {
                    quote! { Some(#explicit_help.to_string()) }
                } else if let Some(rel) = arg_relationships.get(&arg_name) {
                    let help = &rel.description;
                    if !help.is_empty() {
                        quote! { Some(#help.to_string()) }
                    } else {
                        quote! { None }
                    }
                } else {
                    quote! { None }
                }
            } else if let Some(rel) = arg_relationships.get(&arg_name) {
                let help = &rel.description;
                if !help.is_empty() {
                    quote! { Some(#help.to_string()) }
                } else {
                    quote! { None }
                }
            } else {
                quote! { None }
            };

            // Get long_help from #[arg(long_help = "...")]
            let long_help_token = if let Some(config) = &arg_config {
                if let Some(ref lh) = config.long_help {
                    quote! { Some(#lh.to_string()) }
                } else {
                    quote! { None }
                }
            } else {
                quote! { None }
            };

            // Generate tokens for argument attributes
            let short_token = if let Some(config) = &arg_config {
                if let Some(s) = config.short {
                    quote! { Some(#s) }
                } else {
                    quote! { None }
                }
            } else {
                quote! { None }
            };

            let default_value_token = if let Some(config) = &arg_config {
                if let Some(ref dv) = config.default_value {
                    quote! { Some(#dv.to_string()) }
                } else {
                    quote! { None }
                }
            } else {
                quote! { None }
            };

            let env_token = if let Some(config) = &arg_config {
                if let Some(ref e) = config.env {
                    quote! { Some(#e.to_string()) }
                } else {
                    quote! { None }
                }
            } else {
                quote! { None }
            };

            let value_name_token = if let Some(config) = &arg_config {
                if let Some(ref vn) = config.value_name {
                    quote! { Some(#vn.to_string()) }
                } else {
                    quote! { None }
                }
            } else {
                quote! { None }
            };

            let aliases_token = if let Some(config) = &arg_config {
                if !config.aliases.is_empty() {
                    let aliases_vec = &config.aliases;
                    quote! { vec![#(#aliases_vec.to_string()),*] }
                } else {
                    quote! { vec![] }
                }
            } else {
                quote! { vec![] }
            };

            let positional_token = if let Some(config) = &arg_config {
                if let Some(pos) = config.positional {
                    quote! { Some(#pos) }
                } else {
                    quote! { None }
                }
            } else {
                quote! { None }
            };

            // Generate action token - use explicit action if provided, otherwise use inferred
            let action_token = if let Some(config) = &arg_config {
                if let Some(ref act) = config.action {
                    // Parse action string to ArgAction
                    match act.as_str() {
                        "count" => quote! { Some(::clap::ArgAction::Count) },
                        "set" => quote! { Some(::clap::ArgAction::Set) },
                        "set_false" => quote! { Some(::clap::ArgAction::SetFalse) },
                        "set_true" => quote! { Some(::clap::ArgAction::SetTrue) },
                        "append" => quote! { Some(::clap::ArgAction::Append) },
                        _ => quote! { None },
                    }
                } else if let Some(ref inferred) = inferred_action {
                    match inferred.as_str() {
                        "count" => quote! { Some(::clap::ArgAction::Count) },
                        "set_true" => quote! { Some(::clap::ArgAction::SetTrue) },
                        _ => quote! { None },
                    }
                } else {
                    quote! { None }
                }
            } else if let Some(ref inferred) = inferred_action {
                match inferred.as_str() {
                    "count" => quote! { Some(::clap::ArgAction::Count) },
                    "set_true" => quote! { Some(::clap::ArgAction::SetTrue) },
                    _ => quote! { None },
                }
            } else {
                quote! { None }
            };

            // Get group from doc comment or #[arg] attribute
            // Priority: #[arg(group = "...")] > doc comment [group: ...]
            let doc_rel = arg_relationships.get(&arg_name);
            let group_token = if let Some(config) = &arg_config {
                if let Some(ref g) = config.group {
                    quote! { Some(#g.to_string()) }
                } else if let Some(rel) = doc_rel {
                    if let Some(ref g) = rel.group {
                        quote! { Some(#g.to_string()) }
                    } else {
                        quote! { None }
                    }
                } else {
                    quote! { None }
                }
            } else if let Some(rel) = doc_rel {
                if let Some(ref g) = rel.group {
                    quote! { Some(#g.to_string()) }
                } else {
                    quote! { None }
                }
            } else {
                quote! { None }
            };

            // Get requires from doc comment or #[arg] attribute
            // Merges both sources if present
            let requires_token = {
                let mut requires_all = Vec::new();

                // Add from #[arg(requires = "...")]
                if let Some(config) = &arg_config {
                    requires_all.extend(config.requires.iter().cloned());
                }

                // Add from doc comment [requires: ...]
                if let Some(rel) = doc_rel {
                    for req in &rel.requires {
                        if !requires_all.contains(req) {
                            requires_all.push(req.clone());
                        }
                    }
                }

                if requires_all.is_empty() {
                    quote! { vec![] }
                } else {
                    quote! { vec![#(#requires_all.to_string()),*] }
                }
            };

            // Get conflicts_with from doc comment or #[arg] attribute
            // Merges both sources if present
            let conflicts_with_token = {
                let mut conflicts_all = Vec::new();

                // Add from #[arg(conflicts_with = "...")]
                if let Some(config) = &arg_config {
                    conflicts_all.extend(config.conflicts_with.iter().cloned());
                }

                // Add from doc comment [conflicts: ...]
                if let Some(rel) = doc_rel {
                    for conf in &rel.conflicts_with {
                        if !conflicts_all.contains(conf) {
                            conflicts_all.push(conf.clone());
                        }
                    }
                }

                if conflicts_all.is_empty() {
                    quote! { vec![] }
                } else {
                    quote! { vec![#(#conflicts_all.to_string()),*] }
                }
            };

            // Handle value_parser
            // Extract TokenStream string representation if explicit value_parser was specified
            let value_parser_token = if let Some(config) = &arg_config {
                if let Some(ref vp_ts) = config.value_parser {
                    // Explicit value_parser specified - extract string representation
                    // The TokenStream contains a string literal with the expression
                    let ts_str = vp_ts.to_string();
                    // Extract the actual expression string from the literal
                    let expr_str = if ts_str.starts_with('"') && ts_str.ends_with('"') {
                        ts_str.trim_matches('"').to_string()
                    } else {
                        ts_str
                    };
                    quote! { Some(#expr_str.to_string()) }
                } else {
                    // Try auto-inference
                    let inferred_parser = infer_type_parser(&inner_ty);
                    if let Some(ref parser) = inferred_parser {
                        quote! { Some(#parser.to_string()) }
                    } else {
                        quote! { None }
                    }
                }
            } else {
                // Try auto-inference
                let inferred_parser = infer_type_parser(&inner_ty);
                if let Some(ref parser) = inferred_parser {
                    quote! { Some(#parser.to_string()) }
                } else {
                    quote! { None }
                }
            };

            let next_line_help_token = if let Some(config) = &arg_config {
                let value = config.next_line_help;
                quote! { #value }
            } else {
                quote! { false }
            };

            let display_order_token = if let Some(config) = &arg_config {
                if let Some(order) = config.display_order {
                    quote! { Some(#order) }
                } else {
                    quote! { None }
                }
            } else {
                quote! { None }
            };

            let exclusive_token = if let Some(config) = &arg_config {
                if let Some(excl) = config.exclusive {
                    quote! { Some(#excl) }
                } else {
                    quote! { None }
                }
            } else {
                quote! { None }
            };

            let trailing_vararg_token = if let Some(config) = &arg_config {
                let value = config.trailing_vararg;
                quote! { #value }
            } else {
                quote! { false }
            };

            let allow_negative_numbers_token = if let Some(config) = &arg_config {
                let value = config.allow_negative_numbers;
                quote! { #value }
            } else {
                quote! { false }
            };

            // New tokens for extended doc comment tags
            // Priority: #[arg] attribute > doc comment tag

            // Get env from doc comment (already have env_token from #[arg])
            let doc_env_token = if let Some(rel) = doc_rel {
                if let Some(ref env_var) = rel.env {
                    quote! { Some(#env_var.to_string()) }
                } else {
                    quote! { None }
                }
            } else {
                quote! { None }
            };
            // Merge: prefer #[arg(env)] if present, else use doc comment
            let final_env_token = if let Some(config) = &arg_config {
                if config.env.is_some() {
                    env_token.clone()
                } else {
                    doc_env_token
                }
            } else {
                doc_env_token
            };

            // Get hide from doc comment
            let hide_token = if let Some(rel) = doc_rel {
                let hide_val = rel.hide;
                quote! { #hide_val }
            } else {
                quote! { false }
            };

            // Get default_value from doc comment (already have default_value_token from #[arg])
            let doc_default_token = if let Some(rel) = doc_rel {
                if let Some(ref dv) = rel.default_value {
                    quote! { Some(#dv.to_string()) }
                } else {
                    quote! { None }
                }
            } else {
                quote! { None }
            };
            // Merge: prefer #[arg(default_value)] if present, else use doc comment
            let final_default_token = if let Some(config) = &arg_config {
                if config.default_value.is_some() {
                    default_value_token.clone()
                } else {
                    doc_default_token
                }
            } else {
                doc_default_token
            };

            // Get value_hint from doc comment
            let value_hint_token = if let Some(rel) = doc_rel {
                if let Some(ref vh) = rel.value_hint {
                    quote! { Some(#vh.to_string()) }
                } else {
                    quote! { None }
                }
            } else {
                quote! { None }
            };

            // Get global from doc comment
            let global_token = if let Some(rel) = doc_rel {
                let global_val = rel.global;
                quote! { #global_val }
            } else {
                quote! { false }
            };

            // Get exclusive from doc comment (merge with #[arg(exclusive)])
            let doc_exclusive_token = if let Some(rel) = doc_rel {
                if rel.exclusive {
                    quote! { Some(true) }
                } else {
                    quote! { None }
                }
            } else {
                quote! { None }
            };
            let final_exclusive_token = if let Some(config) = &arg_config {
                if config.exclusive.is_some() {
                    exclusive_token.clone()
                } else {
                    doc_exclusive_token
                }
            } else {
                doc_exclusive_token
            };

            // Get help_heading from doc comment
            let help_heading_token = if let Some(rel) = doc_rel {
                if let Some(ref hh) = rel.help_heading {
                    quote! { Some(#hh.to_string()) }
                } else {
                    quote! { None }
                }
            } else {
                quote! { None }
            };

            arg_metadata.push(quote! {
                ::clap_noun_verb::cli::registry::ArgMetadata {
                    name: #arg_name.to_string(),
                    required: !#is_option,
                    is_flag: #is_flag_type,
                    help: #help_token,
                    min_value: #min_value_token,
                    max_value: #max_value_token,
                    min_length: #min_length_token,
                    max_length: #max_length_token,
                    short: #short_token,
                    default_value: #final_default_token,
                    env: #final_env_token,
                    multiple: #multiple_values,
                    value_name: #value_name_token,
                    aliases: #aliases_token,
                    positional: #positional_token,
                    action: #action_token,
                    group: #group_token,
                    requires: #requires_token,
                    conflicts_with: #conflicts_with_token,
                    value_parser: #value_parser_token,
                    hide: #hide_token,
                    next_help_heading: #help_heading_token,
                    long_help: #long_help_token,
                    next_line_help: #next_line_help_token,
                    display_order: #display_order_token,
                    exclusive: #final_exclusive_token,
                    trailing_vararg: #trailing_vararg_token,
                    allow_negative_numbers: #allow_negative_numbers_token,
                    value_hint: #value_hint_token,
                    global: #global_token,
                }
            });
        }
    }

    // GAP 2: Generate duplicate verb detection
    let noun_name_for_check = noun_name.as_deref().unwrap_or("__auto__");
    let duplicate_check =
        validation::generate_duplicate_detection(&verb_name, noun_name_for_check, fn_name);

    // Generate telemetry span instrumentation
    let telemetry_instrumentation = telemetry_validation::generate_verb_instrumentation(
        &verb_name,
        noun_name_for_check,
        fn_name,
    );

    // Generate wrapper function
    // Empty string "" means root-level verb (no noun)
    // "__auto__" means auto-infer from filename
    let noun_name_str = noun_name.as_deref().unwrap_or("__auto__");
    let about_str = about.as_deref().unwrap_or("");

    // Remove #[noun] attribute from output (it's been processed)
    // Keep #[arg] attributes - they're handled by the #[arg] proc macro (pass-through)
    // The #[verb] macro parses them from pat_type.attrs before output
    let mut output_fn = input_fn.clone();
    output_fn.attrs.retain(|attr| {
        let is_noun = attr.path().is_ident("noun")
            || attr.path().segments.last().map(|seg| seg.ident == "noun").unwrap_or(false);
        !is_noun
    });

    // Keep #[arg] attributes on parameters - the #[arg] proc macro will handle them
    // and the #[verb] macro has already parsed them for metadata generation

    let expanded = quote! {
        // Telemetry span declaration for this verb (only when autonomic feature enabled)
        #[cfg(feature = "autonomic")]
        #telemetry_instrumentation

        #output_fn

        // GAP 2: Compile-time duplicate verb detection
        #duplicate_check

        // Wrapper function that adapts HandlerInput to function signature
        // NOTE: Use __handler_input to avoid shadowing if user has an arg named "input"
        fn #wrapper_name(__handler_input: ::clap_noun_verb::logic::HandlerInput) -> ::clap_noun_verb::error::Result<::clap_noun_verb::logic::HandlerOutput> {
            // Telemetry: Start span for this verb execution (only when autonomic feature enabled)
            #[cfg(feature = "autonomic")]
            let mut _verb_span = ::clap_noun_verb::autonomic::telemetry::TraceSpan::new_root(
                concat!(#noun_name_str, ".", #verb_name)
            );
            #[cfg(feature = "autonomic")]
            {
                _verb_span.set_attribute("noun", #noun_name_str);
                _verb_span.set_attribute("verb", #verb_name);
            }

            // Execute handler with argument extraction
            #(#arg_extractions)*
            let result = #fn_name(#(#arg_calls),*)?;

            // Finish span and record duration (only when autonomic feature enabled)
            #[cfg(feature = "autonomic")]
            _verb_span.finish();

            ::clap_noun_verb::logic::HandlerOutput::from_data(result)
        }

        // Auto-generated registration
        // CRITICAL FIX: Use named function instead of closure to satisfy fn() type requirement
        #[allow(non_upper_case_globals)]
        #[linkme::distributed_slice(::clap_noun_verb::cli::registry::__VERB_REGISTRY)]
        static #init_fn_name: fn() = {
            fn __register_impl() {
                // Core team approach: Auto-infer noun name from filename if not explicitly provided
                // Special case: "root" means root-level verb (no noun)
                let (noun_name_static, noun_about_static, verb_name_final) = if #noun_name_str == "root" {
                    // Root-level verb: no noun, verb appears directly under CLI binary
                    // Pass empty string to registry to signal root verb
                    ("", "", #verb_name)
                } else if #noun_name_str == "__auto__" {
                    // Extract noun name from filename using file!() macro
                    let file_path = file!();
                    let inferred_name = ::std::path::Path::new(file_path)
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown")
                        .to_string();

                    // If verb name was auto-inferred, strip noun name from verb name if it appears
                    // Example: show_collector_status() -> verb_name="collector_status", noun="collector" -> verb="status"
                    let mut final_verb_name = #verb_name.to_string();
                    if final_verb_name.starts_with(&inferred_name) && final_verb_name.len() > inferred_name.len() {
                        if final_verb_name.as_bytes()[inferred_name.len()] == b'_' {
                            // Strip noun_ prefix (e.g., "collector_status" -> "status")
                            final_verb_name = final_verb_name[inferred_name.len() + 1..].to_string();
                        }
                    }

                    // Extract module doc from function doc as fallback
                    // Note: Full module doc extraction requires parsing the entire file,
                    // which is complex in proc macros. For now, we use function doc as a fallback.
                    // Users can add module doc (`//!`) at the top of the file, but we can't easily extract it.
                    let noun_about = if !#about_str.is_empty() {
                        #about_str.to_string()
                    } else {
                        String::new()
                    };

                    // Leak strings to get static lifetime for registration (acceptable for CLI construction)
                    let name_static: &'static str = Box::leak(inferred_name.into_boxed_str());
                    let about_static: &'static str = Box::leak(noun_about.into_boxed_str());
                    let verb_static: &'static str = Box::leak(final_verb_name.into_boxed_str());

                    // Auto-register noun with inferred name and doc
                    ::clap_noun_verb::cli::registry::CommandRegistry::register_noun(
                        name_static,
                        about_static,
                    );

                    (name_static, about_static, verb_static)
                } else {
                    // Leak explicit noun name and about to get static lifetime
                    let name_static: &'static str = Box::leak(#noun_name_str.to_string().into_boxed_str());
                    let about_static: &'static str = Box::leak(String::new().into_boxed_str());
                    let verb_static: &'static str = #verb_name;

                    // BUGFIX: Auto-register noun even with explicit noun name
                    // This ensures the noun exists even if no #[noun] attribute was used
                    ::clap_noun_verb::cli::registry::CommandRegistry::register_noun(
                        name_static,
                        about_static,
                    );

                    (name_static, about_static, verb_static)
                };

                let args = vec![#(#arg_metadata),*];
                ::clap_noun_verb::cli::registry::CommandRegistry::register_verb_with_args::<_>(
                    noun_name_static,
                    verb_name_final,
                    #about_str,
                    args,
                    #wrapper_name,
                );
            }
            __register_impl  // Return function pointer (not a call!)
        };
    };

    expanded.into()
}

/// Check if type is Option<T>
fn is_option_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            segment.ident == "Option"
        } else {
            false
        }
    } else {
        false
    }
}

/// Check if type is bool
fn is_bool_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            segment.ident == "bool"
        } else {
            false
        }
    } else {
        false
    }
}

/// Check if type is Vec<T>
fn is_vec_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            segment.ident == "Vec"
        } else {
            false
        }
    } else {
        false
    }
}

/// Validation constraints parsed from attributes
struct ValidationConstraints {
    min_value: Option<String>,
    max_value: Option<String>,
    min_length: Option<usize>,
    max_length: Option<usize>,
}

/// Argument configuration parsed from #[arg(...)] attributes
struct ArgConfig {
    short: Option<char>,
    default_value: Option<String>,
    env: Option<String>,
    multiple: bool,
    value_name: Option<String>,
    aliases: Vec<String>,
    positional: Option<usize>,
    action: Option<String>, // Store as string: "count", "set", "set_false", "set_true", "append"
    group: Option<String>,
    requires: Vec<String>,
    conflicts_with: Vec<String>,
    value_parser: Option<proc_macro2::TokenStream>, // Store TokenStream for compile-time expansion
    help: Option<String>,                           // Override docstring help
    long_help: Option<String>,                      // Long help text
    next_line_help: bool,                           // Next line help formatting
    display_order: Option<usize>,                   // Display order in help
    exclusive: Option<bool>,                        // Exclusive group flag
    trailing_vararg: bool,                          // Trailing varargs support
    allow_negative_numbers: bool,                   // Allow negative numbers
}

/// Parse argument attributes from parameter attributes
///
/// Parses `#[arg(short = 'v', default_value = "50", env = "PORT", multiple, value_name = "FILE")]` attributes
fn parse_arg_attributes(attrs: &[syn::Attribute]) -> Option<ArgConfig> {
    for attr in attrs {
        if attr.path().is_ident("arg") {
            if let syn::Meta::List(list) = &attr.meta {
                // Parse tokens manually to handle both flags (just names) and key-value pairs
                let mut config = ArgConfig {
                    short: None,
                    default_value: None,
                    env: None,
                    multiple: false,
                    value_name: None,
                    aliases: Vec::new(),
                    positional: None,
                    action: None,
                    group: None,
                    requires: Vec::new(),
                    conflicts_with: Vec::new(),
                    value_parser: None,
                    help: None,
                    long_help: None,
                    next_line_help: false,
                    display_order: None,
                    exclusive: None,
                    trailing_vararg: false,
                    allow_negative_numbers: false,
                };

                // Try parsing as MetaList first (handles key=value pairs)
                let parser =
                    syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated;
                if let Ok(meta_list) = parser.parse2(list.tokens.clone()) {
                    for meta in meta_list {
                        match &meta {
                            syn::Meta::NameValue(nv) => {
                                let ident = nv.path.get_ident()?.to_string();
                                match ident.as_str() {
                                    "short" => {
                                        // Parse short = 'v' or short = "v"
                                        if let syn::Expr::Lit(syn::ExprLit {
                                            lit: syn::Lit::Char(c),
                                            ..
                                        }) = &nv.value
                                        {
                                            config.short = Some(c.value());
                                        } else if let syn::Expr::Lit(syn::ExprLit {
                                            lit: syn::Lit::Str(s),
                                            ..
                                        }) = &nv.value
                                        {
                                            let s_val = s.value();
                                            if s_val.len() == 1 {
                                                config.short = s_val.chars().next();
                                            }
                                        }
                                    }
                                    "default_value" => {
                                        if let syn::Expr::Lit(syn::ExprLit {
                                            lit: syn::Lit::Str(s),
                                            ..
                                        }) = &nv.value
                                        {
                                            config.default_value = Some(s.value());
                                        }
                                    }
                                    "env" => {
                                        if let syn::Expr::Lit(syn::ExprLit {
                                            lit: syn::Lit::Str(s),
                                            ..
                                        }) = &nv.value
                                        {
                                            config.env = Some(s.value());
                                        }
                                    }
                                    "value_name" => {
                                        if let syn::Expr::Lit(syn::ExprLit {
                                            lit: syn::Lit::Str(s),
                                            ..
                                        }) = &nv.value
                                        {
                                            config.value_name = Some(s.value());
                                        }
                                    }
                                    "aliases" => {
                                        // Parse aliases = ["verbose", "v"]
                                        if let syn::Expr::Array(arr) = &nv.value {
                                            for expr in &arr.elems {
                                                if let syn::Expr::Lit(syn::ExprLit {
                                                    lit: syn::Lit::Str(s),
                                                    ..
                                                }) = expr
                                                {
                                                    config.aliases.push(s.value());
                                                }
                                            }
                                        }
                                    }
                                    "alias" => {
                                        // Parse alias = "verbose" (single alias)
                                        if let syn::Expr::Lit(syn::ExprLit {
                                            lit: syn::Lit::Str(s),
                                            ..
                                        }) = &nv.value
                                        {
                                            config.aliases.push(s.value());
                                        }
                                    }
                                    "index" => {
                                        // Parse index = 0 (positional argument index)
                                        if let syn::Expr::Lit(syn::ExprLit {
                                            lit: syn::Lit::Int(i),
                                            ..
                                        }) = &nv.value
                                        {
                                            if let Ok(index) = i.base10_parse::<usize>() {
                                                config.positional = Some(index);
                                            }
                                        }
                                    }
                                    "action" => {
                                        // Parse action = "count", action = "set_false", etc.
                                        if let syn::Expr::Lit(syn::ExprLit {
                                            lit: syn::Lit::Str(s),
                                            ..
                                        }) = &nv.value
                                        {
                                            config.action = Some(s.value());
                                        }
                                    }
                                    "group" => {
                                        // Parse group = "filter"
                                        if let syn::Expr::Lit(syn::ExprLit {
                                            lit: syn::Lit::Str(s),
                                            ..
                                        }) = &nv.value
                                        {
                                            config.group = Some(s.value());
                                        }
                                    }
                                    "requires" => {
                                        // Parse requires = ["arg1", "arg2"] or requires = "arg1"
                                        if let syn::Expr::Array(arr) = &nv.value {
                                            for expr in &arr.elems {
                                                if let syn::Expr::Lit(syn::ExprLit {
                                                    lit: syn::Lit::Str(s),
                                                    ..
                                                }) = expr
                                                {
                                                    config.requires.push(s.value());
                                                }
                                            }
                                        } else if let syn::Expr::Lit(syn::ExprLit {
                                            lit: syn::Lit::Str(s),
                                            ..
                                        }) = &nv.value
                                        {
                                            config.requires.push(s.value());
                                        }
                                    }
                                    "conflicts_with" => {
                                        // Parse conflicts_with = ["arg1", "arg2"] or conflicts_with = "arg1"
                                        if let syn::Expr::Array(arr) = &nv.value {
                                            for expr in &arr.elems {
                                                if let syn::Expr::Lit(syn::ExprLit {
                                                    lit: syn::Lit::Str(s),
                                                    ..
                                                }) = expr
                                                {
                                                    config.conflicts_with.push(s.value());
                                                }
                                            }
                                        } else if let syn::Expr::Lit(syn::ExprLit {
                                            lit: syn::Lit::Str(s),
                                            ..
                                        }) = &nv.value
                                        {
                                            config.conflicts_with.push(s.value());
                                        }
                                    }
                                    "value_parser" => {
                                        // Parse value_parser = ...
                                        // Workaround: Convert TokenStream to string and match common patterns
                                        // This allows us to support common expressions like:
                                        // - clap::value_parser!(u16).range(1..=65535)
                                        // - clap::value_parser!(u32).range(1..)
                                        // - clap::value_parser!(PathBuf)
                                        // etc.
                                        // Convert syn::Expr to TokenStream, then to string
                                        let ts = quote::quote! { #nv.value };
                                        let ts_string = ts.to_string();
                                        config.value_parser =
                                            Some(proc_macro2::TokenStream::from_iter(
                                                std::iter::once(proc_macro2::TokenTree::Literal(
                                                    proc_macro2::Literal::string(&ts_string),
                                                )),
                                            ));
                                    }
                                    "help" => {
                                        // Parse help = "..." to override docstring
                                        if let syn::Expr::Lit(syn::ExprLit {
                                            lit: syn::Lit::Str(s),
                                            ..
                                        }) = &nv.value
                                        {
                                            config.help = Some(s.value());
                                        }
                                    }
                                    "long_help" => {
                                        // Parse long_help = "..." for detailed help
                                        if let syn::Expr::Lit(syn::ExprLit {
                                            lit: syn::Lit::Str(s),
                                            ..
                                        }) = &nv.value
                                        {
                                            config.long_help = Some(s.value());
                                        }
                                    }
                                    "display_order" => {
                                        // Parse display_order = N
                                        if let syn::Expr::Lit(syn::ExprLit {
                                            lit: syn::Lit::Int(i),
                                            ..
                                        }) = &nv.value
                                        {
                                            if let Ok(order) = i.base10_parse::<usize>() {
                                                config.display_order = Some(order);
                                            }
                                        }
                                    }
                                    "exclusive" => {
                                        // Parse exclusive = true/false
                                        if let syn::Expr::Lit(syn::ExprLit {
                                            lit: syn::Lit::Bool(b),
                                            ..
                                        }) = &nv.value
                                        {
                                            config.exclusive = Some(b.value);
                                        }
                                    }
                                    "trailing_vararg" => {
                                        // Parse trailing_vararg = true
                                        if let syn::Expr::Lit(syn::ExprLit {
                                            lit: syn::Lit::Bool(b),
                                            ..
                                        }) = &nv.value
                                        {
                                            config.trailing_vararg = b.value;
                                        }
                                    }
                                    "allow_negative_numbers" => {
                                        // Parse allow_negative_numbers = true
                                        if let syn::Expr::Lit(syn::ExprLit {
                                            lit: syn::Lit::Bool(b),
                                            ..
                                        }) = &nv.value
                                        {
                                            config.allow_negative_numbers = b.value;
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            syn::Meta::Path(path) => {
                                // Handle flag attributes like `multiple`, `next_line_help`, `trailing_vararg`
                                if let Some(ident) = path.get_ident() {
                                    match ident.to_string().as_str() {
                                        "multiple" => config.multiple = true,
                                        "next_line_help" => config.next_line_help = true,
                                        "trailing_vararg" => config.trailing_vararg = true,
                                        "allow_negative_numbers" => {
                                            config.allow_negative_numbers = true
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            _ => {}
                        }
                    }

                    return Some(config);
                }
            }
        }
    }
    None
}

/// Parse validation attributes from parameter attributes
///
/// Parses `#[validate(min = 0, max = 100, min_length = 1, max_length = 50)]` attributes
fn parse_validation_attributes(attrs: &[syn::Attribute]) -> Option<ValidationConstraints> {
    for attr in attrs {
        if attr.path().is_ident("validate") {
            if let syn::Meta::List(list) = &attr.meta {
                let parser = syn::punctuated::Punctuated::<syn::MetaNameValue, syn::Token![,]>::parse_terminated;
                if let Ok(meta_list) = parser.parse2(list.tokens.clone()) {
                    let mut constraints = ValidationConstraints {
                        min_value: None,
                        max_value: None,
                        min_length: None,
                        max_length: None,
                    };

                    for meta in meta_list {
                        let ident = meta.path.get_ident()?.to_string();
                        let value = match &meta.value {
                            syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Int(i), .. }) => {
                                if ident == "min" || ident == "min_value" {
                                    constraints.min_value = Some(i.base10_digits().to_string());
                                } else if ident == "max" || ident == "max_value" {
                                    constraints.max_value = Some(i.base10_digits().to_string());
                                } else if ident == "min_length" {
                                    if let Ok(v) = i.base10_parse::<usize>() {
                                        constraints.min_length = Some(v);
                                    }
                                } else if ident == "max_length" {
                                    if let Ok(v) = i.base10_parse::<usize>() {
                                        constraints.max_length = Some(v);
                                    }
                                }
                                None
                            }
                            syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(s), .. }) => {
                                if ident == "min"
                                    || ident == "min_value"
                                    || ident == "max"
                                    || ident == "max_value"
                                {
                                    Some(s.value())
                                } else {
                                    None
                                }
                            }
                            _ => None,
                        };

                        if let Some(val) = value {
                            if ident == "min" || ident == "min_value" {
                                constraints.min_value = Some(val);
                            } else if ident == "max" || ident == "max_value" {
                                constraints.max_value = Some(val);
                            }
                        }
                    }

                    return Some(constraints);
                }
            }
        }
    }
    None
}

/// Get auto-validation constraints for a type
///
/// Returns validation constraints that can be inferred from the type:
/// - `u32`, `u64`, `usize` = min_value = "0"
/// - `u8`, `u16` = min_value = "0", max_value inferred from type max
/// - `i32`, `i64`, `isize` = no auto validation (can be negative)
/// - `String` = no auto validation (but could add min_length/max_length later)
fn get_type_validation(
    ty: &syn::Type,
) -> (Option<String>, Option<String>, Option<usize>, Option<usize>) {
    if let syn::Type::Path(type_path) = ty {
        let type_name =
            type_path.path.segments.last().map(|s| s.ident.to_string()).unwrap_or_default();

        match type_name.as_str() {
            // Unsigned integers: min = 0
            "u8" => (Some("0".to_string()), Some("255".to_string()), None, None),
            "u16" => (Some("0".to_string()), Some("65535".to_string()), None, None),
            "u32" | "u64" | "usize" => (Some("0".to_string()), None, None, None),
            // Signed integers: no auto validation (can be negative)
            "i8" => (Some("-128".to_string()), Some("127".to_string()), None, None),
            "i16" => (Some("-32768".to_string()), Some("32767".to_string()), None, None),
            "i32" | "i64" | "isize" => (None, None, None, None),
            // String: no auto validation (can add min_length/max_length from attributes later)
            "String" => (None, None, None, None),
            _ => (None, None, None, None),
        }
    } else {
        (None, None, None, None)
    }
}

/// Extract inner type from Option<T>, Vec<T>, or return original
fn extract_inner_type(ty: &syn::Type) -> syn::Type {
    if let syn::Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            if segment.ident == "Option" || segment.ident == "Vec" {
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                        return inner_ty.clone();
                    }
                }
            }
        }
    }
    ty.clone()
}

/// Infer type parser for common types
///
/// Returns a string representation of the parser expression for auto-inferred types:
/// - `PathBuf` → `clap::value_parser!(PathBuf)`
/// - `IpAddr` → `clap::value_parser!(IpAddr)`
/// - `Ipv4Addr` → `clap::value_parser!(Ipv4Addr)`
/// - `Ipv6Addr` → `clap::value_parser!(Ipv6Addr)`
/// - `Url` → `clap::value_parser!(Url)` (if url feature available)
/// - Numeric types already handled by validation constraints
fn infer_type_parser(ty: &syn::Type) -> Option<String> {
    if let syn::Type::Path(type_path) = ty {
        let type_name =
            type_path.path.segments.last().map(|s| s.ident.to_string()).unwrap_or_default();

        match type_name.as_str() {
            "PathBuf" => Some("clap::value_parser!(::std::path::PathBuf)".to_string()),
            "IpAddr" => Some("clap::value_parser!(::std::net::IpAddr)".to_string()),
            "Ipv4Addr" => Some("clap::value_parser!(::std::net::Ipv4Addr)".to_string()),
            "Ipv6Addr" => Some("clap::value_parser!(::std::net::Ipv6Addr)".to_string()),
            // Url requires url crate - check if available at compile time
            // For now, we'll include it and let compilation fail if url feature isn't enabled
            "Url" => Some("clap::value_parser!(::url::Url)".to_string()),
            // Duration requires custom parser - defer to explicit specification
            _ => None,
        }
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_doc_relationships_group() {
        let desc = "Export as JSON [group: format]";
        let result = parse_doc_relationships(desc);
        assert_eq!(result.group, Some("format".to_string()));
        assert_eq!(result.description, "Export as JSON");
    }

    #[test]
    fn test_parse_doc_relationships_requires() {
        let desc = "Output filename [requires: format]";
        let result = parse_doc_relationships(desc);
        assert!(result.requires.contains(&"format".to_string()));
        assert_eq!(result.description, "Output filename");
    }

    #[test]
    fn test_parse_doc_relationships_conflicts() {
        let desc = "Output format [conflicts: raw]";
        let result = parse_doc_relationships(desc);
        assert!(result.conflicts_with.contains(&"raw".to_string()));
        assert_eq!(result.description, "Output format");
    }

    #[test]
    fn test_parse_argument_descriptions_with_relationships() {
        let docstring = r#"Test command

# Arguments
* `json` - Export as JSON [group: format]
* `yaml` - Export as YAML [group: format]
* `filename` - Output filename [requires: format]
"#;
        let result = parse_argument_descriptions_with_relationships(docstring);

        assert!(result.contains_key("json"), "Should contain 'json' key");
        assert!(result.contains_key("yaml"), "Should contain 'yaml' key");
        assert!(result.contains_key("filename"), "Should contain 'filename' key");

        let json_rel = result.get("json").unwrap();
        assert_eq!(json_rel.group, Some("format".to_string()), "json should have group='format'");

        let yaml_rel = result.get("yaml").unwrap();
        assert_eq!(yaml_rel.group, Some("format".to_string()), "yaml should have group='format'");

        let filename_rel = result.get("filename").unwrap();
        assert!(
            filename_rel.requires.contains(&"format".to_string()),
            "filename should require 'format'"
        );
    }
}
