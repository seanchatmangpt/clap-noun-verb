use crate::{Execution, Executor, Invocation};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Evidence produced after an explicitly admitted execution.
///
/// `fingerprint` is a deterministic corruption/replay guard, not a cryptographic
/// signature and not a substitute for an external receipt authority.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionRecord {
    pub schema_version: u32,
    pub subject: String,
    pub invocation: Invocation,
    pub execution: Execution,
    pub fingerprint: String,
}

impl ExecutionRecord {
    #[must_use]
    pub fn new(subject: impl Into<String>, invocation: Invocation, execution: Execution) -> Self {
        let subject = subject.into();
        let fingerprint = fingerprint(&subject, &invocation, &execution);
        Self { schema_version: 1, subject, invocation, execution, fingerprint }
    }

    /// Verify that the record has not changed since manufacture.
    #[must_use]
    pub fn verify_integrity(&self) -> bool {
        self.fingerprint == fingerprint(&self.subject, &self.invocation, &self.execution)
    }

    /// Re-execute the exact invocation and compare the observed result.
    pub fn replay<E: Executor>(&self, executor: &E) -> Result<ReplayVerification, ReplayError> {
        if !self.verify_integrity() {
            return Err(ReplayError::RecordIntegrity);
        }
        let observed = executor
            .execute(&self.invocation)
            .map_err(|error| ReplayError::Execution(error.to_string()))?;
        Ok(ReplayVerification {
            matches: observed == self.execution,
            expected: self.execution.clone(),
            observed,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayVerification {
    pub matches: bool,
    pub expected: Execution,
    pub observed: Execution,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ReplayError {
    #[error("execution record integrity check failed")]
    RecordIntegrity,
    #[error("replay execution failed: {0}")]
    Execution(String),
}

fn fingerprint(subject: &str, invocation: &Invocation, execution: &Execution) -> String {
    let mut hash = 0xcbf29ce484222325_u64;
    feed(&mut hash, subject.as_bytes());
    for arg in &invocation.args {
        feed(&mut hash, &[0]);
        feed(&mut hash, arg.as_bytes());
    }
    for (key, value) in &invocation.env {
        feed(&mut hash, &[1]);
        feed(&mut hash, key.as_bytes());
        feed(&mut hash, &[2]);
        feed(&mut hash, value.as_bytes());
    }
    feed(&mut hash, &execution.exit_code.to_le_bytes());
    feed(&mut hash, execution.stdout.as_bytes());
    feed(&mut hash, &[3]);
    feed(&mut hash, execution.stderr.as_bytes());
    format!("fnv1a64:{hash:016x}")
}

fn feed(hash: &mut u64, bytes: &[u8]) {
    for byte in bytes {
        *hash ^= u64::from(*byte);
        *hash = hash.wrapping_mul(0x100000001b3);
    }
}
