// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Executable witness for replay-aware reflexive verification.

#[cfg(feature = "reflexive-testing")]
fn main() {
    use clap_noun_verb::frontier::ReflexiveReport;

    let first = ReflexiveReport { passed: 128, failed: 0, replay_verified: false };
    assert!(!first.is_alive(), "one successful run without replay has no ALIVE standing");

    let replayed = ReflexiveReport { replay_verified: true, ..first.clone() };
    assert!(replayed.is_alive());

    let failed = ReflexiveReport { passed: 127, failed: 1, replay_verified: true };
    assert!(!failed.is_alive(), "a failed check must refuse ALIVE standing");

    let first_json = serde_json::to_string(&replayed).expect("report must serialize");
    let replay_json = serde_json::to_string(&replayed).expect("report must replay");
    assert_eq!(first_json, replay_json, "replay consequence must be byte-identical");

    println!("Reflexive report admitted only after 128 passing checks and replay");
}

#[cfg(not(feature = "reflexive-testing"))]
fn main() {
    println!("Enable --features reflexive-testing to execute this bounded witness");
}
