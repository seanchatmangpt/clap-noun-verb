// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Deterministic synchronization between Rust noun-verb adapters and RDF authority.
//!
//! Observation and planning are side-effect free. `apply_sync` is the explicit
//! filesystem actuator and persists a machine-readable receipt for every batch.

use crate::ggen_to_rdf::{parse_rust_source, verb_definitions_to_ntriples};
use crate::rdf_to_ggen::{
    rdf_triples_to_verb_definitions, ObjectType, RdfTriple, RdfVerbDefinition,
};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// Synchronization operation required for one verb.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum SyncOperation {
    /// Verb exists in code but not in RDF.
    AddToOntology,
    /// Verb exists in both but differs.
    UpdateOntology,
    /// Verb exists in RDF but not in code.
    RemoveFromOntology,
    /// Verb definitions match.
    NoChange,
}

/// One deterministic synchronization decision.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VerbSyncEntry {
    /// Verb name.
    pub verb_name: String,
    /// Parent noun, when present.
    pub noun: Option<String>,
    /// Required operation.
    pub operation: SyncOperation,
    /// Definition parsed from Rust.
    pub code_version: Option<RdfVerbDefinition>,
    /// Definition parsed from RDF.
    pub rdf_version: Option<RdfVerbDefinition>,
    /// Canonically ordered field differences.
    pub differences: Vec<String>,
}

/// Complete bounded synchronization plan.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SyncResult {
    /// Stable observation identifier. The legacy field name is retained for compatibility.
    pub timestamp: String,
    /// Number of distinct verb identities observed across both surfaces.
    pub total_verbs: usize,
    /// Canonically ordered decisions.
    pub changes: Vec<VerbSyncEntry>,
    /// Aggregate plan verdict.
    pub summary: SyncSummary,
}

/// Aggregate synchronization verdict.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SyncSummary {
    /// Verbs absent from RDF.
    pub added: usize,
    /// Verbs whose definitions differ.
    pub modified: usize,
    /// Verbs absent from Rust.
    pub removed: usize,
    /// Verbs with equivalent definitions.
    pub unchanged: usize,
    /// True only when no mutation is required.
    pub conformant: bool,
}

/// Synchronization engine. Planning and actuation remain explicit operations.
#[derive(Debug, Clone)]
pub struct OntologySync {
    /// Rust source files admitted for observation.
    pub source_paths: Vec<PathBuf>,
    /// Managed RDF ontology directory.
    pub ontology_path: PathBuf,
}

impl OntologySync {
    /// Create a synchronization engine.
    #[must_use]
    pub fn new(source_paths: Vec<PathBuf>, ontology_path: PathBuf) -> Self {
        Self { source_paths, ontology_path }
    }

    /// Observe both surfaces and manufacture a deterministic plan.
    pub async fn sync_ggen_with_ontology(&self) -> Result<SyncResult, SyncError> {
        let code_verbs = self.load_verbs_from_code().await?;
        let rdf_verbs = self.load_verbs_from_ontology().await?;
        let changes = compute_diff(&code_verbs, &rdf_verbs)?;
        let summary = summarize_changes(&changes);
        Ok(SyncResult { timestamp: observation_id(), total_verbs: changes.len(), changes, summary })
    }

    async fn load_verbs_from_code(&self) -> Result<Vec<RdfVerbDefinition>, SyncError> {
        let mut paths = self.source_paths.clone();
        paths.sort();
        let mut verbs = Vec::new();
        for path in paths {
            let content = tokio::fs::read_to_string(&path)
                .await
                .map_err(|error| SyncError::IoError(format!("{}: {error}", path.display())))?;
            let mut parsed = parse_rust_source(&content)
                .map_err(|error| SyncError::ParseError(format!("{}: {error}", path.display())))?;
            verbs.append(&mut parsed);
        }
        canonicalize_unique(verbs, "Rust")
    }

    async fn load_verbs_from_ontology(&self) -> Result<Vec<RdfVerbDefinition>, SyncError> {
        let mut reader = tokio::fs::read_dir(&self.ontology_path)
            .await
            .map_err(|error| SyncError::IoError(error.to_string()))?;
        let mut paths = Vec::new();
        while let Some(entry) =
            reader.next_entry().await.map_err(|error| SyncError::IoError(error.to_string()))?
        {
            let path = entry.path();
            if path.extension().is_some_and(|extension| extension == "nt") {
                paths.push(path);
            }
        }
        paths.sort();

        let mut verbs = Vec::new();
        for path in paths {
            let content = tokio::fs::read_to_string(&path)
                .await
                .map_err(|error| SyncError::IoError(format!("{}: {error}", path.display())))?;
            let triples = parse_ntriples(&content)?;
            let mut parsed = rdf_triples_to_verb_definitions(triples)
                .map_err(|error| SyncError::RdfError(format!("{}: {error}", path.display())))?;
            verbs.append(&mut parsed);
        }
        canonicalize_unique(verbs, "RDF")
    }

    /// Apply a complete plan and persist `.ontology-sync-receipt.json`.
    ///
    /// Files are staged before mutation. A second identical application records
    /// `replay_verified=true` because all declared consequences already match.
    pub async fn apply_sync(&self, result: &SyncResult) -> Result<(), SyncError> {
        let expected_summary = summarize_changes(&result.changes);
        if expected_summary != result.summary || result.total_verbs != result.changes.len() {
            return Err(SyncError::ConformanceError(
                "sync result summary does not match its decisions".to_string(),
            ));
        }

        tokio::fs::create_dir_all(&self.ontology_path)
            .await
            .map_err(|error| SyncError::IoError(error.to_string()))?;

        let desired = desired_files(result)?;
        let managed_existing = managed_files(&self.ontology_path).await?;
        let replay_verified =
            consequences_match(&self.ontology_path, &desired, &managed_existing).await?;
        let desired_names: BTreeSet<String> = desired.keys().cloned().collect();
        let stale: Vec<String> = managed_existing.difference(&desired_names).cloned().collect();

        let stage = self.ontology_path.join(".ontology-sync-stage");
        if tokio::fs::try_exists(&stage)
            .await
            .map_err(|error| SyncError::IoError(error.to_string()))?
        {
            tokio::fs::remove_dir_all(&stage)
                .await
                .map_err(|error| SyncError::IoError(error.to_string()))?;
        }
        tokio::fs::create_dir_all(&stage)
            .await
            .map_err(|error| SyncError::IoError(error.to_string()))?;

        let mut written_receipts = Vec::new();
        for (name, content) in &desired {
            tokio::fs::write(stage.join(name), content)
                .await
                .map_err(|error| SyncError::IoError(error.to_string()))?;
            written_receipts.push(FileReceipt {
                path: name.clone(),
                bytes: content.len(),
                digest: fnv1a64(content.as_bytes()),
            });
        }

        let receipt = SyncReceipt {
            schema_version: "1.0.0".to_string(),
            observation: result.timestamp.clone(),
            admission: "ADMITTED".to_string(),
            standing: "PARTIAL_ALIVE".to_string(),
            actuation_performed: !replay_verified,
            replay_verified,
            written: written_receipts,
            removed: stale.clone(),
        };
        let receipt_bytes = serde_json::to_vec_pretty(&receipt)
            .map_err(|error| SyncError::ReceiptError(error.to_string()))?;
        tokio::fs::write(stage.join("receipt.json"), &receipt_bytes)
            .await
            .map_err(|error| SyncError::IoError(error.to_string()))?;

        for name in desired.keys() {
            replace_file(&stage.join(name), &self.ontology_path.join(name)).await?;
        }
        for name in &stale {
            let path = self.ontology_path.join(name);
            if tokio::fs::try_exists(&path)
                .await
                .map_err(|error| SyncError::IoError(error.to_string()))?
            {
                tokio::fs::remove_file(path)
                    .await
                    .map_err(|error| SyncError::IoError(error.to_string()))?;
            }
        }
        replace_file(
            &stage.join("receipt.json"),
            &self.ontology_path.join(".ontology-sync-receipt.json"),
        )
        .await?;
        tokio::fs::remove_dir_all(stage)
            .await
            .map_err(|error| SyncError::IoError(error.to_string()))?;
        Ok(())
    }
}

fn observation_id() -> String {
    std::env::var("SOURCE_DATE_EPOCH")
        .map(|value| format!("source-date-epoch:{value}"))
        .unwrap_or_else(|_| "UNSPECIFIED".to_string())
}

fn identity(verb: &RdfVerbDefinition) -> (String, Option<String>) {
    (verb.name.clone(), verb.noun_name.clone())
}

fn canonicalize_unique(
    verbs: Vec<RdfVerbDefinition>,
    source: &str,
) -> Result<Vec<RdfVerbDefinition>, SyncError> {
    let mut canonical = BTreeMap::new();
    for verb in verbs {
        let key = identity(&verb);
        if canonical.insert(key.clone(), verb).is_some() {
            return Err(SyncError::ConformanceError(format!(
                "duplicate {source} verb identity: {:?}",
                key
            )));
        }
    }
    Ok(canonical.into_values().collect())
}

fn compute_diff(
    code_verbs: &[RdfVerbDefinition],
    rdf_verbs: &[RdfVerbDefinition],
) -> Result<Vec<VerbSyncEntry>, SyncError> {
    let code = canonicalize_unique(code_verbs.to_vec(), "Rust")?
        .into_iter()
        .map(|verb| (identity(&verb), verb))
        .collect::<BTreeMap<_, _>>();
    let rdf = canonicalize_unique(rdf_verbs.to_vec(), "RDF")?
        .into_iter()
        .map(|verb| (identity(&verb), verb))
        .collect::<BTreeMap<_, _>>();
    let keys: BTreeSet<_> = code.keys().chain(rdf.keys()).cloned().collect();
    let mut changes = Vec::new();

    for key in keys {
        let code_version = code.get(&key);
        let rdf_version = rdf.get(&key);
        let (operation, differences) = match (code_version, rdf_version) {
            (Some(_), None) => {
                (SyncOperation::AddToOntology, vec!["Verb missing in RDF ontology".to_string()])
            }
            (None, Some(_)) => (
                SyncOperation::RemoveFromOntology,
                vec!["Verb exists in RDF but not in Rust".to_string()],
            ),
            (Some(code_verb), Some(rdf_verb)) => {
                let differences = compute_verb_differences(code_verb, rdf_verb);
                let operation = if differences.is_empty() {
                    SyncOperation::NoChange
                } else {
                    SyncOperation::UpdateOntology
                };
                (operation, differences)
            }
            (None, None) => {
                return Err(SyncError::ConformanceError(format!(
                    "verb identity disappeared during planning: {:?}",
                    key
                )));
            }
        };
        changes.push(VerbSyncEntry {
            verb_name: key.0,
            noun: key.1,
            operation,
            code_version: code_version.cloned(),
            rdf_version: rdf_version.cloned(),
            differences,
        });
    }
    Ok(changes)
}

fn summarize_changes(changes: &[VerbSyncEntry]) -> SyncSummary {
    let mut summary =
        SyncSummary { added: 0, modified: 0, removed: 0, unchanged: 0, conformant: false };
    for change in changes {
        match change.operation {
            SyncOperation::AddToOntology => summary.added += 1,
            SyncOperation::UpdateOntology => summary.modified += 1,
            SyncOperation::RemoveFromOntology => summary.removed += 1,
            SyncOperation::NoChange => summary.unchanged += 1,
        }
    }
    summary.conformant = summary.added == 0 && summary.modified == 0 && summary.removed == 0;
    summary
}

fn compute_verb_differences(code: &RdfVerbDefinition, rdf: &RdfVerbDefinition) -> Vec<String> {
    let mut differences = Vec::new();
    if code.name != rdf.name {
        differences.push(format!("name: Rust={:?}, RDF={:?}", code.name, rdf.name));
    }
    if code.description != rdf.description {
        differences
            .push(format!("description: Rust={:?}, RDF={:?}", code.description, rdf.description));
    }
    if code.noun_uri != rdf.noun_uri {
        differences.push(format!("noun_uri: Rust={:?}, RDF={:?}", code.noun_uri, rdf.noun_uri));
    }
    if code.noun_name != rdf.noun_name {
        differences.push(format!("noun_name: Rust={:?}, RDF={:?}", code.noun_name, rdf.noun_name));
    }
    if code.arguments != rdf.arguments {
        differences.push("arguments differ".to_string());
    }
    if code.return_type != rdf.return_type {
        differences
            .push(format!("return_type: Rust={:?}, RDF={:?}", code.return_type, rdf.return_type));
    }
    if code.trait_bounds != rdf.trait_bounds {
        differences.push(format!(
            "trait_bounds: Rust={:?}, RDF={:?}",
            code.trait_bounds, rdf.trait_bounds
        ));
    }
    if code.docstring != rdf.docstring {
        differences.push(format!("docstring: Rust={:?}, RDF={:?}", code.docstring, rdf.docstring));
    }
    if code.is_async != rdf.is_async {
        differences.push(format!("is_async: Rust={}, RDF={}", code.is_async, rdf.is_async));
    }
    differences
}

fn managed_filename(noun: Option<&str>) -> String {
    let raw = noun.unwrap_or("root");
    let mut safe = String::new();
    for character in raw.chars() {
        if character.is_ascii_alphanumeric() || matches!(character, '-' | '_') {
            safe.push(character.to_ascii_lowercase());
        } else {
            safe.push('_');
        }
    }
    if safe.is_empty() {
        safe.push_str("root");
    }
    format!("{safe}-verbs.nt")
}

fn desired_files(result: &SyncResult) -> Result<BTreeMap<String, String>, SyncError> {
    let mut groups: BTreeMap<Option<String>, Vec<RdfVerbDefinition>> = BTreeMap::new();
    for entry in &result.changes {
        if let Some(code) = &entry.code_version {
            groups.entry(entry.noun.clone()).or_default().push(code.clone());
        }
    }

    let mut files = BTreeMap::new();
    for (noun, mut verbs) in groups {
        verbs.sort_by(|left, right| left.verb_uri.cmp(&right.verb_uri));
        let name = managed_filename(noun.as_deref());
        if files.insert(name.clone(), verb_definitions_to_ntriples(&verbs)).is_some() {
            return Err(SyncError::ConformanceError(format!("managed filename collision: {name}")));
        }
    }
    Ok(files)
}

async fn managed_files(directory: &Path) -> Result<BTreeSet<String>, SyncError> {
    let mut files = BTreeSet::new();
    let mut reader = tokio::fs::read_dir(directory)
        .await
        .map_err(|error| SyncError::IoError(error.to_string()))?;
    while let Some(entry) =
        reader.next_entry().await.map_err(|error| SyncError::IoError(error.to_string()))?
    {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.ends_with("-verbs.nt") {
            files.insert(name);
        }
    }
    Ok(files)
}

async fn consequences_match(
    directory: &Path,
    desired: &BTreeMap<String, String>,
    existing: &BTreeSet<String>,
) -> Result<bool, SyncError> {
    let desired_names: BTreeSet<_> = desired.keys().cloned().collect();
    if &desired_names != existing {
        return Ok(false);
    }
    for (name, content) in desired {
        let actual = tokio::fs::read(directory.join(name))
            .await
            .map_err(|error| SyncError::IoError(error.to_string()))?;
        if actual != content.as_bytes() {
            return Ok(false);
        }
    }
    Ok(true)
}

async fn replace_file(staged: &Path, destination: &Path) -> Result<(), SyncError> {
    if tokio::fs::try_exists(destination)
        .await
        .map_err(|error| SyncError::IoError(error.to_string()))?
    {
        tokio::fs::remove_file(destination)
            .await
            .map_err(|error| SyncError::IoError(error.to_string()))?;
    }
    tokio::fs::rename(staged, destination)
        .await
        .map_err(|error| SyncError::IoError(error.to_string()))
}

fn parse_ntriples(content: &str) -> Result<Vec<RdfTriple>, SyncError> {
    let statement = Regex::new(r"^<([^>]*)>\s+<([^>]*)>\s+(.+?)\s+\.\s*$")
        .map_err(|error| SyncError::ParseError(error.to_string()))?;
    let mut triples = Vec::new();
    for (index, raw) in content.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let captures = statement.captures(line).ok_or_else(|| {
            SyncError::ParseError(format!("invalid N-Triples statement at line {}", index + 1))
        })?;
        let subject = captures[1].to_string();
        let predicate = captures[2].to_string();
        let token = captures[3].trim();
        let (object, object_type) = if token.starts_with('<') && token.ends_with('>') {
            (token[1..token.len() - 1].to_string(), ObjectType::Reference)
        } else if token.starts_with('"') {
            (parse_literal(token, index + 1)?, ObjectType::Literal)
        } else {
            return Err(SyncError::ParseError(format!(
                "unsupported N-Triples object at line {}",
                index + 1
            )));
        };
        triples.push(RdfTriple { subject, predicate, object, object_type });
    }
    Ok(triples)
}

fn parse_literal(token: &str, line: usize) -> Result<String, SyncError> {
    let mut escaped = false;
    let mut closing = None;
    for (index, character) in token.char_indices().skip(1) {
        if escaped {
            escaped = false;
        } else if character == '\\' {
            escaped = true;
        } else if character == '"' {
            closing = Some(index);
            break;
        }
    }
    let closing = closing
        .ok_or_else(|| SyncError::ParseError(format!("unterminated literal at line {line}")))?;
    let suffix = token[closing + 1..].trim();
    if !(suffix.is_empty() || suffix.starts_with('@') || suffix.starts_with("^^<")) {
        return Err(SyncError::ParseError(format!("invalid literal suffix at line {line}")));
    }
    unescape_literal(&token[1..closing], line)
}

fn unescape_literal(value: &str, line: usize) -> Result<String, SyncError> {
    let mut output = String::new();
    let mut characters = value.chars();
    while let Some(character) = characters.next() {
        if character != '\\' {
            output.push(character);
            continue;
        }
        let escaped = characters
            .next()
            .ok_or_else(|| SyncError::ParseError(format!("trailing escape at line {line}")))?;
        match escaped {
            '\\' => output.push('\\'),
            '"' => output.push('"'),
            'n' => output.push('\n'),
            'r' => output.push('\r'),
            't' => output.push('\t'),
            other => {
                return Err(SyncError::ParseError(format!(
                    "unsupported escape \\{other} at line {line}"
                )))
            }
        }
    }
    Ok(output)
}

fn fnv1a64(value: &[u8]) -> String {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in value {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("fnv1a64:{hash:016x}")
}

#[derive(Debug, Serialize)]
struct FileReceipt {
    path: String,
    bytes: usize,
    digest: String,
}

#[derive(Debug, Serialize)]
struct SyncReceipt {
    schema_version: String,
    observation: String,
    admission: String,
    standing: String,
    actuation_performed: bool,
    replay_verified: bool,
    written: Vec<FileReceipt>,
    removed: Vec<String>,
}

/// Errors produced by ontology planning or actuation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncError {
    /// Filesystem failure.
    IoError(String),
    /// Rust or N-Triples parse failure.
    ParseError(String),
    /// RDF graph admission failure.
    RdfError(String),
    /// Code and RDF cannot be reconciled lawfully.
    ConformanceError(String),
    /// Receipt manufacture failed.
    ReceiptError(String),
}

impl std::fmt::Display for SyncError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IoError(message) => write!(formatter, "IO error: {message}"),
            Self::ParseError(message) => write!(formatter, "Parse error: {message}"),
            Self::RdfError(message) => write!(formatter, "RDF error: {message}"),
            Self::ConformanceError(message) => {
                write!(formatter, "Conformance error: {message}")
            }
            Self::ReceiptError(message) => write!(formatter, "Receipt error: {message}"),
        }
    }
}

impl std::error::Error for SyncError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rdf_to_ggen::{ArgumentType, RdfArgumentDefinition};

    fn verb(name: &str) -> RdfVerbDefinition {
        RdfVerbDefinition {
            verb_uri: format!("http://example.org/{name}Verb"),
            name: name.to_string(),
            description: format!("{name} data"),
            noun_uri: Some("http://example.org/GraphNoun".to_string()),
            noun_name: Some("graph".to_string()),
            arguments: vec![RdfArgumentDefinition {
                arg_uri: format!("http://example.org/{name}PathArg"),
                name: "path".to_string(),
                description: String::new(),
                value_type: "String".to_string(),
                required: true,
                is_flag: false,
                default_value: None,
                short_name: None,
                long_name: None,
                allowed_values: Vec::new(),
                argument_type: ArgumentType::Positional,
            }],
            return_type: "Result<serde_json::Value>".to_string(),
            trait_bounds: Vec::new(),
            docstring: format!("{name} data"),
            is_async: false,
        }
    }

    #[test]
    fn changes_are_canonical_and_any_drift_is_nonconformant() {
        let code = vec![verb("zeta"), verb("alpha")];
        let changes = compute_diff(&code, &[]).expect("valid unique definitions");
        assert_eq!(
            changes.iter().map(|change| change.verb_name.as_str()).collect::<Vec<_>>(),
            vec!["alpha", "zeta"]
        );
        let summary = summarize_changes(&changes);
        assert_eq!(summary.added, 2);
        assert!(!summary.conformant);
    }

    #[test]
    fn duplicate_identities_refuse_planning() {
        let repeated = verb("load");
        assert!(compute_diff(&[repeated.clone(), repeated], &[]).is_err());
    }

    #[test]
    fn ntriples_round_trip_closes_arguments() {
        let original = verb("load");
        let serialized = verb_definitions_to_ntriples(std::slice::from_ref(&original));
        let triples = parse_ntriples(&serialized).expect("canonical N-Triples");
        let reconstructed = rdf_triples_to_verb_definitions(triples).expect("closed RDF graph");
        assert_eq!(reconstructed.len(), 1);
        assert_eq!(reconstructed[0].name, original.name);
        assert_eq!(reconstructed[0].arguments, original.arguments);
        assert_eq!(reconstructed[0].return_type, original.return_type);
    }

    #[test]
    fn malformed_ntriples_refuse_observation() {
        assert!(parse_ntriples("<subject> <predicate> missing-period").is_err());
    }

    #[test]
    fn deterministic_digest_is_stable() {
        assert_eq!(fnv1a64(b"receipt"), fnv1a64(b"receipt"));
        assert_ne!(fnv1a64(b"receipt"), fnv1a64(b"different"));
    }
}
