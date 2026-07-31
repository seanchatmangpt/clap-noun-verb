// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! RDF → ggen projection for typed noun-verb interface adapters.
//!
//! RDF owns names, arguments, documentation, and return types. Generated wrappers
//! own interface projection only; domain behavior remains in `crate::handlers`.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// RDF triple representation flattened from SPARQL or Turtle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RdfTriple {
    /// Subject URI.
    pub subject: String,
    /// Predicate URI.
    pub predicate: String,
    /// Literal value or referenced URI.
    pub object: String,
    /// RDF object classification.
    #[serde(default)]
    pub object_type: ObjectType,
}

/// RDF object classification.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ObjectType {
    /// Literal value.
    #[default]
    Literal,
    /// URI reference.
    Reference,
}

/// RDF verb definition extracted from semantic authority.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RdfVerbDefinition {
    /// Stable verb URI.
    pub verb_uri: String,
    /// CLI verb name.
    pub name: String,
    /// Human-readable description.
    pub description: String,
    /// Parent noun URI.
    #[serde(default)]
    pub noun_uri: Option<String>,
    /// Parent noun local name.
    #[serde(default)]
    pub noun_name: Option<String>,
    /// Ordered arguments.
    #[serde(default)]
    pub arguments: Vec<RdfArgumentDefinition>,
    /// Rust return type.
    pub return_type: String,
    /// Required trait bounds.
    #[serde(default)]
    pub trait_bounds: Vec<String>,
    /// Documentation text.
    #[serde(default)]
    pub docstring: String,
    /// Whether the adapter is asynchronous.
    #[serde(default)]
    pub is_async: bool,
}

/// RDF argument definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RdfArgumentDefinition {
    /// Stable argument URI.
    pub arg_uri: String,
    /// CLI and Rust parameter name.
    pub name: String,
    /// Human-readable description.
    pub description: String,
    /// Rust value type.
    pub value_type: String,
    /// Whether the value is required.
    pub required: bool,
    /// Whether the value is a boolean flag.
    pub is_flag: bool,
    /// Optional default value.
    #[serde(default)]
    pub default_value: Option<String>,
    /// Optional short name.
    #[serde(default)]
    pub short_name: Option<char>,
    /// Optional long name.
    #[serde(default)]
    pub long_name: Option<String>,
    /// Admitted values.
    #[serde(default)]
    pub allowed_values: Vec<String>,
    /// Argument classification.
    pub argument_type: ArgumentType,
}

/// Argument classification.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum ArgumentType {
    /// Positional required argument.
    #[default]
    Positional,
    /// Optional named argument.
    Optional,
    /// Boolean flag.
    Flag,
    /// Repeatable argument.
    Repeating,
    /// Variadic argument.
    Variadic,
}

/// One SPARQL JSON binding.
#[derive(Debug, Deserialize)]
pub struct SparqlBinding {
    /// Bound lexical value or URI.
    #[serde(default)]
    pub value: String,
    /// SPARQL term type.
    #[serde(default)]
    pub r#type: String,
}

/// SPARQL JSON results document.
#[derive(Debug, Deserialize)]
pub struct SparqlResults {
    /// Result rows.
    pub results: SparqlResultList,
}

/// SPARQL result list.
#[derive(Debug, Deserialize)]
pub struct SparqlResultList {
    /// Variable bindings per row.
    pub bindings: Vec<HashMap<String, SparqlBinding>>,
}

fn local_name(value: &str) -> String {
    value
        .rsplit(['#', '/', ':'])
        .find(|part| !part.is_empty())
        .unwrap_or(value)
        .trim_end_matches("Noun")
        .to_ascii_lowercase()
}

fn rust_identifier(value: &str) -> String {
    let mut rendered = String::new();
    let mut previous_separator = false;
    for character in value.chars() {
        if character.is_ascii_alphanumeric() || character == '_' {
            if rendered.is_empty() && character.is_ascii_digit() {
                rendered.push('_');
            }
            rendered.push(character.to_ascii_lowercase());
            previous_separator = false;
        } else if !previous_separator && !rendered.is_empty() {
            rendered.push('_');
            previous_separator = true;
        }
    }
    while rendered.ends_with('_') {
        rendered.pop();
    }
    if rendered.is_empty() {
        "unnamed".to_string()
    } else {
        rendered
    }
}

fn rust_literal(value: &str) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "\"invalid-string\"".to_string())
}

fn adapter_name(verb: &RdfVerbDefinition) -> String {
    verb.noun_name.as_ref().map_or_else(
        || rust_identifier(&verb.name),
        |noun| format!("{}_{}", rust_identifier(noun), rust_identifier(&verb.name)),
    )
}

fn argument_type(argument: &RdfArgumentDefinition) -> String {
    if argument.is_flag {
        "bool".to_string()
    } else if argument.required {
        argument.value_type.clone()
    } else {
        format!("Option<{}>", argument.value_type)
    }
}

/// Project one RDF verb into a compiling interface adapter.
///
/// The adapter delegates to a same-named function in `crate::handlers`, keeping
/// generated interface code separate from authored domain behavior.
#[must_use]
pub fn rdf_spec_to_verb_code(verb: &RdfVerbDefinition) -> String {
    let mut code = String::new();
    let documentation = if verb.docstring.is_empty() {
        &verb.description
    } else {
        &verb.docstring
    };
    for line in documentation.lines() {
        code.push_str(&format!("/// {line}\n"));
    }

    let verb_literal = rust_literal(&verb.name);
    if let Some(noun) = verb.noun_name.as_deref() {
        code.push_str(&format!("#[verb({verb_literal}, {})]\n", rust_literal(noun)));
    } else {
        code.push_str(&format!("#[verb({verb_literal})]\n"));
    }

    let function_name = adapter_name(verb);
    let async_keyword = if verb.is_async { "async " } else { "" };
    code.push_str(&format!("pub {async_keyword}fn {function_name}(\n"));

    let argument_names = if verb.arguments.is_empty() {
        code.push_str("    args: VerbArgs,\n");
        vec!["args".to_string()]
    } else {
        verb.arguments
            .iter()
            .map(|argument| {
                let name = rust_identifier(&argument.name);
                code.push_str(&format!("    {name}: {},\n", argument_type(argument)));
                name
            })
            .collect()
    };

    let return_type = if verb.return_type.trim().is_empty() {
        "serde_json::Value"
    } else {
        verb.return_type.as_str()
    };
    code.push_str(&format!(") -> Result<{return_type}> {{\n"));
    let await_suffix = if verb.is_async { ".await" } else { "" };
    code.push_str(&format!(
        "    crate::handlers::{function_name}({}){await_suffix}\n",
        argument_names.join(", ")
    ));
    code.push_str("}\n");
    code
}

fn required_binding<'a>(
    binding: &'a HashMap<String, SparqlBinding>,
    name: &str,
) -> Result<&'a SparqlBinding, Box<dyn std::error::Error>> {
    binding.get(name).ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Missing ?{name} binding"),
        )
        .into()
    })
}

/// Convert SPARQL JSON results to canonical verb definitions.
pub fn sparql_results_to_verb_definitions(
    sparql_json: &str,
) -> Result<Vec<RdfVerbDefinition>, Box<dyn std::error::Error>> {
    let results: SparqlResults = serde_json::from_str(sparql_json)?;
    let mut verbs: HashMap<String, RdfVerbDefinition> = HashMap::new();

    for binding in results.results.bindings {
        let verb_uri = required_binding(&binding, "verb")?.value.clone();
        let name = required_binding(&binding, "verbName")?.value.clone();
        let verb = verbs.entry(verb_uri.clone()).or_insert_with(|| {
            let noun_uri = binding.get("noun").map(|item| item.value.clone());
            let noun_name = binding
                .get("nounName")
                .map(|item| item.value.clone())
                .or_else(|| noun_uri.as_deref().map(local_name));
            RdfVerbDefinition {
                verb_uri: verb_uri.clone(),
                name,
                description: binding
                    .get("verbAbout")
                    .map(|item| item.value.clone())
                    .unwrap_or_default(),
                noun_uri,
                noun_name,
                arguments: Vec::new(),
                return_type: binding
                    .get("returnType")
                    .map(|item| item.value.clone())
                    .filter(|value| !value.trim().is_empty())
                    .unwrap_or_else(|| "serde_json::Value".to_string()),
                trait_bounds: Vec::new(),
                docstring: binding
                    .get("docstring")
                    .map(|item| item.value.clone())
                    .unwrap_or_default(),
                is_async: binding
                    .get("isAsync")
                    .is_some_and(|item| item.value.eq_ignore_ascii_case("true")),
            }
        });

        if let Some(trait_binding) = binding.get("traitBound") {
            let trait_name = local_name(&trait_binding.value);
            if !verb.trait_bounds.contains(&trait_name) {
                verb.trait_bounds.push(trait_name);
                verb.trait_bounds.sort();
            }
        }
    }

    let mut ordered: Vec<_> = verbs.into_values().collect();
    ordered.sort_by(|left, right| left.verb_uri.cmp(&right.verb_uri));
    Ok(ordered)
}

fn recognized_predicate(predicate: &str) -> bool {
    [
        "hasVerbName",
        "verbAbout",
        "belongsToNoun",
        "returnType",
        "HasTraitBound",
        "docstring",
        "isAsync",
    ]
    .iter()
    .any(|name| predicate.contains(name))
}

/// Convert RDF triples to canonical verb definitions.
pub fn rdf_triples_to_verb_definitions(
    triples: Vec<RdfTriple>,
) -> Result<Vec<RdfVerbDefinition>, Box<dyn std::error::Error>> {
    let mut verb_map: HashMap<String, RdfVerbDefinition> = HashMap::new();

    for triple in triples {
        if !recognized_predicate(&triple.predicate) {
            continue;
        }
        let verb = verb_map.entry(triple.subject.clone()).or_insert_with(|| RdfVerbDefinition {
            verb_uri: triple.subject.clone(),
            name: String::new(),
            description: String::new(),
            noun_uri: None,
            noun_name: None,
            arguments: Vec::new(),
            return_type: "serde_json::Value".to_string(),
            trait_bounds: Vec::new(),
            docstring: String::new(),
            is_async: false,
        });

        match triple.predicate.as_str() {
            predicate if predicate.contains("hasVerbName") => {
                verb.name = triple.object.trim_matches('"').to_string();
            }
            predicate if predicate.contains("verbAbout") => {
                verb.description = triple.object.trim_matches('"').to_string();
            }
            predicate if predicate.contains("belongsToNoun") => {
                verb.noun_name = Some(local_name(&triple.object));
                verb.noun_uri = Some(triple.object);
            }
            predicate if predicate.contains("returnType") => {
                verb.return_type = triple.object.trim_matches('"').to_string();
            }
            predicate if predicate.contains("HasTraitBound") => {
                let trait_name = local_name(&triple.object);
                if !verb.trait_bounds.contains(&trait_name) {
                    verb.trait_bounds.push(trait_name);
                    verb.trait_bounds.sort();
                }
            }
            predicate if predicate.contains("docstring") => {
                verb.docstring = triple.object.trim_matches('"').to_string();
            }
            predicate if predicate.contains("isAsync") => {
                verb.is_async = triple.object.trim_matches('"').eq_ignore_ascii_case("true");
            }
            _ => {}
        }
    }

    let mut ordered: Vec<_> = verb_map.into_values().collect();
    for verb in &ordered {
        if verb.name.trim().is_empty() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Verb {} has no admitted name", verb.verb_uri),
            )
            .into());
        }
    }
    ordered.sort_by(|left, right| left.verb_uri.cmp(&right.verb_uri));
    Ok(ordered)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn load_verb() -> RdfVerbDefinition {
        RdfVerbDefinition {
            verb_uri: "ex:LoadGraphVerb".to_string(),
            name: "load".to_string(),
            description: "Load a graph from file".to_string(),
            noun_uri: Some("ex:GraphNoun".to_string()),
            noun_name: Some("graph".to_string()),
            arguments: vec![
                RdfArgumentDefinition {
                    arg_uri: "ex:PathArg".to_string(),
                    name: "path".to_string(),
                    description: "File path".to_string(),
                    value_type: "String".to_string(),
                    required: true,
                    is_flag: false,
                    default_value: None,
                    short_name: None,
                    long_name: None,
                    allowed_values: vec![],
                    argument_type: ArgumentType::Positional,
                },
                RdfArgumentDefinition {
                    arg_uri: "ex:FormatArg".to_string(),
                    name: "format".to_string(),
                    description: "File format".to_string(),
                    value_type: "String".to_string(),
                    required: false,
                    is_flag: false,
                    default_value: Some("ttl".to_string()),
                    short_name: Some('f'),
                    long_name: None,
                    allowed_values: vec!["ttl".to_string(), "nt".to_string()],
                    argument_type: ArgumentType::Optional,
                },
            ],
            return_type: "GraphLoadedOutput".to_string(),
            trait_bounds: vec!["Send".to_string(), "Sync".to_string()],
            docstring: "Load a graph from file or stdin".to_string(),
            is_async: false,
        }
    }

    #[test]
    fn projects_domain_handler_adapter() {
        let code = rdf_spec_to_verb_code(&load_verb());
        assert!(code.contains("#[verb(\"load\", \"graph\")]"));
        assert!(code.contains("pub fn graph_load("));
        assert!(code.contains("path: String,"));
        assert!(code.contains("format: Option<String>,"));
        assert!(code.contains("crate::handlers::graph_load(path, format)"));
    }

    #[test]
    fn projects_async_handler_adapter() {
        let mut verb = load_verb();
        verb.name = "query".to_string();
        verb.noun_name = Some("database".to_string());
        verb.arguments.clear();
        verb.return_type = "QueryResult".to_string();
        verb.is_async = true;
        let code = rdf_spec_to_verb_code(&verb);
        assert!(code.contains("pub async fn database_query("));
        assert!(code.contains("crate::handlers::database_query(args).await"));
    }

    #[test]
    fn sparql_projection_is_canonical() {
        let sparql_json = r#"{
            "results": {
                "bindings": [
                    {
                        "verb": {"type": "uri", "value": "http://example.org/ZVerb"},
                        "verbName": {"type": "literal", "value": "zeta"}
                    },
                    {
                        "verb": {"type": "uri", "value": "http://example.org/AVerb"},
                        "verbName": {"type": "literal", "value": "alpha"}
                    }
                ]
            }
        }"#;
        let verbs = sparql_results_to_verb_definitions(sparql_json).expect("valid SPARQL");
        assert_eq!(verbs.iter().map(|verb| verb.name.as_str()).collect::<Vec<_>>(), vec!["alpha", "zeta"]);
    }

    #[test]
    fn rdf_projection_admits_all_documented_predicates() {
        let subject = "ex:LoadGraphVerb";
        let triples = vec![
            RdfTriple { subject: subject.into(), predicate: "cnv:hasVerbName".into(), object: "load".into(), object_type: ObjectType::Literal },
            RdfTriple { subject: subject.into(), predicate: "cnv:verbAbout".into(), object: "Load graph".into(), object_type: ObjectType::Literal },
            RdfTriple { subject: subject.into(), predicate: "cnv:belongsToNoun".into(), object: "ex:GraphNoun".into(), object_type: ObjectType::Reference },
            RdfTriple { subject: subject.into(), predicate: "cnv:returnType".into(), object: "GraphLoadedOutput".into(), object_type: ObjectType::Literal },
            RdfTriple { subject: subject.into(), predicate: "cnv:isAsync".into(), object: "true".into(), object_type: ObjectType::Literal },
        ];
        let verbs = rdf_triples_to_verb_definitions(triples).expect("valid RDF projection");
        assert_eq!(verbs[0].noun_name.as_deref(), Some("graph"));
        assert_eq!(verbs[0].description, "Load graph");
        assert!(verbs[0].is_async);
    }
}
