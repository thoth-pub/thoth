//! Conventional request-scoped GraphQL DataLoader foundation (`ADR-0007`).
//!
//! The real GraphQL [`Context`](super::Context) owns one [`RequestLoaders`]
//! value per request, which is the lifecycle boundary every field-specific
//! loader uses. `BE-02` adds the first production consumer,
//! [`PublisherDistributionPlatformLoader`], backing
//! `Publisher.distributionPlatforms`.
//!
//! Test-only consumers below prove the approved batching, scheduling,
//! freshness, failure, Diesel and error semantics of the foundation itself;
//! field-specific evidence lives with the adopting field.

use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::Arc;

use dataloader::non_cached::Loader;
use dataloader::BatchFn;
use juniper::{graphql_value, FieldError};
use thoth_errors::ThothError;
use uuid::Uuid;

use crate::db::PgPool;
use crate::model::publisher_distribution_platform::crud::enabled_assignment_rows;
use crate::model::publisher_distribution_platform::PublisherDistributionPlatformAssignment;

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
/// Field-specific loaders call this constructor (or an equivalent that visibly
/// applies the same values) and use `try_load`; `Loader::load` is not an
/// approved database-loader API because a missing returned key panics.
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
/// Production consumers add typed loader fields here rather than introducing
/// global or application-scoped loader state. `BE-02` adds the first one.
pub(crate) struct RequestLoaders {
    /// `Publisher.distributionPlatforms` (`BE-02`), keyed by `publisher_id`.
    pub(crate) publisher_distribution_platforms: PublisherDistributionPlatformLoader,
    #[cfg(all(test, feature = "backend"))]
    pub(crate) fixture: Option<fixture::FixtureLoaders>,
}

impl RequestLoaders {
    /// Construct the ADR-0007 request-local bundle directly.
    ///
    /// Every loader is built per request and dropped with the request, so no
    /// loader and no completed loader result crosses a request boundary.
    pub(crate) fn for_request(pool: Arc<PgPool>) -> Self {
        Self {
            publisher_distribution_platforms: configured_loader(
                PublisherDistributionPlatformBatcher {
                    pool,
                    #[cfg(all(test, feature = "backend"))]
                    stats: None,
                },
            ),
            #[cfg(all(test, feature = "backend"))]
            fixture: None,
        }
    }

    /// The production bundle with the assignment loader's dispatch chunks
    /// recorded.
    ///
    /// Same constructor, same configuration, same batcher and same statement
    /// as [`Self::for_request`]; only the observation is test-only.
    #[cfg(all(test, feature = "backend"))]
    pub(crate) fn for_request_observed(pool: Arc<PgPool>, stats: Arc<fixture::BatchStats>) -> Self {
        Self {
            publisher_distribution_platforms: configured_loader(
                PublisherDistributionPlatformBatcher {
                    pool,
                    stats: Some(stats),
                },
            ),
            fixture: None,
        }
    }
}

/// The loaded value of one `Publisher.distributionPlatforms` key.
///
/// A publisher with no enabled assignment loads a successful empty vector; a
/// batch-wide backend failure loads a [`SharedBatchError`] for every key in the
/// failed chunk.
pub(crate) type PublisherDistributionPlatformValue =
    Result<Vec<PublisherDistributionPlatformAssignment>, SharedBatchError>;

pub(crate) type PublisherDistributionPlatformLoader =
    Loader<Uuid, PublisherDistributionPlatformValue, PublisherDistributionPlatformBatcher>;

/// Batch function for `Publisher.distributionPlatforms`.
///
/// The key is exactly `publisher_id`: the field takes no result-changing
/// argument, so no second key dimension exists. If the field's contract ever
/// gains one, this key contract must be revisited rather than silently reused.
pub(crate) struct PublisherDistributionPlatformBatcher {
    pool: Arc<PgPool>,
    /// Test-only dispatch observation. The production bundle leaves this
    /// `None`, so measured evidence exercises the production batcher itself.
    #[cfg(all(test, feature = "backend"))]
    stats: Option<Arc<fixture::BatchStats>>,
}

impl BatchFn<Uuid, PublisherDistributionPlatformValue> for PublisherDistributionPlatformBatcher {
    /// Load one dispatch chunk with exactly one set-based statement.
    ///
    /// Only `Arc<PgPool>` and an owned key vector move into the blocking
    /// closure; the Diesel connection is acquired, used and dropped entirely
    /// inside it, so no connection is ever held across an `.await`. There is no
    /// per-parent query loop, no fallback query and no retry.
    ///
    /// The result is total over the requested keys: every key is seeded with a
    /// successful empty vector before rows are grouped in, and a backend
    /// failure replaces every key's value with the shared error rather than
    /// omitting keys or fabricating empty success.
    async fn load(&mut self, keys: &[Uuid]) -> HashMap<Uuid, PublisherDistributionPlatformValue> {
        #[cfg(all(test, feature = "backend"))]
        if let Some(stats) = &self.stats {
            stats.record(keys);
        }
        let pool = Arc::clone(&self.pool);
        let owned_keys = keys.to_vec();
        let result = tokio::task::spawn_blocking(move || {
            let mut connection = pool.get().map_err(ThothError::from)?;
            enabled_assignment_rows(&mut connection, &owned_keys)
        })
        .await;

        let mut output: HashMap<Uuid, PublisherDistributionPlatformValue> =
            keys.iter().map(|key| (*key, Ok(Vec::new()))).collect();
        match result {
            Ok(Ok(rows)) => {
                // Rows arrive ordered by `(publisher_id, platform)`, so pushing
                // in order preserves canonical per-publisher platform order.
                for (publisher_id, platform, enabled_at) in rows {
                    if let Some(Ok(assignments)) = output.get_mut(&publisher_id) {
                        assignments.push(PublisherDistributionPlatformAssignment {
                            platform,
                            enabled_at,
                        });
                    }
                }
            }
            Ok(Err(error)) => {
                let error = SharedBatchError::from_thoth(error, ASSIGNMENT_ERROR_CONVENTION);
                for key in keys {
                    output.insert(*key, Err(error.clone()));
                }
            }
            Err(join_error) => {
                let error = SharedBatchError::from_thoth(
                    ThothError::InternalError(join_error.to_string()),
                    ASSIGNMENT_ERROR_CONVENTION,
                );
                for key in keys {
                    output.insert(*key, Err(error.clone()));
                }
            }
        }
        output
    }
}

/// `Publisher.distributionPlatforms` joins the publisher field family, whose
/// convention is `ThothResult -> FieldResult` through `.map_err(Into::into)`.
/// The loader must therefore produce the same message-only shape as the field's
/// direct synchronous baseline and invent no `extensions.type`.
pub(crate) const ASSIGNMENT_ERROR_CONVENTION: FieldErrorConvention =
    FieldErrorConvention::Conventional;

/// Project one loaded value onto the field's `FieldResult`.
///
/// `try_load` is the only approved database-loader API: a defective batch that
/// omits a requested key fails closed here instead of panicking inside
/// `Loader::load`.
pub(crate) fn unpack_assignments(
    outcome: Result<PublisherDistributionPlatformValue, std::io::Error>,
) -> Result<Vec<PublisherDistributionPlatformAssignment>, FieldError> {
    match outcome {
        Ok(Ok(assignments)) => Ok(assignments),
        Ok(Err(error)) => Err(error.to_field_error()),
        Err(missing) => Err(FieldError::new(
            format!("loader returned no entry: {missing}"),
            graphql_value!(None),
        )),
    }
}

/// Which existing GraphQL error conversion convention a field family uses.
///
/// The DataLoader foundation preserves the field's current convention rather
/// than normalizing the repository globally.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FieldErrorConvention {
    /// Ordinary `.map_err(Into::into)`: message only, no extensions object.
    Conventional,
    /// Explicit `ThothError::into_field_error`: preserve `extensions.type`.
    ///
    /// No production loader-backed field uses this convention yet; it exists
    /// so a future adopting field in that family keeps its current
    /// GraphQL-visible error shape.
    #[cfg_attr(not(test), allow(dead_code))]
    ExplicitThoth,
}

/// Cloneable, non-panicking projection of one batch-wide backend failure.
///
/// `ThothError` itself is not `Clone`. A failed batch must still provide an
/// error value for every requested key, so the batch boundary snapshots only
/// the GraphQL-visible information needed by the owning field convention.
/// There is deliberately no JSON/serde round trip here.
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
mod failure_tests;
#[cfg(all(test, feature = "backend"))]
pub(crate) mod fixture;
#[cfg(all(test, feature = "backend"))]
mod tests;
