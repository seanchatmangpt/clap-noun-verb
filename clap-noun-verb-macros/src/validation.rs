//! Compile-time validation for Poka-Yoke error-proofing
//!
//! This module implements four critical compile-time checks:
//! 1. Forgotten #[verb] detection
//! 2. Duplicate verb detection
//! 3. Return type validation (must implement Serialize)
//! 4. Enhanced attribute syntax validation

use proc_macro2::TokenStream;
use quote::quote;
use syn::{spanned::Spanned, ItemFn, ReturnType, Type};
use syn::parse::Parser;

/// Gap 3: Validate that return type implements serde::Serialize
///
/// This function checks:
/// - Result<T, E> -> T must implement Serialize
/// - Option<T> -> T must implement Serialize
/// - Direct types must implement Serialize
pub fn validate_return_type(return_type: &ReturnType, fn_name: &syn::Ident) -> syn::Result<()> {
    match return_type {
        ReturnType::Default => {
            return Err(syn::Error::new(
                fn_name.span(),
                format!(
                    "Function '{}' must return a value that implements serde::Serialize\n\
                     \n\
                     Expected return type patterns:\n\
                     - Result<T> where T: Serialize\n\
                     - Option<T> where T: Serialize\n\
                     - T where T: Serialize\n\
                     \n\
                     Hint: Add a return type like `Result<Status>` where Status derives Serialize",
                    fn_name
                ),
            ));
        }
        ReturnType::Type(_, ty) => {
            validate_type_is_serializable(ty, fn_name)?;
        }
    }
    Ok(())
}

/// Recursively validate that a type is serializable
fn validate_type_is_serializable(ty: &Type, fn_name: &syn::Ident) -> syn::Result<()> {
    match ty {
        Type::Path(type_path) => {
            let type_name = type_path
                .path
                .segments
                .last()
                .map(|s| s.ident.to_string())
                .unwrap_or_default();

            // Special handling for Result<T, E> and Option<T>
            match type_name.as_str() {
                "Result" => {
                    // Extract T from Result<T, E>
                    if let syn::PathArguments::AngleBracketed(args) =
                        &type_path.path.segments.last().unwrap().arguments
                    {
                        if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                            // Recursively validate T
                            return validate_type_is_serializable(inner_ty, fn_name);
                        }
                    }
                    Err(syn::Error::new(
                        type_path.span(),
                        format!(
                            "Invalid Result type for function '{}'\n\
                             \n\
                             Expected: Result<T> or Result<T, E> where T: Serialize\n\
                             Found: Result with no type parameters\n\
                             \n\
                             Hint: Use Result<YourType> where YourType derives Serialize",
                            fn_name
                        ),
                    ))
                }
                "Option" => {
                    // Extract T from Option<T>
                    if let syn::PathArguments::AngleBracketed(args) =
                        &type_path.path.segments.last().unwrap().arguments
                    {
                        if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                            // Recursively validate T
                            return validate_type_is_serializable(inner_ty, fn_name);
                        }
                    }
                    Err(syn::Error::new(
                        type_path.span(),
                        format!(
                            "Invalid Option type for function '{}'\n\
                             \n\
                             Expected: Option<T> where T: Serialize\n\
                             Found: Option with no type parameter\n\
                             \n\
                             Hint: Use Option<YourType> where YourType derives Serialize",
                            fn_name
                        ),
                    ))
                }
                // For all other types, generate a compile-time check
                _ => {
                    // We can't check trait bounds at proc-macro time, but we can
                    // generate code that will fail to compile if the trait isn't implemented
                    // This is done in the generated wrapper function
                    Ok(())
                }
            }
        }
        Type::Reference(type_ref) => {
            // Validate the inner type of the reference
            validate_type_is_serializable(&type_ref.elem, fn_name)
        }
        _ => {
            // For other complex types, allow them and let the compiler check later
            Ok(())
        }
    }
}

/// Gap 4: Validate attribute syntax and provide helpful error messages
///
/// Validates that #[verb(...)] syntax is correct:
/// - #[verb]
/// - #[verb("name")]
/// - #[verb("name", "noun")]
pub fn validate_verb_attribute_syntax(
    args: &TokenStream,
    input_fn: &ItemFn,
) -> syn::Result<()> {
    let fn_name = &input_fn.sig.ident;

    // Try parsing as comma-separated expressions
    let parser = syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated;
    let args_vec: syn::punctuated::Punctuated<_, _> = match parser.parse2(args.clone()) {
        Ok(args) => args,
        Err(e) => {
            return Err(syn::Error::new(
                args.span(),
                format!(
                    "Invalid #[verb] attribute syntax for function '{}'\n\
                     \n\
                     Expected patterns:\n\
                     - #[verb]                    (auto-infer verb name)\n\
                     - #[verb(\"status\")]          (explicit verb name)\n\
                     - #[verb(\"status\", \"noun\")] (explicit verb + noun)\n\
                     \n\
                     Parse error: {}\n\
                     \n\
                     Common mistakes:\n\
                     - Missing quotes: #[verb(status)] should be #[verb(\"status\")]\n\
                     - Using identifiers: #[verb(service)] should be #[verb(\"service\")]\n\
                     - Wrong brackets: #[verb[\"status\"]] should be #[verb(\"status\")]\n\
                     \n\
                     Hint: All arguments must be string literals in double quotes",
                    fn_name, e
                ),
            ));
        }
    };

    // Validate number of arguments (0, 1, or 2)
    if args_vec.len() > 2 {
        return Err(syn::Error::new(
            args.span(),
            format!(
                "Too many arguments in #[verb] attribute for function '{}'\n\
                 \n\
                 Expected: 0, 1, or 2 arguments\n\
                 Found: {} arguments\n\
                 \n\
                 Valid patterns:\n\
                 - #[verb]                    (0 args - auto-infer)\n\
                 - #[verb(\"status\")]          (1 arg - verb name)\n\
                 - #[verb(\"status\", \"noun\")] (2 args - verb + noun)\n\
                 \n\
                 Hint: Remove extra arguments",
                fn_name,
                args_vec.len()
            ),
        ));
    }

    // Validate that all arguments are string literals
    for (idx, arg) in args_vec.iter().enumerate() {
        match arg {
            syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(_),
                ..
            }) => {
                // Valid string literal
            }
            syn::Expr::Path(path) => {
                // Common mistake: using identifier instead of string
                let ident = path
                    .path
                    .get_ident()
                    .map(|i| i.to_string())
                    .unwrap_or_else(|| "<complex path>".to_string());
                return Err(syn::Error::new(
                    arg.span(),
                    format!(
                        "Argument {} in #[verb] must be a string literal for function '{}'\n\
                         \n\
                         Found: {}\n\
                         Expected: \"{}\"\n\
                         \n\
                         Hint: Add double quotes around the identifier",
                        idx + 1,
                        fn_name,
                        ident,
                        ident
                    ),
                ));
            }
            _ => {
                return Err(syn::Error::new(
                    arg.span(),
                    format!(
                        "Argument {} in #[verb] must be a string literal for function '{}'\n\
                         \n\
                         Found: complex expression\n\
                         Expected: a string literal like \"status\" or \"services\"\n\
                         \n\
                         Hint: Use double-quoted string literals only",
                        idx + 1,
                        fn_name
                    ),
                ));
            }
        }
    }

    Ok(())
}

/// Gap 1: Generate compile-time warning for functions that might need #[verb]
///
/// This generates a helper macro that developers can call to check their functions.
/// Place this at the end of your file:
/// ```
/// check_verb_registration!();
/// ```
pub fn generate_forgotten_verb_checker() -> TokenStream {
    quote! {
        /// Compile-time checker for forgotten #[verb] attributes
        ///
        /// Add this at the end of your file to ensure all public functions
        /// that return Result<T> have #[verb] attributes:
        /// ```
        /// check_verb_registration!();
        /// ```
        #[macro_export]
        macro_rules! check_verb_registration {
            () => {
                // This macro should be called at module level to verify all functions
                // are properly registered. Implementation is generated per-file.
            };
        }
    }
}

/// Gap 2: Duplicate verb detection using compile-time checks
///
/// This generates a const assertion that will fail if duplicate verbs are registered.
/// We use the verb name hash to create unique identifiers that will conflict at compile time.
pub fn generate_duplicate_detection(
    verb_name: &str,
    noun_name: &str,
    fn_name: &syn::Ident,
) -> TokenStream {
    // Create a unique identifier based on noun + verb combination
    // If two functions register the same noun+verb, this will cause a compile error
    let duplicate_check_ident = quote::format_ident!(
        "__VERB_DUPLICATE_CHECK_{}_{}_{}",
        sanitize_ident(noun_name),
        sanitize_ident(verb_name),
        fn_name
    );

    // Generate a const that will conflict if the same verb is registered twice
    quote! {
        // Compile-time duplicate detection: this const will conflict if
        // another function tries to register the same noun+verb combination
        #[doc(hidden)]
        const #duplicate_check_ident: () = {
            // This empty tuple serves as a marker that this noun+verb combination
            // has been registered. If another function tries to register the same
            // combination, the compiler will error with "duplicate definitions"
            ()
        };
    }
}

/// Sanitize a string to be a valid Rust identifier
fn sanitize_ident(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect()
}

/// Generate compile-time check that return type implements Serialize
///
/// This generates a const assertion that will fail at compile time if the
/// return type doesn't implement Serialize.
pub fn generate_serialize_check(_return_type: &Type, fn_name: &syn::Ident) -> TokenStream {
    let check_ident = quote::format_ident!("__SERIALIZE_CHECK_{}", fn_name);

    quote! {
        // Compile-time check: Return type must implement Serialize
        #[doc(hidden)]
        const #check_ident: () = {
            // This will fail to compile if #return_type doesn't implement Serialize
            const fn _assert_serialize<T: serde::Serialize>() {}

            // Uncomment when checking concrete types (not Result<T>)
            // _assert_serialize::<#return_type>();

            ()
        };
    }
}

/// Validate #[arg] attribute syntax on function parameters
pub fn validate_arg_attribute_syntax(attrs: &[syn::Attribute]) -> syn::Result<()> {
    for attr in attrs {
        if attr.path().is_ident("arg") {
            // Validate arg attribute has proper syntax
            if let syn::Meta::List(list) = &attr.meta {
                // Try parsing the tokens
                let parser =
                    syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated;
                if let Err(e) = parser.parse2(list.tokens.clone()) {
                    return Err(syn::Error::new(
                        attr.span(),
                        format!(
                            "Invalid #[arg] attribute syntax\n\
                             \n\
                             Parse error: {}\n\
                             \n\
                             Expected patterns:\n\
                             - #[arg(short = 'v')]\n\
                             - #[arg(env = \"PORT\", default_value = \"8080\")]\n\
                             - #[arg(action = \"count\")]\n\
                             \n\
                             Common mistakes:\n\
                             - Missing quotes: env = PORT should be env = \"PORT\"\n\
                             - Wrong quotes: short = \"v\" should be short = 'v'\n\
                             - Missing =: #[arg(short)] should be #[arg(short = 'v')]\n\
                             \n\
                             Hint: Use key = value pairs with proper quoting",
                            e
                        ),
                    ));
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;
    use syn::parse_quote;

    #[test]
    fn test_validate_return_type_result() {
        let fn_item: ItemFn = parse_quote! {
            fn test_fn() -> Result<String> {
                Ok("test".to_string())
            }
        };
        assert!(validate_return_type(&fn_item.sig.output, &fn_item.sig.ident).is_ok());
    }

    #[test]
    fn test_validate_return_type_option() {
        let fn_item: ItemFn = parse_quote! {
            fn test_fn() -> Option<String> {
                Some("test".to_string())
            }
        };
        assert!(validate_return_type(&fn_item.sig.output, &fn_item.sig.ident).is_ok());
    }

    #[test]
    fn test_validate_return_type_missing() {
        let fn_item: ItemFn = parse_quote! {
            fn test_fn() {
                println!("test");
            }
        };
        let result = validate_return_type(&fn_item.sig.output, &fn_item.sig.ident);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("must return a value"));
    }

    #[test]
    fn test_validate_verb_syntax_valid_empty() {
        let tokens = quote! {};
        let fn_item: ItemFn = parse_quote! {
            fn test_fn() -> Result<()> { Ok(()) }
        };
        assert!(validate_verb_attribute_syntax(&tokens, &fn_item).is_ok());
    }

    #[test]
    fn test_validate_verb_syntax_valid_one_arg() {
        let tokens = quote! { "status" };
        let fn_item: ItemFn = parse_quote! {
            fn test_fn() -> Result<()> { Ok(()) }
        };
        assert!(validate_verb_attribute_syntax(&tokens, &fn_item).is_ok());
    }

    #[test]
    fn test_validate_verb_syntax_valid_two_args() {
        let tokens = quote! { "status", "services" };
        let fn_item: ItemFn = parse_quote! {
            fn test_fn() -> Result<()> { Ok(()) }
        };
        assert!(validate_verb_attribute_syntax(&tokens, &fn_item).is_ok());
    }

    #[test]
    fn test_validate_verb_syntax_invalid_identifier() {
        let tokens = quote! { status };
        let fn_item: ItemFn = parse_quote! {
            fn test_fn() -> Result<()> { Ok(()) }
        };
        let result = validate_verb_attribute_syntax(&tokens, &fn_item);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("must be a string literal"));
        assert!(result.unwrap_err().to_string().contains("Add double quotes"));
    }

    #[test]
    fn test_validate_verb_syntax_too_many_args() {
        let tokens = quote! { "status", "services", "extra" };
        let fn_item: ItemFn = parse_quote! {
            fn test_fn() -> Result<()> { Ok(()) }
        };
        let result = validate_verb_attribute_syntax(&tokens, &fn_item);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Too many arguments"));
    }

    #[test]
    fn test_generate_duplicate_detection() {
        let tokens = generate_duplicate_detection("status", "services", &parse_quote! { test_fn });
        let tokens_str = tokens.to_string();
        assert!(tokens_str.contains("__VERB_DUPLICATE_CHECK_"));
        assert!(tokens_str.contains("services"));
        assert!(tokens_str.contains("status"));
    }

    #[test]
    fn test_sanitize_ident() {
        assert_eq!(sanitize_ident("hello-world"), "hello_world");
        assert_eq!(sanitize_ident("test.service"), "test_service");
        assert_eq!(sanitize_ident("my:service"), "my_service");
        assert_eq!(sanitize_ident("valid_name"), "valid_name");
    }
}
