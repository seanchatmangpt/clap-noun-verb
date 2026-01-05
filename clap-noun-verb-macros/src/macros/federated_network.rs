//! Federated Semantic Network Macros
//!
//! This module provides procedural macros for enabling CLI federation and semantic networking.
//! CLIs can discover, advertise, and invoke capabilities across a federated network using
//! RDF-based semantic descriptions and type-safe remote invocation.
//!
//! # Architecture
//!
//! - **CapabilityAdvertiser**: Publishes RDF descriptions to discovery service
//! - **RemoteResolver**: Finds and validates remote capabilities via SPARQL
//! - **InvocationProxy**: Type-safe cross-CLI method calls with serialization
//! - **TrustValidator**: Cryptographic verification of remote CLI signatures
//! - **FederationRegistry**: Tracks network topology and peer relationships
//!
//! # Protocols
//!
//! - Capability advertisement: DCAT over HTTP
//! - Federation queries: SPARQL
//! - Remote invocation: REST with CBOR encoding
//! - Trust establishment: Ed25519 signatures with certificate chains

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse::Parser, ItemFn, ItemStruct};

/// Mark a CLI as participatory in the federated network
///
/// This macro generates the necessary boilerplate for CLI federation:
/// - Capability advertisement on startup
/// - Discovery service registration
/// - Peer authentication setup
/// - Network topology tracking
///
/// # Example
///
/// ```rust,ignore
/// #[federated(
///     discovery_url = "https://cli-federation.example.com",
///     identity = "my-cli-v1.0",
///     trust_anchor = "./certs/root.pem"
/// )]
/// struct MyCli;
/// ```
pub fn federated_impl(args: TokenStream, input: TokenStream) -> syn::Result<TokenStream> {
    let input_struct = syn::parse2::<ItemStruct>(input)?;
    let struct_name = &input_struct.ident;

    // Parse macro arguments
    let config = parse_federated_config(args)?;
    let discovery_url = config.discovery_url;
    let identity = config.identity;
    let trust_anchor = config.trust_anchor;

    // Generate federation initialization code
    let expanded = quote! {
        #input_struct

        impl ::clap_noun_verb::federation::Federated for #struct_name {
            fn discovery_url(&self) -> &str {
                #discovery_url
            }

            fn identity(&self) -> &str {
                #identity
            }

            fn trust_anchor(&self) -> &str {
                #trust_anchor
            }

            fn initialize_federation(&self) -> ::clap_noun_verb::error::Result<()> {
                // Initialize capability advertiser
                let advertiser = ::clap_noun_verb::federation::CapabilityAdvertiser::new(
                    self.identity(),
                    self.discovery_url(),
                )?;

                // Initialize trust validator
                let validator = ::clap_noun_verb::federation::TrustValidator::new(
                    self.trust_anchor(),
                )?;

                // Initialize federation registry
                let registry = ::clap_noun_verb::federation::FederationRegistry::new(
                    self.identity(),
                    validator,
                )?;

                // Register with federation
                advertiser.advertise_startup()?;
                registry.register_self()?;

                Ok(())
            }

            fn shutdown_federation(&self) -> ::clap_noun_verb::error::Result<()> {
                let advertiser = ::clap_noun_verb::federation::CapabilityAdvertiser::get_instance()?;
                advertiser.advertise_shutdown()?;
                Ok(())
            }
        }

        // Auto-initialize federation on startup
        #[linkme::distributed_slice(::clap_noun_verb::cli::registry::__STARTUP_HOOKS)]
        static __INIT_FEDERATION: fn() = || {
            let cli = #struct_name;
            if let Err(e) = cli.initialize_federation() {
                eprintln!("Warning: Failed to initialize CLI federation: {}", e);
            }
        };

        // Auto-shutdown federation on exit
        #[linkme::distributed_slice(::clap_noun_verb::cli::registry::__SHUTDOWN_HOOKS)]
        static __SHUTDOWN_FEDERATION: fn() = || {
            let cli = #struct_name;
            if let Err(e) = cli.shutdown_federation() {
                eprintln!("Warning: Failed to shutdown CLI federation: {}", e);
            }
        };
    };

    Ok(expanded)
}

/// Advertise a capability to the federated network
///
/// This macro generates RDF metadata for a CLI command and publishes it to the
/// discovery service. Remote CLIs can query and invoke this capability.
///
/// # Example
///
/// ```rust,ignore
/// #[advertise_capability(
///     capability_id = "process-data",
///     description = "Process data files with advanced algorithms",
///     inputs = ["file:path", "format:string"],
///     outputs = ["result:json"]
/// )]
/// #[verb("process")]
/// fn process_data(file: PathBuf, format: String) -> Result<ProcessResult> {
///     // Implementation
/// }
/// ```
pub fn advertise_capability_impl(
    args: TokenStream,
    input: TokenStream,
) -> syn::Result<TokenStream> {
    let input_fn = syn::parse2::<ItemFn>(input.clone())?;
    let fn_name = &input_fn.sig.ident;

    // Parse macro arguments
    let config = parse_capability_config(args)?;
    let capability_id = config.capability_id;
    let description = config.description;
    let inputs = config.inputs;
    let outputs = config.outputs;

    // Generate RDF advertisement code
    let expanded = quote! {
        #input_fn

        // Auto-register capability advertisement
        #[linkme::distributed_slice(::clap_noun_verb::federation::__CAPABILITY_REGISTRY)]
        static #fn_name: fn() = || {
            let capability = ::clap_noun_verb::federation::CapabilityDescriptor {
                id: #capability_id,
                description: #description,
                inputs: vec![#(#inputs.to_string()),*],
                outputs: vec![#(#outputs.to_string()),*],
                handler: stringify!(#fn_name),
            };

            if let Ok(advertiser) = ::clap_noun_verb::federation::CapabilityAdvertiser::get_instance() {
                if let Err(e) = advertiser.advertise_capability(&capability) {
                    eprintln!("Warning: Failed to advertise capability '{}': {}", #capability_id, e);
                }
            }
        };
    };

    Ok(expanded)
}

/// Enable remote invocation of a CLI capability
///
/// This macro generates type-safe RPC stubs for calling remote CLI commands.
/// It handles serialization, network transport, authentication, and result validation.
///
/// # Example
///
/// ```rust,ignore
/// #[remote_invoke(
///     target = "remote-cli-v1.0",
///     capability = "process-data",
///     timeout_ms = 5000
/// )]
/// fn remote_process(file: PathBuf, format: String) -> Result<ProcessResult>;
/// ```
pub fn remote_invoke_impl(args: TokenStream, input: TokenStream) -> syn::Result<TokenStream> {
    let input_fn = syn::parse2::<ItemFn>(input)?;
    #[allow(unused_variables)] // Used in quote! macro
    let fn_name = &input_fn.sig.ident;
    let fn_sig = &input_fn.sig;

    // Parse macro arguments
    let config = parse_remote_invoke_config(args)?;
    let target = config.target;
    let capability = config.capability;
    let timeout_ms = config.timeout_ms;

    // Extract parameter names and types
    let mut param_names = Vec::new();
    let mut param_types = Vec::new();

    for input in &fn_sig.inputs {
        if let syn::FnArg::Typed(pat_type) = input {
            if let syn::Pat::Ident(ident) = &*pat_type.pat {
                param_names.push(&ident.ident);
                param_types.push(&*pat_type.ty);
            }
        }
    }

    // Extract return type
    let return_type = match &fn_sig.output {
        syn::ReturnType::Type(_, ty) => ty.clone(),
        syn::ReturnType::Default => {
            return Err(syn::Error::new_spanned(
                fn_sig,
                "Remote invoke requires explicit return type (Result<T>)",
            ));
        }
    };

    // Generate remote invocation implementation
    let expanded = quote! {
        #fn_sig {
            // Initialize remote resolver
            let resolver = ::clap_noun_verb::federation::RemoteResolver::new()?;

            // Resolve remote capability
            let endpoint = resolver.resolve_capability(#target, #capability)?;

            // Initialize invocation proxy
            let proxy = ::clap_noun_verb::federation::InvocationProxy::new(
                endpoint,
                ::std::time::Duration::from_millis(#timeout_ms),
            )?;

            // Serialize parameters
            let params = ::clap_noun_verb::federation::InvocationParams {
                capability: #capability.to_string(),
                args: vec![
                    #(
                        (
                            stringify!(#param_names).to_string(),
                            ::clap_noun_verb::federation::serialize_param(&#param_names)?
                        )
                    ),*
                ],
            };

            // Invoke remote capability
            let result = proxy.invoke(&params)?;

            // Deserialize result
            ::clap_noun_verb::federation::deserialize_result::<#return_type>(&result)
        }
    };

    Ok(expanded)
}

/// Configuration for #[federated] macro
struct FederatedConfig {
    discovery_url: String,
    identity: String,
    trust_anchor: String,
}

/// Configuration for #[advertise_capability] macro
struct CapabilityConfig {
    capability_id: String,
    description: String,
    inputs: Vec<String>,
    outputs: Vec<String>,
}

/// Configuration for #[remote_invoke] macro
struct RemoteInvokeConfig {
    target: String,
    capability: String,
    timeout_ms: u64,
}

/// Parse #[federated(...)] arguments
pub fn parse_federated_config(args: TokenStream) -> syn::Result<FederatedConfig> {
    let parser = syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated;
    let meta_list = parser.parse2(args)?;

    let mut discovery_url = None;
    let mut identity = None;
    let mut trust_anchor = None;

    for meta in meta_list {
        match &meta {
            syn::Meta::NameValue(nv) => {
                let ident = nv
                    .path
                    .get_ident()
                    .ok_or_else(|| syn::Error::new_spanned(&nv.path, "Expected identifier"))?
                    .to_string();

                if let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(s), .. }) = &nv.value {
                    match ident.as_str() {
                        "discovery_url" => discovery_url = Some(s.value()),
                        "identity" => identity = Some(s.value()),
                        "trust_anchor" => trust_anchor = Some(s.value()),
                        _ => {
                            return Err(syn::Error::new_spanned(
                                &nv.path,
                                format!("Unknown attribute: {}", ident),
                            ));
                        }
                    }
                }
            }
            _ => {
                return Err(syn::Error::new_spanned(
                    &meta,
                    "Expected name-value pairs (e.g., discovery_url = \"...\")",
                ));
            }
        }
    }

    Ok(FederatedConfig {
        discovery_url: discovery_url.ok_or_else(|| {
            syn::Error::new(
                proc_macro2::Span::call_site(),
                "Missing required attribute: discovery_url",
            )
        })?,
        identity: identity.ok_or_else(|| {
            syn::Error::new(proc_macro2::Span::call_site(), "Missing required attribute: identity")
        })?,
        trust_anchor: trust_anchor.ok_or_else(|| {
            syn::Error::new(
                proc_macro2::Span::call_site(),
                "Missing required attribute: trust_anchor",
            )
        })?,
    })
}

/// Parse #[advertise_capability(...)] arguments
pub fn parse_capability_config(args: TokenStream) -> syn::Result<CapabilityConfig> {
    let parser = syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated;
    let meta_list = parser.parse2(args)?;

    let mut capability_id = None;
    let mut description = None;
    let mut inputs = Vec::new();
    let mut outputs = Vec::new();

    for meta in meta_list {
        match &meta {
            syn::Meta::NameValue(nv) => {
                let ident = nv
                    .path
                    .get_ident()
                    .ok_or_else(|| syn::Error::new_spanned(&nv.path, "Expected identifier"))?
                    .to_string();

                match ident.as_str() {
                    "capability_id" | "description" => {
                        if let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(s), .. }) =
                            &nv.value
                        {
                            match ident.as_str() {
                                "capability_id" => capability_id = Some(s.value()),
                                "description" => description = Some(s.value()),
                                _ => {}
                            }
                        }
                    }
                    "inputs" | "outputs" => {
                        if let syn::Expr::Array(arr) = &nv.value {
                            let values: Vec<String> = arr
                                .elems
                                .iter()
                                .filter_map(|expr| {
                                    if let syn::Expr::Lit(syn::ExprLit {
                                        lit: syn::Lit::Str(s),
                                        ..
                                    }) = expr
                                    {
                                        Some(s.value())
                                    } else {
                                        None
                                    }
                                })
                                .collect();

                            match ident.as_str() {
                                "inputs" => inputs = values,
                                "outputs" => outputs = values,
                                _ => {}
                            }
                        }
                    }
                    _ => {
                        return Err(syn::Error::new_spanned(
                            &nv.path,
                            format!("Unknown attribute: {}", ident),
                        ));
                    }
                }
            }
            _ => {
                return Err(syn::Error::new_spanned(&meta, "Expected name-value pairs"));
            }
        }
    }

    Ok(CapabilityConfig {
        capability_id: capability_id.ok_or_else(|| {
            syn::Error::new(
                proc_macro2::Span::call_site(),
                "Missing required attribute: capability_id",
            )
        })?,
        description: description.unwrap_or_default(),
        inputs,
        outputs,
    })
}

/// Parse #[remote_invoke(...)] arguments
pub fn parse_remote_invoke_config(args: TokenStream) -> syn::Result<RemoteInvokeConfig> {
    let parser = syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated;
    let meta_list = parser.parse2(args)?;

    let mut target = None;
    let mut capability = None;
    let mut timeout_ms = 5000u64; // Default timeout

    for meta in meta_list {
        match &meta {
            syn::Meta::NameValue(nv) => {
                let ident = nv
                    .path
                    .get_ident()
                    .ok_or_else(|| syn::Error::new_spanned(&nv.path, "Expected identifier"))?
                    .to_string();

                match ident.as_str() {
                    "target" | "capability" => {
                        if let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(s), .. }) =
                            &nv.value
                        {
                            match ident.as_str() {
                                "target" => target = Some(s.value()),
                                "capability" => capability = Some(s.value()),
                                _ => {}
                            }
                        }
                    }
                    "timeout_ms" => {
                        if let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Int(i), .. }) =
                            &nv.value
                        {
                            timeout_ms = i.base10_parse::<u64>()?;
                        }
                    }
                    _ => {
                        return Err(syn::Error::new_spanned(
                            &nv.path,
                            format!("Unknown attribute: {}", ident),
                        ));
                    }
                }
            }
            _ => {
                return Err(syn::Error::new_spanned(&meta, "Expected name-value pairs"));
            }
        }
    }

    Ok(RemoteInvokeConfig {
        target: target.ok_or_else(|| {
            syn::Error::new(proc_macro2::Span::call_site(), "Missing required attribute: target")
        })?,
        capability: capability.ok_or_else(|| {
            syn::Error::new(
                proc_macro2::Span::call_site(),
                "Missing required attribute: capability",
            )
        })?,
        timeout_ms,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    #[test]
    fn test_parse_federated_config() {
        let args = quote! {
            discovery_url = "https://example.com",
            identity = "my-cli",
            trust_anchor = "./certs/root.pem"
        };

        let config = parse_federated_config(args).expect("Failed to parse config");
        assert_eq!(config.discovery_url, "https://example.com");
        assert_eq!(config.identity, "my-cli");
        assert_eq!(config.trust_anchor, "./certs/root.pem");
    }

    #[test]
    fn test_parse_capability_config() {
        let args = quote! {
            capability_id = "process-data",
            description = "Process data files",
            inputs = ["file:path", "format:string"],
            outputs = ["result:json"]
        };

        let config = parse_capability_config(args).expect("Failed to parse config");
        assert_eq!(config.capability_id, "process-data");
        assert_eq!(config.description, "Process data files");
        assert_eq!(config.inputs, vec!["file:path", "format:string"]);
        assert_eq!(config.outputs, vec!["result:json"]);
    }

    #[test]
    fn test_parse_remote_invoke_config() {
        let args = quote! {
            target = "remote-cli",
            capability = "process-data",
            timeout_ms = 10000
        };

        let config = parse_remote_invoke_config(args).expect("Failed to parse config");
        assert_eq!(config.target, "remote-cli");
        assert_eq!(config.capability, "process-data");
        assert_eq!(config.timeout_ms, 10000);
    }

    #[test]
    fn test_parse_remote_invoke_config_default_timeout() {
        let args = quote! {
            target = "remote-cli",
            capability = "process-data"
        };

        let config = parse_remote_invoke_config(args).expect("Failed to parse config");
        assert_eq!(config.timeout_ms, 5000); // Default timeout
    }
}
