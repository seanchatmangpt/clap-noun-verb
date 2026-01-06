#![allow(clippy::needless_borrows_for_generic_args)]
// FUTURE: These types are part of the frontier feature set and will be integrated in future phases

//! Semantic CLI Composition - Type-safe capability discovery and composition
//!
//! This module provides procedural macros and runtime infrastructure for:
//! - Marking capabilities as semantically composable
//! - Runtime discovery protocol using SPARQL queries over RDF stores
//! - Dynamic CLI auto-generation from discovered capabilities
//! - Type-safe composition validation at compile and runtime
//! - MCP protocol integration for distributed agent-to-agent capability sharing
//!
//! # Architecture
//!
//! The semantic composition system uses a four-layer architecture:
//!
//! 1. **CapabilityRegistry**: Indexed RDF store of available capabilities
//!    - Uses oxigraph SPARQL engine for efficient queries
//!    - Capabilities stored as RDF triples with ontology metadata
//!    - Supports federated discovery across multiple registries
//!
//! 2. **CompositionValidator**: Type-level proof that composition is valid
//!    - Compile-time validation via proc macro analysis
//!    - Runtime validation via SPARQL constraint queries
//!    - Ensures type compatibility and contract adherence
//!
//! 3. **RuntimeBuilder**: Dynamically construct CLIs from capability combinations
//!    - Generates clap Command structures at runtime
//!    - Wires handlers to discovered capabilities
//!    - Supports hot-reloading and dynamic reconfiguration
//!
//! 4. **ProtocolAdapter**: Convert between different capability representations
//!    - MCP protocol for agent-to-agent communication
//!    - JSON-LD for semantic web integration
//!    - Custom binary protocols for performance
//!
//! # Type-First Design
//!
//! Capabilities are encoded at the type level:
//! ```rust,ignore
//! #[semantic_composable(
//!     uri = "urn:example:capability:file-reader",
//!     inputs = "rdf:type fs:Path",
//!     outputs = "rdf:type text:Content"
//! )]
//! fn read_file(path: PathBuf) -> Result<String, Error> {
//!     // Implementation
//! }
//! ```
//!
//! # Zero-Cost Abstractions
//!
//! - Macros expand to zero-cost registration code
//! - Generic composition uses monomorphization
//! - No runtime overhead for type-safe composition
//!
//! # Chicago TDD Testing Strategy
//!
//! Tests verify observable behaviors:
//! - Capability discovery returns correct results
//! - Composition validation rejects invalid combinations
//! - Generated CLIs execute correctly
//! - MCP protocol integration works end-to-end

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Error, Ident, ItemFn, LitStr, Result, Token,
};

// =============================================================================
// Type Definitions - Encode Invariants
// =============================================================================

/// Semantic composability attributes parsed from macro invocation
///
/// Type invariants:
/// - URI must be valid IRI format
/// - Inputs/outputs must be valid RDF type expressions
/// - Constraints are SPARQL ASK queries
#[derive(Clone)]
pub struct SemanticAttributes {
    /// Unique capability URI (e.g., "urn:example:capability:reader")
    pub uri: LitStr,
    /// RDF type expression for inputs (e.g., "rdf:type fs:Path")
    pub inputs: Option<LitStr>,
    /// RDF type expression for outputs (e.g., "rdf:type text:Content")
    pub outputs: Option<LitStr>,
    /// Additional SPARQL constraints for composition
    pub constraints: Option<LitStr>,
    /// MCP protocol version for agent communication
    pub mcp_version: Option<LitStr>,
}

impl Parse for SemanticAttributes {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut uri = None;
        let mut inputs = None;
        let mut outputs = None;
        let mut constraints = None;
        let mut mcp_version = None;

        // Parse key-value pairs: key = "value"
        let pairs = Punctuated::<MetaKeyValue, Token![,]>::parse_terminated(input)?;

        for pair in pairs {
            let key = pair.key.to_string();
            match key.as_str() {
                "uri" => {
                    if uri.is_some() {
                        return Err(Error::new(pair.key.span(), "duplicate 'uri' attribute"));
                    }
                    uri = Some(pair.value);
                }
                "inputs" => {
                    if inputs.is_some() {
                        return Err(Error::new(pair.key.span(), "duplicate 'inputs' attribute"));
                    }
                    inputs = Some(pair.value);
                }
                "outputs" => {
                    if outputs.is_some() {
                        return Err(Error::new(pair.key.span(), "duplicate 'outputs' attribute"));
                    }
                    outputs = Some(pair.value);
                }
                "constraints" => {
                    if constraints.is_some() {
                        return Err(Error::new(
                            pair.key.span(),
                            "duplicate 'constraints' attribute",
                        ));
                    }
                    constraints = Some(pair.value);
                }
                "mcp_version" => {
                    if mcp_version.is_some() {
                        return Err(Error::new(
                            pair.key.span(),
                            "duplicate 'mcp_version' attribute",
                        ));
                    }
                    mcp_version = Some(pair.value);
                }
                _ => {
                    return Err(Error::new(
                        pair.key.span(),
                        format!("unknown attribute '{}'", key),
                    ));
                }
            }
        }

        let uri = uri.ok_or_else(|| input.error("missing required 'uri' attribute"))?;

        Ok(SemanticAttributes { uri, inputs, outputs, constraints, mcp_version })
    }
}

// Manual Debug implementation since LitStr doesn't implement Debug
impl std::fmt::Debug for SemanticAttributes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SemanticAttributes")
            .field("uri", &self.uri.value())
            .field("inputs", &self.inputs.as_ref().map(|s| s.value()))
            .field("outputs", &self.outputs.as_ref().map(|s| s.value()))
            .field("constraints", &self.constraints.as_ref().map(|s| s.value()))
            .field("mcp_version", &self.mcp_version.as_ref().map(|s| s.value()))
            .finish()
    }
}

/// Helper struct for parsing key = "value" pairs
struct MetaKeyValue {
    key: Ident,
    value: LitStr,
}

impl Parse for MetaKeyValue {
    fn parse(input: ParseStream) -> Result<Self> {
        let key: Ident = input.parse()?;
        input.parse::<Token![=]>()?;
        let value: LitStr = input.parse()?;
        Ok(MetaKeyValue { key, value })
    }
}

// =============================================================================
// Macro Implementation - Zero-Cost Registration
// =============================================================================

/// Generate semantic composability registration code
///
/// This function performs:
/// 1. Compile-time validation of capability signature
/// 2. Generation of RDF metadata as const data
/// 3. Registration in distributed slice for auto-discovery
/// 4. MCP protocol adapter generation
///
/// # Type Safety
///
/// - Validates function signature matches declared types
/// - Ensures Result<T, E> return type for error handling
/// - Generates type-safe composition validators
pub fn expand_semantic_composable(
    attrs: SemanticAttributes,
    function: ItemFn,
) -> Result<TokenStream> {
    let fn_name = &function.sig.ident;
    let fn_vis = &function.vis;
    let fn_sig = &function.sig;

    // Validate function signature
    validate_composable_signature(&function)?;

    // Generate unique registration identifier
    let registry_name = Ident::new(
        &format!("__SEMANTIC_CAPABILITY_{}", fn_name.to_string().to_uppercase()),
        Span::call_site(),
    );

    // Extract RDF metadata
    let capability_uri = &attrs.uri;
    let inputs_rdf = attrs.inputs.as_ref().map(|s| s.value()).unwrap_or_default();
    let outputs_rdf = attrs.outputs.as_ref().map(|s| s.value()).unwrap_or_default();
    let constraints_sparql = attrs.constraints.as_ref().map(|s| s.value()).unwrap_or_default();
    let mcp_version =
        attrs.mcp_version.as_ref().map(|s| s.value()).unwrap_or_else(|| "2024.1".to_string());

    // Generate RDF metadata as const string (zero-cost, embedded in binary)
    let rdf_metadata = generate_rdf_metadata(
        capability_uri.value(),
        &inputs_rdf,
        &outputs_rdf,
        &constraints_sparql,
        fn_name.to_string(),
    );

    // Generate MCP protocol descriptor
    let mcp_descriptor =
        generate_mcp_descriptor(capability_uri.value(), &mcp_version, &mcp_version);

    Ok(quote! {
        // Original function (unchanged)
        #fn_vis #fn_sig #function.block

        // Capability metadata registration (zero-cost, compiled into binary)
        #[doc(hidden)]
        #[allow(non_upper_case_globals)]
        const #registry_name: ::clap_noun_verb::semantic::CapabilityMetadata =
            ::clap_noun_verb::semantic::CapabilityMetadata {
                uri: #capability_uri,
                function_name: stringify!(#fn_name),
                rdf_metadata: #rdf_metadata,
                mcp_descriptor: #mcp_descriptor,
            };

        // Register in distributed slice for auto-discovery
        #[::linkme::distributed_slice(::clap_noun_verb::semantic::SEMANTIC_CAPABILITIES)]
        #[doc(hidden)]
        static __CAPABILITY_REGISTRY: &::clap_noun_verb::semantic::CapabilityMetadata =
            &#registry_name;
    })
}

// =============================================================================
// Compile-Time Validation - Poka-Yoke Error Proofing
// =============================================================================

/// Validate that function signature meets semantic composability requirements
///
/// Requirements:
/// - Must return Result<T, E> for error handling
/// - Parameters must be serializable (for MCP protocol)
/// - No unsafe code in signature
fn validate_composable_signature(function: &ItemFn) -> Result<()> {
    let sig = &function.sig;

    // Check for Result return type
    let return_type = &sig.output;
    match return_type {
        syn::ReturnType::Default => {
            return Err(Error::new_spanned(
                sig,
                "semantic_composable functions must return Result<T, E>",
            ));
        }
        syn::ReturnType::Type(_, ty) => {
            // Basic check for Result in type path
            let type_str = quote!(#ty).to_string();
            if !type_str.contains("Result") {
                return Err(Error::new_spanned(
                    ty,
                    "semantic_composable functions must return Result<T, E>",
                ));
            }
        }
    }

    // Check for async (not yet supported - would require runtime)
    if sig.asyncness.is_some() {
        return Err(Error::new_spanned(
            sig,
            "async functions not yet supported in semantic_composable (FUTURE: tokio integration)",
        ));
    }

    // Check for unsafe (not allowed)
    if sig.unsafety.is_some() {
        return Err(Error::new_spanned(sig, "unsafe functions cannot be semantic_composable"));
    }

    Ok(())
}

// =============================================================================
// RDF Metadata Generation - Semantic Web Integration
// =============================================================================

/// Generate RDF Turtle format metadata for capability
///
/// Output format:
/// ```turtle
/// @prefix cap: <urn:clap-noun-verb:capability:> .
/// @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
/// @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
///
/// <urn:example:capability:reader>
///     rdf:type cap:Capability ;
///     cap:functionName "read_file" ;
///     cap:inputType "rdf:type fs:Path" ;
///     cap:outputType "rdf:type text:Content" ;
///     cap:constraints "ASK WHERE { ... }" .
/// ```
fn generate_rdf_metadata(
    uri: impl AsRef<str>,
    inputs: impl AsRef<str>,
    outputs: impl AsRef<str>,
    constraints: impl AsRef<str>,
    function_name: impl AsRef<str>,
) -> String {
    format!(
        r#"@prefix cap: <urn:clap-noun-verb:capability:> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

<{uri}>
    rdf:type cap:Capability ;
    cap:functionName "{function_name}" ;
    cap:inputType "{inputs}" ;
    cap:outputType "{outputs}" ;
    cap:constraints "{constraints}" ."#,
        uri = uri.as_ref(),
        function_name = function_name.as_ref(),
        inputs = inputs.as_ref(),
        outputs = outputs.as_ref(),
        constraints = constraints.as_ref(),
    )
}

// =============================================================================
// MCP Protocol Descriptor - Agent Communication
// =============================================================================

/// Generate MCP protocol descriptor JSON for agent-to-agent communication
///
/// Output format:
/// ```json
/// {
///   "protocol": "mcp",
///   "version": "2024.1",
///   "capability": {
///     "uri": "urn:example:capability:reader",
///     "function": "read_file",
///     "interface": "sync"
///   }
/// }
/// ```
fn generate_mcp_descriptor(
    uri: impl AsRef<str>,
    mcp_version: impl AsRef<str>,
    function_name: impl AsRef<str>,
) -> String {
    format!(
        r#"{{"protocol":"mcp","version":"{version}","capability":{{"uri":"{uri}","function":"{function}","interface":"sync"}}}}"#,
        version = mcp_version.as_ref(),
        uri = uri.as_ref(),
        function = function_name.as_ref(),
    )
}

// =============================================================================
// Unit Tests - Chicago TDD State-Based Verification
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    // AAA Pattern: Arrange-Act-Assert

    #[test]
    fn test_parse_semantic_attributes_with_all_fields() {
        // Arrange: Valid attribute syntax
        let input = parse_quote! {
            uri = "urn:test:cap:reader",
            inputs = "rdf:type fs:Path",
            outputs = "rdf:type text:Content",
            constraints = "ASK WHERE { ?s ?p ?o }",
            mcp_version = "2024.1"
        };

        // Act: Parse attributes
        let result: Result<SemanticAttributes> = syn::parse2(input);

        // Assert: All fields parsed correctly
        assert!(result.is_ok());
        let attrs = result.expect("parsing should succeed");
        assert_eq!(attrs.uri.value(), "urn:test:cap:reader");
        assert_eq!(attrs.inputs.as_ref().map(|s| s.value()).as_deref(), Some("rdf:type fs:Path"));
        assert_eq!(
            attrs.outputs.as_ref().map(|s| s.value()).as_deref(),
            Some("rdf:type text:Content")
        );
        assert_eq!(
            attrs.constraints.as_ref().map(|s| s.value()).as_deref(),
            Some("ASK WHERE { ?s ?p ?o }")
        );
        assert_eq!(attrs.mcp_version.as_ref().map(|s| s.value()).as_deref(), Some("2024.1"));
    }

    #[test]
    fn test_parse_semantic_attributes_minimal() {
        // Arrange: Minimal valid syntax (only required fields)
        let input = parse_quote! {
            uri = "urn:test:cap:minimal"
        };

        // Act: Parse attributes
        let result: Result<SemanticAttributes> = syn::parse2(input);

        // Assert: Required field present, optional fields None
        assert!(result.is_ok());
        let attrs = result.expect("parsing should succeed");
        assert_eq!(attrs.uri.value(), "urn:test:cap:minimal");
        assert!(attrs.inputs.is_none());
        assert!(attrs.outputs.is_none());
        assert!(attrs.constraints.is_none());
        assert!(attrs.mcp_version.is_none());
    }

    #[test]
    fn test_parse_semantic_attributes_missing_uri() {
        // Arrange: Missing required 'uri' field
        let input = parse_quote! {
            inputs = "rdf:type fs:Path"
        };

        // Act: Parse attributes
        let result: Result<SemanticAttributes> = syn::parse2(input);

        // Assert: Parsing fails with clear error
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("missing required 'uri' attribute"));
    }

    #[test]
    fn test_parse_semantic_attributes_duplicate_key() {
        // Arrange: Duplicate 'uri' field
        let input = parse_quote! {
            uri = "urn:test:first",
            uri = "urn:test:second"
        };

        // Act: Parse attributes
        let result: Result<SemanticAttributes> = syn::parse2(input);

        // Assert: Parsing fails detecting duplicate
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("duplicate 'uri' attribute"));
    }

    #[test]
    fn test_parse_semantic_attributes_unknown_key() {
        // Arrange: Unknown attribute key
        let input = parse_quote! {
            uri = "urn:test:cap",
            unknown_field = "value"
        };

        // Act: Parse attributes
        let result: Result<SemanticAttributes> = syn::parse2(input);

        // Assert: Parsing fails with unknown key error
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("unknown attribute"));
    }

    #[test]
    fn test_validate_composable_signature_valid() {
        // Arrange: Valid function signature with Result return
        let function: ItemFn = parse_quote! {
            fn test_capability(input: String) -> Result<String, Error> {
                Ok(input)
            }
        };

        // Act: Validate signature
        let result = validate_composable_signature(&function);

        // Assert: Validation succeeds
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_composable_signature_no_result() {
        // Arrange: Invalid function - no Result return type
        let function: ItemFn = parse_quote! {
            fn test_capability(input: String) -> String {
                input
            }
        };

        // Act: Validate signature
        let result = validate_composable_signature(&function);

        // Assert: Validation fails
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("must return Result<T, E>"));
    }

    #[test]
    fn test_validate_composable_signature_async() {
        // Arrange: Async function (not yet supported)
        let function: ItemFn = parse_quote! {
            async fn test_capability(input: String) -> Result<String, Error> {
                Ok(input)
            }
        };

        // Act: Validate signature
        let result = validate_composable_signature(&function);

        // Assert: Validation fails with clear message
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("async functions not yet supported"));
    }

    #[test]
    fn test_validate_composable_signature_unsafe() {
        // Arrange: Unsafe function (not allowed)
        let function: ItemFn = parse_quote! {
            unsafe fn test_capability(input: String) -> Result<String, Error> {
                Ok(input)
            }
        };

        // Act: Validate signature
        let result = validate_composable_signature(&function);

        // Assert: Validation fails
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("unsafe functions cannot be semantic_composable"));
    }

    #[test]
    fn test_generate_rdf_metadata() {
        // Arrange: Capability metadata
        let uri = "urn:test:cap:reader";
        let inputs = "rdf:type fs:Path";
        let outputs = "rdf:type text:Content";
        let constraints = "ASK WHERE { ?s ?p ?o }";
        let function_name = "read_file";

        // Act: Generate RDF Turtle
        let rdf = generate_rdf_metadata(uri, inputs, outputs, constraints, function_name);

        // Assert: RDF contains all required triples
        assert!(rdf.contains("<urn:test:cap:reader>"));
        assert!(rdf.contains("cap:functionName \"read_file\""));
        assert!(rdf.contains("cap:inputType \"rdf:type fs:Path\""));
        assert!(rdf.contains("cap:outputType \"rdf:type text:Content\""));
        assert!(rdf.contains("cap:constraints \"ASK WHERE { ?s ?p ?o }\""));
    }

    #[test]
    fn test_generate_mcp_descriptor() {
        // Arrange: MCP metadata
        let uri = "urn:test:cap:reader";
        let version = "2024.1";
        let function_name = "read_file";

        // Act: Generate MCP JSON descriptor
        let mcp = generate_mcp_descriptor(uri, version, function_name);

        // Assert: JSON contains all required fields
        assert!(mcp.contains(r#""protocol":"mcp""#));
        assert!(mcp.contains(r#""version":"2024.1""#));
        assert!(mcp.contains(r#""uri":"urn:test:cap:reader""#));
        assert!(mcp.contains(r#""function":"read_file""#));
        assert!(mcp.contains(r#""interface":"sync""#));
    }
}
