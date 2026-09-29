// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Real fault-injection tests for `ocel::record_invocation` and
//! `autonomic::record_receipt` under concurrent writers and adverse
//! filesystem conditions: many real OS threads racing real file writes to
//! the SAME real ledger path, and a real permission-denied primary path
//! forcing the real fallback.
//!
//! Both `record_invocation` and `record_receipt` are best-effort and
//! process-serialize via the whole-document read-modify-write pattern
//! (`load_or_new` -> mutate -> `save`) with NO cross-process/cross-thread
//! lock of their own beyond the file itself -- so concurrent writers are a
//! REAL race the ledger's own file-replace semantics must survive without
//! corrupting the file into invalid JSON/JSONL, even though individual
//! events can legitimately be lost to the classic read-modify-write race
//! (documented here as a real, current limitation, not silently assumed
//! away).

use clap_noun_verb::autonomic::{self, Effect};
use clap_noun_verb::ocel;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

static ENV_LOCK: Mutex<()> = Mutex::new(());

fn temp_dir(label: &str) -> PathBuf {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let dir = std::env::temp_dir().join(format!("cnv-fault-injection-{label}-{nanos}"));
    fs::create_dir_all(&dir).expect("create real temp dir");
    dir
}

#[test]
fn record_receipt_survives_real_concurrent_writers_without_corrupting_the_ledger() {
    // Arrange
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let dir = temp_dir("receipt-concurrent");
    let path = dir.join("receipts.jsonl");
    std::env::set_var("CLAP_NOUN_VERB_RECEIPT_PATH", &path);

    // Act: 16 real OS threads, each appending 10 real receipts concurrently
    // against the SAME real file.
    const THREADS: usize = 16;
    const PER_THREAD: usize = 10;
    let handles: Vec<_> = (0..THREADS)
        .map(|thread_idx| {
            std::thread::spawn(move || {
                for i in 0..PER_THREAD {
                    autonomic::record_receipt(
                        "fleet",
                        &format!("thread-{thread_idx}-{i}"),
                        Effect::ReadOnly,
                        true,
                    );
                }
            })
        })
        .collect();
    for handle in handles {
        handle.join().expect("writer thread must not panic");
    }

    std::env::remove_var("CLAP_NOUN_VERB_RECEIPT_PATH");

    // Assert: the ledger is real, valid JSON-Lines -- every line parses,
    // and every parsed receipt's own digest is internally self-consistent
    // (a corrupted or torn write would produce an unparseable line or a
    // receipt whose digest doesn't match its own fields).
    let contents = fs::read_to_string(&path).expect("read real ledger after concurrent writes");
    let mut real_receipts = Vec::new();
    for (line_no, line) in contents.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let receipt: autonomic::Receipt = serde_json::from_str(line)
            .unwrap_or_else(|e| panic!("line {line_no} must be valid JSON: {e}\nline: {line}"));
        assert!(
            receipt.digest_is_consistent(),
            "receipt at line {line_no} must have a self-consistent digest"
        );
        real_receipts.push(receipt);
    }

    // Document the real, current limitation: concurrent read-modify-write
    // races CAN drop individual receipts (this is not a silent-corruption
    // bug -- it's the documented cost of no cross-thread lock beyond the
    // file itself). What must NEVER happen is a corrupted file or a
    // receipt with an inconsistent digest, both asserted above. At least
    // some receipts from the real concurrent run must have landed.
    assert!(!real_receipts.is_empty(), "at least some concurrent writes must have landed");
    assert!(
        real_receipts.len() <= THREADS * PER_THREAD,
        "must never observe MORE receipts than were ever written"
    );

    fs::remove_dir_all(&dir).ok();
}

#[test]
fn record_invocation_survives_real_concurrent_writers_without_corrupting_the_document() {
    // Arrange
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let dir = temp_dir("ocel-concurrent");
    let path = dir.join("ocel.json");
    std::env::set_var("CLAP_NOUN_VERB_OCEL_PATH", &path);

    // Act: 16 real OS threads racing real writes to the same real
    // whole-document JSON file.
    const THREADS: usize = 16;
    const PER_THREAD: usize = 10;
    let handles: Vec<_> = (0..THREADS)
        .map(|thread_idx| {
            std::thread::spawn(move || {
                for i in 0..PER_THREAD {
                    ocel::record_invocation("fleet", &format!("thread-{thread_idx}-{i}"), true, 1);
                }
            })
        })
        .collect();
    for handle in handles {
        handle.join().expect("writer thread must not panic");
    }

    std::env::remove_var("CLAP_NOUN_VERB_OCEL_PATH");

    // Assert: the final document is real, valid, spec-shaped JSON -- a
    // torn concurrent write would leave invalid JSON on disk, which
    // ocel::read_document would surface as a real error here.
    let document = ocel::read_document(&path)
        .expect("the OCEL document must remain valid JSON after concurrent writes");

    // As with the receipt ledger, individual events can legitimately be
    // lost to the read-modify-write race (a real, documented limitation);
    // what must never happen is document corruption. At least some
    // concurrent writes landed.
    assert!(!document.events.is_empty(), "at least some concurrent writes must have landed");
    assert!(
        document.events.len() <= THREADS * PER_THREAD,
        "must never observe MORE events than were ever written"
    );

    fs::remove_dir_all(&dir).ok();
}

#[test]
fn record_receipt_falls_back_under_a_real_permission_denied_primary_path() {
    // Arrange: a primary path through a real read-only directory --
    // distinct from the existing "path through a file" unwritable-path
    // test in src/autonomic.rs, this exercises a real permission denial
    // rather than a structurally-impossible path.
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let dir = temp_dir("permission-denied");
    let readonly_subdir = dir.join("readonly");
    fs::create_dir_all(&readonly_subdir).expect("create real subdir");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&readonly_subdir).expect("real metadata").permissions();
        perms.set_mode(0o555); // real read+execute, no write
        fs::set_permissions(&readonly_subdir, perms).expect("real chmod");
    }

    let unwritable_path = readonly_subdir.join("receipts.jsonl");
    std::env::set_var("CLAP_NOUN_VERB_RECEIPT_PATH", &unwritable_path);

    // Act: must not panic even under a real permission-denied primary path.
    autonomic::record_receipt("diag", "permission-check", Effect::ReadOnly, true);

    std::env::remove_var("CLAP_NOUN_VERB_RECEIPT_PATH");

    // Assert
    #[cfg(unix)]
    {
        assert!(!unwritable_path.exists(), "the real read-only primary path must stay untouched");
        let fallback_receipts = autonomic::read_and_verify_ledger(&autonomic::fallback_path())
            .expect("read real fallback ledger");
        assert!(
            fallback_receipts.iter().any(|r| r.noun == "diag" && r.verb == "permission-check"),
            "the receipt must have landed in the real fallback location instead"
        );
    }

    // Restore write permission so temp cleanup can remove the directory.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&readonly_subdir).expect("real metadata").permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&readonly_subdir, perms).ok();
    }
    fs::remove_dir_all(&dir).ok();
}
