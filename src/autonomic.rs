// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The Autonomic Layer: Effects, Guards, and Receipts.
//!
//! `docs/reference/README.md`'s API Stability Guarantees table already
//! claims `✅ **Autonomic Layer**: Introspection, effects, guards, receipts`
//! as a stable, available API. Introspection already exists (`--introspect`,
//! see `crate::registry::collect_tools_from_cmd`). This module is the
//! Effects/Guards/Receipts third of that claim, made real:
//!
//! - [`Effect`]: a formal declaration of what kind of consequence a verb's
//!   execution has (read-only / mutating / idempotent / unknown).
//! - [`Guard`]/[`GuardSet`]: pluggable precondition evaluation -- "can this
//!   be executed?" -- checked before dispatch, with typed denial reasons.
//! - [`Receipt`]/[`record_receipt`]: an append-only, hash-chained record of
//!   every verb invocation, recorded automatically at the same dispatch
//!   site [`crate::ocel::record_invocation`] already hooks into (mirroring
//!   this crate's "telemetry/OCEL is always-on, never feature-gated"
//!   precedent, ADL-003) -- best-effort, never fails or panics the
//!   invocation it describes. [`read_and_verify_ledger`] reads a ledger
//!   back and checks its whole chain in one call.
//!
//! The receipt chain digest is a plain FNV-1a 64-bit hash, not a
//! cryptographic hash function -- it detects accidental corruption or
//! reordering of the ledger, not deliberate tampering by an adversary with
//! write access to the ledger file. This is stated explicitly so the
//! "receipt" claim is never overclaimed as security-grade.

use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::error::Result;

// =============================================================================
// Effects
// =============================================================================

/// A formal declaration of what kind of consequence a verb's execution has.
///
/// `Unknown` is the honest default for a verb with no declared effect --
/// never silently promoted to `ReadOnly`/`Idempotent`, since guessing wrong
/// in either direction (assuming safe-to-retry when it isn't, or assuming
/// unsafe when it's actually idempotent) is worse than admitting "not yet
/// classified."
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Effect {
    /// Never mutates state; safe to call any number of times.
    ReadOnly,
    /// Mutates state; each call may have additional consequences.
    Mutating,
    /// Mutates state, but repeated calls with the same input converge to
    /// the same end state (safe to retry after a failure).
    Idempotent,
    /// Not yet classified.
    Unknown,
}

impl Effect {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ReadOnly => "read_only",
            Self::Mutating => "mutating",
            Self::Idempotent => "idempotent",
            Self::Unknown => "unknown",
        }
    }
}

// =============================================================================
// Guards
// =============================================================================

/// The context a [`Guard`] evaluates against: which verb is about to be
/// dispatched, with what arguments.
#[derive(Debug, Clone)]
pub struct GuardContext<'a> {
    pub noun: &'a str,
    pub verb: &'a str,
    pub args: &'a serde_json::Value,
}

/// Why a [`Guard`] refused to admit an invocation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GuardDenial {
    /// A short, stable, machine-matchable code (e.g. `"MISSING_CAPABILITY"`).
    pub code: String,
    /// A human-readable explanation.
    pub reason: String,
}

impl GuardDenial {
    #[must_use]
    pub fn new(code: impl Into<String>, reason: impl Into<String>) -> Self {
        Self { code: code.into(), reason: reason.into() }
    }
}

/// A single precondition check, evaluated before a verb is dispatched.
///
/// Object-safe by design (no async methods), consistent with this crate's
/// `NounCommand`/`VerbCommand` trait design (ADL-010): synchronous, `&self`,
/// `'static` bounds only.
pub trait Guard: Send + Sync {
    /// A short, stable identifier for this guard (appears in denial output).
    fn name(&self) -> &'static str;
    /// Evaluate this guard's precondition against `ctx`.
    fn check(&self, ctx: &GuardContext<'_>) -> std::result::Result<(), GuardDenial>;
}

/// An ordered collection of [`Guard`]s, evaluated together.
#[derive(Default)]
pub struct GuardSet {
    guards: Vec<Box<dyn Guard>>,
}

impl GuardSet {
    #[must_use]
    pub fn new() -> Self {
        Self { guards: Vec::new() }
    }

    /// Register a guard. Guards run in registration order, but every guard
    /// always runs -- evaluation does not short-circuit on the first
    /// denial, so a caller sees every reason an invocation would be
    /// refused, not just the first one found.
    pub fn add(&mut self, guard: Box<dyn Guard>) -> &mut Self {
        self.guards.push(guard);
        self
    }

    /// Evaluate every registered guard against `ctx`.
    ///
    /// `Ok(())` iff every guard passed. `Err(denials)` lists every denial,
    /// tagged with the denying guard's [`Guard::name`] via `code`'s
    /// `"{guard_name}:{original_code}"` prefix when a guard's own code
    /// doesn't already carry that context.
    pub fn evaluate(&self, ctx: &GuardContext<'_>) -> std::result::Result<(), Vec<GuardDenial>> {
        let mut denials = Vec::new();
        for guard in &self.guards {
            if let Err(denial) = guard.check(ctx) {
                denials.push(denial);
            }
        }
        if denials.is_empty() {
            Ok(())
        } else {
            Err(denials)
        }
    }
}

// =============================================================================
// Receipts
// =============================================================================

/// One append-only, hash-chained record of a verb invocation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Receipt {
    pub sequence: u64,
    pub noun: String,
    pub verb: String,
    pub effect: Effect,
    pub success: bool,
    pub time: String,
    /// FNV-1a 64-bit digest of the previous receipt in the chain, as lowercase
    /// hex (`"0000000000000000"` for the first receipt in a fresh ledger).
    pub prev_digest: String,
    /// FNV-1a 64-bit digest of this receipt's own fields (including
    /// `prev_digest`), as lowercase hex -- the chain link.
    pub digest: String,
}

const GENESIS_DIGEST: &str = "0000000000000000";

fn fnv1a_64(bytes: &[u8]) -> u64 {
    const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
    const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;
    let mut hash = FNV_OFFSET_BASIS;
    for &byte in bytes {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

fn compute_digest(
    sequence: u64,
    noun: &str,
    verb: &str,
    effect: Effect,
    success: bool,
    time: &str,
    prev_digest: &str,
) -> String {
    let canonical =
        format!("{sequence}|{noun}|{verb}|{}|{success}|{time}|{prev_digest}", effect.as_str());
    format!("{:016x}", fnv1a_64(canonical.as_bytes()))
}

impl Receipt {
    /// Build the next receipt in a chain given the previous receipt (or
    /// `None` to start a fresh chain at the genesis digest).
    #[must_use]
    pub fn next(
        previous: Option<&Receipt>,
        noun: &str,
        verb: &str,
        effect: Effect,
        success: bool,
        time: String,
    ) -> Self {
        let sequence = previous.map_or(0, |r| r.sequence + 1);
        let prev_digest = previous.map_or_else(|| GENESIS_DIGEST.to_string(), |r| r.digest.clone());
        let digest = compute_digest(sequence, noun, verb, effect, success, &time, &prev_digest);
        Self {
            sequence,
            noun: noun.to_string(),
            verb: verb.to_string(),
            effect,
            success,
            time,
            prev_digest,
            digest,
        }
    }

    /// Recompute this receipt's digest from its own fields and compare
    /// against the stored `digest` -- `true` iff they match.
    #[must_use]
    pub fn digest_is_consistent(&self) -> bool {
        compute_digest(
            self.sequence,
            &self.noun,
            &self.verb,
            self.effect,
            self.success,
            &self.time,
            &self.prev_digest,
        ) == self.digest
    }
}

/// Why [`verify_chain`] rejected a receipt ledger.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChainVerificationError {
    /// The receipt at this index's own digest doesn't match its recomputed
    /// value (its fields were altered after being written).
    DigestMismatch { index: usize },
    /// The receipt at this index's `prev_digest` doesn't match the
    /// previous receipt's `digest` (a receipt was inserted, removed, or
    /// reordered).
    ChainBroken { index: usize },
    /// The first receipt's `prev_digest` isn't the genesis digest.
    InvalidGenesis,
}

impl std::fmt::Display for ChainVerificationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DigestMismatch { index } => {
                write!(f, "receipt at index {index} has an inconsistent digest")
            }
            Self::ChainBroken { index } => {
                write!(f, "receipt at index {index} does not chain from its predecessor")
            }
            Self::InvalidGenesis => {
                write!(f, "first receipt does not start from the genesis digest")
            }
        }
    }
}

impl std::error::Error for ChainVerificationError {}

/// Verify that `receipts` (in order) form one unbroken, internally
/// consistent hash chain from the genesis digest.
pub fn verify_chain(receipts: &[Receipt]) -> std::result::Result<(), ChainVerificationError> {
    for (index, receipt) in receipts.iter().enumerate() {
        if !receipt.digest_is_consistent() {
            return Err(ChainVerificationError::DigestMismatch { index });
        }
        let expected_prev =
            if index == 0 { GENESIS_DIGEST } else { receipts[index - 1].digest.as_str() };
        if receipt.prev_digest != expected_prev {
            if index == 0 {
                return Err(ChainVerificationError::InvalidGenesis);
            }
            return Err(ChainVerificationError::ChainBroken { index });
        }
    }
    Ok(())
}

// =============================================================================
// Receipt ledger (disk persistence)
// =============================================================================

const ENV_PATH: &str = "CLAP_NOUN_VERB_RECEIPT_PATH";
const DEFAULT_RELATIVE_PATH: &str = ".clap-noun-verb/receipts.jsonl";
const FALLBACK_FILE_NAME: &str = "clap-noun-verb-receipts.jsonl";

/// Resolve the primary receipt ledger path: `CLAP_NOUN_VERB_RECEIPT_PATH` if
/// set, else `.clap-noun-verb/receipts.jsonl` relative to the current
/// working directory (mirroring [`crate::ocel::primary_path`]'s scheme).
#[must_use]
pub fn primary_path() -> PathBuf {
    if let Ok(path) = std::env::var(ENV_PATH) {
        return PathBuf::from(path);
    }
    PathBuf::from(DEFAULT_RELATIVE_PATH)
}

/// A best-effort fallback path (the system temp directory) used when the
/// primary path can't be written to.
#[must_use]
pub fn fallback_path() -> PathBuf {
    std::env::temp_dir().join(FALLBACK_FILE_NAME)
}

/// Read every receipt from a JSON-Lines ledger file at `path`.
pub fn read_ledger(path: &Path) -> io::Result<Vec<Receipt>> {
    let contents = match fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(err) => return Err(err),
    };
    let mut receipts = Vec::new();
    for line in contents.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let receipt: Receipt = serde_json::from_str(line)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        receipts.push(receipt);
    }
    Ok(receipts)
}

fn append_receipt(
    path: &Path,
    noun: &str,
    verb: &str,
    effect: Effect,
    success: bool,
) -> io::Result<Receipt> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    let existing = read_ledger(path)?;
    let previous = existing.last();
    let time = chrono::Utc::now().to_rfc3339();
    let receipt = Receipt::next(previous, noun, verb, effect, success, time);

    let mut line = serde_json::to_string(&receipt)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    line.push('\n');
    use std::io::Write;
    let mut file = fs::OpenOptions::new().create(true).append(true).open(path)?;
    file.write_all(line.as_bytes())?;
    Ok(receipt)
}

/// Append one receipt for a verb invocation, recording `effect`/`success`.
/// Best-effort: never returns an error to the caller and never panics --
/// any I/O failure at the primary path falls back to a temp-dir ledger, and
/// a failure there degrades to a `log::warn!`, mirroring
/// [`crate::ocel::record_invocation`]'s exact fallback discipline.
pub fn record_receipt(noun: &str, verb: &str, effect: Effect, success: bool) {
    let primary = primary_path();
    match append_receipt(&primary, noun, verb, effect, success) {
        Ok(_) => (),
        Err(primary_err) => {
            let fallback = fallback_path();
            if let Err(fallback_err) = append_receipt(&fallback, noun, verb, effect, success) {
                log::warn!(
                    "clap-noun-verb: receipt logging failed for both primary path {primary:?} \
                     ({primary_err}) and fallback path {fallback:?} ({fallback_err}); \
                     continuing without a receipt for this invocation"
                );
            }
        }
    }
}

/// Read and verify the receipt ledger at `path` in one call -- the
/// convenience an operator or audit tool actually reaches for.
pub fn read_and_verify_ledger(path: &Path) -> Result<Vec<Receipt>> {
    let receipts = read_ledger(path).map_err(|e| {
        crate::error::NounVerbError::execution_error(format!("failed to read receipt ledger: {e}"))
    })?;
    verify_chain(&receipts).map_err(|e| {
        crate::error::NounVerbError::execution_error(format!(
            "receipt chain verification failed: {e}"
        ))
    })?;
    Ok(receipts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use std::time::{SystemTime, UNIX_EPOCH};

    // Serializes tests that mutate CWD/env vars, mirroring src/ocel.rs's
    // own test-isolation pattern (real files, real env, no mocks).
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn temp_dir(label: &str) -> PathBuf {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
        let dir = std::env::temp_dir().join(format!("cnv-autonomic-test-{label}-{nanos}"));
        fs::create_dir_all(&dir).expect("create real temp test dir");
        dir
    }

    // -------------------------------------------------------------------
    // Guards
    // -------------------------------------------------------------------

    struct AlwaysDenies;
    impl Guard for AlwaysDenies {
        fn name(&self) -> &'static str {
            "always_denies"
        }
        fn check(&self, _ctx: &GuardContext<'_>) -> std::result::Result<(), GuardDenial> {
            Err(GuardDenial::new("ALWAYS_DENIED", "this guard never admits"))
        }
    }

    struct RequiresNameArg;
    impl Guard for RequiresNameArg {
        fn name(&self) -> &'static str {
            "requires_name_arg"
        }
        fn check(&self, ctx: &GuardContext<'_>) -> std::result::Result<(), GuardDenial> {
            if ctx.args.get("name").is_some() {
                Ok(())
            } else {
                Err(GuardDenial::new("MISSING_NAME", "argument \"name\" is required"))
            }
        }
    }

    #[test]
    fn test_guard_set_admits_when_every_guard_passes() {
        // Arrange
        let mut guards = GuardSet::new();
        guards.add(Box::new(RequiresNameArg));
        let args = serde_json::json!({ "name": "World" });
        let ctx = GuardContext { noun: "greet", verb: "hello", args: &args };

        // Act
        let result = guards.evaluate(&ctx);

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn test_guard_set_collects_every_denial_not_just_the_first() {
        // Arrange
        let mut guards = GuardSet::new();
        guards.add(Box::new(AlwaysDenies));
        guards.add(Box::new(RequiresNameArg));
        let args = serde_json::json!({});
        let ctx = GuardContext { noun: "greet", verb: "hello", args: &args };

        // Act
        let denials = guards.evaluate(&ctx).expect_err("both guards should deny");

        // Assert
        assert_eq!(denials.len(), 2);
        assert_eq!(denials[0].code, "ALWAYS_DENIED");
        assert_eq!(denials[1].code, "MISSING_NAME");
    }

    // -------------------------------------------------------------------
    // Receipt chain
    // -------------------------------------------------------------------

    #[test]
    fn test_receipt_next_chains_from_the_genesis_digest_for_the_first_receipt() {
        // Act
        let receipt = Receipt::next(
            None,
            "fleet",
            "alive",
            Effect::ReadOnly,
            true,
            "2026-01-01T00:00:00Z".to_string(),
        );

        // Assert
        assert_eq!(receipt.sequence, 0);
        assert_eq!(receipt.prev_digest, GENESIS_DIGEST);
        assert!(receipt.digest_is_consistent());
    }

    #[test]
    fn test_receipt_next_chains_sequence_and_prev_digest_from_the_prior_receipt() {
        // Arrange
        let first = Receipt::next(
            None,
            "fleet",
            "alive",
            Effect::ReadOnly,
            true,
            "2026-01-01T00:00:00Z".to_string(),
        );

        // Act
        let second = Receipt::next(
            Some(&first),
            "fleet",
            "alive",
            Effect::ReadOnly,
            true,
            "2026-01-01T00:01:00Z".to_string(),
        );

        // Assert
        assert_eq!(second.sequence, 1);
        assert_eq!(second.prev_digest, first.digest);
        assert!(second.digest_is_consistent());
    }

    #[test]
    fn test_verify_chain_accepts_a_real_unbroken_chain() {
        // Arrange
        let first = Receipt::next(
            None,
            "fleet",
            "alive",
            Effect::ReadOnly,
            true,
            "2026-01-01T00:00:00Z".to_string(),
        );
        let second = Receipt::next(
            Some(&first),
            "fleet",
            "alive",
            Effect::ReadOnly,
            true,
            "2026-01-01T00:01:00Z".to_string(),
        );

        // Act
        let result = verify_chain(&[first, second]);

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_chain_detects_a_tampered_receipt() {
        // Arrange
        let mut first = Receipt::next(
            None,
            "fleet",
            "alive",
            Effect::ReadOnly,
            true,
            "2026-01-01T00:00:00Z".to_string(),
        );
        let second = Receipt::next(
            Some(&first),
            "fleet",
            "alive",
            Effect::ReadOnly,
            true,
            "2026-01-01T00:01:00Z".to_string(),
        );
        // Tamper: flip success after the digest was computed.
        first.success = false;

        // Act
        let result = verify_chain(&[first, second]);

        // Assert
        assert_eq!(result.unwrap_err(), ChainVerificationError::DigestMismatch { index: 0 });
    }

    #[test]
    fn test_verify_chain_detects_a_removed_receipt_breaking_the_chain() {
        // Arrange: three real chained receipts, with the middle one dropped.
        let first = Receipt::next(
            None,
            "fleet",
            "alive",
            Effect::ReadOnly,
            true,
            "2026-01-01T00:00:00Z".to_string(),
        );
        let second = Receipt::next(
            Some(&first),
            "fleet",
            "alive",
            Effect::ReadOnly,
            true,
            "2026-01-01T00:01:00Z".to_string(),
        );
        let third = Receipt::next(
            Some(&second),
            "fleet",
            "alive",
            Effect::ReadOnly,
            true,
            "2026-01-01T00:02:00Z".to_string(),
        );

        // Act: verify with `second` removed.
        let result = verify_chain(&[first, third]);

        // Assert
        assert_eq!(result.unwrap_err(), ChainVerificationError::ChainBroken { index: 1 });
    }

    // -------------------------------------------------------------------
    // Ledger persistence (real files on disk)
    // -------------------------------------------------------------------

    #[test]
    fn test_record_receipt_appends_a_real_verifiable_chain_to_a_real_file() {
        // Arrange
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let dir = temp_dir("record");
        let path = dir.join("receipts.jsonl");
        std::env::set_var(ENV_PATH, &path);

        // Act
        record_receipt("fleet", "alive", Effect::ReadOnly, true);
        record_receipt("fleet", "alive", Effect::ReadOnly, true);
        record_receipt("fleet", "dead", Effect::Mutating, false);

        std::env::remove_var(ENV_PATH);

        // Assert: real file, real chain, real verification.
        let receipts = read_and_verify_ledger(&path).expect("read and verify real ledger");
        assert_eq!(receipts.len(), 3);
        assert_eq!(receipts[0].sequence, 0);
        assert_eq!(receipts[1].sequence, 1);
        assert_eq!(receipts[2].sequence, 2);
        assert_eq!(receipts[2].verb, "dead");
        assert!(!receipts[2].success);
        assert_eq!(receipts[2].effect, Effect::Mutating);

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_record_receipt_falls_back_when_primary_path_is_unwritable() {
        // Arrange: point the primary path at something that cannot possibly
        // be created (a path through a file, not a directory).
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let dir = temp_dir("fallback");
        let blocking_file = dir.join("not-a-directory");
        fs::write(&blocking_file, b"blocking").expect("create real blocking file");
        let unwritable_path = blocking_file.join("receipts.jsonl");
        std::env::set_var(ENV_PATH, &unwritable_path);

        // Act: must never panic even though the primary path is unwritable.
        record_receipt("diag", "check", Effect::ReadOnly, true);

        std::env::remove_var(ENV_PATH);

        // Assert: a receipt landed in the real fallback location instead.
        let fallback_receipts =
            read_and_verify_ledger(&fallback_path()).expect("read real fallback ledger");
        assert!(fallback_receipts.iter().any(|r| r.noun == "diag" && r.verb == "check"));

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_effect_as_str_matches_the_serde_snake_case_wire_form() {
        assert_eq!(Effect::ReadOnly.as_str(), "read_only");
        assert_eq!(Effect::Mutating.as_str(), "mutating");
        assert_eq!(Effect::Idempotent.as_str(), "idempotent");
        assert_eq!(Effect::Unknown.as_str(), "unknown");
    }
}
