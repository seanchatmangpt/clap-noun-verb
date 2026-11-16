use crate::Invocation;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use thiserror::Error;

/// Admission decision made before an invocation can reach an executor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "decision", rename_all = "snake_case")]
pub enum Admission {
    Admitted,
    Refused { reason: String },
}

impl Admission {
    #[must_use]
    pub const fn is_admitted(&self) -> bool {
        matches!(self, Self::Admitted)
    }
}

/// Pure admission boundary. Implementations inspect but never execute an invocation.
pub trait AdmissionPolicy: Send + Sync {
    fn admit(&self, invocation: &Invocation) -> Admission;
}

/// Admit schema-validated argv while refusing per-invocation environment mutation.
#[derive(Debug, Clone, Copy, Default)]
pub struct AdmitValidated;

impl AdmissionPolicy for AdmitValidated {
    fn admit(&self, invocation: &Invocation) -> Admission {
        admit_no_environment(invocation)
    }
}

/// Least-authority policy for selecting explicit callable command paths.
#[derive(Debug, Clone, Default)]
pub struct CommandAllowList {
    commands: BTreeSet<Vec<String>>,
}

impl CommandAllowList {
    #[must_use]
    pub fn new(commands: impl IntoIterator<Item = Vec<String>>) -> Self {
        Self { commands: commands.into_iter().filter(|command| !command.is_empty()).collect() }
    }

    #[must_use]
    pub fn allow(mut self, command_path: impl IntoIterator<Item = impl Into<String>>) -> Self {
        let command = command_path.into_iter().map(Into::into).collect::<Vec<_>>();
        if !command.is_empty() {
            self.commands.insert(command);
        }
        self
    }
}

impl AdmissionPolicy for CommandAllowList {
    fn admit(&self, invocation: &Invocation) -> Admission {
        if !invocation.env.is_empty() {
            return admit_no_environment(invocation);
        }
        if self.commands.iter().any(|command| invocation.args.starts_with(command.as_slice())) {
            Admission::Admitted
        } else {
            Admission::Refused {
                reason: "command path is outside the admitted allow-list".to_owned(),
            }
        }
    }
}

/// Explicitly permits selected per-invocation environment names after another policy admits argv.
#[derive(Debug, Clone)]
pub struct EnvironmentAllowList<P> {
    inner: P,
    names: BTreeSet<String>,
}

impl<P> EnvironmentAllowList<P> {
    #[must_use]
    pub fn new(inner: P, names: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self { inner, names: names.into_iter().map(Into::into).collect() }
    }
}

impl<P: AdmissionPolicy> AdmissionPolicy for EnvironmentAllowList<P> {
    fn admit(&self, invocation: &Invocation) -> Admission {
        let mut argv_only = invocation.clone();
        argv_only.env.clear();
        match self.inner.admit(&argv_only) {
            Admission::Admitted => {}
            refused => return refused,
        }
        if let Some(name) = invocation.env.keys().find(|name| !self.names.contains(*name)) {
            return Admission::Refused {
                reason: format!("environment variable '{name}' is outside the admitted allow-list"),
            };
        }
        Admission::Admitted
    }
}

fn admit_no_environment(invocation: &Invocation) -> Admission {
    if invocation.env.is_empty() {
        Admission::Admitted
    } else {
        Admission::Refused {
            reason: "per-invocation environment mutation is not admitted by default".to_owned(),
        }
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum AdmissionError {
    #[error("invocation refused: {0}")]
    Refused(String),
}
