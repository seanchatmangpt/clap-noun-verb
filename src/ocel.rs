// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Always-on, zero-configuration OCEL 2.0 event logging.
//!
//! Every CLI invocation dispatched through the standard [`crate::cli::run`] entry
//! point (via `CommandRegistry::execute_verb` / `execute_root_verb`) is recorded,
//! with no per-verb opt-in, as one OCEL 2.0 `cli_invocation` event into a real
//! OCEL 2.0 JSON document on disk -- mirroring this crate's "telemetry is always
//! compiled, never feature-gated" precedent (see `src/telemetry.rs`, ADL-003).
//!
//! OCEL logging is a best-effort convenience layer: it must never fail or panic
//! the CLI invocation it is describing. Any I/O failure is downgraded to a
//! `log::warn!` and the invocation proceeds normally.

use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::{NounVerbError, Result};

// =============================================================================
// OCEL 2.0 data model (spec-exact field names and shape)
// =============================================================================

/// An OCEL 2.0 attribute type declaration, shared by object types and event types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeType {
    pub name: String,
    /// One of "string" | "integer" | "float" | "boolean" | "time".
    #[serde(rename = "type")]
    pub attr_type: String,
}

/// Declaration of an object type and the attributes its objects may carry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectType {
    pub name: String,
    #[serde(default)]
    pub attributes: Vec<AttributeType>,
}

/// Declaration of an event type and the attributes its events may carry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventType {
    pub name: String,
    #[serde(default)]
    pub attributes: Vec<AttributeType>,
}

/// A time-varying attribute value recorded on an object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectAttributeValue {
    pub name: String,
    pub time: String,
    pub value: serde_json::Value,
}

/// An OCEL 2.0 object (e.g. a "command" or a "process").
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcelObject {
    pub id: String,
    #[serde(rename = "type")]
    pub obj_type: String,
    #[serde(default)]
    pub attributes: Vec<ObjectAttributeValue>,
}

/// A (non-time-varying) attribute value recorded on an event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventAttributeValue {
    pub name: String,
    pub value: serde_json::Value,
}

/// A link from an event to one of the objects it concerns.
///
/// `objectId` is the spec-mandated camelCase key on the wire; keep the Rust
/// field snake_case and rename it explicitly via `#[serde(rename = ...)]`
/// (applies symmetrically to serialization and deserialization).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    #[serde(rename = "objectId")]
    pub object_id: String,
    pub qualifier: String,
}

/// An OCEL 2.0 event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcelEvent {
    pub id: String,
    #[serde(rename = "type")]
    pub event_type: String,
    pub time: String,
    #[serde(default)]
    pub attributes: Vec<EventAttributeValue>,
    #[serde(default)]
    pub relationships: Vec<Relationship>,
}

/// The full OCEL 2.0 JSON document.
#[derive(Debug, Clone)]
pub struct OcelDocument {
    pub object_types: Vec<ObjectType>,
    pub event_types: Vec<EventType>,
    pub objects: Vec<OcelObject>,
    pub events: Vec<OcelEvent>,
}

// Spec-exact top-level keys ("objectTypes"/"eventTypes") differ from the
// idiomatic Rust field names above; a manual Serialize/Deserialize keeps the
// wire format exactly as specified without a crate-wide rename attribute.
impl Serialize for OcelDocument {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut s = serializer.serialize_struct("OcelDocument", 4)?;
        s.serialize_field("objectTypes", &self.object_types)?;
        s.serialize_field("eventTypes", &self.event_types)?;
        s.serialize_field("objects", &self.objects)?;
        s.serialize_field("events", &self.events)?;
        s.end()
    }
}

impl<'de> Deserialize<'de> for OcelDocument {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Wire {
            #[serde(default, rename = "objectTypes")]
            object_types: Vec<ObjectType>,
            #[serde(default, rename = "eventTypes")]
            event_types: Vec<EventType>,
            #[serde(default)]
            objects: Vec<OcelObject>,
            #[serde(default)]
            events: Vec<OcelEvent>,
        }
        let wire = Wire::deserialize(deserializer)?;
        Ok(OcelDocument {
            object_types: wire.object_types,
            event_types: wire.event_types,
            objects: wire.objects,
            events: wire.events,
        })
    }
}

impl Default for OcelDocument {
    fn default() -> Self {
        Self {
            object_types: Vec::new(),
            event_types: Vec::new(),
            objects: Vec::new(),
            events: Vec::new(),
        }
    }
}

impl OcelDocument {
    /// An empty, spec-shaped document.
    pub fn empty() -> Self {
        Self::default()
    }

    fn ensure_object_type(&mut self, name: &str) {
        if !self.object_types.iter().any(|t| t.name == name) {
            self.object_types.push(ObjectType { name: name.to_string(), attributes: Vec::new() });
        }
    }

    fn ensure_event_type(&mut self, name: &str) {
        if !self.event_types.iter().any(|t| t.name == name) {
            self.event_types.push(EventType { name: name.to_string(), attributes: Vec::new() });
        }
    }

    fn ensure_object(&mut self, id: &str, obj_type: &str) {
        if !self.objects.iter().any(|o| o.id == id) {
            self.objects.push(OcelObject {
                id: id.to_string(),
                obj_type: obj_type.to_string(),
                attributes: Vec::new(),
            });
        }
    }
}

// =============================================================================
// Identifier generation (no new dependency: mirrors telemetry.rs's approach)
// =============================================================================

fn now_rfc3339() -> String {
    chrono::Utc::now().to_rfc3339()
}

fn generate_event_id() -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let count = COUNTER.fetch_add(1, Ordering::SeqCst);
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    format!("evt-{nanos:x}-{count:x}")
}

/// A single, process-lifetime-stable id for the OS process object, derived from
/// the PID plus the process's own start timestamp.
fn process_object_id() -> &'static str {
    static PROCESS_ID: OnceLock<String> = OnceLock::new();
    PROCESS_ID.get_or_init(|| {
        let pid = std::process::id();
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
        format!("proc-{pid}-{nanos:x}")
    })
}

// =============================================================================
// Path resolution
// =============================================================================

const ENV_PATH: &str = "CLAP_NOUN_VERB_OCEL_PATH";
const DEFAULT_RELATIVE_PATH: &str = ".clap-noun-verb/ocel.json";
const FALLBACK_FILE_NAME: &str = "clap-noun-verb-ocel.json";

/// Resolve the primary OCEL document path: `CLAP_NOUN_VERB_OCEL_PATH` if set,
/// else `.clap-noun-verb/ocel.json` relative to the current working directory.
pub fn primary_path() -> PathBuf {
    if let Ok(path) = std::env::var(ENV_PATH) {
        return PathBuf::from(path);
    }
    PathBuf::from(DEFAULT_RELATIVE_PATH)
}

/// Resolve the fallback OCEL document path under `TMPDIR` (or `/tmp`).
pub fn fallback_path() -> PathBuf {
    let tmp_dir = std::env::var("TMPDIR").unwrap_or_else(|_| "/tmp".to_string());
    Path::new(&tmp_dir).join(FALLBACK_FILE_NAME)
}

// =============================================================================
// Load / save
// =============================================================================

/// Load an OCEL 2.0 document from `path`. Returns an empty document if the
/// file does not exist; returns an error if the file exists but is not valid
/// JSON or does not match the OCEL 2.0 shape.
pub fn read_document(path: &Path) -> Result<OcelDocument> {
    if !path.exists() {
        return Ok(OcelDocument::empty());
    }
    let contents = fs::read_to_string(path).map_err(|e| {
        NounVerbError::execution_error(format!("Failed to read OCEL log {path:?}: {e}"))
    })?;
    serde_json::from_str(&contents)
        .map_err(|e| NounVerbError::execution_error(format!("Invalid OCEL log at {path:?}: {e}")))
}

fn load_or_new(path: &Path) -> io::Result<OcelDocument> {
    if !path.exists() {
        return Ok(OcelDocument::empty());
    }
    let contents = fs::read_to_string(path)?;
    serde_json::from_str(&contents).or_else(|_| Ok(OcelDocument::empty()))
}

fn save(path: &Path, doc: &OcelDocument) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    let json = serde_json::to_string_pretty(doc)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
    fs::write(path, json)
}

// =============================================================================
// Corpus aggregation
// =============================================================================

/// Read N real OCEL 2.0 documents from disk and fold them into one, unioning
/// `objectTypes`/`eventTypes` by name (deduped) and concatenating
/// `objects`/`events` (objects deduped by id, first occurrence wins),
/// preserving the spec-exact shape of the merged result.
///
/// This is the structural seam that lets a fleet of independently deployed
/// (and independently wrapped, non-Rust) CLIs become one comparable,
/// aggregable observability corpus instead of N bespoke logs.
pub fn merge_documents(paths: &[impl AsRef<Path>]) -> Result<OcelDocument> {
    let mut merged = OcelDocument::empty();
    for path in paths {
        let document = read_document(path.as_ref())?;
        for object_type in document.object_types {
            if !merged.object_types.iter().any(|existing| existing.name == object_type.name) {
                merged.object_types.push(object_type);
            }
        }
        for event_type in document.event_types {
            if !merged.event_types.iter().any(|existing| existing.name == event_type.name) {
                merged.event_types.push(event_type);
            }
        }
        for object in document.objects {
            if !merged.objects.iter().any(|existing| existing.id == object.id) {
                merged.objects.push(object);
            }
        }
        merged.events.extend(document.events);
    }
    Ok(merged)
}

// =============================================================================
// Drift detection (admitted schema vs. observed corpus)
// =============================================================================

/// The result of comparing an "admitted" set of command ids (e.g. the noun/verb
/// surface a deployment is supposed to expose) against the command ids actually
/// exercised in an observed [`OcelDocument`].
///
/// Command ids use the same `"{noun}:{verb}"` scheme as [`record_invocation`]'s
/// `command` object id (see `try_record_invocation`), so callers can pass ids
/// derived from a deployment manifest without re-deriving that scheme.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DriftReport {
    /// Command ids present in the admitted set with zero matching events.
    pub admitted_never_exercised: Vec<String>,
    /// Command ids present in the admitted set with at least one matching event.
    pub exercised: Vec<String>,
    /// `exercised.len() / admitted.len()`, or `0.0` if the admitted set is empty.
    pub coverage_ratio: f64,
}

/// Compare an admitted `"{noun}:{verb}"` command-id set against `observed`'s
/// real recorded events (via each event's `regards` relationship into a
/// `command` object), reporting which admitted commands were never exercised.
///
/// `admitted` is a slice of `(noun, verb)` pairs; ids are formed with the exact
/// same `format!("{noun}:{verb}")` scheme `try_record_invocation` already uses.
pub fn drift_report(admitted: &[(&str, &str)], observed: &OcelDocument) -> DriftReport {
    let exercised_ids: std::collections::HashSet<&str> = observed
        .events
        .iter()
        .flat_map(|event| event.relationships.iter())
        .map(|rel| rel.object_id.as_str())
        .collect();

    let mut admitted_never_exercised = Vec::new();
    let mut exercised = Vec::new();

    for (noun, verb) in admitted {
        let id = format!("{noun}:{verb}");
        if exercised_ids.contains(id.as_str()) {
            exercised.push(id);
        } else {
            admitted_never_exercised.push(id);
        }
    }

    let coverage_ratio =
        if admitted.is_empty() { 0.0 } else { exercised.len() as f64 / admitted.len() as f64 };

    DriftReport { admitted_never_exercised, exercised, coverage_ratio }
}

// =============================================================================
// Prune candidates
// =============================================================================

/// Command ids that are candidates for pruning from an admitted surface,
/// because the underlying command has gone stale in the observed corpus.
///
/// Definition used here: a command object is a prune candidate if its **most
/// recent** associated event's `time` is older than `now - min_age`. A command
/// object with zero associated events is *not* returned by this function --
/// "never exercised" is a distinct concept from "exercised, but not recently"
/// (see [`drift_report`] for the former). Events with an unparseable `time`
/// are treated as absent (they cannot support a recency claim either way).
pub fn prune_candidates(
    document: &OcelDocument,
    min_age: std::time::Duration,
    now: chrono::DateTime<chrono::Utc>,
) -> Vec<String> {
    let cutoff =
        now - chrono::Duration::from_std(min_age).unwrap_or_else(|_| chrono::Duration::seconds(0));

    let mut most_recent: std::collections::HashMap<&str, chrono::DateTime<chrono::Utc>> =
        std::collections::HashMap::new();

    for event in &document.events {
        let Ok(event_time) = chrono::DateTime::parse_from_rfc3339(&event.time) else {
            continue;
        };
        let event_time = event_time.with_timezone(&chrono::Utc);
        for rel in &event.relationships {
            most_recent
                .entry(rel.object_id.as_str())
                .and_modify(|existing| {
                    if event_time > *existing {
                        *existing = event_time;
                    }
                })
                .or_insert(event_time);
        }
    }

    let mut candidates: Vec<String> = most_recent
        .into_iter()
        .filter(|(_, last_seen)| *last_seen < cutoff)
        .map(|(id, _)| id.to_string())
        .collect();
    candidates.sort();
    candidates
}

// =============================================================================
// Pack-selection signals -- the closure seam from observed OCEL evidence back
// into what ggen should generate next.
// =============================================================================

/// What a single admitted command's real invocation history recommends doing
/// with it at the next generation cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SignalRecommendation {
    /// Declared/admitted in the ontology, but never observed executing at
    /// all across the whole merged corpus -- the strongest "stop generating
    /// this" signal, since it means the command has never once been used.
    Prune,
    /// Observed executing at least once, but its most recent invocation is
    /// older than the recency window checked -- a candidate for human
    /// review, not an automatic prune (it may simply be seasonal/rare).
    Review,
    /// Observed executing recently, but below the configured success-rate
    /// threshold -- a candidate for hardening (its behavior, argument
    /// validation, or documentation), not removal.
    Harden,
    /// Observed executing recently and at or above the success-rate
    /// threshold -- no action recommended.
    Keep,
}

impl SignalRecommendation {
    /// The exact lowercase token this recommendation serializes to in RDF
    /// (and that a ggen SPARQL gate matches against, e.g.
    /// `cnv-ocel:recommendation "prune"`).
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Prune => "prune",
            Self::Review => "review",
            Self::Harden => "harden",
            Self::Keep => "keep",
        }
    }
}

/// One admitted command's real-world usage signal, derived from a merged
/// OCEL corpus (see [`merge_documents`]) -- the input a pack-selection gate
/// (or a human deciding what to prune/harden by hand) consumes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PackSelectionSignal {
    /// `"{noun}:{verb}"`, the same id scheme [`record_invocation`]/
    /// [`drift_report`]/[`prune_candidates`] already use.
    pub command_id: String,
    /// Total events regarding this command across the merged corpus.
    pub invocation_count: u64,
    /// Of those, how many carried a `success` attribute of `true`.
    pub success_count: u64,
    /// `success_count / invocation_count`, or `0.0` if never invoked.
    pub success_rate: f64,
    pub recommendation: SignalRecommendation,
}

/// Compute one [`PackSelectionSignal`] per admitted `(noun, verb)` pair from
/// `observed`'s real recorded events.
///
/// `min_age`/`now` are the same recency window [`prune_candidates`] uses
/// (an admitted-but-never-exercised command is always [`SignalRecommendation::Prune`]
/// regardless of this window -- see [`drift_report`] for that distinction).
/// `min_success_rate` is the threshold below which an exercised, recent
/// command is recommended for [`SignalRecommendation::Harden`] rather than
/// [`SignalRecommendation::Keep`].
pub fn compute_signals(
    admitted: &[(&str, &str)],
    observed: &OcelDocument,
    min_age: std::time::Duration,
    now: chrono::DateTime<chrono::Utc>,
    min_success_rate: f64,
) -> Vec<PackSelectionSignal> {
    let drift = drift_report(admitted, observed);
    let never_exercised: std::collections::HashSet<&str> =
        drift.admitted_never_exercised.iter().map(String::as_str).collect();
    let stale: std::collections::HashSet<String> =
        prune_candidates(observed, min_age, now).into_iter().collect();

    let mut signals = Vec::with_capacity(admitted.len());
    for (noun, verb) in admitted {
        let command_id = format!("{noun}:{verb}");

        if never_exercised.contains(command_id.as_str()) {
            signals.push(PackSelectionSignal {
                command_id,
                invocation_count: 0,
                success_count: 0,
                success_rate: 0.0,
                recommendation: SignalRecommendation::Prune,
            });
            continue;
        }

        let mut invocation_count = 0u64;
        let mut success_count = 0u64;
        for event in &observed.events {
            let regards_this_command = event
                .relationships
                .iter()
                .any(|rel| rel.qualifier == "regards" && rel.object_id == command_id);
            if !regards_this_command {
                continue;
            }
            invocation_count += 1;
            let succeeded = event
                .attributes
                .iter()
                .find(|attr| attr.name == "success")
                .and_then(|attr| attr.value.as_bool())
                .unwrap_or(false);
            if succeeded {
                success_count += 1;
            }
        }

        let success_rate = if invocation_count == 0 {
            0.0
        } else {
            success_count as f64 / invocation_count as f64
        };

        let recommendation = if stale.contains(&command_id) {
            SignalRecommendation::Review
        } else if success_rate < min_success_rate {
            SignalRecommendation::Harden
        } else {
            SignalRecommendation::Keep
        };

        signals.push(PackSelectionSignal {
            command_id,
            invocation_count,
            success_count,
            success_rate,
            recommendation,
        });
    }
    signals
}

/// Project [`PackSelectionSignal`]s as Turtle RDF individuals of
/// `cnv-ocel:Signal`, in the same `cnv-ocel:` vocabulary [`to_rdf`] uses.
///
/// The output is meant to be written into (or composed as) a ggen pack's own
/// `ontology.ttl`, so a SPARQL admission gate can query the union graph for
/// `cnv-ocel:Signal ; cnv-ocel:commandId "{noun}:{verb}" ; cnv-ocel:recommendation "prune"`
/// matching a `cnv:Command` currently being admitted, and refuse to keep
/// generating a command real usage evidence says is dead -- the concrete
/// closure of observed corpus evidence back into a generation decision.
#[must_use]
pub fn signals_to_rdf(signals: &[PackSelectionSignal]) -> String {
    let mut out = String::new();
    out.push_str("@prefix cnv-ocel: <");
    out.push_str(RDF_BASE_IRI);
    out.push_str("> .\n");
    out.push_str("@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n\n");

    for signal in signals {
        let subject = format!("cnv-ocel:signal-{}", turtle_local_name(&signal.command_id));
        out.push_str(&format!("{subject} a cnv-ocel:Signal ;\n"));
        out.push_str(&format!(
            "    cnv-ocel:commandId \"{}\" ;\n",
            escape_turtle_literal(&signal.command_id)
        ));
        out.push_str(&format!(
            "    cnv-ocel:invocationCount \"{}\"^^xsd:integer ;\n",
            signal.invocation_count
        ));
        out.push_str(&format!(
            "    cnv-ocel:successRate \"{}\"^^xsd:double ;\n",
            signal.success_rate
        ));
        out.push_str(&format!(
            "    cnv-ocel:recommendation \"{}\" .\n\n",
            signal.recommendation.as_str()
        ));
    }

    out
}

const SIGNAL_PACK_TOML: &str = r#"[pack]
name = "ocel-signals"
version = "0.0.0"
description = "Regenerated each cycle from a real merged OCEL corpus via clap_noun_verb::ocel::write_signal_pack. Do not hand-edit -- overwritten on every regeneration."
"#;

const SIGNAL_PACK_TEMPLATE: &str = r#"---
to: "docs/signals/{{ command_id | slugify }}.md"
force: true
sparql:
  primary: |
    PREFIX cnv-ocel: <https://clap-noun-verb.dev/ontology/ocel#>
    SELECT ?command_id ?recommendation ?invocation_count ?success_rate WHERE {
      ?signal a cnv-ocel:Signal ;
              cnv-ocel:commandId ?command_id ;
              cnv-ocel:recommendation ?recommendation ;
              cnv-ocel:invocationCount ?invocation_count ;
              cnv-ocel:successRate ?success_rate .
    }
---
`{{ command_id }}`: **{{ recommendation }}** ({{ invocation_count }} invocations, {{ success_rate }} success rate)
"#;

/// Write a complete, ggen-composable \"signals pack\" directory from
/// `signals` -- `pack.toml`, `ontology.ttl` (via [`signals_to_rdf`]), and one
/// minimal per-signal status template -- ready to compose alongside a
/// project's own `cnv:Cli` ontology via `[packs] <name> = { path = ... }`
/// next to `ocel-feedback-pack` (see
/// `~/ggen-marketplace/packs/ocel-feedback-pack`).
///
/// This is the concrete, single-call closure of the loop: a real merged
/// OCEL corpus in, a real ggen-composable pack directory out. `dir` is
/// overwritten wholesale on every call -- this pack is meant to be
/// regenerated fresh each cycle, never hand-edited.
///
/// Callers must place `dir` **outside** any project directory whose own
/// `[templates] dir` would otherwise also discover this pack's templates
/// and double-render them (mirroring how every other composed ggen pack in
/// this ecosystem is referenced by an absolute path, not nested under the
/// consuming project).
pub fn write_signal_pack(dir: &Path, signals: &[PackSelectionSignal]) -> io::Result<()> {
    fs::create_dir_all(dir)?;
    let templates_dir = dir.join("templates");
    fs::create_dir_all(&templates_dir)?;

    fs::write(dir.join("pack.toml"), SIGNAL_PACK_TOML)?;
    fs::write(dir.join("ontology.ttl"), signals_to_rdf(signals))?;
    fs::write(templates_dir.join("signal-status.md.tmpl"), SIGNAL_PACK_TEMPLATE)?;
    Ok(())
}

const DRIFT_PACK_TOML: &str = r#"[pack]
name = "ocel-drift"
version = "0.0.0"
description = "Regenerated each cycle from a real DriftReport via clap_noun_verb::ocel::write_drift_pack. Do not hand-edit -- overwritten on every regeneration."
"#;

// This pack's own template deliberately does NOT render the human-facing
// status report -- `ocel-drift-pack` (the sibling pack this data pack is
// always composed alongside) already does, at `docs/DRIFT_STATUS.md`.
// Rendering the same report from both packs would collide on that output
// path the moment both are composed together (every ggen pack must ship
// at least one template, so this one exists purely to satisfy that rule
// with a real, distinct, non-conflicting artifact).
const DRIFT_PACK_TEMPLATE: &str = r#"---
to: ".drift-data-provenance.md"
force: true
sparql:
  drift: |
    PREFIX cnv-ocel: <https://clap-noun-verb.dev/ontology/ocel#>
    SELECT ?coverage_ratio ?min_coverage_ratio WHERE {
      ?report a cnv-ocel:DriftReport ;
              cnv-ocel:coverageRatio ?coverage_ratio ;
              cnv-ocel:minCoverageRatio ?min_coverage_ratio .
    }
---
Regenerated drift data (coverage_ratio={{ drift[0].coverage_ratio }},
min_coverage_ratio={{ drift[0].min_coverage_ratio }}). See
`docs/DRIFT_STATUS.md` (rendered by `ocel-drift-pack`) for the
human-facing report.
"#;

/// Project a [`DriftReport`] plus a chosen coverage floor as Turtle RDF: one
/// `cnv-ocel:DriftReport` individual carrying `coverageRatio` (the report's
/// real, observed value) and `minCoverageRatio` (the floor a gate should
/// enforce against it).
#[must_use]
pub fn drift_report_to_rdf(report: &DriftReport, min_coverage_ratio: f64) -> String {
    format!(
        "@prefix cnv-ocel: <{RDF_BASE_IRI}> .\n\
         @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n\n\
         cnv-ocel:drift-report a cnv-ocel:DriftReport ;\n\
         \x20   cnv-ocel:coverageRatio \"{}\"^^xsd:double ;\n\
         \x20   cnv-ocel:minCoverageRatio \"{}\"^^xsd:double .\n",
        report.coverage_ratio, min_coverage_ratio
    )
}

/// Write a complete, ggen-composable "drift pack" directory from a
/// [`DriftReport`] and a chosen coverage floor -- `pack.toml` +
/// `ontology.ttl` (via [`drift_report_to_rdf`]) + one status template --
/// ready to compose alongside a project's own `cnv:Cli` ontology next to
/// `ocel-drift-pack` (see `~/ggen-marketplace/packs/ocel-drift-pack`), the
/// same single-call closure [`write_signal_pack`] gives for per-command
/// signals, but for the project-wide coverage ratio instead.
pub fn write_drift_pack(
    dir: &Path,
    report: &DriftReport,
    min_coverage_ratio: f64,
) -> io::Result<()> {
    fs::create_dir_all(dir)?;
    let templates_dir = dir.join("templates");
    fs::create_dir_all(&templates_dir)?;

    fs::write(dir.join("pack.toml"), DRIFT_PACK_TOML)?;
    fs::write(dir.join("ontology.ttl"), drift_report_to_rdf(report, min_coverage_ratio))?;
    fs::write(templates_dir.join("drift-status.md.tmpl"), DRIFT_PACK_TEMPLATE)?;
    Ok(())
}

// =============================================================================
// RDF (Turtle) export
// =============================================================================

const RDF_BASE_IRI: &str = "https://clap-noun-verb.dev/ontology/ocel#";

/// Escape a string for use inside a Turtle string literal (`"..."`).
fn escape_turtle_literal(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n").replace('\r', "\\r")
}

/// Turn a raw id into a Turtle-safe local name for a `cnv-ocel:` IRI (spaces
/// and colons -- common in `"{noun}:{verb}"` command ids -- are not valid bare
/// local-name characters, so they are percent-encoded).
fn turtle_local_name(id: &str) -> String {
    let mut out = String::with_capacity(id.len());
    for ch in id.chars() {
        if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
            out.push(ch);
        } else {
            for byte in ch.to_string().as_bytes() {
                out.push_str(&format!("%{byte:02X}"));
            }
        }
    }
    out
}

fn json_value_to_turtle_literal(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(s) => format!("\"{}\"", escape_turtle_literal(s)),
        serde_json::Value::Bool(b) => format!("\"{b}\"^^xsd:boolean"),
        serde_json::Value::Number(n) => format!("\"{n}\"^^xsd:double"),
        other => format!("\"{}\"", escape_turtle_literal(&other.to_string())),
    }
}

/// Serialize an [`OcelDocument`] as Turtle RDF using a small inline
/// `cnv-ocel:` vocabulary (base IRI `https://clap-noun-verb.dev/ontology/ocel#`).
///
/// Each [`OcelObject`] becomes a `cnv-ocel:Object` individual with
/// `cnv-ocel:objectType`/`cnv-ocel:objectId` triples (plus one literal triple
/// per attribute, named after the attribute); each [`OcelEvent`] becomes a
/// `cnv-ocel:Event` individual with `cnv-ocel:eventType`/`cnv-ocel:time`
/// (typed `xsd:dateTime`) plus one `cnv-ocel:relatesTo` triple per
/// relationship, annotated with the relationship's qualifier via a
/// `cnv-ocel:qualifier` triple on a blank node (kept deliberately simple --
/// one fixed predicate rather than one generated predicate per qualifier).
///
/// This hand-emits syntactically valid Turtle directly; no RDF library
/// dependency is introduced for this.
pub fn to_rdf(document: &OcelDocument) -> String {
    let mut out = String::new();
    out.push_str("@prefix cnv-ocel: <");
    out.push_str(RDF_BASE_IRI);
    out.push_str("> .\n");
    out.push_str("@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n\n");

    for object in &document.objects {
        let subject = format!("cnv-ocel:object-{}", turtle_local_name(&object.id));
        out.push_str(&format!("{subject} a cnv-ocel:Object ;\n"));
        out.push_str(&format!(
            "    cnv-ocel:objectId \"{}\" ;\n",
            escape_turtle_literal(&object.id)
        ));
        if object.attributes.is_empty() {
            out.push_str(&format!(
                "    cnv-ocel:objectType \"{}\" .\n\n",
                escape_turtle_literal(&object.obj_type)
            ));
        } else {
            out.push_str(&format!(
                "    cnv-ocel:objectType \"{}\" ;\n",
                escape_turtle_literal(&object.obj_type)
            ));
            for (idx, attr) in object.attributes.iter().enumerate() {
                let terminator = if idx + 1 == object.attributes.len() { " .\n\n" } else { " ;\n" };
                out.push_str(&format!(
                    "    cnv-ocel:hasAttribute [ cnv-ocel:attributeName \"{}\" ; \
                     cnv-ocel:attributeValue {} ]{terminator}",
                    escape_turtle_literal(&attr.name),
                    json_value_to_turtle_literal(&attr.value),
                ));
            }
        }
    }

    for event in &document.events {
        let subject = format!("cnv-ocel:event-{}", turtle_local_name(&event.id));
        out.push_str(&format!("{subject} a cnv-ocel:Event ;\n"));
        out.push_str(&format!(
            "    cnv-ocel:eventType \"{}\" ;\n",
            escape_turtle_literal(&event.event_type)
        ));
        out.push_str(&format!(
            "    cnv-ocel:time \"{}\"^^xsd:dateTime",
            escape_turtle_literal(&event.time)
        ));

        for attr in &event.attributes {
            out.push_str(" ;\n");
            out.push_str(&format!(
                "    cnv-ocel:hasAttribute [ cnv-ocel:attributeName \"{}\" ; \
                 cnv-ocel:attributeValue {} ]",
                escape_turtle_literal(&attr.name),
                json_value_to_turtle_literal(&attr.value),
            ));
        }

        for rel in &event.relationships {
            let object_subject = format!("cnv-ocel:object-{}", turtle_local_name(&rel.object_id));
            out.push_str(" ;\n");
            out.push_str(&format!(
                "    cnv-ocel:relatesTo [ cnv-ocel:relatesToObject {object_subject} ; \
                 cnv-ocel:qualifier \"{}\" ]",
                escape_turtle_literal(&rel.qualifier)
            ));
        }

        out.push_str(" .\n\n");
    }

    out
}

/// Unescape a Turtle string-literal body (the inverse of
/// [`escape_turtle_literal`]).
fn unescape_turtle_literal(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut chars = value.chars();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.next() {
                Some('n') => out.push('\n'),
                Some('r') => out.push('\r'),
                Some('"') => out.push('"'),
                Some('\\') => out.push('\\'),
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
                None => out.push('\\'),
            }
        } else {
            out.push(ch);
        }
    }
    out
}

/// Extract the body of the first `"..."` literal in `text` (handling
/// `\"`-escaped quotes), starting the search at `text[start..]`. Returns
/// `(unescaped_body, index_just_past_the_closing_quote)`.
fn extract_quoted_literal(text: &str, start: usize) -> Option<(String, usize)> {
    let bytes = text.as_bytes();
    let open = text[start..].find('"')? + start;
    let mut idx = open + 1;
    let mut escaped = false;
    while idx < bytes.len() {
        match bytes[idx] {
            b'\\' if !escaped => escaped = true,
            b'"' if !escaped => {
                let body = &text[open + 1..idx];
                return Some((unescape_turtle_literal(body), idx + 1));
            }
            _ => escaped = false,
        }
        idx += 1;
    }
    None
}

/// Parse a Turtle literal token (`"..."`, `"..."^^xsd:boolean`, or
/// `"..."^^xsd:double`) starting at `text[start..]` into the
/// [`serde_json::Value`] [`json_value_to_turtle_literal`] would have
/// produced it from. Numeric literals always parse back as
/// `serde_json::Value::Number` via `f64` -- [`to_rdf`] always emits
/// `xsd:double` for every JSON number regardless of whether the original
/// value was an integer or a float, so the int/float distinction is lost
/// by that emitter's own design, not by this parser. Returns the value
/// plus the index just past the literal (including any `^^xsd:...` type
/// suffix).
fn parse_turtle_literal(text: &str, start: usize) -> Option<(serde_json::Value, usize)> {
    let (body, mut idx) = extract_quoted_literal(text, start)?;
    if let Some(rest) = text[idx..].strip_prefix("^^xsd:boolean") {
        idx = text.len() - rest.len();
        let value = body.parse::<bool>().ok()?;
        Some((serde_json::json!(value), idx))
    } else if let Some(rest) = text[idx..].strip_prefix("^^xsd:double") {
        idx = text.len() - rest.len();
        let value = body.parse::<f64>().ok()?;
        Some((serde_json::json!(value), idx))
    } else if let Some(rest) = text[idx..].strip_prefix("^^xsd:dateTime") {
        idx = text.len() - rest.len();
        Some((serde_json::json!(body), idx))
    } else {
        Some((serde_json::json!(body), idx))
    }
}

/// Decode a [`turtle_local_name`]-encoded local name back to its original
/// string (reversing the `%XX` percent-encoding of every non
/// `[A-Za-z0-9_-]` byte).
fn decode_turtle_local_name(encoded: &str) -> String {
    let bytes = encoded.as_bytes();
    let mut decoded_bytes = Vec::with_capacity(bytes.len());
    let mut idx = 0;
    while idx < bytes.len() {
        if bytes[idx] == b'%' && idx + 2 < bytes.len() {
            if let Ok(byte) = u8::from_str_radix(&encoded[idx + 1..idx + 3], 16) {
                decoded_bytes.push(byte);
                idx += 3;
                continue;
            }
        }
        decoded_bytes.push(bytes[idx]);
        idx += 1;
    }
    String::from_utf8(decoded_bytes).unwrap_or_else(|_| encoded.to_string())
}

/// Extract the single-line content between the first `[` and its matching
/// `]` starting the search at `text[start..]` (no nested brackets --
/// [`to_rdf`] never emits any).
fn extract_bracketed(text: &str, start: usize) -> Option<(&str, usize)> {
    let open = text[start..].find('[')? + start;
    let close = text[open..].find(']')? + open;
    Some((&text[open + 1..close], close + 1))
}

/// Parse Turtle RDF produced by [`to_rdf`] back into an [`OcelDocument`].
///
/// This is the narrow inverse of `to_rdf`'s own fixed, hand-emitted
/// grammar -- **not** a general Turtle/RDF parser, and it will not parse
/// arbitrary Turtle from another source. `object_types`/`event_types` are
/// re-derived from the parsed `objects`/`events` (registering `"command"`/
/// `"process"`/`"cli_invocation"` etc. the same way [`OcelDocument::ensure_object_type`]
/// would), since `to_rdf` never serializes the type-declaration arrays in
/// the first place -- there is nothing to recover them from.
///
/// Numeric attribute values always round-trip as `f64` (see
/// [`parse_turtle_literal`]): `to_rdf` itself always emits `xsd:double`
/// for every JSON number, so the original int/float distinction cannot be
/// recovered by any parser reading only this Turtle output.
pub fn from_rdf(turtle: &str) -> Result<OcelDocument> {
    let mut document = OcelDocument::empty();

    // Split on blank lines into per-subject blocks (to_rdf always
    // terminates one subject's triples with " .\n\n").
    for block in turtle.split("\n\n") {
        let block = block.trim();
        if block.is_empty() || block.starts_with("@prefix") {
            continue;
        }

        if let Some(rest) = block.strip_prefix("cnv-ocel:object-") {
            let (local_name, _) =
                rest.split_once(' ').ok_or_else(|| malformed_rdf("object subject line"))?;
            let id = decode_turtle_local_name(local_name);

            let (object_id, mut idx) = extract_quoted_literal(
                block,
                block.find("cnv-ocel:objectId").ok_or_else(|| malformed_rdf("objectId"))?,
            )
            .ok_or_else(|| malformed_rdf("objectId literal"))?;

            let type_start =
                block.find("cnv-ocel:objectType").ok_or_else(|| malformed_rdf("objectType"))?;
            let (obj_type, new_idx) = extract_quoted_literal(block, type_start)
                .ok_or_else(|| malformed_rdf("objectType literal"))?;
            idx = idx.max(new_idx);

            let mut attributes = Vec::new();
            let mut search_from = idx;
            while let Some(attr_marker) = block[search_from..].find("cnv-ocel:hasAttribute") {
                let attr_start = search_from + attr_marker;
                let (bracket_body, next_idx) = extract_bracketed(block, attr_start)
                    .ok_or_else(|| malformed_rdf("attribute blank node"))?;
                let (name, _) = extract_quoted_literal(
                    bracket_body,
                    bracket_body
                        .find("cnv-ocel:attributeName")
                        .ok_or_else(|| malformed_rdf("attributeName"))?,
                )
                .ok_or_else(|| malformed_rdf("attributeName literal"))?;
                let value_start = bracket_body
                    .find("cnv-ocel:attributeValue")
                    .ok_or_else(|| malformed_rdf("attributeValue"))?;
                let (value, _) = parse_turtle_literal(bracket_body, value_start)
                    .ok_or_else(|| malformed_rdf("attributeValue literal"))?;
                attributes.push(ObjectAttributeValue { name, time: now_rfc3339(), value });
                search_from = next_idx;
            }

            document.ensure_object_type(&obj_type);
            document.objects.push(OcelObject { id: object_id.clone(), obj_type, attributes });
            let _ = id; // decoded solely for validation the block parsed; objectId literal is authoritative
        } else if let Some(rest) = block.strip_prefix("cnv-ocel:event-") {
            let (local_name, _) =
                rest.split_once(' ').ok_or_else(|| malformed_rdf("event subject line"))?;
            let event_id = decode_turtle_local_name(local_name);

            let (event_type, _) = extract_quoted_literal(
                block,
                block.find("cnv-ocel:eventType").ok_or_else(|| malformed_rdf("eventType"))?,
            )
            .ok_or_else(|| malformed_rdf("eventType literal"))?;

            let time_start = block.find("cnv-ocel:time").ok_or_else(|| malformed_rdf("time"))?;
            let (time_value, mut idx) = parse_turtle_literal(block, time_start)
                .ok_or_else(|| malformed_rdf("time literal"))?;
            let time =
                time_value.as_str().ok_or_else(|| malformed_rdf("time literal type"))?.to_string();

            let mut attributes = Vec::new();
            let mut search_from = idx;
            while let Some(attr_marker) = block[search_from..].find("cnv-ocel:hasAttribute") {
                let attr_start = search_from + attr_marker;
                let (bracket_body, next_idx) = extract_bracketed(block, attr_start)
                    .ok_or_else(|| malformed_rdf("attribute blank node"))?;
                let (name, _) = extract_quoted_literal(
                    bracket_body,
                    bracket_body
                        .find("cnv-ocel:attributeName")
                        .ok_or_else(|| malformed_rdf("attributeName"))?,
                )
                .ok_or_else(|| malformed_rdf("attributeName literal"))?;
                let value_start = bracket_body
                    .find("cnv-ocel:attributeValue")
                    .ok_or_else(|| malformed_rdf("attributeValue"))?;
                let (value, _) = parse_turtle_literal(bracket_body, value_start)
                    .ok_or_else(|| malformed_rdf("attributeValue literal"))?;
                attributes.push(EventAttributeValue { name, value });
                search_from = next_idx;
                idx = next_idx;
            }

            let mut relationships = Vec::new();
            search_from = idx;
            while let Some(rel_marker) = block[search_from..].find("cnv-ocel:relatesTo") {
                let rel_start = search_from + rel_marker;
                let (bracket_body, next_idx) = extract_bracketed(block, rel_start)
                    .ok_or_else(|| malformed_rdf("relationship blank node"))?;
                let object_marker = bracket_body
                    .find("cnv-ocel:relatesToObject")
                    .ok_or_else(|| malformed_rdf("relatesToObject"))?;
                let after_marker =
                    &bracket_body[object_marker + "cnv-ocel:relatesToObject".len()..];
                let object_ref = after_marker
                    .split([';', ' '])
                    .find(|s| !s.is_empty())
                    .ok_or_else(|| malformed_rdf("relatesToObject value"))?;
                let object_local = object_ref
                    .strip_prefix("cnv-ocel:object-")
                    .ok_or_else(|| malformed_rdf("relatesToObject reference"))?;
                let object_id = decode_turtle_local_name(object_local);

                let (qualifier, _) = extract_quoted_literal(
                    bracket_body,
                    bracket_body
                        .find("cnv-ocel:qualifier")
                        .ok_or_else(|| malformed_rdf("qualifier"))?,
                )
                .ok_or_else(|| malformed_rdf("qualifier literal"))?;

                relationships.push(Relationship { object_id, qualifier });
                search_from = next_idx;
            }

            document.ensure_event_type(&event_type);
            document.events.push(OcelEvent {
                id: event_id,
                event_type,
                time,
                attributes,
                relationships,
            });
        } else {
            return Err(malformed_rdf("unrecognized subject block"));
        }
    }

    Ok(document)
}

fn malformed_rdf(what: &str) -> NounVerbError {
    NounVerbError::execution_error(format!("from_rdf: malformed or unsupported Turtle ({what})"))
}

// =============================================================================
// Recording
// =============================================================================

/// Append one `cli_invocation` event to the OCEL document, registering the
/// "command" and "process" object types/objects and the "cli_invocation" event
/// type idempotently. Best-effort: never returns an error to the caller and
/// never panics -- all I/O failures degrade to a `log::warn!`.
pub fn record_invocation(noun: &str, verb: &str, success: bool, duration_ms: u128) {
    let primary = primary_path();
    match try_record_invocation(&primary, noun, verb, success, duration_ms) {
        Ok(()) => (),
        Err(primary_err) => {
            let fallback = fallback_path();
            if let Err(fallback_err) =
                try_record_invocation(&fallback, noun, verb, success, duration_ms)
            {
                log::warn!(
                    "clap-noun-verb: OCEL logging failed for both primary path {primary:?} \
                     ({primary_err}) and fallback path {fallback:?} ({fallback_err}); \
                     continuing without OCEL logging for this invocation"
                );
            }
        }
    }
}

fn try_record_invocation(
    path: &Path,
    noun: &str,
    verb: &str,
    success: bool,
    duration_ms: u128,
) -> io::Result<()> {
    let mut doc = load_or_new(path)?;

    doc.ensure_object_type("command");
    doc.ensure_object_type("process");
    doc.ensure_event_type("cli_invocation");

    let command_id = format!("{noun}:{verb}");
    doc.ensure_object(&command_id, "command");

    let process_id = process_object_id();
    doc.ensure_object(process_id, "process");

    let event = OcelEvent {
        id: generate_event_id(),
        event_type: "cli_invocation".to_string(),
        time: now_rfc3339(),
        attributes: vec![
            EventAttributeValue { name: "noun".to_string(), value: serde_json::json!(noun) },
            EventAttributeValue { name: "verb".to_string(), value: serde_json::json!(verb) },
            EventAttributeValue { name: "success".to_string(), value: serde_json::json!(success) },
            EventAttributeValue {
                name: "duration_ms".to_string(),
                value: serde_json::json!(duration_ms as u64),
            },
        ],
        relationships: vec![
            Relationship { object_id: command_id, qualifier: "regards".to_string() },
            Relationship {
                object_id: process_id.to_string(),
                qualifier: "performed_by".to_string(),
            },
        ],
    };
    doc.events.push(event);

    save(path, &doc)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;
    use std::sync::Mutex;

    // Serializes tests that mutate process-wide env vars / CWD so they don't
    // race each other (Chicago style: real files, real env, no mocks).
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn temp_dir(label: &str) -> PathBuf {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
        let dir = std::env::temp_dir().join(format!("cnv-ocel-test-{label}-{nanos}"));
        fs::create_dir_all(&dir).expect("create temp test dir");
        dir
    }

    #[test]
    fn test_document_shape_matches_ocel_2_0_spec() {
        // Arrange
        let mut doc = OcelDocument::empty();
        doc.ensure_object_type("command");
        doc.ensure_event_type("cli_invocation");
        doc.ensure_object("noun:verb", "command");
        doc.events.push(OcelEvent {
            id: "evt-1".to_string(),
            event_type: "cli_invocation".to_string(),
            time: now_rfc3339(),
            attributes: vec![EventAttributeValue {
                name: "noun".to_string(),
                value: serde_json::json!("noun"),
            }],
            relationships: vec![Relationship {
                object_id: "noun:verb".to_string(),
                qualifier: "regards".to_string(),
            }],
        });

        // Act
        let json = serde_json::to_string(&doc).expect("serialize document");
        let value: serde_json::Value = serde_json::from_str(&json).expect("parse back as Value");

        // Assert: exact OCEL 2.0 top-level shape
        let obj = value.as_object().expect("top-level object");
        assert!(obj.contains_key("objectTypes"));
        assert!(obj.contains_key("eventTypes"));
        assert!(obj.contains_key("objects"));
        assert!(obj.contains_key("events"));
        assert!(value["objectTypes"].is_array());
        assert!(value["eventTypes"].is_array());
        assert!(value["objects"].is_array());
        assert!(value["events"].is_array());

        let event = &value["events"][0];
        assert_eq!(event["type"], "cli_invocation");
        assert!(event["relationships"][0]["objectId"].is_string());
        assert_eq!(event["relationships"][0]["objectId"], "noun:verb");
        assert_eq!(event["relationships"][0]["qualifier"], "regards");

        let object = &value["objects"][0];
        assert_eq!(object["type"], "command");
        assert_eq!(object["id"], "noun:verb");
    }

    #[test]
    fn test_record_invocation_appends_command_and_process_objects_and_event() {
        // Arrange
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let dir = temp_dir("record");
        let path = dir.join("ocel.json");
        std::env::set_var(ENV_PATH, &path);

        // Act: two distinct real commands through the real recording path
        record_invocation("things", "list", true, 5);
        record_invocation("things", "create", true, 12);

        std::env::remove_var(ENV_PATH);

        // Assert: real document on disk, real structure
        let doc = read_document(&path).expect("read back real OCEL document");
        assert_eq!(doc.events.len(), 2);

        let command_objects: Vec<&OcelObject> =
            doc.objects.iter().filter(|o| o.obj_type == "command").collect();
        assert_eq!(command_objects.len(), 2);
        assert!(command_objects.iter().any(|o| o.id == "things:list"));
        assert!(command_objects.iter().any(|o| o.id == "things:create"));

        let process_objects: Vec<&OcelObject> =
            doc.objects.iter().filter(|o| o.obj_type == "process").collect();
        // Same process invocation records both events against one process object.
        assert_eq!(process_objects.len(), 1);

        let object_ids: std::collections::HashSet<&str> =
            doc.objects.iter().map(|o| o.id.as_str()).collect();
        for event in &doc.events {
            assert_eq!(event.event_type, "cli_invocation");
            assert_eq!(event.relationships.len(), 2);
            for rel in &event.relationships {
                assert!(
                    object_ids.contains(rel.object_id.as_str()),
                    "event relationship {} must point at a real registered object",
                    rel.object_id
                );
            }
        }

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_merge_documents_unions_two_real_recorded_documents() {
        // Arrange: two real OCEL documents produced by the real writer
        // (`record_invocation`) against two independent temp dirs.
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let dir_a = temp_dir("merge-a");
        let path_a = dir_a.join("ocel.json");
        std::env::set_var(ENV_PATH, &path_a);
        record_invocation("things", "list", true, 5);
        std::env::remove_var(ENV_PATH);

        let dir_b = temp_dir("merge-b");
        let path_b = dir_b.join("ocel.json");
        std::env::set_var(ENV_PATH, &path_b);
        record_invocation("orders", "create", true, 7);
        std::env::remove_var(ENV_PATH);

        // Act
        let merged = merge_documents(&[&path_a, &path_b]).expect("merge two real documents");

        // Assert: union of object/event types and objects, concatenated events
        assert_eq!(merged.events.len(), 2);
        let event_types: BTreeSet<&str> =
            merged.event_types.iter().map(|t| t.name.as_str()).collect();
        assert_eq!(event_types, BTreeSet::from(["cli_invocation"]));
        let object_types: BTreeSet<&str> =
            merged.object_types.iter().map(|t| t.name.as_str()).collect();
        assert_eq!(object_types, BTreeSet::from(["command", "process"]));
        let command_ids: BTreeSet<&str> = merged
            .objects
            .iter()
            .filter(|o| o.obj_type == "command")
            .map(|o| o.id.as_str())
            .collect();
        assert_eq!(command_ids, BTreeSet::from(["things:list", "orders:create"]));

        fs::remove_dir_all(&dir_a).ok();
        fs::remove_dir_all(&dir_b).ok();
    }

    #[test]
    fn test_merge_documents_dedupes_shared_object_across_documents() {
        // Arrange: both documents record against the *same* noun/verb, so the
        // resulting "command" object id is identical in both real files.
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let dir_a = temp_dir("dedupe-a");
        let path_a = dir_a.join("ocel.json");
        std::env::set_var(ENV_PATH, &path_a);
        record_invocation("things", "list", true, 1);
        std::env::remove_var(ENV_PATH);

        let dir_b = temp_dir("dedupe-b");
        let path_b = dir_b.join("ocel.json");
        std::env::set_var(ENV_PATH, &path_b);
        record_invocation("things", "list", true, 2);
        std::env::remove_var(ENV_PATH);

        // Act
        let merged = merge_documents(&[&path_a, &path_b]).expect("merge two real documents");

        // Assert: 2 events (concatenated) but only 1 deduped "command" object
        assert_eq!(merged.events.len(), 2);
        let command_objects: Vec<&OcelObject> =
            merged.objects.iter().filter(|o| o.obj_type == "command").collect();
        assert_eq!(command_objects.len(), 1);
        assert_eq!(command_objects[0].id, "things:list");

        fs::remove_dir_all(&dir_a).ok();
        fs::remove_dir_all(&dir_b).ok();
    }

    #[test]
    fn test_record_invocation_falls_back_when_primary_path_is_unwritable() {
        // Arrange: point the primary path at a deep directory under a path
        // component that is actually a *file* -- creating any child of a file
        // always fails, deterministically, on every platform/user, unlike a
        // permissions-based test.
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let dir = temp_dir("fallback");
        let blocking_file = dir.join("not-a-directory");
        fs::write(&blocking_file, b"x").expect("create blocking file");
        let unwritable_primary = blocking_file.join("deep").join("nested").join("ocel.json");
        std::env::set_var(ENV_PATH, &unwritable_primary);

        // Act: recording must not panic and must complete
        record_invocation("diag", "check", true, 1);

        std::env::remove_var(ENV_PATH);

        // Assert: primary path was never created (still not a real path)
        assert!(!unwritable_primary.exists());

        // Assert: the fallback path received the event instead
        let fallback = fallback_path();
        let doc = read_document(&fallback).expect("read back fallback OCEL document");
        assert!(doc.events.iter().any(|e| {
            e.attributes.iter().any(|a| a.name == "verb" && a.value == serde_json::json!("check"))
        }));

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_drift_report_names_never_exercised_command_and_computes_coverage() {
        // Arrange: a real OCEL document via 2 real record_invocation calls.
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let dir = temp_dir("drift");
        let path = dir.join("ocel.json");
        std::env::set_var(ENV_PATH, &path);

        record_invocation("things", "list", true, 5);
        record_invocation("things", "create", true, 12);

        std::env::remove_var(ENV_PATH);

        let observed = read_document(&path).expect("read back real OCEL document");
        let admitted = [("things", "list"), ("things", "create"), ("things", "delete")];

        // Act
        let report = drift_report(&admitted, &observed);

        // Assert
        assert_eq!(report.admitted_never_exercised, vec!["things:delete".to_string()]);
        let exercised: BTreeSet<&str> = report.exercised.iter().map(|s| s.as_str()).collect();
        assert_eq!(exercised, BTreeSet::from(["things:list", "things:create"]));
        assert!((report.coverage_ratio - (2.0 / 3.0)).abs() < f64::EPSILON);

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_drift_report_coverage_ratio_is_zero_for_empty_admitted_set() {
        // Arrange
        let observed = OcelDocument::empty();

        // Act
        let report = drift_report(&[], &observed);

        // Assert: guarded against divide-by-zero
        assert_eq!(report.coverage_ratio, 0.0);
        assert!(report.admitted_never_exercised.is_empty());
        assert!(report.exercised.is_empty());
    }

    #[test]
    fn test_prune_candidates_returns_only_command_older_than_threshold() {
        // Arrange: two real commands, one old event, one recent event.
        let mut doc = OcelDocument::empty();
        doc.ensure_object_type("command");
        doc.ensure_event_type("cli_invocation");
        doc.ensure_object("stale:verb", "command");
        doc.ensure_object("fresh:verb", "command");

        let now = chrono::Utc::now();
        let old_time = (now - chrono::Duration::days(30)).to_rfc3339();
        let recent_time = (now - chrono::Duration::minutes(1)).to_rfc3339();

        doc.events.push(OcelEvent {
            id: "evt-old".to_string(),
            event_type: "cli_invocation".to_string(),
            time: old_time,
            attributes: vec![],
            relationships: vec![Relationship {
                object_id: "stale:verb".to_string(),
                qualifier: "regards".to_string(),
            }],
        });
        doc.events.push(OcelEvent {
            id: "evt-recent".to_string(),
            event_type: "cli_invocation".to_string(),
            time: recent_time,
            attributes: vec![],
            relationships: vec![Relationship {
                object_id: "fresh:verb".to_string(),
                qualifier: "regards".to_string(),
            }],
        });

        // Act: threshold of 1 day catches only the 30-day-old command.
        let candidates = prune_candidates(&doc, std::time::Duration::from_secs(60 * 60 * 24), now);

        // Assert
        assert_eq!(candidates, vec!["stale:verb".to_string()]);
    }

    #[test]
    fn test_prune_candidates_uses_most_recent_event_not_first() {
        // Arrange: same command, one old event and one recent event -- the
        // most-recent-wins rule means this command must NOT be a candidate.
        let mut doc = OcelDocument::empty();
        let now = chrono::Utc::now();
        let old_time = (now - chrono::Duration::days(30)).to_rfc3339();
        let recent_time = (now - chrono::Duration::minutes(1)).to_rfc3339();

        doc.events.push(OcelEvent {
            id: "evt-1".to_string(),
            event_type: "cli_invocation".to_string(),
            time: old_time,
            attributes: vec![],
            relationships: vec![Relationship {
                object_id: "things:list".to_string(),
                qualifier: "regards".to_string(),
            }],
        });
        doc.events.push(OcelEvent {
            id: "evt-2".to_string(),
            event_type: "cli_invocation".to_string(),
            time: recent_time,
            attributes: vec![],
            relationships: vec![Relationship {
                object_id: "things:list".to_string(),
                qualifier: "regards".to_string(),
            }],
        });

        // Act
        let candidates = prune_candidates(&doc, std::time::Duration::from_secs(60 * 60 * 24), now);

        // Assert
        assert!(candidates.is_empty());
    }

    #[test]
    fn test_to_rdf_emits_syntactically_structured_turtle_for_object_event_and_relationship() {
        // Arrange: a small real OcelDocument built via record_invocation.
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let dir = temp_dir("rdf");
        let path = dir.join("ocel.json");
        std::env::set_var(ENV_PATH, &path);

        record_invocation("things", "list", true, 5);

        std::env::remove_var(ENV_PATH);

        let doc = read_document(&path).expect("read back real OCEL document");

        // Act
        let turtle = to_rdf(&doc);

        // Assert: valid-looking Turtle structure.
        assert!(
            turtle.starts_with("@prefix cnv-ocel: <https://clap-noun-verb.dev/ontology/ocel#> .\n")
        );
        assert!(turtle.contains("@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n"));

        // The "command" object appears as a real cnv-ocel:Object individual.
        assert!(turtle.contains("cnv-ocel:object-things%3Alist a cnv-ocel:Object ;"));
        assert!(turtle.contains("cnv-ocel:objectId \"things:list\" ;"));
        assert!(turtle.contains("cnv-ocel:objectType \"command\""));

        // The cli_invocation event appears as a real cnv-ocel:Event individual.
        assert!(turtle.contains("a cnv-ocel:Event ;"));
        assert!(turtle.contains("cnv-ocel:eventType \"cli_invocation\" ;"));
        assert!(turtle.contains("^^xsd:dateTime"));

        // The event -> command relationship is present.
        assert!(turtle.contains("cnv-ocel:relatesTo [ cnv-ocel:relatesToObject cnv-ocel:object-things%3Alist ; cnv-ocel:qualifier \"regards\" ]"));

        // Every statement terminates cleanly; no dangling unescaped literal
        // newline (all literal content on one physical line per `"..."`
        // pair -- we only ever emit `\n` as the escaped two-char sequence).
        for line in turtle.lines() {
            let quote_count = line.matches('"').count();
            assert_eq!(
                quote_count % 2,
                0,
                "line has an odd number of unescaped quotes (unterminated literal): {line}"
            );
        }
        assert!(turtle.trim_end().ends_with('.'));

        fs::remove_dir_all(&dir).ok();
    }

    // =========================================================================
    // compute_signals / signals_to_rdf -- the pack-selection closure seam.
    // =========================================================================

    fn signal_event(
        command_id: &str,
        success: bool,
        time: chrono::DateTime<chrono::Utc>,
    ) -> OcelEvent {
        OcelEvent {
            id: generate_event_id(),
            event_type: "cli_invocation".to_string(),
            time: time.to_rfc3339(),
            attributes: vec![EventAttributeValue {
                name: "success".to_string(),
                value: serde_json::json!(success),
            }],
            relationships: vec![Relationship {
                object_id: command_id.to_string(),
                qualifier: "regards".to_string(),
            }],
        }
    }

    #[test]
    fn test_compute_signals_recommends_prune_for_a_command_with_zero_real_invocations() {
        // Arrange: "fleet:legacy" is admitted (declared in the ontology) but
        // never actually invoked anywhere in the merged real corpus.
        let now = chrono::Utc::now();
        let observed = OcelDocument::empty();
        let admitted = [("fleet", "legacy")];

        // Act
        let signals = compute_signals(
            &admitted,
            &observed,
            std::time::Duration::from_secs(60 * 60 * 24 * 30),
            now,
            0.5,
        );

        // Assert: real, unambiguous "stop generating this" signal.
        assert_eq!(signals.len(), 1);
        assert_eq!(signals[0].command_id, "fleet:legacy");
        assert_eq!(signals[0].invocation_count, 0);
        assert_eq!(signals[0].recommendation, SignalRecommendation::Prune);
        assert_eq!(signals[0].recommendation.as_str(), "prune");
    }

    #[test]
    fn test_compute_signals_recommends_harden_for_a_command_below_the_success_threshold() {
        // Arrange: "fleet:flaky" ran 4 times, only 1 succeeded (25%), all
        // recent -- below the 50% threshold, but genuinely exercised, so
        // this must be Harden, not Prune or Review.
        let now = chrono::Utc::now();
        let mut observed = OcelDocument::empty();
        observed.events.push(signal_event("fleet:flaky", true, now));
        observed.events.push(signal_event("fleet:flaky", false, now));
        observed.events.push(signal_event("fleet:flaky", false, now));
        observed.events.push(signal_event("fleet:flaky", false, now));
        let admitted = [("fleet", "flaky")];

        // Act
        let signals = compute_signals(
            &admitted,
            &observed,
            std::time::Duration::from_secs(60 * 60 * 24 * 30),
            now,
            0.5,
        );

        // Assert
        assert_eq!(signals.len(), 1);
        assert_eq!(signals[0].invocation_count, 4);
        assert_eq!(signals[0].success_count, 1);
        assert!((signals[0].success_rate - 0.25).abs() < f64::EPSILON);
        assert_eq!(signals[0].recommendation, SignalRecommendation::Harden);
    }

    #[test]
    fn test_compute_signals_recommends_review_for_an_exercised_but_stale_command() {
        // Arrange: "fleet:seasonal" succeeded every time it ran, but its last
        // (only) run was well outside the recency window -- Review, not
        // Prune (it HAS real usage evidence) and not Harden (it's healthy).
        let now = chrono::Utc::now();
        let old = now - chrono::Duration::days(90);
        let mut observed = OcelDocument::empty();
        observed.events.push(signal_event("fleet:seasonal", true, old));
        let admitted = [("fleet", "seasonal")];

        // Act
        let signals = compute_signals(
            &admitted,
            &observed,
            std::time::Duration::from_secs(60 * 60 * 24 * 30),
            now,
            0.5,
        );

        // Assert
        assert_eq!(signals.len(), 1);
        assert_eq!(signals[0].invocation_count, 1);
        assert_eq!(signals[0].recommendation, SignalRecommendation::Review);
    }

    #[test]
    fn test_compute_signals_recommends_keep_for_a_healthy_recently_exercised_command() {
        // Arrange: "fleet:healthy" ran 3 times, all recent, all succeeded.
        let now = chrono::Utc::now();
        let mut observed = OcelDocument::empty();
        observed.events.push(signal_event("fleet:healthy", true, now));
        observed.events.push(signal_event("fleet:healthy", true, now));
        observed.events.push(signal_event("fleet:healthy", true, now));
        let admitted = [("fleet", "healthy")];

        // Act
        let signals = compute_signals(
            &admitted,
            &observed,
            std::time::Duration::from_secs(60 * 60 * 24 * 30),
            now,
            0.5,
        );

        // Assert
        assert_eq!(signals.len(), 1);
        assert_eq!(signals[0].invocation_count, 3);
        assert_eq!(signals[0].success_count, 3);
        assert!((signals[0].success_rate - 1.0).abs() < f64::EPSILON);
        assert_eq!(signals[0].recommendation, SignalRecommendation::Keep);
    }

    #[test]
    fn test_signals_to_rdf_emits_a_queryable_signal_individual_per_command() {
        // Arrange: one real signal of each recommendation kind.
        let signals = vec![
            PackSelectionSignal {
                command_id: "fleet:legacy".to_string(),
                invocation_count: 0,
                success_count: 0,
                success_rate: 0.0,
                recommendation: SignalRecommendation::Prune,
            },
            PackSelectionSignal {
                command_id: "fleet:healthy".to_string(),
                invocation_count: 3,
                success_count: 3,
                success_rate: 1.0,
                recommendation: SignalRecommendation::Keep,
            },
        ];

        // Act
        let turtle = signals_to_rdf(&signals);

        // Assert: real, syntactically well-formed Turtle a SPARQL gate can
        // query for `cnv-ocel:recommendation "prune"` matching a command id.
        assert!(turtle.contains("cnv-ocel:signal-fleet%3Alegacy a cnv-ocel:Signal ;"));
        assert!(turtle.contains("cnv-ocel:commandId \"fleet:legacy\" ;"));
        assert!(turtle.contains("cnv-ocel:invocationCount \"0\"^^xsd:integer ;"));
        assert!(turtle.contains("cnv-ocel:recommendation \"prune\" ."));
        assert!(turtle.contains("cnv-ocel:signal-fleet%3Ahealthy a cnv-ocel:Signal ;"));
        assert!(turtle.contains("cnv-ocel:recommendation \"keep\" ."));

        for line in turtle.lines() {
            let quote_count = line.matches('"').count();
            assert_eq!(
                quote_count % 2,
                0,
                "line has an odd number of unescaped quotes (unterminated literal): {line}"
            );
        }
    }

    #[test]
    fn test_write_signal_pack_writes_a_real_composable_pack_directory() {
        // Arrange
        let dir = temp_dir("signal-pack");
        let signals = vec![PackSelectionSignal {
            command_id: "fleet:dead".to_string(),
            invocation_count: 0,
            success_count: 0,
            success_rate: 0.0,
            recommendation: SignalRecommendation::Prune,
        }];

        // Act
        write_signal_pack(&dir, &signals).expect("write real signal pack directory");

        // Assert: real files on disk, not an in-memory description of them.
        let pack_toml =
            fs::read_to_string(dir.join("pack.toml")).expect("read real pack.toml back");
        assert!(pack_toml.contains("name = \"ocel-signals\""));

        let ontology =
            fs::read_to_string(dir.join("ontology.ttl")).expect("read real ontology.ttl back");
        assert!(ontology.contains("cnv-ocel:commandId \"fleet:dead\""));
        assert!(ontology.contains("cnv-ocel:recommendation \"prune\""));

        let template = fs::read_to_string(dir.join("templates").join("signal-status.md.tmpl"))
            .expect("read real template back");
        assert!(template.starts_with("---\n"));
        assert!(template.contains("cnv-ocel:Signal"));

        fs::remove_dir_all(&dir).ok();
    }

    // =========================================================================
    // write_drift_pack -- the ggen-composable projection of drift_report.
    // =========================================================================

    #[test]
    fn test_drift_report_to_rdf_emits_a_real_queryable_drift_report_individual() {
        // Arrange: a real drift report over a real merged corpus.
        let mut observed = OcelDocument::empty();
        observed.events.push(regards_event_for_test("fleet:alive"));
        let admitted = [("fleet", "alive"), ("fleet", "dead")];
        let report = drift_report(&admitted, &observed);
        assert!((report.coverage_ratio - 0.5).abs() < f64::EPSILON);

        // Act
        let turtle = drift_report_to_rdf(&report, 0.8);

        // Assert
        assert!(turtle.contains("cnv-ocel:drift-report a cnv-ocel:DriftReport ;"));
        assert!(turtle.contains("cnv-ocel:coverageRatio \"0.5\"^^xsd:double ;"));
        assert!(turtle.contains("cnv-ocel:minCoverageRatio \"0.8\"^^xsd:double ."));
    }

    #[test]
    fn test_write_drift_pack_writes_a_real_composable_pack_directory() {
        // Arrange
        let dir = temp_dir("drift-pack");
        let report = DriftReport {
            admitted_never_exercised: vec!["fleet:dead".to_string()],
            exercised: vec!["fleet:alive".to_string()],
            coverage_ratio: 0.5,
        };

        // Act
        write_drift_pack(&dir, &report, 0.8).expect("write real drift pack directory");

        // Assert: real files on disk.
        let pack_toml =
            fs::read_to_string(dir.join("pack.toml")).expect("read real pack.toml back");
        assert!(pack_toml.contains("name = \"ocel-drift\""));

        let ontology =
            fs::read_to_string(dir.join("ontology.ttl")).expect("read real ontology.ttl back");
        assert!(ontology.contains("cnv-ocel:coverageRatio \"0.5\"^^xsd:double"));
        assert!(ontology.contains("cnv-ocel:minCoverageRatio \"0.8\"^^xsd:double"));

        let template = fs::read_to_string(dir.join("templates").join("drift-status.md.tmpl"))
            .expect("read real template back");
        assert!(template.starts_with("---\n"));
        assert!(template.contains("cnv-ocel:DriftReport"));

        fs::remove_dir_all(&dir).ok();
    }

    fn regards_event_for_test(command_id: &str) -> OcelEvent {
        OcelEvent {
            id: generate_event_id(),
            event_type: "cli_invocation".to_string(),
            time: now_rfc3339(),
            attributes: vec![EventAttributeValue {
                name: "success".to_string(),
                value: serde_json::json!(true),
            }],
            relationships: vec![Relationship {
                object_id: command_id.to_string(),
                qualifier: "regards".to_string(),
            }],
        }
    }

    // =========================================================================
    // from_rdf -- the real inverse of to_rdf.
    // =========================================================================

    #[test]
    fn test_from_rdf_round_trips_a_real_document_built_via_record_invocation() {
        // Arrange: a real document via two real record_invocation calls
        // (the exact same fixture pattern test_to_rdf_... uses).
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let dir = temp_dir("from-rdf-roundtrip");
        let path = dir.join("ocel.json");
        std::env::set_var(ENV_PATH, &path);

        record_invocation("things", "list", true, 5);
        record_invocation("things", "create", false, 12);

        std::env::remove_var(ENV_PATH);

        let original = read_document(&path).expect("read real OCEL document");
        let turtle = to_rdf(&original);

        // Act
        let round_tripped = from_rdf(&turtle).expect("parse real to_rdf output back");

        // Assert: every real object survived, with its real id/type/attributes.
        assert_eq!(round_tripped.objects.len(), original.objects.len());
        for original_object in &original.objects {
            let recovered = round_tripped
                .objects
                .iter()
                .find(|o| o.id == original_object.id)
                .unwrap_or_else(|| panic!("object {} must round-trip", original_object.id));
            assert_eq!(recovered.obj_type, original_object.obj_type);
        }

        // Assert: every real event survived, with its real type/time and
        // every real attribute value (duration_ms/success round-trip
        // through xsd:double -- compared as f64, since to_rdf's own
        // design always emits doubles regardless of the original JSON
        // number's int/float-ness).
        assert_eq!(round_tripped.events.len(), original.events.len());
        for original_event in &original.events {
            let recovered = round_tripped
                .events
                .iter()
                .find(|e| e.id == original_event.id)
                .unwrap_or_else(|| panic!("event {} must round-trip", original_event.id));
            assert_eq!(recovered.event_type, original_event.event_type);
            assert_eq!(recovered.time, original_event.time);
            assert_eq!(recovered.relationships.len(), original_event.relationships.len());
            for original_rel in &original_event.relationships {
                assert!(
                    recovered.relationships.iter().any(|r| r.object_id == original_rel.object_id
                        && r.qualifier == original_rel.qualifier),
                    "relationship to {} ({}) must round-trip",
                    original_rel.object_id,
                    original_rel.qualifier
                );
            }
            for original_attr in &original_event.attributes {
                let recovered_attr = recovered
                    .attributes
                    .iter()
                    .find(|a| a.name == original_attr.name)
                    .unwrap_or_else(|| panic!("attribute {} must round-trip", original_attr.name));
                match (&original_attr.value, &recovered_attr.value) {
                    (serde_json::Value::Number(orig), serde_json::Value::Number(recov)) => {
                        assert!(
                            (orig.as_f64().unwrap_or(f64::NAN)
                                - recov.as_f64().unwrap_or(f64::NAN))
                            .abs()
                                < f64::EPSILON,
                            "numeric attribute {} must round-trip its real value (as f64): \
                             {orig} vs {recov}",
                            original_attr.name
                        );
                    }
                    (orig, recov) => assert_eq!(
                        orig, recov,
                        "attribute {} must round-trip exactly",
                        original_attr.name
                    ),
                }
            }
        }

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_from_rdf_round_trips_object_ids_containing_colons_and_special_characters() {
        // Arrange: a real document with a command id containing ':' (the
        // real "{noun}:{verb}" scheme) -- proving the percent-encode/decode
        // round trip in turtle_local_name/decode_turtle_local_name is real,
        // not just theoretically symmetric.
        let mut doc = OcelDocument::empty();
        doc.ensure_object_type("command");
        doc.objects.push(OcelObject {
            id: "fleet:special-verb".to_string(),
            obj_type: "command".to_string(),
            attributes: Vec::new(),
        });

        // Act
        let turtle = to_rdf(&doc);
        let round_tripped = from_rdf(&turtle).expect("parse real to_rdf output back");

        // Assert
        assert_eq!(round_tripped.objects.len(), 1);
        assert_eq!(round_tripped.objects[0].id, "fleet:special-verb");
    }

    #[test]
    fn test_from_rdf_round_trips_a_string_valued_attribute_exactly() {
        // Arrange: an object with a real string attribute (never re-typed
        // as xsd:double, unlike numbers) -- must round-trip byte-for-byte,
        // including a value containing a real escaped quote.
        let mut doc = OcelDocument::empty();
        doc.ensure_object_type("command");
        doc.objects.push(OcelObject {
            id: "fleet:quoted".to_string(),
            obj_type: "command".to_string(),
            attributes: vec![ObjectAttributeValue {
                name: "note".to_string(),
                time: now_rfc3339(),
                value: serde_json::json!("has a \"quoted\" word"),
            }],
        });

        // Act
        let turtle = to_rdf(&doc);
        let round_tripped = from_rdf(&turtle).expect("parse real to_rdf output back");

        // Assert
        assert_eq!(round_tripped.objects.len(), 1);
        assert_eq!(round_tripped.objects[0].attributes.len(), 1);
        assert_eq!(
            round_tripped.objects[0].attributes[0].value,
            serde_json::json!("has a \"quoted\" word")
        );
    }

    #[test]
    fn test_from_rdf_rejects_turtle_it_did_not_itself_produce() {
        // Act: hand-typed Turtle with a subject shape from_rdf never
        // emits -- must be refused, not silently misparsed.
        let result = from_rdf("@prefix ex: <http://example.org/> .\n\nex:thing a ex:Widget .\n");

        // Assert
        assert!(result.is_err(), "unrecognized Turtle must be refused, not silently accepted");
    }
}
