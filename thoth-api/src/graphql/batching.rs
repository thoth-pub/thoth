//! Request-scoped GraphQL batch store (`ADR-0006` sections 4.1-4.12).
//!
//! # Lifetime and ownership
//!
//! The store is owned by exactly one GraphQL request, lives on the request's
//! [`Context`](super::Context), and dies with it. There is no global, static or
//! process-wide cache and no singleton: two concurrent requests share nothing.
//!
//! # Identity
//!
//! Every entry is keyed by the full four-part identity of `ADR-0006`
//! invariant 13:
//!
//! ```text
//! (top-level GraphQL response key, loader identity, normalized load shape, parent key)
//! ```
//!
//! applied **uniformly** to queries and mutation payloads. Storage lifetime and
//! reuse namespace are distinct concepts (invariant 20): the store never
//! crosses requests, and reuse is confined to one top-level response key
//! *within* a request.
//!
//! There is deliberately **no** separate namespace for indirectly (descendant-)
//! prefetched entries (`ADR-0006` section 4.19.2). Within one scope, an entry
//! prefetched from an ancestor and one prefetched from the terminal field's own
//! parent list are the same entry and satisfy each other's lookups. Across
//! scopes they are distinct entries.
//!
//! # Availability
//!
//! Store availability is **derived** from [`MutationGuardMode`] and from
//! nothing else, so `OFF + store available` and `OBSERVE + store available` are
//! structurally unrepresentable (`ADR-0006` invariant 30). There is no second
//! "enable loaders" flag anywhere in the codebase.

// THOTH-GQL-BATCH-01 delivers this foundation and **adopts it in no production
// field**, so in a non-test build every loader-facing item below is legitimately
// unreferenced. That is the specified merged state, not an oversight: the store
// is also unavailable outside `ENFORCE`, and the merged guard mode is `OFF`.
// The `#[cfg(test)]` proof fixture and test matrix exercise all of it. The first
// adopting task removes this attribute when it wires a production loader.
#![cfg_attr(not(test), allow(dead_code))]

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
/// This is a **closed** discriminant: a new loader must add a variant here
/// rather than using a stringly-typed cache namespace, so two loaders can never
/// accidentally share a namespace by spelling the same string.
///
/// It currently carries only this task's proof loader, and that is the honest
/// encoding of this task's scope: the foundation adopts **no production
/// field**, so no production loader exists yet. An adopting task adds its own
/// variant under its own authorization.
///
/// The variant is deliberately **not** gated on `cfg(test)`. Gating it would
/// leave this enum — and [`LoadShapeKey`] with it — uninhabited in a production
/// build, which in turn makes the store's generic code statically unreachable.
/// One compiled-in, never-constructed discriminant is a better encoding than an
/// uninhabited key type, and it names no `BE-02` concept.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum LoaderIdentity {
    /// Proof loader used by this task's `#[cfg(test)]` fixture: imprints keyed
    /// by `publisher_id`. No production field maps to it.
    TestImprints,
}

/// Normalized, loader-owned cache-shape identity (`ADR-0006` section 4.4.2).
///
/// A loader constructs this from its typed GraphQL shape through a **single**
/// loader-owned constructor used by both the prefetch site and the child
/// lookup, so the two cannot drift.
///
/// Schema defaults must be applied *before* this key is constructed: juniper's
/// `LookAheadSelection::arguments()` reads only literal AST arguments and does
/// not apply schema defaults, while the child resolver receives the
/// default-applied value (`ADR-0006` section 4.4.3). Normalizing here is what
/// makes an omitted argument and an explicitly supplied schema default resolve
/// against the same entry.
///
/// This is a typed value, never a serialized GraphQL argument string.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
/// Like [`LoaderIdentity`], the variant is not `cfg(test)`-gated, so this type
/// stays inhabited in a production build.
pub(crate) enum LoadShapeKey {
    /// Proof shape for the test fixture's loader, carrying the normalized
    /// `limit` argument.
    TestImprints { limit: i32 },
}

/// Type-safe parent-key representation used by the shared store.
///
/// Keys reach the store only from already-resolved, already-authorized parents
/// (`ADR-0006` invariant 3); this type is not a general "load by id" facility
/// and must never carry a key taken from user input.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum StoredParentKey {
    Uuid(Uuid),
}

/// The response-key namespace for one top-level GraphQL field occurrence.
///
/// This is the GraphQL **response key** — therefore the alias when one is
/// present — and is never normalized to the schema field name
/// (`ADR-0006` invariant 21). It is produced solely by
/// [`super::scope::top_level_response_key`].
///
/// No source-position or AST-occurrence component is part of it: `ADR-0006`
/// section 4.12.6.3 rejects that on evidence, because `path()` and `location()`
/// can both be identical across two distinct top-level mutation executions.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct ScopeKey(String);

impl ScopeKey {
    pub(crate) fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

/// The full four-part store identity of `ADR-0006` invariant 13.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct StoreKey {
    scope: ScopeKey,
    loader: LoaderIdentity,
    shape: LoadShapeKey,
    parent: StoredParentKey,
}

/// Failure identity: `(scope, loader, shape, attempted key set)`.
///
/// The scope component is never dropped, so a `LoadFailed` recorded under one
/// scope can never poison another (`ADR-0006` sections 4.9.2, 4.12;
/// invariant 31).
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct DispatchFailureKey {
    scope: ScopeKey,
    loader: LoaderIdentity,
    shape: LoadShapeKey,
    attempted_keys: Vec<StoredParentKey>,
}

/// The `extensions.type` discriminant a failure must reproduce.
///
/// `ThothError` is not `Clone` (`thoth-errors/src/lib.rs`), so a shareable
/// representation is retained instead of the error itself. The classification
/// is preserved so the prefetched path produces the **same** `extensions.type`
/// as the direct path (`ADR-0006` section 4.9.3).
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

/// One stored entry. Absence from the map is `NotLoaded`.
///
/// The three states of `ADR-0006` section 4.7 are mutually unambiguous:
/// `Loaded(vec![])` is a present entry holding an empty vector, and is
/// therefore distinguishable both from absence and from `LoadFailed`.
enum StoredEntry {
    Loaded(Arc<dyn Any + Send + Sync>),
    LoadFailed(Arc<SharedLoadError>),
}

/// Child resolver lookup state (`ADR-0006` section 4.7).
///
/// | State | Child resolver must | Must not |
/// |---|---|---|
/// | `NotLoaded` | execute its ordinary direct per-parent query | — |
/// | `Loaded(rows)` | return `rows` | issue any database query |
/// | `Loaded([])` | return the empty result | query, or treat it as a miss |
/// | `LoadFailed(e)` | return the field error | query, retry, or return empty |
///
/// **Only `NotLoaded` may direct-fallback.** `LoadFailed` must neither retry
/// nor become empty data.
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum BatchLookup<V> {
    NotLoaded,
    Loaded(Vec<V>),
    LoadFailed(FieldError),
}

/// Outcome of one dispatch, for test and evidence purposes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum DispatchResult {
    /// The store is unavailable (guard mode is not `ENFORCE`), so nothing was
    /// loaded and every lookup will read `NotLoaded`.
    Unavailable,
    /// Every requested key already had `Loaded` or `LoadFailed` state under
    /// this `(scope, loader, shape)`; no SQL was issued.
    AlreadyLoaded,
    /// One set-based statement was issued and its result partitioned.
    Loaded,
    /// The dispatch failed; the failure is retained and will not be retried.
    Failed,
}

/// Contract implemented by each set-based loader (`ADR-0006` section 4.5).
///
/// The loader owns its typed shape and the normalization that maps it to the
/// shared [`LoadShapeKey`]. The store never serializes shapes or keys to
/// strings.
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) trait BatchLoader: Send + Sync + 'static {
    type Key: Clone + Debug + Eq + Hash + Send + Sync + 'static;
    type Value: Clone + Debug + Send + Sync + 'static;
    type Shape: Clone + Debug + Eq + Hash + Send + Sync + 'static;

    const IDENTITY: LoaderIdentity;

    /// The **single** loader-owned shape constructor, used by both the prefetch
    /// site and the child lookup so the two cannot drift.
    fn shape_key(shape: &Self::Shape) -> LoadShapeKey;
    fn stored_key(key: &Self::Key) -> StoredParentKey;
    /// Which parent bucket a returned row belongs to.
    fn key_for_value(value: &Self::Value) -> Self::Key;

    /// Execute **exactly one** set-based statement for the supplied keys, using
    /// `.eq_any(..)` (`WHERE key = ANY(..)`), returning raw canonical model rows
    /// rather than GraphQL objects. Never iterate keys issuing per-key
    /// statements.
    fn load(db: &PgPool, keys: &[Self::Key], shape: &Self::Shape) -> ThothResult<Vec<Self::Value>>;
}

/// Request-scoped GraphQL batch store.
///
/// Interior mutability is `RwLock`, which keeps `Context: Sync` so the async
/// execution path continues to compile (`ADR-0006` section 4.1).
///
/// Availability is derived directly from the guard mode, so
/// `OFF + store enabled` and `OBSERVE + store enabled` cannot be represented by
/// this type: there is no constructor taking an independent enable flag.
pub(crate) struct GraphqlBatchStore {
    mode: MutationGuardMode,
    entries: RwLock<HashMap<StoreKey, StoredEntry>>,
    failures: RwLock<HashMap<DispatchFailureKey, Arc<SharedLoadError>>>,
}

impl GraphqlBatchStore {
    /// Construct an empty store for one request, in the request's guard mode.
    pub(crate) fn new(mode: MutationGuardMode) -> Self {
        Self {
            mode,
            entries: RwLock::new(HashMap::new()),
            failures: RwLock::new(HashMap::new()),
        }
    }

    /// Whether the store may be used, derived from the guard mode alone.
    pub(crate) fn is_available(&self) -> bool {
        self.mode.store_available()
    }

    /// Read the state for one `(scope, loader, shape, parent key)`.
    ///
    /// **Reads are non-destructive**: the entry stays in the store, so a second
    /// read of the same entry returns the same bucket (`ADR-0006` section 4.6).
    ///
    /// When the store is unavailable every lookup reads `NotLoaded`, so every
    /// path takes its always-correct direct fallback.
    #[cfg_attr(not(test), allow(dead_code))]
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

    /// Load, in one set-based statement, only those keys that have no existing
    /// `Loaded` or `LoadFailed` state under this `(scope, loader, shape)`.
    ///
    /// Dispatch happens **once per unique `(scope, loader, shape)`** over the
    /// de-duplicated key set — never once per parent, and never shared across
    /// argument variants or across scopes (`ADR-0006` section 4.4.4).
    ///
    /// Database/load failures are retained in the store and surfaced to child
    /// resolvers through [`BatchLookup::LoadFailed`]; they are deliberately
    /// **not** retried by a later read or dispatch in the same scope
    /// (`ADR-0006` section 4.9).
    #[cfg_attr(not(test), allow(dead_code))]
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

        // De-duplicate the requested keys, preserving first-seen order.
        //
        // Written as an explicit loop with a separate membership set rather
        // than a `filter(|k| seen.insert(k.clone()))` expression: the
        // side-effecting-predicate form is easy to misread, and its correctness
        // silently depends on `HashSet::insert`'s return value. `n` references
        // to one key must yield exactly one key.
        let mut seen_keys: HashSet<L::Key> = HashSet::with_capacity(keys.len());
        let mut unique_keys: Vec<L::Key> = Vec::with_capacity(keys.len());
        for key in keys {
            let is_new = seen_keys.insert(key.clone());
            if is_new {
                unique_keys.push(key.clone());
            }
        }

        // Of those, keep only the ones with no state yet. An existing
        // `LoadFailed` counts as state, so a failure is never retried.
        let missing_keys = {
            let entries = self.entries.read().map_err(|_| {
                ThothError::InternalError("GraphQL batch store read lock poisoned".to_string())
            })?;
            let mut missing = Vec::with_capacity(unique_keys.len());
            for key in &unique_keys {
                let store_key = StoreKey {
                    scope: scope.clone(),
                    loader: L::IDENTITY,
                    shape: shape_key.clone(),
                    parent: L::stored_key(key),
                };
                if !entries.contains_key(&store_key) {
                    missing.push(key.clone());
                }
            }
            missing
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

        // Seed an exact empty bucket for every requested key first, so a key
        // the statement returned no rows for becomes `Loaded([])` — genuinely
        // empty — rather than absent. `Loaded([])` must never be represented as
        // absence (`ADR-0006` section 4.7).
        let mut partitions: HashMap<L::Key, Vec<L::Value>> = missing_keys
            .iter()
            .cloned()
            .map(|key| (key, Vec::new()))
            .collect();

        // Partitioning is a pure function of the returned rows and the input
        // keys: every returned row lands in the bucket for its own key and no
        // other, preserving the statement's row order within each bucket so the
        // per-key ordering matches the direct per-parent result.
        for value in values {
            let parent_key = L::key_for_value(&value);
            let Some(bucket) = partitions.get_mut(&parent_key) else {
                // A row for a key we did not ask for means the loader's
                // `key_for_value` and its statement disagree. Fail closed
                // rather than silently dropping or misattributing the row.
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

    /// Record a dispatch failure once per `(scope, loader, shape)` with the
    /// attempted key set, and mark every covered key `LoadFailed`.
    ///
    /// The parent list resolver still returns its parents successfully: the
    /// failure does not become the parent list field's error. Each covered
    /// child resolver returns the derived `FieldError`, and **no retry query is
    /// issued**.
    fn record_failure<L: BatchLoader>(
        &self,
        scope: &ScopeKey,
        shape: &LoadShapeKey,
        keys: &[L::Key],
        error: &ThothError,
    ) -> ThothResult<()> {
        let error = Arc::new(SharedLoadError::from_thoth_error(error));
        let mut attempted_keys: Vec<StoredParentKey> = keys.iter().map(L::stored_key).collect();
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

    /// Clear every cached success and failure across **all** scopes, loaders,
    /// shapes and keys (`ADR-0006` section 4.12.5).
    ///
    /// Required by the architecture and deliberately **unused** by this task:
    /// ordinary correctness, including mutation read-after-write, comes from
    /// top-level response-key partitioning, not from write-time invalidation.
    /// Avoiding that retrofit is a principal reason `ADR-0006` partitions by
    /// scope, and it is why no production mutation resolver is modified.
    #[cfg_attr(not(test), allow(dead_code))]
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

    #[cfg(test)]
    pub(crate) fn entry_count(&self) -> usize {
        self.entries.read().expect("batch store lock").len()
    }
}
