use std::{
    any::Any,
    collections::{HashMap, HashSet},
    fmt::Debug,
    hash::Hash,
    sync::{Arc, RwLock},
};

use juniper::{graphql_value, FieldError};
use thoth_errors::{ThothError, ThothResult};
use uuid::Uuid;

use crate::db::PgPool;

use super::mutation_guard::MutationGuardMode;

/// Stable identity of a loader namespace.
///
/// New production loaders must add a closed variant here rather than using a
/// stringly-typed cache namespace.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum LoaderIdentity {
    PublisherDistributionPlatforms,
    #[cfg(test)]
    TestImprints,
}

/// Normalized, loader-owned cache-shape identity.
///
/// A loader constructs this from its typed GraphQL shape. Schema defaults must
/// be applied before this key is constructed.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum LoadShapeKey {
    Unit,
    #[cfg(test)]
    TestImprints { limit: i32 },
}

/// Type-safe parent-key representation used by the shared store.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum StoredParentKey {
    Uuid(Uuid),
}

/// The response-key namespace for one top-level GraphQL field occurrence.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct ScopeKey(String);

impl ScopeKey {
    pub(crate) fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    #[cfg(test)]
    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct StoreKey {
    scope: ScopeKey,
    loader: LoaderIdentity,
    shape: LoadShapeKey,
    parent: StoredParentKey,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct DispatchFailureKey {
    scope: ScopeKey,
    loader: LoaderIdentity,
    shape: LoadShapeKey,
    attempted_keys: Vec<StoredParentKey>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SharedErrorType {
    InvalidSubjectCode,
    NoAccess,
    InternalError,
}

#[derive(Clone, Debug)]
struct SharedLoadError {
    message: String,
    error_type: SharedErrorType,
}

impl SharedLoadError {
    fn from_thoth_error(error: &ThothError) -> Self {
        let (message, error_type) = match error {
            ThothError::InvalidSubjectCode { .. } => {
                (error.to_string(), SharedErrorType::InvalidSubjectCode)
            }
            ThothError::Unauthorised => ("Unauthorized".to_string(), SharedErrorType::NoAccess),
            _ => (error.to_string(), SharedErrorType::InternalError),
        };
        Self {
            message,
            error_type,
        }
    }

    fn to_field_error(&self) -> FieldError {
        let extensions = match self.error_type {
            SharedErrorType::InvalidSubjectCode => {
                graphql_value!({ "type": "INVALID_SUBJECT_CODE" })
            }
            SharedErrorType::NoAccess => graphql_value!({ "type": "NO_ACCESS" }),
            SharedErrorType::InternalError => graphql_value!({ "type": "INTERNAL_ERROR" }),
        };
        FieldError::new(self.message.clone(), extensions)
    }
}

enum StoredEntry {
    Loaded(Arc<dyn Any + Send + Sync>),
    LoadFailed(Arc<SharedLoadError>),
}

/// Child resolver lookup state. Only `NotLoaded` permits a direct fallback.
pub(crate) enum BatchLookup<V> {
    NotLoaded,
    Loaded(Vec<V>),
    LoadFailed(FieldError),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DispatchResult {
    Unavailable,
    AlreadyLoaded,
    Loaded,
    Failed,
}

/// Contract implemented by each set-based loader.
///
/// The loader owns its typed shape and the normalization that maps it to the
/// shared `LoadShapeKey`. The store never serializes shapes or keys to strings.
pub(crate) trait BatchLoader: Send + Sync + 'static {
    type Key: Clone + Debug + Eq + Hash + Send + Sync + 'static;
    type Value: Clone + Debug + Send + Sync + 'static;
    type Shape: Clone + Debug + Eq + Hash + Send + Sync + 'static;

    const IDENTITY: LoaderIdentity;

    fn shape_key(shape: &Self::Shape) -> LoadShapeKey;
    fn stored_key(key: &Self::Key) -> StoredParentKey;
    fn key_for_value(value: &Self::Value) -> Self::Key;

    /// Execute exactly one set-based statement for the supplied keys.
    fn load(db: &PgPool, keys: &[Self::Key], shape: &Self::Shape) -> ThothResult<Vec<Self::Value>>;
}

/// Request-scoped GraphQL batch store.
///
/// Store availability is derived directly from the guard mode. There is no
/// independent enable flag, so `OFF + store enabled` and `OBSERVE + store
/// enabled` cannot be represented by this type.
pub(crate) struct GraphqlBatchStore {
    mode: MutationGuardMode,
    entries: RwLock<HashMap<StoreKey, StoredEntry>>,
    failures: RwLock<HashMap<DispatchFailureKey, Arc<SharedLoadError>>>,
}

impl GraphqlBatchStore {
    pub(crate) fn new(mode: MutationGuardMode) -> Self {
        Self {
            mode,
            entries: RwLock::new(HashMap::new()),
            failures: RwLock::new(HashMap::new()),
        }
    }

    pub(crate) fn is_available(&self) -> bool {
        self.mode.store_available()
    }

    pub(crate) fn lookup<L: BatchLoader>(
        &self,
        scope: &ScopeKey,
        shape: &L::Shape,
        key: &L::Key,
    ) -> ThothResult<BatchLookup<L::Value>> {
        if !self.is_available() {
            return Ok(BatchLookup::NotLoaded);
        }

        let store_key = StoreKey {
            scope: scope.clone(),
            loader: L::IDENTITY,
            shape: L::shape_key(shape),
            parent: L::stored_key(key),
        };
        let entries = self.entries.read().map_err(|_| {
            ThothError::InternalError("GraphQL batch store read lock poisoned".to_string())
        })?;

        match entries.get(&store_key) {
            None => Ok(BatchLookup::NotLoaded),
            Some(StoredEntry::LoadFailed(error)) => {
                Ok(BatchLookup::LoadFailed(error.to_field_error()))
            }
            Some(StoredEntry::Loaded(value)) => value
                .as_ref()
                .downcast_ref::<Vec<L::Value>>()
                .cloned()
                .map(BatchLookup::Loaded)
                .ok_or_else(|| {
                    ThothError::InternalError(format!(
                        "GraphQL batch store type mismatch for loader {:?}",
                        L::IDENTITY
                    ))
                }),
        }
    }

    /// Load only keys that have no existing `Loaded` or `LoadFailed` state.
    ///
    /// Database/load failures are retained in the store and returned to child
    /// resolvers through `BatchLookup::LoadFailed`; they are deliberately not
    /// retried by a later read or dispatch in the same scope.
    pub(crate) fn dispatch<L: BatchLoader>(
        &self,
        db: &PgPool,
        scope: &ScopeKey,
        shape: &L::Shape,
        keys: &[L::Key],
    ) -> ThothResult<DispatchResult> {
        if !self.is_available() {
            return Ok(DispatchResult::Unavailable);
        }

        let shape_key = L::shape_key(shape);
        let mut seen = HashSet::new();
        let unique_keys: Vec<L::Key> = keys
            .iter()
            .filter(|key| seen.insert((*key).clone()))
            .cloned()
            .collect();

        let missing_keys = {
            let entries = self.entries.read().map_err(|_| {
                ThothError::InternalError("GraphQL batch store read lock poisoned".to_string())
            })?;
            unique_keys
                .iter()
                .filter(|key| {
                    let store_key = StoreKey {
                        scope: scope.clone(),
                        loader: L::IDENTITY,
                        shape: shape_key.clone(),
                        parent: L::stored_key(key),
                    };
                    !entries.contains_key(&store_key)
                })
                .cloned()
                .collect::<Vec<_>>()
        };

        if missing_keys.is_empty() {
            return Ok(DispatchResult::AlreadyLoaded);
        }

        let values = match L::load(db, &missing_keys, shape) {
            Ok(values) => values,
            Err(error) => {
                self.record_failure::<L>(scope, &shape_key, &missing_keys, &error)?;
                return Ok(DispatchResult::Failed);
            }
        };

        let mut partitions: HashMap<L::Key, Vec<L::Value>> = missing_keys
            .iter()
            .cloned()
            .map(|key| (key, Vec::new()))
            .collect();

        for value in values {
            let parent_key = L::key_for_value(&value);
            let Some(bucket) = partitions.get_mut(&parent_key) else {
                let error = ThothError::InternalError(format!(
                    "GraphQL batch loader {:?} returned an unrequested parent key",
                    L::IDENTITY
                ));
                self.record_failure::<L>(scope, &shape_key, &missing_keys, &error)?;
                return Ok(DispatchResult::Failed);
            };
            bucket.push(value);
        }

        let mut entries = self.entries.write().map_err(|_| {
            ThothError::InternalError("GraphQL batch store write lock poisoned".to_string())
        })?;
        for (key, values) in partitions {
            entries.insert(
                StoreKey {
                    scope: scope.clone(),
                    loader: L::IDENTITY,
                    shape: shape_key.clone(),
                    parent: L::stored_key(&key),
                },
                StoredEntry::Loaded(Arc::new(values)),
            );
        }
        Ok(DispatchResult::Loaded)
    }

    fn record_failure<L: BatchLoader>(
        &self,
        scope: &ScopeKey,
        shape: &LoadShapeKey,
        keys: &[L::Key],
        error: &ThothError,
    ) -> ThothResult<()> {
        let error = Arc::new(SharedLoadError::from_thoth_error(error));
        let mut attempted_keys: Vec<StoredParentKey> =
            keys.iter().map(L::stored_key).collect();
        attempted_keys.sort();
        attempted_keys.dedup();

        {
            let mut failures = self.failures.write().map_err(|_| {
                ThothError::InternalError("GraphQL batch failure lock poisoned".to_string())
            })?;
            failures.insert(
                DispatchFailureKey {
                    scope: scope.clone(),
                    loader: L::IDENTITY,
                    shape: shape.clone(),
                    attempted_keys,
                },
                Arc::clone(&error),
            );
        }

        let mut entries = self.entries.write().map_err(|_| {
            ThothError::InternalError("GraphQL batch store write lock poisoned".to_string())
        })?;
        for key in keys {
            entries.insert(
                StoreKey {
                    scope: scope.clone(),
                    loader: L::IDENTITY,
                    shape: shape.clone(),
                    parent: L::stored_key(key),
                },
                StoredEntry::LoadFailed(Arc::clone(&error)),
            );
        }
        Ok(())
    }

    /// Clear every cached success and failure in the request-scoped store.
    pub(crate) fn invalidate_all(&self) -> ThothResult<()> {
        self.entries
            .write()
            .map_err(|_| {
                ThothError::InternalError("GraphQL batch store write lock poisoned".to_string())
            })?
            .clear();
        self.failures
            .write()
            .map_err(|_| {
                ThothError::InternalError("GraphQL batch failure lock poisoned".to_string())
            })?
            .clear();
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn failure_count(&self) -> usize {
        self.failures.read().expect("batch failure lock").len()
    }
}
