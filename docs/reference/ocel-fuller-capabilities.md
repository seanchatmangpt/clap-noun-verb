# Reference: OCEL Fuller Capabilities -- Drift, Pruning, RDF Export

**Source**: `src/ocel.rs`
**Version**: 26.8.22

Three functions extend the OCEL 2.0 event log (`docs/reference/ocel-v2.md`) from a
passive recording mechanism into something a fleet operator or generator (ggen)
can act on: `drift_report` (which admitted commands are unused), `prune_candidates`
(which commands have gone stale), and `to_rdf` (export the corpus as Turtle RDF).

All three operate on an already-loaded `OcelDocument` -- none of them perform I/O
themselves; callers pair them with `ocel::read_document` (see `docs/reference/ocel-v2.md`).

---

## `drift_report` -- admitted surface vs. observed corpus

```rust
pub fn drift_report(admitted: &[(&str, &str)], observed: &OcelDocument) -> DriftReport
```

```rust
pub struct DriftReport {
    pub admitted_never_exercised: Vec<String>,
    pub exercised: Vec<String>,
    pub coverage_ratio: f64,
}
```

**Problem it solves**: a generated or wrapped CLI fleet (ggen packs, `cnv-any`
wrapped executables) can declare a noun/verb surface that nobody ever actually
uses. `drift_report` compares an *admitted* set of `(noun, verb)` pairs -- the
surface a deployment manifest says it exposes -- against the command ids
actually present in an observed `OcelDocument`'s events (via each event's
`regards` relationship into a `command` object). Command ids use the same
`"{noun}:{verb}"` scheme `record_invocation` already uses, so callers can pass
ids straight from a deployment manifest without re-deriving the format.

`coverage_ratio` is `exercised.len() / admitted.len()`, guarded to `0.0` when
`admitted` is empty (no divide-by-zero).

### Usage example (from `test_drift_report_names_never_exercised_command_and_computes_coverage`)

```rust
record_invocation("things", "list", true, 5);
record_invocation("things", "create", true, 12);

let observed = read_document(&path).expect("read back real OCEL document");
let admitted = [("things", "list"), ("things", "create"), ("things", "delete")];

let report = drift_report(&admitted, &observed);

assert_eq!(report.admitted_never_exercised, vec!["things:delete".to_string()]);
// report.exercised contains "things:list" and "things:create"
// report.coverage_ratio == 2.0 / 3.0
```

---

## `prune_candidates` -- staleness detection

```rust
pub fn prune_candidates(
    document: &OcelDocument,
    min_age: std::time::Duration,
    now: chrono::DateTime<chrono::Utc>,
) -> Vec<String>
```

**Problem it solves**: naming stale commands for removal from an admitted
surface. A command object is a prune candidate if its **most recent**
associated event's `time` is older than `now - min_age`. This is deliberately
a different concept from `drift_report`'s "never exercised": a command with
zero associated events is *not* returned by `prune_candidates` at all (it has
no recency to judge -- that's `drift_report`'s job). Events with an unparseable
`time` are treated as absent and cannot support a recency claim either way.
When a command has multiple events, the most recent one wins the recency
check, not the first one seen.

### Usage example (from `test_prune_candidates_returns_only_command_older_than_threshold`)

```rust
// "stale:verb" has one event 30 days old; "fresh:verb" has one event 1 minute old.
let candidates = prune_candidates(&doc, std::time::Duration::from_secs(60 * 60 * 24), now);

assert_eq!(candidates, vec!["stale:verb".to_string()]);
```

A command with both an old and a recent event is *not* a candidate --
confirmed by `test_prune_candidates_uses_most_recent_event_not_first`, which
gives `"things:list"` one 30-day-old event and one 1-minute-old event under
the same 1-day threshold and asserts `candidates.is_empty()`.

---

## `to_rdf` -- Turtle RDF export

```rust
pub fn to_rdf(document: &OcelDocument) -> String
```

**Problem it solves**: ggen already maintains an ontology graph queried via
SPARQL for inference over generated CLI schemas. Real invocation evidence
(the OCEL corpus) lives in a separate, incompatible format (OCEL JSON). `to_rdf`
serializes an `OcelDocument` as Turtle using a small inline `cnv-ocel:`
vocabulary (base IRI `https://clap-noun-verb.dev/ontology/ocel#`), so the same
SPARQL layer that already queries ggen's ontology graph *could* query real
invocation evidence alongside it. This function only produces the Turtle text
-- no live ggen integration exists yet; nothing loads this output into a
triple store or wires it into ggen's SPARQL layer today. It hand-emits
syntactically valid Turtle directly; no RDF library dependency is introduced.

Each `OcelObject` becomes a `cnv-ocel:Object` individual with
`cnv-ocel:objectType`/`cnv-ocel:objectId` triples (plus one `cnv-ocel:hasAttribute`
blank node per attribute). Each `OcelEvent` becomes a `cnv-ocel:Event`
individual with `cnv-ocel:eventType`/`cnv-ocel:time` (typed `xsd:dateTime`)
plus one `cnv-ocel:relatesTo` blank node per relationship, carrying the
relationship's qualifier (e.g. `regards`, `performed_by`) via a
`cnv-ocel:qualifier` triple.

### Usage example (from `test_to_rdf_emits_syntactically_structured_turtle_for_object_event_and_relationship`)

```rust
record_invocation("things", "list", true, 5);
let doc = read_document(&path).expect("read back real OCEL document");

let turtle = to_rdf(&doc);

assert!(turtle.starts_with(
    "@prefix cnv-ocel: <https://clap-noun-verb.dev/ontology/ocel#> .\n"
));
assert!(turtle.contains("@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n"));
assert!(turtle.contains("cnv-ocel:object-things%3Alist a cnv-ocel:Object ;"));
```

(`things:list`'s `:` is percent-encoded to `%3A` in the Turtle local name,
since `:` is not a valid bare local-name character.)

---

## `from_rdf` -- the narrow inverse of `to_rdf`

```rust
pub fn from_rdf(turtle: &str) -> Result<OcelDocument>
```

`from_rdf` is **not a general Turtle/RDF parser** -- it is a narrow,
hand-rolled parser that inverts exactly `to_rdf`'s own fixed emission
grammar (no external RDF-parsing dependency was added). It refuses any
Turtle it did not itself emit the shape of. Two documented, inherent
lossy points:

- Numeric literals always round-trip as `f64` (via `xsd:double`), since
  `to_rdf` itself always emits doubles regardless of the original
  int/float-ness of the value.
- Blank-node extraction (`extract_bracketed`) only handles single-line
  `[ ... ]` blocks with no nested brackets, because `to_rdf` never emits
  any nested structure.

## `drift_report_to_rdf` / `write_drift_pack` -- the aggregate coverage gate

```rust
pub fn drift_report_to_rdf(report: &DriftReport, min_coverage_ratio: f64) -> String
pub fn write_drift_pack(dir: &Path, report: &DriftReport, min_coverage_ratio: f64) -> io::Result<()>
```

Mirrors `signals_to_rdf`/`write_signal_pack`'s pattern (see
[OCEL Feedback Loop](ocel-feedback-loop.md)) but for the aggregate
`DriftReport.coverage_ratio` rather than per-command signals: emits one
`cnv-ocel:DriftReport` individual with `coverageRatio`/`minCoverageRatio`,
plus a composable pack directory (`pack.toml`/`ontology.ttl`) a consuming
project's `ggen.toml` can point at directly. Closed into a real
ggen-marketplace gate, `ocel-drift-pack`
(`gates/010_coverage_floor.rq`, fail-closed on
`coverage_ratio < min_coverage_ratio`) -- proven with a real subprocess
`ggen sync run` in `tests/ocel_drift_gate.rs`.

---

## See Also

- `docs/reference/ocel-v2.md` -- the underlying OCEL 2.0 event log these three
  functions operate on
- `docs/reference/cnv-any.md` -- `merge_documents`, the seam for aggregating a
  fleet's worth of invocation evidence that `drift_report`/`prune_candidates`
  can then be run against
- `docs/reference/ocel-feedback-loop.md` -- `compute_signals`/`write_signal_pack`
  and `ocel-feedback-pack`'s gate: this evidence closing the loop back into a
  real ggen generation decision
- `src/ocel.rs` -- full implementation and inline doc comments
