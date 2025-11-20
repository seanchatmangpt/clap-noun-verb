//! RDF triple generation from macro metadata
//!
//! This module generates Turtle RDF triples and SHACL shapes from verb metadata
//! extracted during macro expansion. The generated RDF describes commands, arguments,
//! and validation constraints in a machine-queryable format.
//!
//! FUTURE: v5.1 - RDF/SHACL semantic metadata export

#![allow(dead_code)] // FUTURE: v5.1 - complete RDF export feature

use proc_macro2::TokenStream;
use quote::quote;

/// Argument metadata extracted from function parameters
#[derive(Debug, Clone)]
pub struct ArgMetadata {
    pub name: String,
    pub ty: String,
    pub required: bool,
    pub doc: Option<String>,
    pub bounds: Option<(i64, i64)>,
    pub pattern: Option<String>,
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
}

/// Generate Turtle RDF triples for a verb
///
/// Creates triples describing the command, its noun/verb names, documentation,
/// and all arguments with their types and constraints.
///
/// # Example Output
/// ```turtle
/// cli:services-status a cnv:Command ;
///     cnv:name "services-status" ;
///     cnv:nounName "services" ;
///     rdfs:comment "Show service status" ;
///     cnv:hasArgument cli:arg-verbose, cli:arg-json .
///
/// cli:arg-verbose a cnv:Argument ;
///     cnv:name "verbose" ;
///     cnv:type xsd:boolean ;
///     cnv:required false .
/// ```
pub fn generate_rdf_for_verb(name: &str, noun: &str, doc: &str, args: &[ArgMetadata]) -> String {
    let mut rdf = String::new();

    // Command URI
    let command_uri = format!("cli:{}-{}", noun, name);

    // Command definition
    rdf.push_str(&format!("{} a cnv:Command ;\n", command_uri));
    rdf.push_str(&format!("    cnv:name \"{}\" ;\n", name));
    rdf.push_str(&format!("    cnv:nounName \"{}\" ;\n", noun));

    if !doc.is_empty() {
        rdf.push_str(&format!("    rdfs:comment \"{}\" ;\n", escape_string(doc)));
    }

    // Argument URIs
    if !args.is_empty() {
        rdf.push_str("    cnv:hasArgument ");
        let arg_uris: Vec<String> =
            args.iter().map(|arg| format!("cli:arg-{}", arg.name)).collect();
        rdf.push_str(&arg_uris.join(", "));
        rdf.push_str(" .\n");
    } else {
        // Remove trailing semicolon and add period
        if rdf.ends_with(";\n") {
            rdf.truncate(rdf.len() - 2);
            rdf.push_str(" .\n");
        }
    }

    // Argument definitions
    for arg in args {
        rdf.push('\n');
        rdf.push_str(&generate_argument_rdf(arg));
    }

    rdf
}

/// Generate RDF for a single argument
fn generate_argument_rdf(arg: &ArgMetadata) -> String {
    let mut rdf = String::new();
    let arg_uri = format!("cli:arg-{}", arg.name);

    rdf.push_str(&format!("{} a cnv:Argument ;\n", arg_uri));
    rdf.push_str(&format!("    cnv:name \"{}\" ;\n", arg.name));
    rdf.push_str(&format!("    cnv:type {} ;\n", map_rust_type_to_xsd(&arg.ty)));
    rdf.push_str(&format!("    cnv:required {} ", arg.required));

    // Add constraints if present
    let has_constraints = arg.bounds.is_some()
        || arg.pattern.is_some()
        || arg.min_length.is_some()
        || arg.max_length.is_some()
        || arg.doc.is_some();

    if has_constraints {
        rdf.push_str(";\n");

        if let Some((min, max)) = arg.bounds {
            rdf.push_str(&format!("    cnv:minValue \"{}\"^^xsd:integer ;\n", min));
            rdf.push_str(&format!("    cnv:maxValue \"{}\"^^xsd:integer ;\n", max));
        }

        if let Some(ref pattern) = arg.pattern {
            rdf.push_str(&format!("    cnv:pattern \"{}\" ;\n", escape_string(pattern)));
        }

        if let Some(min_len) = arg.min_length {
            rdf.push_str(&format!("    cnv:minLength \"{}\"^^xsd:nonNegativeInteger ;\n", min_len));
        }

        if let Some(max_len) = arg.max_length {
            rdf.push_str(&format!("    cnv:maxLength \"{}\"^^xsd:nonNegativeInteger ;\n", max_len));
        }

        if let Some(ref doc) = arg.doc {
            rdf.push_str(&format!("    rdfs:comment \"{}\" ;\n", escape_string(doc)));
        }

        // Remove trailing semicolon and add period
        if rdf.ends_with(";\n") {
            rdf.truncate(rdf.len() - 2);
            rdf.push_str(" .\n");
        }
    } else {
        rdf.push_str(".\n");
    }

    rdf
}

/// Generate SHACL shapes for verb validation
///
/// Creates SHACL NodeShape constraints that validate command invocations
/// against argument requirements, types, and bounds.
///
/// # Example Output
/// ```turtle
/// :services-status-shape a sh:NodeShape ;
///     sh:targetNode cli:services-status ;
///     sh:property [
///         sh:path cnv:argument ;
///         sh:name "verbose" ;
///         sh:datatype xsd:boolean ;
///         sh:minCount 0 ;
///         sh:maxCount 1 ;
///     ] .
/// ```
pub fn generate_shacl_shapes_for_verb(name: &str, noun: &str, args: &[ArgMetadata]) -> String {
    let mut shacl = String::new();

    let shape_name = format!(":{}-{}-shape", noun, name);
    let command_uri = format!("cli:{}-{}", noun, name);

    shacl.push_str(&format!("{} a sh:NodeShape ;\n", shape_name));
    shacl.push_str(&format!("    sh:targetNode {} ", command_uri));

    if args.is_empty() {
        shacl.push_str(".\n");
        return shacl;
    }

    shacl.push_str(";\n");

    // Generate property shapes for each argument
    for (i, arg) in args.iter().enumerate() {
        shacl.push_str("    sh:property [\n");
        shacl.push_str("        sh:path cnv:argument ;\n");
        shacl.push_str(&format!("        sh:name \"{}\" ;\n", arg.name));
        shacl.push_str(&format!("        sh:datatype {} ;\n", map_rust_type_to_xsd(&arg.ty)));

        // Required constraint
        if arg.required {
            shacl.push_str("        sh:minCount 1 ;\n");
            shacl.push_str("        sh:maxCount 1 ;\n");
        } else {
            shacl.push_str("        sh:minCount 0 ;\n");
            shacl.push_str("        sh:maxCount 1 ;\n");
        }

        // Value constraints
        if let Some((min, max)) = arg.bounds {
            shacl.push_str(&format!("        sh:minInclusive {} ;\n", min));
            shacl.push_str(&format!("        sh:maxInclusive {} ;\n", max));
        }

        if let Some(ref pattern) = arg.pattern {
            shacl.push_str(&format!("        sh:pattern \"{}\" ;\n", escape_string(pattern)));
        }

        if let Some(min_len) = arg.min_length {
            shacl.push_str(&format!("        sh:minLength {} ;\n", min_len));
        }

        if let Some(max_len) = arg.max_length {
            shacl.push_str(&format!("        sh:maxLength {} ;\n", max_len));
        }

        if let Some(ref doc) = arg.doc {
            shacl.push_str(&format!("        sh:description \"{}\" ;\n", escape_string(doc)));
        }

        // Remove trailing semicolon
        if shacl.ends_with(";\n") {
            shacl.truncate(shacl.len() - 2);
            shacl.push('\n');
        }

        // Close property shape
        if i < args.len() - 1 {
            shacl.push_str("    ] ;\n");
        } else {
            shacl.push_str("    ] .\n");
        }
    }

    shacl
}

/// Map Rust types to XSD datatypes
fn map_rust_type_to_xsd(rust_type: &str) -> String {
    match rust_type {
        "bool" => "xsd:boolean".to_string(),
        "String" | "str" | "&str" => "xsd:string".to_string(),
        "u8" | "u16" | "u32" | "u64" | "usize" => "xsd:nonNegativeInteger".to_string(),
        "i8" | "i16" | "i32" | "i64" | "isize" => "xsd:integer".to_string(),
        "f32" | "f64" => "xsd:decimal".to_string(),
        "PathBuf" | "Path" => "xsd:anyURI".to_string(),
        _ => {
            // Handle generic types like Option<T>, Vec<T>
            if rust_type.starts_with("Option<") {
                let inner = rust_type
                    .strip_prefix("Option<")
                    .and_then(|s| s.strip_suffix('>'))
                    .unwrap_or("String");
                map_rust_type_to_xsd(inner)
            } else if rust_type.starts_with("Vec<") {
                let inner = rust_type
                    .strip_prefix("Vec<")
                    .and_then(|s| s.strip_suffix('>'))
                    .unwrap_or("String");
                map_rust_type_to_xsd(inner)
            } else {
                "xsd:string".to_string()
            }
        }
    }
}

/// Escape special characters in strings for Turtle syntax
fn escape_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

/// Generate the distributed slice registration code
pub fn generate_rdf_registration(
    verb_name: &str,
    noun_name: &str,
    fn_name: &syn::Ident,
) -> TokenStream {
    let rdf_static_name = quote::format_ident!("__RDF_{}", fn_name.to_string().to_uppercase());

    quote! {
        #[allow(non_upper_case_globals)]
        #[linkme::distributed_slice(::clap_noun_verb::rdf::macro_integration::__VERB_RDF)]
        static #rdf_static_name: fn() -> (&'static str, &'static str) = || {
            // RDF triples and SHACL shapes will be generated at compile time
            // and stored as static strings
            (
                concat!(
                    "# RDF for verb: ", #verb_name, "\n",
                    "# Noun: ", #noun_name, "\n"
                ),
                concat!(
                    "# SHACL shapes for verb: ", #verb_name, "\n"
                )
            )
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_simple_rdf() {
        let args = vec![ArgMetadata {
            name: "verbose".to_string(),
            ty: "bool".to_string(),
            required: false,
            doc: Some("Enable verbose output".to_string()),
            bounds: None,
            pattern: None,
            min_length: None,
            max_length: None,
        }];

        let rdf = generate_rdf_for_verb("status", "services", "Show service status", &args);

        assert!(rdf.contains("cli:services-status a cnv:Command"));
        assert!(rdf.contains("cnv:name \"status\""));
        assert!(rdf.contains("cnv:nounName \"services\""));
        assert!(rdf.contains("cli:arg-verbose a cnv:Argument"));
        assert!(rdf.contains("cnv:type xsd:boolean"));
    }

    #[test]
    fn test_generate_rdf_with_constraints() {
        let args = vec![ArgMetadata {
            name: "port".to_string(),
            ty: "u16".to_string(),
            required: true,
            doc: Some("Server port".to_string()),
            bounds: Some((1, 65535)),
            pattern: None,
            min_length: None,
            max_length: None,
        }];

        let rdf = generate_rdf_for_verb("start", "server", "Start server", &args);

        assert!(rdf.contains("cli:arg-port"));
        assert!(rdf.contains("cnv:minValue \"1\""));
        assert!(rdf.contains("cnv:maxValue \"65535\""));
        assert!(rdf.contains("cnv:required true"));
    }

    #[test]
    fn test_generate_shacl_shapes() {
        let args = vec![ArgMetadata {
            name: "name".to_string(),
            ty: "String".to_string(),
            required: true,
            doc: Some("Resource name".to_string()),
            bounds: None,
            pattern: Some("^[a-z0-9-]+$".to_string()),
            min_length: Some(3),
            max_length: Some(50),
        }];

        let shacl = generate_shacl_shapes_for_verb("create", "resource", &args);

        assert!(shacl.contains(":resource-create-shape a sh:NodeShape"));
        assert!(shacl.contains("sh:targetNode cli:resource-create"));
        assert!(shacl.contains("sh:name \"name\""));
        assert!(shacl.contains("sh:datatype xsd:string"));
        assert!(shacl.contains("sh:minCount 1"));
        assert!(shacl.contains("sh:pattern"));
        assert!(shacl.contains("sh:minLength 3"));
        assert!(shacl.contains("sh:maxLength 50"));
    }

    #[test]
    fn test_map_rust_types() {
        assert_eq!(map_rust_type_to_xsd("bool"), "xsd:boolean");
        assert_eq!(map_rust_type_to_xsd("String"), "xsd:string");
        assert_eq!(map_rust_type_to_xsd("u32"), "xsd:nonNegativeInteger");
        assert_eq!(map_rust_type_to_xsd("i32"), "xsd:integer");
        assert_eq!(map_rust_type_to_xsd("f64"), "xsd:decimal");
        assert_eq!(map_rust_type_to_xsd("Option<String>"), "xsd:string");
        assert_eq!(map_rust_type_to_xsd("Vec<u32>"), "xsd:nonNegativeInteger");
    }

    #[test]
    fn test_escape_string() {
        assert_eq!(escape_string("hello"), "hello");
        assert_eq!(escape_string("hello \"world\""), "hello \\\"world\\\"");
        assert_eq!(escape_string("line1\nline2"), "line1\\nline2");
        assert_eq!(escape_string("tab\there"), "tab\\there");
    }

    #[test]
    fn test_generate_rdf_no_args() {
        let rdf = generate_rdf_for_verb("list", "services", "List all services", &[]);

        assert!(rdf.contains("cli:services-list a cnv:Command"));
        assert!(rdf.contains("cnv:name \"list\""));
        assert!(!rdf.contains("cnv:hasArgument"));
    }

    #[test]
    fn test_generate_shacl_no_args() {
        let shacl = generate_shacl_shapes_for_verb("list", "services", &[]);

        assert!(shacl.contains(":services-list-shape a sh:NodeShape"));
        assert!(shacl.contains("sh:targetNode cli:services-list"));
        assert!(!shacl.contains("sh:property"));
    }
}
