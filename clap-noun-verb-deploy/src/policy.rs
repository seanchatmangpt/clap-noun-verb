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

/// Policy which admits every invocation that already passed schema validation.
#[derive(Debug, Clone, Copy, Default)]
pub struct AdmitValidated;

impl AdmissionPolicy for AdmitValidated {
    fn admit(&self, _invocation: &Invocation) -> Admission {
        Admission::Admitted
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
        Self {
            commands: commands.into_iter().collect(),
        }
    }

    #[must_use]
    pub fn allow(mut self, command_path: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.commands
            .insert(command_path.into_iter().map(Into::into).collect());
        self
    }
}

impl AdmissionPolicy for CommandAllowList {
    fn admit(&self, invocation: &Invocation) -> Admission {
        if self
            .commands
            .iter()
            .any(|command| invocation.args.starts_with(command.as_slice()))
        {
            Admission::Admitted
        } else {
            Admission::Refused {
                reason: "command path is outside the admitted allow-list".to_owned(),
            }
        }
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum AdmissionError {
    #[error("invocation refused: {0}")]
    Refused(String),
}
