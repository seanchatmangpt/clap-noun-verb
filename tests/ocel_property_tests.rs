// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Property-based tests for `clap_noun_verb::ocel::merge_documents` and
//! `compute_signals`, covering adversarial/malformed inputs a
//! hand-picked example test would likely miss: empty documents, documents
//! with duplicate object ids across sources, events with malformed or
//! missing `time`/`success` attributes, and empty admitted sets.

use clap_noun_verb::ocel::{
    compute_signals, merge_documents, EventAttributeValue, OcelDocument, OcelEvent, OcelObject,
    Relationship, SignalRecommendation,
};
use proptest::prelude::*;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn temp_dir(label: &str) -> PathBuf {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let dir = std::env::temp_dir().join(format!("cnv-ocel-proptest-{label}-{nanos}"));
    fs::create_dir_all(&dir).expect("create real temp dir");
    dir
}

fn write_document(path: &PathBuf, doc: &OcelDocument) {
    let json = serde_json::to_string(doc).expect("serialize real OcelDocument");
    fs::write(path, json).expect("write real OCEL document to disk");
}

fn regards_event(id: &str, command_id: &str, success: bool, time: &str) -> OcelEvent {
    OcelEvent {
        id: id.to_string(),
        event_type: "cli_invocation".to_string(),
        time: time.to_string(),
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

// A restricted alphabet keeps generated command ids valid as both JSON
// map-safe strings and OCEL object ids without needing per-case escaping,
// while still exercising a wide space of distinct real values.
fn command_id_strategy() -> impl Strategy<Value = String> {
    "[a-z]{1,8}:[a-z]{1,8}"
}

proptest! {
    /// merge_documents must never lose or duplicate a real object: the
    /// merged document's distinct object ids are exactly the union of
    /// every source document's distinct object ids, for any number of
    /// real documents (including documents that share object ids, and
    /// documents with none).
    #[test]
    fn merge_documents_object_ids_are_exactly_the_real_union(
        doc_object_ids in prop::collection::vec(
            prop::collection::vec(command_id_strategy(), 0..6),
            0..5,
        )
    ) {
        // Arrange: one real OCEL document per generated list of object ids.
        let dir = temp_dir("union");
        let mut paths = Vec::new();
        let mut expected_union: std::collections::HashSet<String> =
            std::collections::HashSet::new();

        for (idx, ids) in doc_object_ids.iter().enumerate() {
            let mut doc = OcelDocument::empty();
            for id in ids {
                doc.objects.push(OcelObject {
                    id: id.clone(),
                    obj_type: "command".to_string(),
                    attributes: Vec::new(),
                });
                expected_union.insert(id.clone());
            }
            let path = dir.join(format!("doc-{idx}.json"));
            write_document(&path, &doc);
            paths.push(path);
        }

        // Act
        let merged = merge_documents(&paths).expect("merge real documents");

        // Assert
        let actual_union: std::collections::HashSet<String> =
            merged.objects.iter().map(|o| o.id.clone()).collect();
        prop_assert_eq!(actual_union, expected_union);

        fs::remove_dir_all(&dir).ok();
    }

    /// merge_documents must never drop a real event: the merged document's
    /// event count is exactly the sum of every source document's event
    /// count, regardless of how object ids overlap across sources.
    #[test]
    fn merge_documents_never_drops_a_real_event(
        events_per_doc in prop::collection::vec(0usize..5, 0..5),
    ) {
        let dir = temp_dir("event-count");
        let mut paths = Vec::new();
        let mut expected_total = 0usize;

        for (doc_idx, count) in events_per_doc.iter().enumerate() {
            let mut doc = OcelDocument::empty();
            for event_idx in 0..*count {
                // Deliberately reuse the SAME command id "shared:command"
                // across every document -- the adversarial case where
                // object-id deduping could accidentally eat events too.
                doc.events.push(regards_event(
                    &format!("evt-{doc_idx}-{event_idx}"),
                    "shared:command",
                    event_idx % 2 == 0,
                    "2026-01-01T00:00:00Z",
                ));
            }
            expected_total += count;
            let path = dir.join(format!("doc-{doc_idx}.json"));
            write_document(&path, &doc);
            paths.push(path);
        }

        let merged = merge_documents(&paths).expect("merge real documents");
        prop_assert_eq!(merged.events.len(), expected_total);

        fs::remove_dir_all(&dir).ok();
    }

    /// compute_signals must produce exactly one signal per admitted
    /// command, no more and no fewer, regardless of how many real events
    /// (with any real success/failure mix) were observed for it, and
    /// regardless of whether extra, unrelated commands were also
    /// observed in the same document.
    #[test]
    fn compute_signals_produces_exactly_one_signal_per_admitted_command(
        admitted_ids in prop::collection::vec(command_id_strategy(), 0..8),
        noise_event_count in 0usize..5,
    ) {
        let mut observed = OcelDocument::empty();
        // Noise: events regarding a command that is NOT in the admitted
        // set -- must never produce a spurious signal.
        for i in 0..noise_event_count {
            observed.events.push(regards_event(
                &format!("noise-{i}"),
                "unadmitted:noise",
                true,
                "2026-01-01T00:00:00Z",
            ));
        }

        let admitted: Vec<(&str, &str)> = admitted_ids
            .iter()
            .filter_map(|id| id.split_once(':'))
            .collect();

        let signals = compute_signals(
            &admitted,
            &observed,
            Duration::from_secs(60 * 60 * 24 * 30),
            chrono::Utc::now(),
            0.5,
        );

        // Exactly one signal per DISTINCT admitted (noun, verb) pair.
        let distinct_admitted: std::collections::HashSet<(&str, &str)> =
            admitted.iter().copied().collect();
        prop_assert_eq!(signals.len(), distinct_admitted.len());

        // No signal was produced for the unadmitted noise command.
        prop_assert!(!signals.iter().any(|s| s.command_id == "unadmitted:noise"));
    }

    /// A command with real, recorded events whose `success` attribute is
    /// EITHER missing or a non-boolean JSON value must never be treated
    /// as a real success -- compute_signals's success-rate computation
    /// must degrade to "not successful" for that event, not panic, not
    /// silently count it as true.
    #[test]
    fn compute_signals_never_panics_on_malformed_success_attributes(
        malformed_kind in 0u8..3,
    ) {
        let mut observed = OcelDocument::empty();
        let event = match malformed_kind {
            0 => {
                // Missing success attribute entirely.
                OcelEvent {
                    id: "evt-missing".to_string(),
                    event_type: "cli_invocation".to_string(),
                    time: chrono::Utc::now().to_rfc3339(),
                    attributes: Vec::new(),
                    relationships: vec![Relationship {
                        object_id: "fleet:malformed".to_string(),
                        qualifier: "regards".to_string(),
                    }],
                }
            }
            1 => {
                // success attribute present but a string, not a bool. Time
                // must be real and recent -- a stale timestamp would make
                // Review the correct classification instead of Harden,
                // conflating recency with the success-attribute question
                // this test actually targets.
                let now = chrono::Utc::now().to_rfc3339();
                let mut e = regards_event("evt-string", "fleet:malformed", true, &now);
                e.attributes = vec![EventAttributeValue {
                    name: "success".to_string(),
                    value: serde_json::json!("yes"),
                }];
                e
            }
            _ => {
                // success attribute present but a number, not a bool.
                let now = chrono::Utc::now().to_rfc3339();
                let mut e = regards_event("evt-number", "fleet:malformed", true, &now);
                e.attributes = vec![EventAttributeValue {
                    name: "success".to_string(),
                    value: serde_json::json!(1),
                }];
                e
            }
        };
        observed.events.push(event);

        let admitted = [("fleet", "malformed")];
        // Act: must not panic.
        let signals = compute_signals(
            &admitted,
            &observed,
            Duration::from_secs(60 * 60 * 24 * 30),
            chrono::Utc::now(),
            0.5,
        );

        // Assert: the malformed-success event was counted as observed
        // (invocation_count == 1) but never as a real success.
        prop_assert_eq!(signals.len(), 1);
        prop_assert_eq!(signals[0].invocation_count, 1);
        prop_assert_eq!(signals[0].success_count, 0);
        prop_assert_eq!(signals[0].recommendation, SignalRecommendation::Harden);
    }

    /// merge_documents on a set of purely empty real documents must
    /// itself be a real, valid, empty document -- never an error.
    #[test]
    fn merge_documents_of_only_empty_documents_is_a_real_empty_document(
        doc_count in 0usize..5,
    ) {
        let dir = temp_dir("all-empty");
        let mut paths = Vec::new();
        for i in 0..doc_count {
            let path = dir.join(format!("doc-{i}.json"));
            write_document(&path, &OcelDocument::empty());
            paths.push(path);
        }

        let merged = merge_documents(&paths).expect("merge real empty documents");
        prop_assert!(merged.objects.is_empty());
        prop_assert!(merged.events.is_empty());

        fs::remove_dir_all(&dir).ok();
    }
}
