//! Integration proofs for v26.8.8 feature completion.

#[cfg(feature = "process-data")]
#[test]
fn process_data_transform_is_replayable() {
    use clap_noun_verb::{ProcessDataPipeline, ProcessDataStep};
    use serde_json::json;

    let pipeline = ProcessDataPipeline::new()
        .with_step(ProcessDataStep::SelectPointer("/payload".to_string()))
        .with_step(ProcessDataStep::RemoveNullFields);
    let input = json!({"payload": {"kept": 1, "removed": null}});

    let first = pipeline.transform(&input).expect("first execution");
    let replay = pipeline.transform(&input).expect("replay execution");
    assert_eq!(first, json!({"kept": 1}));
    assert_eq!(replay, first);
}

#[cfg(feature = "contrib")]
#[test]
fn contrib_list_is_deterministic_and_duplicate_safe() {
    use clap_noun_verb::{Contributor, ContributorRegistry};

    let mut registry = ContributorRegistry::new();
    registry
        .register(Contributor::new("beta", "Beta").expect("valid contributor"))
        .expect("unique contributor");
    registry
        .register(Contributor::new("alpha", "Alpha").expect("valid contributor"))
        .expect("unique contributor");

    let listed: Vec<_> = registry
        .list()
        .iter()
        .map(|contributor| contributor.id.as_str())
        .collect();
    assert_eq!(listed, vec!["alpha", "beta"]);
    assert!(registry
        .register(Contributor::new("alpha", "Duplicate").expect("valid contributor"))
        .is_err());
}
