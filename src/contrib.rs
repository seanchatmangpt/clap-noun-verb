// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Deterministic contributor metadata helpers.
//!
//! The `contrib` capability is an in-memory construction surface. It records
//! admitted contributor identities and roles, then lists them deterministically.
//! It performs no network, filesystem, Git, or process actuation.

use crate::{NounVerbError, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// An admitted contributor identity and its declared roles.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Contributor {
    /// Stable contributor identifier, such as a GitHub login.
    pub id: String,
    /// Human-readable display name.
    pub display_name: String,
    /// Deterministically ordered contributor roles.
    pub roles: BTreeSet<String>,
}

impl Contributor {
    /// Construct a contributor with no roles.
    ///
    /// # Errors
    ///
    /// Returns [`NounVerbError`] when `id` or `display_name` is blank.
    pub fn new(id: impl Into<String>, display_name: impl Into<String>) -> Result<Self> {
        let id = id.into();
        let display_name = display_name.into();
        if id.trim().is_empty() {
            return Err(NounVerbError::argument_error("contributor id must not be blank"));
        }
        if display_name.trim().is_empty() {
            return Err(NounVerbError::argument_error(
                "contributor display name must not be blank",
            ));
        }
        Ok(Self {
            id,
            display_name,
            roles: BTreeSet::new(),
        })
    }

    /// Add one non-empty role.
    ///
    /// # Errors
    ///
    /// Returns [`NounVerbError`] when the role is blank.
    pub fn add_role(&mut self, role: impl Into<String>) -> Result<bool> {
        let role = role.into();
        if role.trim().is_empty() {
            return Err(NounVerbError::argument_error("contributor role must not be blank"));
        }
        Ok(self.roles.insert(role))
    }
}

/// In-memory contributor registry with deterministic `list` semantics.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContributorRegistry {
    contributors: BTreeMap<String, Contributor>,
}

impl ContributorRegistry {
    /// Create an empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register one contributor without replacing an existing identity.
    ///
    /// # Errors
    ///
    /// Returns [`NounVerbError`] when the contributor id is already registered.
    pub fn register(&mut self, contributor: Contributor) -> Result<()> {
        if self.contributors.contains_key(&contributor.id) {
            return Err(NounVerbError::argument_error(format!(
                "contributor already registered: {}",
                contributor.id
            )));
        }
        self.contributors.insert(contributor.id.clone(), contributor);
        Ok(())
    }

    /// Return contributors ordered by stable identifier.
    #[must_use]
    pub fn list(&self) -> Vec<&Contributor> {
        self.contributors.values().collect()
    }

    /// Resolve a contributor by stable identifier.
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&Contributor> {
        self.contributors.get(id)
    }

    /// Return the number of registered contributors.
    #[must_use]
    pub fn len(&self) -> usize {
        self.contributors.len()
    }

    /// Return whether no contributors are registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.contributors.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_is_deterministic_by_identifier() {
        let mut registry = ContributorRegistry::new();
        registry
            .register(Contributor::new("zeta", "Zeta").expect("valid contributor"))
            .expect("unique contributor");
        registry
            .register(Contributor::new("alpha", "Alpha").expect("valid contributor"))
            .expect("unique contributor");

        let ids: Vec<_> = registry.list().iter().map(|contributor| contributor.id.as_str()).collect();
        assert_eq!(ids, vec!["alpha", "zeta"]);
    }

    #[test]
    fn duplicate_identity_is_refused() {
        let mut registry = ContributorRegistry::new();
        registry
            .register(Contributor::new("alpha", "Alpha").expect("valid contributor"))
            .expect("first registration");
        let error = registry
            .register(Contributor::new("alpha", "Other Alpha").expect("valid contributor"))
            .expect_err("duplicate must be refused");
        assert!(error.to_string().contains("already registered"));
    }

    #[test]
    fn roles_are_unique_and_sorted() {
        let mut contributor = Contributor::new("alpha", "Alpha").expect("valid contributor");
        assert!(contributor.add_role("reviewer").expect("valid role"));
        assert!(contributor.add_role("maintainer").expect("valid role"));
        assert!(!contributor.add_role("reviewer").expect("duplicate role is bounded"));
        assert_eq!(
            contributor.roles.into_iter().collect::<Vec<_>>(),
            vec!["maintainer", "reviewer"]
        );
    }

    #[test]
    fn blank_identity_is_refused() {
        let error = Contributor::new("  ", "Nobody").expect_err("blank id must be refused");
        assert!(error.to_string().contains("id must not be blank"));
    }
}
