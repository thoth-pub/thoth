//! Conventional request-scoped GraphQL DataLoader foundation (`ADR-0007`).
//!
//! Production fields do not adopt a loader in `THOTH-GQL-DATALOADER-01`.
//! The real GraphQL [`Context`](super::Context) nevertheless owns one
//! [`RequestLoaders`] value per request, providing the lifecycle boundary that
//! future field-specific loaders must use. Test-only consumers below prove the
//! approved batching, scheduling, freshness, failure, Diesel, and error
//! semantics without changing the production SDL.

use std::fmt::Debug;
use std::hash::Hash;
use std::sync::Arc;

use dataloader::non_cached::Loader;
use dataloader::BatchFn;
use juniper::{graphql_value, FieldError};
use thoth_errors::ThothError;

/// Approved explicit maximum number of unique keys in one dispatch chunk.
pub(crate) const MAX_BATCH_SIZE: usize = 200;
/// Approved explicit scheduler-yield budget before dispatch.
pub(crate) const YIELD_COUNT: usize = 10;

/// The configuration every production Thoth DataLoader must use initially.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct LoaderConfig {
    pub(crate) max_batch_size: usize,
    pub(crate) yield_count: usize,
}

pub(crate) const LOADER_CONFIG: LoaderConfig = LoaderConfig {
    max_batch_size: MAX_BATCH_SIZE,
    yield_count: YIELD_COUNT,
};

/// Build the approved non-cached loader without relying on crate defaults.
///
/// Future field-specific loaders call this constructor (or an equivalent that
/// visibly applies the same values) and use `try_load`; `Loader::load` is not
/// an approved database-loader API because a missing returned key panics.
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn configured_loader<K, V, F>(batch_fn: F) -> Loader<K, V, F>
where
    K: Eq + Hash + Clone + Debug,
    V: Clone,
    F: BatchFn<K, V>,
{
    Loader::new(batch_fn)
        .with_max_batch_size(LOADER_CONFIG.max_batch_size)
        .with_yield_count(LOADER_CONFIG.yield_count)
}

/// Request-local loader bundle owned by the real GraphQL context.
///
/// `THOTH-GQL-DATALOADER-01` deliberately adopts no production child field, so
/// the production bundle has no field-specific loader yet. This value is still
/// constructed with every `Context` and dropped with it; future production
/// consumers add typed loader fields here rather than introducing global or
/// application-scoped loader state.
pub(crate) struct RequestLoaders {
    #[cfg(all(test, feature = "backend"))]
    pub(crate) fixture: Option<fixture::FixtureLoaders>,
}

impl RequestLoaders {
    /// Construct the ADR-0007 request-local bundle directly.
    pub(crate) fn for_request() -> Self {
        Self {
            #[cfg(all(test, feature = "backend"))]
            fixture: None,
        }
    }

    /// Temporary source-compatibility constructor for the old
    /// `GraphqlBatchStore::new(mode)` call in `Context` while that large file is
    /// migrated in this same implementation. The argument is deliberately
    /// ignored: guard mode cannot enable or disable DataLoader availability.
    pub(crate) fn new<T>(_legacy_guard_mode: T) -> Self {
        Self::for_request()
    }
}

impl Default for RequestLoaders {
    fn default() -> Self {
        Self::for_request()
    }
}

/// Which existing GraphQL error conversion convention a field family uses.
///
/// The DataLoader foundation preserves the field's current convention rather
/// than normalizing the repository globally.
#[cfg_attr(not(test), allow(dead_code))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FieldErrorConvention {
    /// Ordinary `.map_err(Into::into)`: message only, no extensions object.
    Conventional,
    /// Explicit `ThothError::into_field_error`: preserve `extensions.type`.
    ExplicitThoth,
}

/// Cloneable, non-panicking projection of one batch-wide backend failure.
///
/// `ThothError` itself is not `Clone`. A failed batch must still provide an
/// error value for every requested key, so the batch boundary snapshots only
/// the GraphQL-visible information needed by the owning field convention.
/// There is deliberately no JSON/serde round trip here.
#[cfg_attr(not(test), allow(dead_code))]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SharedBatchError {
    message: Arc<str>,
    extension_type: Option<&'static str>,
}

impl SharedBatchError {
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn from_message(message: impl Into<String>) -> Self {
        let message: String = message.into();
        Self {
            message: Arc::<str>::from(message),
            extension_type: None,
        }
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn from_thoth(error: ThothError, convention: FieldErrorConvention) -> Self {
        let (message, extension_type) = match convention {
            FieldErrorConvention::Conventional => (error.to_string(), None),
            FieldErrorConvention::ExplicitThoth => match &error {
                ThothError::InvalidSubjectCode { .. } => {
                    (error.to_string(), Some("INVALID_SUBJECT_CODE"))
                }
                ThothError::Unauthorised => ("Unauthorized".to_string(), Some("NO_ACCESS")),
                _ => (error.to_string(), Some("INTERNAL_ERROR")),
            },
        };
        Self {
            message: Arc::<str>::from(message),
            extension_type,
        }
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn to_field_error(&self) -> FieldError {
        match self.extension_type {
            Some(kind) => {
                FieldError::new(self.message.to_string(), graphql_value!({ "type": kind }))
            }
            None => FieldError::new(self.message.to_string(), graphql_value!(None)),
        }
    }
}

#[cfg(all(test, feature = "backend"))]
pub(crate) mod fixture;
#[cfg(all(test, feature = "backend"))]
mod failure_tests;
#[cfg(all(test, feature = "backend"))]
mod tests;
