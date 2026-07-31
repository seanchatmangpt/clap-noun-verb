// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Rust noun-verb adapter → canonical RDF projection.
//!
//! Parsing is strict: malformed admitted attributes or signatures refuse the
//! complete projection. RDF and SPARQL outputs are canonically ordered and carry
//! no clock-derived fields, so identical source produces byte-identical output.

use crate::rdf_to_ggen::{ArgumentType, RdfArgumentDefinition, RdfVerbDefinition};
use regex::Regex;
use std::collections::BTreeSet;

const CNV: &str = "http://clap-noun-verb.io/ontology#";
const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
const XSD_BOOLEAN: &str = "http://www.w3.org/2001/XMLSchema#boolean";

#[derive(Debug, Clone, PartialEq, Eq)]
struct VerbAttribute {
    verb: String,
    noun: Option<String>,
}

/// Parse Rust source containing `#[verb]` interface adapters.
pub fn parse_rust_source(source: &str) -> Result<Vec<RdfVerbDefinition>, ParseError> {
    let lines: Vec<&str> = source.lines().collect();
    let mut verbs = Vec::new();
    let mut index = 0;

    while index < lines.len() {
        let mut documentation = Vec::new();
        while index < lines.len() && lines[index].trim().starts_with("///") {
            documentation.push(
                lines[index]
                    .trim()
                    .trim_start_matches("///")
                    .trim()
                    .to_string(),
            );
            index += 1;
        }

        if index >= lines.len() || !lines[index].contains("#[verb(") {
            index += 1;
            continue;
        }

        let attribute = extract_verb_attribute(lines[index])?;
        index += 1;
        let mut signature = String::new();
        let mut found_body = false;
        while index < lines.len() {
            signature.push_str(lines[index].trim());
            signature.push(' ');
            if lines[index].contains('{') {
                found_body = true;
                index += 1;
                break;
            }
            index += 1;
        }
        if !found_body {
            return Err(ParseError::InvalidSignature);
        }

        let mut verb = parse_function_signature(&signature, &attribute)?;
        verb.docstring = documentation.join(" ");
        verb.description = verb.docstring.clone();
        verbs.push(verb);
    }

    verbs.sort_by(|left, right| left.verb_uri.cmp(&right.verb_uri));
    Ok(verbs)
}

fn extract_verb_attribute(line: &str) -> Result<VerbAttribute, ParseError> {
    let expression = Regex::new(
        r#"#\[verb\(\s*\"([^\"]+)\"(?:\s*,\s*\"([^\"]+)\")?\s*\)\]"#,
    )
    .map_err(|_| ParseError::InvalidAttribute)?;
    let captures = expression.captures(line).ok_or(ParseError::InvalidAttribute)?;
    let verb = captures
        .get(1)
        .map(|value| value.as_str().to_string())
        .ok_or(ParseError::InvalidAttribute)?;
    let noun = captures.get(2).map(|value| value.as_str().to_string());
    Ok(VerbAttribute { verb, noun })
}

fn parse_function_signature(
    signature: &str,
    attribute: &VerbAttribute,
) -> Result<RdfVerbDefinition, ParseError> {
    let normalized = signature.split_whitespace().collect::<Vec<_>>().join(" ");
    let function_expression =
        Regex::new(r"(?:pub\s+)?(?:async\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(")
            .map_err(|_| ParseError::InvalidSignature)?;
    let function_name = function_expression
        .captures(&normalized)
        .and_then(|captures| captures.get(1))
        .map(|value| value.as_str())
        .ok_or(ParseError::InvalidSignature)?;

    let noun_name = attribute.noun.clone().or_else(|| {
        function_name
            .split_once('_')
            .map(|(noun, _)| noun.to_string())
            .filter(|noun| !noun.is_empty())
    });
    let return_type = extract_return_type(&normalized).ok_or(ParseError::InvalidReturnType)?;
    let arguments = extract_parameters(&normalized)?;

    Ok(RdfVerbDefinition {
        verb_uri: format!("http://example.org/{}Verb", pascal_case(&attribute.verb)),
        name: attribute.verb.clone(),
        description: String::new(),
        noun_uri: noun_name
            .as_ref()
            .map(|noun| format!("http://example.org/{}Noun", pascal_case(noun))),
        noun_name,
        arguments,
        return_type,
        trait_bounds: extract_trait_bounds(&normalized),
        docstring: String::new(),
        is_async: normalized.contains("async fn"),
    })
}

fn extract_return_type(signature: &str) -> Option<String> {
    let arrow = signature.find("->")?;
    let after_arrow = signature[arrow + 2..].trim();
    let body = after_arrow
        .rfind('{')
        .map_or(after_arrow, |index| &after_arrow[..index]);
    let return_type = body.trim();
    (!return_type.is_empty()).then(|| return_type.to_string())
}

fn matching_parenthesis(signature: &str, opening: usize) -> Option<usize> {
    let mut depth = 0_u32;
    for (offset, character) in signature[opening..].char_indices() {
        match character {
            '(' => depth += 1,
            ')' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(opening + offset);
                }
            }
            _ => {}
        }
    }
    None
}

fn split_top_level(value: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut start = 0;
    let mut angle = 0_i32;
    let mut round = 0_i32;
    let mut square = 0_i32;
    for (index, character) in value.char_indices() {
        match character {
            '<' => angle += 1,
            '>' => angle -= 1,
            '(' => round += 1,
            ')' => round -= 1,
            '[' => square += 1,
            ']' => square -= 1,
            ',' if angle == 0 && round == 0 && square == 0 => {
                parts.push(value[start..index].trim());
                start = index + 1;
            }
            _ => {}
        }
    }
    parts.push(value[start..].trim());
    parts.into_iter().filter(|part| !part.is_empty()).collect()
}

fn extract_parameters(signature: &str) -> Result<Vec<RdfArgumentDefinition>, ParseError> {
    let opening = signature.find('(').ok_or(ParseError::InvalidSignature)?;
    let closing = matching_parenthesis(signature, opening).ok_or(ParseError::InvalidSignature)?;
    let parameters = &signature[opening + 1..closing];
    let mut arguments = Vec::new();

    for parameter in split_top_level(parameters) {
        let parameter = parameter
            .trim_start_matches(|character: char| character == '&' || character.is_whitespace());
        if parameter.is_empty() || parameter == "args: VerbArgs" || parameter == "self" {
            continue;
        }
        let (name, value_type) = parameter
            .split_once(':')
            .ok_or(ParseError::MissingParameter)?;
        let name = name.trim().trim_start_matches("mut ");
        let value_type = value_type.trim();
        let optional = value_type.starts_with("Option<") && value_type.ends_with('>');
        let unwrapped = if optional {
            value_type[7..value_type.len() - 1].trim().to_string()
        } else {
            value_type.to_string()
        };
        let flag = unwrapped == "bool";
        arguments.push(RdfArgumentDefinition {
            arg_uri: format!("http://example.org/{}Arg", pascal_case(name)),
            name: name.to_string(),
            description: String::new(),
            value_type: unwrapped,
            required: !optional && !flag,
            is_flag: flag,
            default_value: None,
            short_name: None,
            long_name: None,
            allowed_values: Vec::new(),
            argument_type: if flag {
                ArgumentType::Flag
            } else if optional {
                ArgumentType::Optional
            } else {
                ArgumentType::Positional
            },
        });
    }
    Ok(arguments)
}

fn extract_trait_bounds(signature: &str) -> Vec<String> {
    ["Deserialize", "Send", "Serialize", "Sync"]
        .into_iter()
        .filter(|bound| signature.contains(bound))
        .map(str::to_string)
        .collect()
}

fn canonical_verbs(verbs: &[RdfVerbDefinition]) -> Vec<&RdfVerbDefinition> {
    let mut ordered: Vec<_> = verbs.iter().collect();
    ordered.sort_by(|left, right| left.verb_uri.cmp(&right.verb_uri));
    ordered
}

fn canonical_arguments(verb: &RdfVerbDefinition) -> Vec<&RdfArgumentDefinition> {
    let mut ordered: Vec<_> = verb.arguments.iter().collect();
    ordered.sort_by(|left, right| left.arg_uri.cmp(&right.arg_uri));
    ordered
}

fn canonical_allowed_values(argument: &RdfArgumentDefinition) -> BTreeSet<&str> {
    argument.allowed_values.iter().map(String::as_str).collect()
}

/// Emit canonical N-Triples for verb definitions.
#[must_use]
pub fn verb_definitions_to_ntriples(verbs: &[RdfVerbDefinition]) -> String {
    let mut output = String::from(
        "# Generated by clap-noun-verb ggen_to_rdf\n# Format: N-Triples (RDF 1.1)\n\n",
    );
    for verb in canonical_verbs(verbs) {
        output.push_str(&verb_triples(verb));
        output.push('\n');
    }
    output
}

fn triple(subject: &str, predicate: &str, object: &str) -> String {
    format!("<{subject}> <{predicate}> {object} .\n")
}

fn literal(value: &str) -> String {
    format!("\"{}\"", escape_rdf_string(value))
}

fn language_literal(value: &str) -> String {
    format!("{}@en", literal(value))
}

fn boolean_literal(value: bool) -> String {
    format!("\"{value}\"^^<{XSD_BOOLEAN}>")
}

fn verb_triples(verb: &RdfVerbDefinition) -> String {
    let mut output = String::new();
    output.push_str(&triple(&verb.verb_uri, RDF_TYPE, &format!("<{CNV}Verb>")));
    output.push_str(&triple(
        &verb.verb_uri,
        &format!("{CNV}hasVerbName"),
        &language_literal(&verb.name),
    ));
    if !verb.description.is_empty() {
        output.push_str(&triple(
            &verb.verb_uri,
            &format!("{CNV}verbAbout"),
            &language_literal(&verb.description),
        ));
    }
    if !verb.docstring.is_empty() && verb.docstring != verb.description {
        output.push_str(&triple(
            &verb.verb_uri,
            &format!("{CNV}docstring"),
            &language_literal(&verb.docstring),
        ));
    }
    if let Some(noun_uri) = &verb.noun_uri {
        output.push_str(&triple(
            &verb.verb_uri,
            &format!("{CNV}belongsToNoun"),
            &format!("<{noun_uri}>"),
        ));
    }
    output.push_str(&triple(
        &verb.verb_uri,
        &format!("{CNV}returnType"),
        &language_literal(&verb.return_type),
    ));
    if verb.is_async {
        output.push_str(&triple(
            &verb.verb_uri,
            &format!("{CNV}isAsync"),
            &boolean_literal(true),
        ));
    }

    for argument in canonical_arguments(verb) {
        output.push_str(&triple(
            &verb.verb_uri,
            &format!("{CNV}hasArguments"),
            &format!("<{}>", argument.arg_uri),
        ));
        output.push_str(&argument_triples(argument));
    }

    let bounds: BTreeSet<_> = verb.trait_bounds.iter().collect();
    for bound in bounds {
        output.push_str(&triple(
            &verb.verb_uri,
            &format!("{CNV}HasTraitBound"),
            &format!("<{CNV}{bound}>"),
        ));
    }
    output
}

fn argument_triples(argument: &RdfArgumentDefinition) -> String {
    let mut output = String::new();
    output.push_str(&triple(
        &argument.arg_uri,
        RDF_TYPE,
        &format!("<{CNV}Argument>"),
    ));
    output.push_str(&triple(
        &argument.arg_uri,
        &format!("{CNV}hasArgumentName"),
        &language_literal(&argument.name),
    ));
    if !argument.description.is_empty() {
        output.push_str(&triple(
            &argument.arg_uri,
            &format!("{CNV}argumentAbout"),
            &language_literal(&argument.description),
        ));
    }
    output.push_str(&triple(
        &argument.arg_uri,
        &format!("{CNV}valueType"),
        &language_literal(&argument.value_type),
    ));
    output.push_str(&triple(
        &argument.arg_uri,
        &format!("{CNV}required"),
        &boolean_literal(argument.required),
    ));
    output.push_str(&triple(
        &argument.arg_uri,
        &format!("{CNV}argumentType"),
        &format!("<{CNV}{:?}>", argument.argument_type),
    ));
    if let Some(default_value) = &argument.default_value {
        output.push_str(&triple(
            &argument.arg_uri,
            &format!("{CNV}defaultValue"),
            &language_literal(default_value),
        ));
    }
    if let Some(short_name) = argument.short_name {
        output.push_str(&triple(
            &argument.arg_uri,
            &format!("{CNV}shortName"),
            &language_literal(&short_name.to_string()),
        ));
    }
    if let Some(long_name) = &argument.long_name {
        output.push_str(&triple(
            &argument.arg_uri,
            &format!("{CNV}longName"),
            &language_literal(long_name),
        ));
    }
    for allowed in canonical_allowed_values(argument) {
        output.push_str(&triple(
            &argument.arg_uri,
            &format!("{CNV}allowedValue"),
            &language_literal(allowed),
        ));
    }
    output
}

/// Emit a canonical SPARQL `INSERT DATA` operation containing the same graph as N-Triples.
#[must_use]
pub fn verb_definitions_to_sparql_insert(verbs: &[RdfVerbDefinition]) -> String {
    let mut output = String::from(
        "PREFIX cnv: <http://clap-noun-verb.io/ontology#>\n\
         PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>\n\
         PREFIX xsd: <http://www.w3.org/2001/XMLSchema#>\n\n\
         INSERT DATA {\n",
    );
    for verb in canonical_verbs(verbs) {
        output.push_str(&format!("  <{}> a cnv:Verb .\n", verb.verb_uri));
        output.push_str(&format!(
            "  <{}> cnv:hasVerbName {}@en .\n",
            verb.verb_uri,
            literal(&verb.name)
        ));
        if !verb.description.is_empty() {
            output.push_str(&format!(
                "  <{}> cnv:verbAbout {}@en .\n",
                verb.verb_uri,
                literal(&verb.description)
            ));
        }
        if !verb.docstring.is_empty() && verb.docstring != verb.description {
            output.push_str(&format!(
                "  <{}> cnv:docstring {}@en .\n",
                verb.verb_uri,
                literal(&verb.docstring)
            ));
        }
        if let Some(noun_uri) = &verb.noun_uri {
            output.push_str(&format!(
                "  <{}> cnv:belongsToNoun <{}> .\n",
                verb.verb_uri, noun_uri
            ));
        }
        output.push_str(&format!(
            "  <{}> cnv:returnType {}@en .\n",
            verb.verb_uri,
            literal(&verb.return_type)
        ));
        if verb.is_async {
            output.push_str(&format!(
                "  <{}> cnv:isAsync \"true\"^^xsd:boolean .\n",
                verb.verb_uri
            ));
        }
        for bound in verb.trait_bounds.iter().collect::<BTreeSet<_>>() {
            output.push_str(&format!(
                "  <{}> cnv:HasTraitBound <{}{}> .\n",
                verb.verb_uri, CNV, bound
            ));
        }
        for argument in canonical_arguments(verb) {
            output.push_str(&format!(
                "  <{}> cnv:hasArguments <{}> .\n",
                verb.verb_uri, argument.arg_uri
            ));
            output.push_str(&argument_sparql(argument));
        }
    }
    output.push_str("}\n");
    output
}

fn argument_sparql(argument: &RdfArgumentDefinition) -> String {
    let mut output = String::new();
    output.push_str(&format!("  <{}> a cnv:Argument .\n", argument.arg_uri));
    output.push_str(&format!(
        "  <{}> cnv:hasArgumentName {}@en .\n",
        argument.arg_uri,
        literal(&argument.name)
    ));
    if !argument.description.is_empty() {
        output.push_str(&format!(
            "  <{}> cnv:argumentAbout {}@en .\n",
            argument.arg_uri,
            literal(&argument.description)
        ));
    }
    output.push_str(&format!(
        "  <{}> cnv:valueType {}@en .\n",
        argument.arg_uri,
        literal(&argument.value_type)
    ));
    output.push_str(&format!(
        "  <{}> cnv:required \"{}\"^^xsd:boolean .\n",
        argument.arg_uri, argument.required
    ));
    output.push_str(&format!(
        "  <{}> cnv:argumentType cnv:{:?} .\n",
        argument.arg_uri, argument.argument_type
    ));
    if let Some(default_value) = &argument.default_value {
        output.push_str(&format!(
            "  <{}> cnv:defaultValue {}@en .\n",
            argument.arg_uri,
            literal(default_value)
        ));
    }
    if let Some(short_name) = argument.short_name {
        output.push_str(&format!(
            "  <{}> cnv:shortName {}@en .\n",
            argument.arg_uri,
            literal(&short_name.to_string())
        ));
    }
    if let Some(long_name) = &argument.long_name {
        output.push_str(&format!(
            "  <{}> cnv:longName {}@en .\n",
            argument.arg_uri,
            literal(long_name)
        ));
    }
    for allowed in canonical_allowed_values(argument) {
        output.push_str(&format!(
            "  <{}> cnv:allowedValue {}@en .\n",
            argument.arg_uri,
            literal(allowed)
        ));
    }
    output
}

fn pascal_case(value: &str) -> String {
    value
        .split(|character: char| !character.is_ascii_alphanumeric())
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut characters = part.chars();
            characters.next().map_or_else(String::new, |first| {
                first.to_uppercase().collect::<String>() + characters.as_str()
            })
        })
        .collect()
}

fn escape_rdf_string(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

/// Errors produced while parsing Rust adapters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseError {
    /// The verb attribute is malformed.
    InvalidAttribute,
    /// The function signature could not be parsed.
    InvalidSignature,
    /// A required parameter declaration was malformed.
    MissingParameter,
    /// The return type could not be parsed.
    InvalidReturnType,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::InvalidAttribute => "Invalid verb attribute",
            Self::InvalidSignature => "Invalid function signature",
            Self::MissingParameter => "Missing parameter declaration",
            Self::InvalidReturnType => "Invalid return type",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for ParseError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn source() -> &'static str {
        r#"
/// Load a graph from file
#[verb("load", "graph")]
pub fn graph_load(path: String, format: Option<String>) -> Result<GraphLoadedOutput> {
    crate::handlers::graph_load(path, format)
}
"#
    }

    fn rich_verb() -> RdfVerbDefinition {
        RdfVerbDefinition {
            verb_uri: "http://example.org/LoadVerb".to_string(),
            name: "load".to_string(),
            description: "Load a graph".to_string(),
            noun_uri: Some("http://example.org/GraphNoun".to_string()),
            noun_name: Some("graph".to_string()),
            arguments: vec![RdfArgumentDefinition {
                arg_uri: "http://example.org/FormatArg".to_string(),
                name: "format".to_string(),
                description: "Input format".to_string(),
                value_type: "String".to_string(),
                required: false,
                is_flag: false,
                default_value: Some("ttl".to_string()),
                short_name: Some('f'),
                long_name: Some("format".to_string()),
                allowed_values: vec!["ttl".to_string(), "jsonld".to_string()],
                argument_type: ArgumentType::Optional,
            }],
            return_type: "Result<GraphLoadedOutput>".to_string(),
            trait_bounds: vec!["Sync".to_string(), "Send".to_string()],
            docstring: "Load an admitted graph".to_string(),
            is_async: true,
        }
    }

    #[test]
    fn parses_two_argument_attribute_and_signature() {
        let verbs = parse_rust_source(source()).expect("valid adapter source");
        assert_eq!(verbs.len(), 1);
        assert_eq!(verbs[0].name, "load");
        assert_eq!(verbs[0].noun_name.as_deref(), Some("graph"));
        assert_eq!(verbs[0].arguments.len(), 2);
        assert_eq!(verbs[0].return_type, "Result<GraphLoadedOutput>");
    }

    #[test]
    fn malformed_attribute_refuses_projection() {
        let result = parse_rust_source("#[verb()]\npub fn bad() -> Result<()> { Ok(()) }");
        assert_eq!(result, Err(ParseError::InvalidAttribute));
    }

    #[test]
    fn ntriples_are_byte_identical_and_canonical() {
        let verb = rich_verb();
        let first = verb_definitions_to_ntriples(std::slice::from_ref(&verb));
        let second = verb_definitions_to_ntriples(std::slice::from_ref(&verb));
        assert_eq!(first, second);
        assert!(!first.contains("Timestamp"));
        assert!(first.contains("defaultValue"));
        assert!(first.contains("shortName"));
        assert!(first.contains("longName"));
        assert!(first.contains("allowedValue"));
        assert!(first.find("jsonld").is_some_and(|left| {
            first.find("ttl").is_some_and(|right| left < right)
        }));
    }

    #[test]
    fn sparql_preserves_complete_metadata() {
        let sparql = verb_definitions_to_sparql_insert(&[rich_verb()]);
        assert!(sparql.contains("PREFIX xsd:"));
        assert!(sparql.contains("cnv:hasVerbName \"load\"@en"));
        assert!(sparql.contains("cnv:docstring"));
        assert!(sparql.contains("cnv:HasTraitBound"));
        assert!(sparql.contains("cnv:argumentType cnv:Optional"));
        assert!(sparql.contains("cnv:defaultValue \"ttl\"@en"));
        assert!(sparql.contains("cnv:shortName \"f\"@en"));
        assert!(sparql.contains("cnv:longName \"format\"@en"));
        assert!(sparql.contains("cnv:allowedValue \"jsonld\"@en"));
    }
}
