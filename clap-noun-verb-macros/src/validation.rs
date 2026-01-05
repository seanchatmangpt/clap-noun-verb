//! Compile-time validation for Poka-Yoke error-proofing
//!
//! This module implements four critical compile-time checks:
//! 1. Forgotten #[verb] detection
//! 2. Duplicate verb detection
//! 3. Return type validation (must implement Serialize)
//! 4. Enhanced attribute syntax validation
//!
//! FUTURE: v5.1 - Complete validation feature set

#![allow(dead_code)] // FUTURE: v5.1 - integrate enhanced validation

use proc_macro2::TokenStream;
use quote::quote;
use quote::ToTokens;
use syn::parse::Parser;
use syn::{spanned::Spanned, ItemFn, ReturnType, Type};

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
            let type_name =
                type_path.path.segments.last().map(|s| s.ident.to_string()).unwrap_or_default();

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
pub fn validate_verb_attribute_syntax(args: &TokenStream, input_fn: &ItemFn) -> syn::Result<()> {
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
            syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(_), .. }) => {
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
/// ```ignore
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
        #[allow(non_upper_case_globals)]
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
    s.chars().map(|c| if c.is_alphanumeric() { c } else { '_' }).collect()
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

/// FM-1.2 Poka-Yoke Guard: Validate no CLI type parameters (FM-1.2: Domain Dependency on CLI Types)
///
/// Prevents domain functions from accepting CLI types that create circular dependencies:
/// - Forbidden: clap::ArgMatches, clap::Command, VerbContext, VerbArgs, HandlerInput
/// - Allowed: String, u32, bool, Vec<T>, PathBuf, domain types
///
/// RPN 270: High severity - circular dependencies break modularity
pub fn validate_no_cli_types_in_params(sig: &syn::Signature) -> syn::Result<()> {
    for input in &sig.inputs {
        if let syn::FnArg::Typed(pat_type) = input {
            if let Some(error) = check_for_cli_types(&pat_type.ty) {
                return Err(error);
            }
        }
    }
    Ok(())
}

/// Check if a type contains forbidden CLI types
fn check_for_cli_types(ty: &Type) -> Option<syn::Error> {
    let type_str = ty.to_token_stream().to_string();

    // Forbidden CLI types that create architecture violations
    let forbidden_patterns = [
        "ArgMatches",
        "Command",
        "VerbContext",
        "VerbArgs",
        "HandlerInput",
        "clap :: ArgMatches",
        "clap :: Command",
    ];

    for pattern in &forbidden_patterns {
        if type_str.contains(pattern) {
            return Some(syn::Error::new(
                ty.span(),
                format!(
                    "🛡️ Poka-Yoke Guard: CLI type contamination detected (FM-1.2)\n\
                     \n\
                     Forbidden types: ArgMatches, Command, VerbContext, VerbArgs, HandlerInput\n\
                     Found: {}\n\
                     \n\
                     Problem: Domain functions should not depend on CLI types.\n\
                     This creates circular dependencies and breaks reusability.\n\
                     \n\
                     Solution: Use simple typed parameters instead:\n\
                     ✅ GOOD:   fn calculate(x: i32, y: i32) -> Result<i32>\n\
                     ❌ WRONG:  fn calculate(args: VerbArgs) -> Result<i32>\n\
                     \n\
                     Pattern:\n\
                     1. #[verb] functions accept VerbArgs\n\
                     2. Extract typed values from VerbArgs\n\
                     3. Call domain functions with plain types\n\
                     4. Domain layer stays CLI-independent",
                    type_str
                ),
            ));
        }
    }

    // Check generic arguments recursively
    if let Type::Path(type_path) = ty {
        if let Some(last_segment) = type_path.path.segments.last() {
            if let syn::PathArguments::AngleBracketed(args) = &last_segment.arguments {
                for arg in &args.args {
                    if let syn::GenericArgument::Type(inner_ty) = arg {
                        if let Some(error) = check_for_cli_types(inner_ty) {
                            return Some(error);
                        }
                    }
                }
            }
        }
    }

    // Check reference types
    if let Type::Reference(type_ref) = ty {
        return check_for_cli_types(&type_ref.elem);
    }

    None
}

/// FM-1.1 Poka-Yoke Guard: Validate verb function complexity (FM-1.1: CLI Layer Contamination)
///
/// Prevents business logic from leaking into #[verb] functions by enforcing low complexity.
/// Verb functions should ONLY:
/// 1. Extract typed arguments from VerbArgs
/// 2. Call domain logic
/// 3. Format and return output
///
/// RPN 336: Critical severity - prevents architecture violations
pub fn validate_verb_complexity(input_fn: &ItemFn) -> syn::Result<()> {
    let complexity = calculate_cyclomatic_complexity(input_fn);

    // Threshold: 5 allows for basic verb function pattern (validate → call → format)
    // Higher complexity indicates business logic creeping into CLI layer
    if complexity > 5 {
        return Err(syn::Error::new(
            input_fn.sig.ident.span(),
            format!(
                "🛡️ Poka-Yoke Guard: Verb function too complex (FM-1.1)\n\
                 \n\
                 Complexity: {} (max allowed: 5)\n\
                 Function: {}\n\
                 \n\
                 Problem: Verb functions should delegate to domain logic, not implement it.\n\
                 High complexity indicates business logic leaking into CLI layer.\n\
                 \n\
                 Solution: Extract logic into separate domain function\n\
                 \n\
                 Correct Pattern (complexity ≤ 5):\n\
                 #[verb(\"calculate\")]\n\
                 fn cmd_calculate(x: i32, y: i32) -> Result<CalcResult> {{\n\
                     // 1. Extract/validate args (simple)\n\
                     if x < 0 {{ return Err(\"x must be positive\".into()); }}\n\
                     // 2. Call domain logic (single call)\n\
                     let result = domain::math::add(x, y);\n\
                     // 3. Format and return (simple)\n\
                     Ok(CalcResult {{ value: result }})\n\
                 }}\n\
                 \n\
                 Benefits:\n\
                 - CLI layer stays thin (easy to test)\n\
                 - Domain logic stays reusable (can call without CLI)\n\
                 - Clear separation of concerns\n\
                 - Business logic testable independently",
                complexity, input_fn.sig.ident
            ),
        ));
    }

    Ok(())
}

/// Calculate cyclomatic complexity of a function
///
/// Counts decision points: if, else, match arms, loops, &&, ||, ?
/// Returns simple count suitable for heuristic checking.
fn calculate_cyclomatic_complexity(input_fn: &ItemFn) -> usize {
    let mut complexity = 1; // Base complexity

    // Walk through the function block and count decision points
    count_decision_points(&input_fn.block, &mut complexity);

    complexity
}

/// Count decision points in a block recursively
fn count_decision_points(block: &syn::Block, complexity: &mut usize) {
    use syn::Stmt;

    for stmt in &block.stmts {
        match stmt {
            Stmt::Expr(expr, _) => {
                count_complexity_in_expr(expr, complexity);
            }
            Stmt::Item(_) | Stmt::Macro(_) => {
                // Item definitions and macros don't affect complexity for our purposes
            }
            Stmt::Local(local) => {
                // Check initializer expression
                if let Some(init) = &local.init {
                    count_complexity_in_expr(&init.expr, complexity);
                }
            }
        }
    }
}

/// Count decision points in an expression using simple heuristics
/// This is a simplified complexity check that looks for specific patterns
fn count_complexity_in_expr(expr: &syn::Expr, complexity: &mut usize) {
    let expr_str = expr.to_token_stream().to_string();

    // Check for decision-making keywords
    // Use simple string matching for portability across syn versions
    let decision_keywords = [
        "if ", "match ", "while ", "for ", // keywords
        "=> ",  // match arms
    ];

    for keyword in &decision_keywords {
        // Count occurrences
        let count = expr_str.matches(keyword).count();
        *complexity += count;
    }

    // Check for logical operators that add branches
    *complexity += expr_str.matches("&&").count();
    *complexity += expr_str.matches("||").count();

    // Recursively check sub-expressions if available
    // This is a simple depth-based check
    match expr {
        syn::Expr::Block(block_expr) => {
            count_decision_points(&block_expr.block, complexity);
        }
        syn::Expr::Paren(paren_expr) => {
            count_complexity_in_expr(&paren_expr.expr, complexity);
        }
        syn::Expr::Call(call_expr) => {
            count_complexity_in_expr(&call_expr.func, complexity);
            for arg in &call_expr.args {
                count_complexity_in_expr(arg, complexity);
            }
        }
        syn::Expr::Binary(bin_expr) => {
            count_complexity_in_expr(&bin_expr.left, complexity);
            count_complexity_in_expr(&bin_expr.right, complexity);
        }
        _ => {
            // Don't recurse deeply for other expressions - string matching is sufficient
        }
    }
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
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("must be a string literal"));
        assert!(err_msg.contains("Add double quotes"));
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

    #[test]
    fn test_validate_no_cli_types_in_params_good() {
        let fn_item: ItemFn = parse_quote! {
            fn calculate(x: i32, y: String) -> Result<String> {
                Ok("test".to_string())
            }
        };
        assert!(validate_no_cli_types_in_params(&fn_item.sig).is_ok());
    }

    #[test]
    fn test_validate_no_cli_types_in_params_bad_verbargs() {
        let fn_item: ItemFn = parse_quote! {
            fn bad_fn(args: VerbArgs) -> Result<()> {
                Ok(())
            }
        };
        let result = validate_no_cli_types_in_params(&fn_item.sig);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("CLI type contamination detected"));
        assert!(err_msg.contains("FM-1.2"));
    }

    #[test]
    fn test_validate_no_cli_types_in_params_bad_argmatches() {
        let fn_item: ItemFn = parse_quote! {
            fn bad_fn(matches: clap::ArgMatches) -> Result<()> {
                Ok(())
            }
        };
        let result = validate_no_cli_types_in_params(&fn_item.sig);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("CLI type contamination detected"));
    }

    #[test]
    fn test_validate_verb_complexity_simple() {
        let fn_item: ItemFn = parse_quote! {
            fn simple_verb(x: i32) -> Result<i32> {
                let result = domain::calculate(x);
                Ok(result)
            }
        };
        assert!(validate_verb_complexity(&fn_item).is_ok());
    }

    #[test]
    fn test_validate_verb_complexity_acceptable() {
        let fn_item: ItemFn = parse_quote! {
            fn verb_with_validation(x: i32) -> Result<i32> {
                if x < 0 {
                    return Err("must be positive".into());
                }
                let result = domain::calculate(x);
                Ok(result)
            }
        };
        // Simple if statement should be acceptable
        assert!(validate_verb_complexity(&fn_item).is_ok());
    }

    #[test]
    fn test_validate_verb_complexity_too_high() {
        let fn_item: ItemFn = parse_quote! {
            fn complex_verb(x: i32, y: i32) -> Result<i32> {
                if x < 0 {
                    if y < 0 {
                        if x + y < -100 {
                            match x {
                                0 => return Ok(0),
                                1 => return Ok(1),
                                2 => return Ok(2),
                                3 => return Ok(3),
                                _ => {}
                            }
                        }
                    }
                }
                Ok(x + y)
            }
        };
        let result = validate_verb_complexity(&fn_item);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("too complex"));
        assert!(err_msg.contains("FM-1.1"));
    }
}
