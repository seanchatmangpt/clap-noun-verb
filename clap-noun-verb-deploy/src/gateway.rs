use crate::{Admission, AdmissionPolicy, ExecutionRecord, Executor, Invocation};
use thiserror::Error;

/// The sole high-level DO path: admission precedes execution and every successful
/// executor call manufactures an execution record.
pub struct Gateway<E, P> {
    subject: String,
    executor: E,
    policy: P,
}

impl<E, P> Gateway<E, P> {
    #[must_use]
    pub fn new(subject: impl Into<String>, executor: E, policy: P) -> Self {
        Self { subject: subject.into(), executor, policy }
    }

    #[must_use]
    pub fn subject(&self) -> &str {
        &self.subject
    }
}

#[derive(Debug, Error)]
pub enum GatewayError {
    #[error("invocation refused: {0}")]
    Refused(String),
    #[error("executor failed: {0}")]
    Execution(String),
}

impl<E, P> Gateway<E, P>
where
    E: Executor,
    P: AdmissionPolicy,
{
    pub fn execute(&self, invocation: Invocation) -> Result<ExecutionRecord, GatewayError> {
        match self.policy.admit(&invocation) {
            Admission::Admitted => {}
            Admission::Refused { reason } => return Err(GatewayError::Refused(reason)),
        }
        let execution = self
            .executor
            .execute(&invocation)
            .map_err(|error| GatewayError::Execution(error.to_string()))?;
        Ok(ExecutionRecord::new(self.subject.clone(), invocation, execution))
    }
}
