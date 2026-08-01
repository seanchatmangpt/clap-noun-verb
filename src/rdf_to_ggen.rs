// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! RDF → typed noun-verb adapter projection.
//!
//! RDF owns semantic identity and interface shape. Generated wrappers own only
//! parsing/routing projection and delegate domain behavior to `crate::handlers`.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap};

const CNV: &str = "http://clap-noun-verb.io/ontology#";

/// RDF triple flattened from SPARQL or an RDF serialization.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RdfTriple {
    /// Subject URI.
    pub subject: String,
    /// Predicate URI.
    pub predicate: String,
    /// Literal value or referenced URI.
    pub object: String,
    /// Object classification.
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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
    /// Canonically ordered arguments.
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
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

fn resource_local_name(value: &str) -> &str {
    value
        .rsplit(|character| matches!(character, '#' | '/' | ':'))
        .find(|part| !part.is_empty())
        .unwrap_or(value)
}

fn noun_local_name(value: &str) -> String {
    resource_local_name(value)
        .trim_end_matches("Noun")
        .to_ascii_lowercase()
}

fn rust_identifier(value: &str) -> String {
    const KEYWORDS: &[&str] = &[
        "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else",
        "enum", "extern", "false", "fn", "for", "if", "impl", "in", "let", "loop",
        "match", "mod", "move", "mut", "pub", "ref", "return", "self", "static",
        "struct", "super", "trait", "true", "type", "unsafe", "use", "where", "while",
    ];

    let mut rendered = String::new();
    let mut separator = false;
    for character in value.chars() {
        if character.is_ascii_alphanumeric() || character == '_' {
            if rendered.is_empty() && character.is_ascii_digit() {
                rendered.push('_');
            }
            rendered.push(character.to_ascii_lowercase());
            separator = false;
        } else if !separator && !rendered.is_empty() {
            rendered.push('_');
            separator = true;
        }
    }
    while rendered.ends_with('_') {
        rendered.pop();
    }
    if rendered.is_empty() {
        rendered.push_str("unnamed");
    }
    if KEYWORDS.contains(&rendered.as_str()) {
        rendered.push('_');
    }
    rendered
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

fn rust_parameter_type(argument: &RdfArgumentDefinition) -> String {
    if argument.is_flag {
        "bool".to_string()
    } else if argument.required {
        argument.value_type.clone()
    } else {
        format!("Option<{}>", argument.value_type)
    }
}

fn projected_return_type(return_type: &str) -> String {
    let trimmed = return_type.trim();
    if trimmed.is_empty() {
        "Result<serde_json::Value>".to_string()
    } else if trimmed.starts_with("Result<")
        || trimmed.starts_with("clap_noun_verb::Result<")
        || trimmed.starts_with("crate::Result<")
        || trimmed.starts_with("std::result::Result<")
    {
        trimmed.to_string()
    } else {
        format!("Result<{trimmed}>")
    }
}

/// Project one RDF verb into a compiling interface adapter.
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
        code.push_str(&format!(
            "#[verb({verb_literal}, {})]\n",
            rust_literal(noun)
        ));
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
                code.push_str(&format!("    {name}: {},\n", rust_parameter_type(argument)));
                name
            })
            .collect()
    };

    code.push_str(&format!(
        ") -> {} {{\n",
        projected_return_type(&verb.return_type)
    ));
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
    let mut verbs: BTreeMap<String, RdfVerbDefinition> = BTreeMap::new();

    for binding in results.results.bindings {
        let verb_uri = required_binding(&binding, "verb")?.value.clone();
        let name = required_binding(&binding, "verbName")?.value.clone();
        let verb = verbs.entry(verb_uri.clone()).or_insert_with(|| {
            let noun_uri = binding.get("noun").map(|item| item.value.clone());
            let noun_name = binding
                .get("nounName")
                .map(|item| item.value.clone())
                .or_else(|| noun_uri.as_deref().map(noun_local_name));
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
        if let Some(bound) = binding.get("traitBound") {
            let name = resource_local_name(&bound.value).to_string();
            if !verb.trait_bounds.contains(&name) {
                verb.trait_bounds.push(name);
                verb.trait_bounds.sort();
            }
        }
    }
    Ok(verbs.into_values().collect())
}

#[derive(Debug, Clone, Default)]
struct ArgumentAccumulator {
    name: String,
    description: String,
    value_type: String,
    required: bool,
    default_value: Option<String>,
    short_name: Option<char>,
    long_name: Option<String>,
    allowed_values: BTreeSet<String>,
    argument_type: ArgumentType,
}

impl ArgumentAccumulator {
    fn finish(self, arg_uri: String) -> Result<RdfArgumentDefinition, Box<dyn std::error::Error>> {
        if self.name.trim().is_empty() || self.value_type.trim().is_empty() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Argument {arg_uri} requires an admitted name and value type"),
            )
            .into());
        }
        let is_flag = self.argument_type == ArgumentType::Flag || self.value_type == "bool";
        let required = self.required && !is_flag && self.argument_type != ArgumentType::Optional;
        Ok(RdfArgumentDefinition {
            arg_uri,
            name: self.name,
            description: self.description,
            value_type: self.value_type,
            required,
            is_flag,
            default_value: self.default_value,
            short_name: self.short_name,
            long_name: self.long_name,
            allowed_values: self.allowed_values.into_iter().collect(),
            argument_type: self.argument_type,
        })
    }
}

fn empty_verb(uri: String) -> RdfVerbDefinition {
    RdfVerbDefinition {
        verb_uri: uri,
        name: String::new(),
        description: String::new(),
        noun_uri: None,
        noun_name: None,
        arguments: Vec::new(),
        return_type: "serde_json::Value".to_string(),
        trait_bounds: Vec::new(),
        docstring: String::new(),
        is_async: false,
    }
}

fn parse_argument_type(value: &str) -> Result<ArgumentType, Box<dyn std::error::Error>> {
    match resource_local_name(value) {
        "Positional" => Ok(ArgumentType::Positional),
        "Optional" => Ok(ArgumentType::Optional),
        "Flag" => Ok(ArgumentType::Flag),
        "Repeating" => Ok(ArgumentType::Repeating),
        "Variadic" => Ok(ArgumentType::Variadic),
        other => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Unknown argument type: {other}"),
        )
        .into()),
    }
}

/// Convert an RDF graph into canonical verb definitions with closed argument edges.
pub fn rdf_triples_to_verb_definitions(
    triples: Vec<RdfTriple>,
) -> Result<Vec<RdfVerbDefinition>, Box<dyn std::error::Error>> {
    let mut verbs: BTreeMap<String, RdfVerbDefinition> = BTreeMap::new();
    let mut arguments: BTreeMap<String, ArgumentAccumulator> = BTreeMap::new();
    let mut links: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();

    for triple in triples {
        let predicate = resource_local_name(&triple.predicate);
        match predicate {
            "hasVerbName" => {
                verbs
                    .entry(triple.subject.clone())
                    .or_insert_with(|| empty_verb(triple.subject))
                    .name = triple.object;
            }
            "verbAbout" => {
                verbs
                    .entry(triple.subject.clone())
                    .or_insert_with(|| empty_verb(triple.subject))
                    .description = triple.object;
            }
            "belongsToNoun" => {
                let verb = verbs
                    .entry(triple.subject.clone())
                    .or_insert_with(|| empty_verb(triple.subject));
                verb.noun_name = Some(noun_local_name(&triple.object));
                verb.noun_uri = Some(triple.object);
            }
            "returnType" => {
                verbs
                    .entry(triple.subject.clone())
                    .or_insert_with(|| empty_verb(triple.subject))
                    .return_type = triple.object;
            }
            "HasTraitBound" => {
                let verb = verbs
                    .entry(triple.subject.clone())
                    .or_insert_with(|| empty_verb(triple.subject));
                let bound = resource_local_name(&triple.object).to_string();
                if !verb.trait_bounds.contains(&bound) {
                    verb.trait_bounds.push(bound);
                    verb.trait_bounds.sort();
                }
            }
            "docstring" => {
                verbs
                    .entry(triple.subject.clone())
                    .or_insert_with(|| empty_verb(triple.subject))
                    .docstring = triple.object;
            }
            "isAsync" => {
                verbs
                    .entry(triple.subject.clone())
                    .or_insert_with(|| empty_verb(triple.subject))
                    .is_async = triple.object.eq_ignore_ascii_case("true");
            }
            "hasArguments" => {
                verbs
                    .entry(triple.subject.clone())
                    .or_insert_with(|| empty_verb(triple.subject.clone()));
                links.entry(triple.subject).or_default().insert(triple.object);
            }
            "hasArgumentName" => {
                arguments.entry(triple.subject).or_default().name = triple.object;
            }
            "argumentAbout" => {
                arguments.entry(triple.subject).or_default().description = triple.object;
            }
            "valueType" => {
                arguments.entry(triple.subject).or_default().value_type = triple.object;
            }
            "required" => {
                arguments.entry(triple.subject).or_default().required =
                    triple.object.eq_ignore_ascii_case("true");
            }
            "argumentType" => {
                arguments.entry(triple.subject).or_default().argument_type =
                    parse_argument_type(&triple.object)?;
            }
            "defaultValue" => {
                arguments.entry(triple.subject).or_default().default_value = Some(triple.object);
            }
            "shortName" => {
                let mut characters = triple.object.chars();
                let short = characters.next().ok_or_else(|| {
                    std::io::Error::new(std::io::ErrorKind::InvalidData, "shortName is empty")
                })?;
                if characters.next().is_some() {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "shortName must contain one character",
                    )
                    .into());
                }
                arguments.entry(triple.subject).or_default().short_name = Some(short);
            }
            "longName" => {
                arguments.entry(triple.subject).or_default().long_name = Some(triple.object);
            }
            "allowedValue" => {
                arguments
                    .entry(triple.subject)
                    .or_default()
                    .allowed_values
                    .insert(triple.object);
            }
            _ => {}
        }
    }

    for (verb_uri, argument_uris) in links {
        let verb = verbs.get_mut(&verb_uri).ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Argument links reference missing verb: {verb_uri}"),
            )
        })?;
        for argument_uri in argument_uris {
            let accumulator = arguments.remove(&argument_uri).ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Verb {verb_uri} references missing argument: {argument_uri}"),
                )
            })?;
            verb.arguments.push(accumulator.finish(argument_uri)?);
        }
        verb.arguments
            .sort_by(|left, right| left.arg_uri.cmp(&right.arg_uri));
    }

    if let Some(orphan) = arguments.keys().next() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Argument is not linked to an admitted verb: {orphan}"),
        )
        .into());
    }

    for verb in verbs.values() {
        if verb.name.trim().is_empty() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Verb {} has no admitted name", verb.verb_uri),
            )
            .into());
        }
    }
    Ok(verbs.into_values().collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn load_verb() -> RdfVerbDefinition {
        RdfVerbDefinition {
            verb_uri: "https://example.org/LoadGraphVerb".to_string(),
            name: "load".to_string(),
            description: "Load a graph from file".to_string(),
            noun_uri: Some("https://example.org/GraphNoun".to_string()),
            noun_name: Some("graph".to_string()),
            arguments: vec![RdfArgumentDefinition {
                arg_uri: "https://example.org/PathArg".to_string(),
                name: "path".to_string(),
                description: "File path".to_string(),
                value_type: "String".to_string(),
                required: true,
                is_flag: false,
                default_value: None,
                short_name: None,
                long_name: None,
                allowed_values: Vec::new(),
                argument_type: ArgumentType::Positional,
            }],
            return_type: "GraphLoadedOutput".to_string(),
            trait_bounds: vec!["Send".to_string()],
            docstring: "Load a graph from file or stdin".to_string(),
            is_async: false,
        }
    }

    #[test]
    fn projects_domain_handler_adapter() {
        let code = rdf_spec_to_verb_code(&load_verb());
        assert!(code.contains("#[verb(\"load\", \"graph\")]"));
        assert!(code.contains("crate::handlers::graph_load(path)"));
        assert!(code.contains("-> Result<GraphLoadedOutput>"));
    }

    #[test]
    fn preserves_existing_result_carrier() {
        let mut verb = load_verb();
        verb.return_type = "Result<GraphLoadedOutput>".to_string();
        let code = rdf_spec_to_verb_code(&verb);
        assert!(code.contains("-> Result<GraphLoadedOutput>"));
        assert!(!code.contains("Result<Result<"));
    }

    #[test]
    fn sanitizes_reserved_identifiers() {
        let mut verb = load_verb();
        verb.name = "type".to_string();
        verb.noun_name = None;
        let code = rdf_spec_to_verb_code(&verb);
        assert!(code.contains("pub fn type_("));
        assert!(code.contains("crate::handlers::type_(path)"));
    }

    #[test]
    fn sparql_projection_is_canonical() {
        let sparql_json = r#"{
            "results": {"bindings": [
                {"verb": {"value": "https://example.org/Z"}, "verbName": {"value": "zeta"}},
                {"verb": {"value": "https://example.org/A"}, "verbName": {"value": "alpha"}}
            ]}
        }"#;
        let verbs = sparql_results_to_verb_definitions(sparql_json).expect("valid SPARQL");
        assert_eq!(
            verbs
                .iter()
                .map(|verb| verb.name.as_str())
                .collect::<Vec<_>>(),
            vec!["alpha", "zeta"]
        );
    }

    #[test]
    fn closes_argument_edges() {
        let verb = "https://example.org/LoadGraphVerb";
        let argument = "https://example.org/PathArg";
        let triples = vec![
            RdfTriple { subject: verb.into(), predicate: format!("{CNV}hasVerbName"), object: "load".into(), object_type: ObjectType::Literal },
            RdfTriple { subject: verb.into(), predicate: format!("{CNV}hasArguments"), object: argument.into(), object_type: ObjectType::Reference },
            RdfTriple { subject: argument.into(), predicate: format!("{CNV}hasArgumentName"), object: "path".into(), object_type: ObjectType::Literal },
            RdfTriple { subject: argument.into(), predicate: format!("{CNV}valueType"), object: "String".into(), object_type: ObjectType::Literal },
            RdfTriple { subject: argument.into(), predicate: format!("{CNV}required"), object: "true".into(), object_type: ObjectType::Literal },
            RdfTriple { subject: argument.into(), predicate: format!("{CNV}argumentType"), object: format!("{CNV}Positional"), object_type: ObjectType::Reference },
        ];
        let verbs = rdf_triples_to_verb_definitions(triples).expect("closed graph");
        assert_eq!(verbs[0].arguments.len(), 1);
        assert_eq!(verbs[0].arguments[0].name, "path");
    }

    #[test]
    fn incomplete_argument_refuses_graph() {
        let verb = "https://example.org/LoadGraphVerb";
        let argument = "https://example.org/PathArg";
        let triples = vec![
            RdfTriple { subject: verb.into(), predicate: format!("{CNV}hasVerbName"), object: "load".into(), object_type: ObjectType::Literal },
            RdfTriple { subject: verb.into(), predicate: format!("{CNV}hasArguments"), object: argument.into(), object_type: ObjectType::Reference },
            RdfTriple { subject: argument.into(), predicate: format!("{CNV}hasArgumentName"), object: "path".into(), object_type: ObjectType::Literal },
        ];
        assert!(rdf_triples_to_verb_definitions(triples).is_err());
    }

    #[test]
    fn orphan_argument_refuses_graph() {
        let argument = "https://example.org/PathArg";
        let triples = vec![
            RdfTriple { subject: argument.into(), predicate: format!("{CNV}hasArgumentName"), object: "path".into(), object_type: ObjectType::Literal },
            RdfTriple { subject: argument.into(), predicate: format!("{CNV}valueType"), object: "String".into(), object_type: ObjectType::Literal },
        ];
        assert!(rdf_triples_to_verb_definitions(triples).is_err());
    }
}
