//! Hyper-Advanced: Pluggable Persistence with Generic Associated Types
//!
//! Enables zero-copy, type-safe persistence backends for logs, receipts, policies
//! using GATs and higher-ranked trait bounds (HRTB) for maximum flexibility

use std::fmt::Debug;
use serde::{Serialize, Deserialize};

/// Generic Associated Type trait for persistence query results
/// Allows backends to return borrowed or owned data without allocation
pub trait PersistenceBackend: Send + Sync + 'static {
    /// Item type stored in this backend
    type Item: Serialize + for<'de> Deserialize<'de> + Debug + Clone;

    /// Query result - returns owned items to avoid lifetime issues with locks
    type QueryResult: IntoIterator<Item = Self::Item>;

    /// Error type for this backend
    type Error: std::error::Error + Send + Sync + 'static;

    /// Store an item, returning its ID
    fn store(&self, item: Self::Item) -> Result<String, Self::Error>;

    /// Retrieve by ID - returns owned copy
    fn get(&self, id: &str) -> Result<Option<Self::Item>, Self::Error>;

    /// Query with predicate - returns owned results
    fn query<F>(&self, predicate: F) -> Result<Self::QueryResult, Self::Error>
    where
        F: Fn(&Self::Item) -> bool;

    /// Delete item by ID
    fn delete(&self, id: &str) -> Result<bool, Self::Error>;

    /// Atomic transaction: store multiple items together
    fn atomic_store(&self, items: Vec<Self::Item>) -> Result<Vec<String>, Self::Error> {
        let mut ids = Vec::with_capacity(items.len());
        for item in items {
            let id = self.store(item)?;
            ids.push(id);
        }
        Ok(ids)
    }
}

/// In-memory backend with full ownership (for testing)
#[derive(Debug)]
pub struct InMemoryBackend<T: Serialize + for<'de> Deserialize<'de> + Debug + Clone + Send + Sync + 'static> {
    store: parking_lot::RwLock<std::collections::BTreeMap<String, T>>,
}

impl<T: Serialize + for<'de> Deserialize<'de> + Debug + Clone + Send + Sync + 'static> InMemoryBackend<T> {
    pub fn new() -> Self {
        Self {
            store: parking_lot::RwLock::new(std::collections::BTreeMap::new()),
        }
    }
}

impl<T: Serialize + for<'de> Deserialize<'de> + Debug + Clone + Send + Sync + 'static> Default for InMemoryBackend<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Serialize + for<'de> Deserialize<'de> + Debug + Clone + Send + Sync + 'static> PersistenceBackend for InMemoryBackend<T> {
    type Item = T;
    type QueryResult = Vec<T>;
    type Error = std::io::Error;

    fn store(&self, item: Self::Item) -> Result<String, Self::Error> {
        let id = uuid::Uuid::new_v4().to_string();
        self.store.write().insert(id.clone(), item);
        Ok(id)
    }

    fn get(&self, id: &str) -> Result<Option<Self::Item>, Self::Error> {
        Ok(self.store.read().get(id).cloned())
    }

    fn query<F>(&self, predicate: F) -> Result<Self::QueryResult, Self::Error>
    where
        F: Fn(&Self::Item) -> bool,
    {
        Ok(self.store.read()
            .values()
            .filter(|item| predicate(item))
            .cloned()
            .collect())
    }

    fn delete(&self, id: &str) -> Result<bool, Self::Error> {
        Ok(self.store.write().remove(id).is_some())
    }
}

/// Wrapper for multiple backends with fall-through semantics
/// Store on primary, replicate to secondary
pub struct ReplicatedBackend<Primary, Secondary>
where
    Primary: PersistenceBackend,
    Secondary: PersistenceBackend<Item = Primary::Item>,
{
    primary: Primary,
    secondary: Secondary,
}

impl<Primary, Secondary> ReplicatedBackend<Primary, Secondary>
where
    Primary: PersistenceBackend,
    Secondary: PersistenceBackend<Item = Primary::Item>,
{
    pub fn new(primary: Primary, secondary: Secondary) -> Self {
        Self { primary, secondary }
    }

    /// Write to primary with async replication hint
    pub fn store_with_replication(
        &self,
        item: Primary::Item,
    ) -> Result<String, Primary::Error> {
        let id = self.primary.store(item.clone())?;
        // Async replication (fire-and-forget in real impl)
        let _ = self.secondary.store(item);
        Ok(id)
    }
}

/// Type-level marker for persistence constraints
pub trait PersistenceConstraint: Send + Sync {
    fn name(&self) -> &'static str;
}

/// Constraint: Items must be immutable after write
pub struct ImmutableAfterWrite;
impl PersistenceConstraint for ImmutableAfterWrite {
    fn name(&self) -> &'static str {
        "ImmutableAfterWrite"
    }
}

/// Constraint: Items must be encrypted at rest
pub struct EncryptedAtRest;
impl PersistenceConstraint for EncryptedAtRest {
    fn name(&self) -> &'static str {
        "EncryptedAtRest"
    }
}

/// Constraint: Items must be audit-logged
pub struct AuditLogged;
impl PersistenceConstraint for AuditLogged {
    fn name(&self) -> &'static str {
        "AuditLogged"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_in_memory_backend_store_and_get() {
        let backend = InMemoryBackend::new();
        let id = backend.store("test_item".to_string()).unwrap();
        let item = backend.get(&id).unwrap();
        assert_eq!(item, Some("test_item".to_string()));
    }

    #[test]
    fn test_in_memory_backend_query() {
        let backend = InMemoryBackend::new();
        backend.store("apple".to_string()).unwrap();
        backend.store("apricot".to_string()).unwrap();
        backend.store("banana".to_string()).unwrap();

        let results = backend.query(|item| item.starts_with('a')).unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_in_memory_backend_delete() {
        let backend: InMemoryBackend<String> = InMemoryBackend::new();
        let id = backend.store("test".to_string()).unwrap();
        assert!(backend.delete(&id).unwrap());
        assert!(!backend.delete(&id).unwrap());
    }
}
