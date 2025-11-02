//! Procedural macros for clap-noun-verb
//!
//! This crate provides attribute macros `#[noun]` and `#[verb]` for
//! declarative CLI command registration.

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse::Parser, parse_macro_input, ItemFn};

// Note: #[arg(...)] attributes on function parameters cannot be a real proc_macro_attribute
// because Rust doesn't allow proc_macro_attribute on parameters - only on items.
// The #[verb] macro parses #[arg(...)] attributes directly from pat_type.attrs.
// Users should use #[allow(unknown_attributes)] if they get compiler warnings.

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
        #[linkme::distributed_slice(::clap_noun_verb::cli::registry::__NOUN_REGISTRY)]
        static #init_fn_name: fn() = || {
            ::clap_noun_verb::cli::registry::CommandRegistry::register_noun(
                #name_str,
                #about_str,
            );
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
#[proc_macro_attribute]
pub fn verb(args: TokenStream, input: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(input as ItemFn);

    // Parse verb name from args
    let parser = syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated;
    let args_vec: syn::punctuated::Punctuated<_, _> =
        match Parser::parse2(parser, proc_macro2::TokenStream::from(args)) {
            Ok(args) => args,
            Err(_) => {
                // If parsing fails, extract verb name from function name
                let verb_name = extract_verb_name_from_fn_name(&input_fn);
                let docstring = extract_docstring(&input_fn);
                let arg_descriptions = parse_argument_descriptions(&docstring);
                return generate_verb_registration(
                    input_fn,
                    verb_name,
                    None,
                    None,
                    arg_descriptions,
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

    // Parse argument descriptions from docstring
    let arg_descriptions = parse_argument_descriptions(&docstring);

    generate_verb_registration(input_fn, verb_name, noun_name, Some(docstring), arg_descriptions)
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
        .join(" ")
        .trim()
        .to_string()
}

/// Parse argument descriptions from docstring
///
/// Extracts individual argument descriptions from a `# Arguments` section
/// in the docstring. Format:
/// ```rust
/// /// # Arguments
/// /// * `arg_name` - Description of argument
/// /// * `other_arg` - Another description
/// ```
///
/// Returns a HashMap mapping argument names to their descriptions
fn parse_argument_descriptions(docstring: &str) -> std::collections::HashMap<String, String> {
    let mut descriptions = std::collections::HashMap::new();

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
        // Format: `* `arg_name` - description` or `* arg_name - description`
        if in_arguments_section && trimmed.starts_with('*') {
            let rest = trimmed[1..].trim();

            // Extract argument name and description
            // Support formats:
            // - `* `arg_name` - description`
            // - `* arg_name - description`
            // - `* \`arg_name\` - description`
            if let Some(dash_pos) = rest.find('-') {
                let before_dash = rest[..dash_pos].trim();
                let description = rest[dash_pos + 1..].trim().to_string();

                // Extract argument name (remove backticks and asterisks)
                let arg_name =
                    before_dash.trim_start_matches('*').trim().trim_matches('`').trim().to_string();

                if !arg_name.is_empty() && !description.is_empty() {
                    descriptions.insert(arg_name, description);
                }
            }
        }
    }

    descriptions
}

/// Generate verb registration code with full type inference
fn generate_verb_registration(
    input_fn: ItemFn,
    verb_name: String,
    noun_name: Option<String>,
    about: Option<String>,
    arg_descriptions: std::collections::HashMap<String, String>,
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
            let is_vec = is_vec_type(&pat_type.ty);
            let vec_inner_type = if is_vec { extract_inner_type(&pat_type.ty) } else { (*pat_type.ty).clone() };

            if is_flag {
                // Boolean flags
                arg_extractions.push(quote! {
                    let #arg_name = input.opts.get(#arg_name_str)
                        .map(|v| v.parse::<bool>().unwrap_or(false))
                        .unwrap_or(false);
                });
                arg_calls.push(quote! { #arg_name });
            } else if is_vec {
                // Vec<T> types - extract from input.args as comma-separated string, then parse
                // The registry extracts multiple values and joins them
                arg_extractions.push(quote! {
                    let #arg_name: #pat_type.ty = if let Some(value_str) = input.args.get(#arg_name_str) {
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
                    let #arg_name = input.args.get(#arg_name_str)
                        .and_then(|v| v.parse::<#inner_type>().ok());
                });
                arg_calls.push(quote! { #arg_name });
            } else {
                // Required arguments
                arg_extractions.push(quote! {
                    let #arg_name = input.args.get(#arg_name_str)
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
            let arg_config = parse_arg_attributes(&pat_type.attrs);

            // Auto-detect multiple values from Vec<T> type
            let is_vec_type = is_vec_type(&inner_ty);
            let multiple_values = arg_config.as_ref().map(|c| c.multiple).unwrap_or(false) || is_vec_type;

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

            // Get help text from docstring if available
            let help_text = arg_descriptions.get(&arg_name);
            let help_token = if let Some(help) = help_text {
                quote! { Some(#help.to_string()) }
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

            arg_metadata.push(quote! {
                ::clap_noun_verb::cli::registry::ArgMetadata {
                    name: #arg_name.to_string(),
                    required: !#is_option,
                    is_flag: #is_flag,
                    help: #help_token,
                    min_value: #min_value_token,
                    max_value: #max_value_token,
                    min_length: #min_length_token,
                    max_length: #max_length_token,
                    short: #short_token,
                    default_value: #default_value_token,
                    env: #env_token,
                    multiple: #multiple_values,
                    value_name: #value_name_token,
                    aliases: #aliases_token,
                }
            });
        }
    }

    // Generate wrapper function
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
        #output_fn

        // Wrapper function that adapts HandlerInput to function signature
        fn #wrapper_name(input: ::clap_noun_verb::logic::HandlerInput) -> ::clap_noun_verb::error::Result<::clap_noun_verb::logic::HandlerOutput> {
            #(#arg_extractions)*
            let result = #fn_name(#(#arg_calls),*)?;
            ::clap_noun_verb::logic::HandlerOutput::from_data(result)
        }

        // Auto-generated registration
        #[linkme::distributed_slice(::clap_noun_verb::cli::registry::__VERB_REGISTRY)]
        static #init_fn_name: fn() = || {
            // Core team approach: Auto-infer noun name from filename if not explicitly provided
            let (noun_name_static, noun_about_static, verb_name_final) = if #noun_name_str == "__auto__" {
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
                                    _ => {}
                                }
                            }
                            syn::Meta::Path(path) => {
                                // Handle flag attributes like `multiple`
                                if let Some(ident) = path.get_ident() {
                                    match ident.to_string().as_str() {
                                        "multiple" => config.multiple = true,
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
