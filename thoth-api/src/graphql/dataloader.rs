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
use crate::model::distribution_job::crud::{
    attempts_for_jobs, latest_back_catalogue_job_payloads, targets_for_jobs,
};
use crate::model::distribution_job::{
    DistributionJobAttempt, DistributionJobPayload, DistributionJobTarget,
};
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
    /// `PublisherServiceConfigurationSummary.latestBackCatalogueJob` (`BE-04`),
    /// keyed by `publisher_id`, valued by the **complete** field value.
    ///
    /// This is the report path's only job loader. The two loaders below are
    /// **not** reachable from the report.
    pub(crate) latest_back_catalogue_jobs: LatestBackCatalogueJobLoader,
    /// `DistributionJob.targets` (`BE-04`), keyed by `distribution_job_id`.
    ///
    /// Retained **only** for the single-job mutation payloads of
    /// `completeDistributionJob`, `failDistributionJob` and
    /// `cancelDistributionJob`, where the cohort is one job and the
    /// dependent-arrival question does not arise. It records zero dispatches on
    /// the report path.
    pub(crate) distribution_job_targets: DistributionJobTargetLoader,
    /// `DistributionJob.attempts` (`BE-04`), keyed by `distribution_job_id`.
    ///
    /// Retained on the same single-job mutation-payload basis as
    /// [`Self::distribution_job_targets`].
    pub(crate) distribution_job_attempts: DistributionJobAttemptLoader,
    #[cfg(all(test, feature = "backend"))]
    pub(crate) fixture: Option<fixture::FixtureLoaders>,
}

/// Per-loader dispatch observation for the whole production bundle.
#[cfg(all(test, feature = "backend"))]
#[derive(Default)]
pub(crate) struct ObservedLoaderStats {
    pub(crate) publisher_distribution_platforms: Arc<fixture::BatchStats>,
    pub(crate) latest_back_catalogue_jobs: Arc<fixture::BatchStats>,
    pub(crate) distribution_job_targets: Arc<fixture::BatchStats>,
    pub(crate) distribution_job_attempts: Arc<fixture::BatchStats>,
}

impl RequestLoaders {
    /// Construct the ADR-0007 request-local bundle directly.
    ///
    /// Every loader is built per request and dropped with the request, so no
    /// loader and no completed loader result crosses a request boundary.
    #[cfg(all(test, feature = "backend"))]
    pub(crate) fn for_request(pool: Arc<PgPool>) -> Self {
        Self::build(pool, None)
    }

    /// Construct the ADR-0007 request-local bundle directly.
    ///
    /// Every loader is built per request and dropped with the request, so no
    /// loader and no completed loader result crosses a request boundary.
    #[cfg(not(all(test, feature = "backend")))]
    pub(crate) fn for_request(pool: Arc<PgPool>) -> Self {
        Self::build(pool)
    }

    /// The production bundle with the assignment loader's dispatch chunks
    /// recorded.
    ///
    /// Same constructor, same configuration, same batcher and same statement
    /// as [`Self::for_request`]; only the observation is test-only.
    #[cfg(all(test, feature = "backend"))]
    pub(crate) fn for_request_observed(pool: Arc<PgPool>, stats: Arc<fixture::BatchStats>) -> Self {
        Self::build(
            pool,
            Some(ObservedLoaderStats {
                publisher_distribution_platforms: stats,
                ..ObservedLoaderStats::default()
            }),
        )
    }

    /// The production bundle with **every** loader's dispatch chunks recorded.
    ///
    /// This is what `BE-04`'s selection-dependent statement-count evidence uses:
    /// it must show not only how many statements ran, but which loaders
    /// dispatched and in how many chunks.
    #[cfg(all(test, feature = "backend"))]
    pub(crate) fn for_request_observed_all(pool: Arc<PgPool>, stats: ObservedLoaderStats) -> Self {
        Self::build(pool, Some(stats))
    }

    #[cfg(all(test, feature = "backend"))]
    fn build(pool: Arc<PgPool>, stats: Option<ObservedLoaderStats>) -> Self {
        let stats = stats.unwrap_or_default();
        Self {
            publisher_distribution_platforms: configured_loader(
                PublisherDistributionPlatformBatcher {
                    pool: Arc::clone(&pool),
                    stats: Some(stats.publisher_distribution_platforms),
                },
            ),
            latest_back_catalogue_jobs: configured_loader(LatestBackCatalogueJobBatcher {
                pool: Arc::clone(&pool),
                stats: Some(stats.latest_back_catalogue_jobs),
            }),
            distribution_job_targets: configured_loader(DistributionJobTargetBatcher {
                pool: Arc::clone(&pool),
                stats: Some(stats.distribution_job_targets),
            }),
            distribution_job_attempts: configured_loader(DistributionJobAttemptBatcher {
                pool,
                stats: Some(stats.distribution_job_attempts),
            }),
            fixture: None,
        }
    }

    #[cfg(not(all(test, feature = "backend")))]
    fn build(pool: Arc<PgPool>) -> Self {
        Self {
            publisher_distribution_platforms: configured_loader(
                PublisherDistributionPlatformBatcher {
                    pool: Arc::clone(&pool),
                },
            ),
            latest_back_catalogue_jobs: configured_loader(LatestBackCatalogueJobBatcher {
                pool: Arc::clone(&pool),
            }),
            distribution_job_targets: configured_loader(DistributionJobTargetBatcher {
                pool: Arc::clone(&pool),
            }),
            distribution_job_attempts: configured_loader(DistributionJobAttemptBatcher { pool }),
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

// ---------------------------------------------------------------------------
// `BE-04` job loaders
// ---------------------------------------------------------------------------

/// The loaded value of one `latestBackCatalogueJob` key: the **complete** field
/// value, not a bare job needing further loads.
///
/// A publisher with no durable job loads a successful `None`, which is the only
/// representation of "no job" this codebase has: nothing fabricates a
/// placeholder job, a synthetic status or a zero count.
pub(crate) type LatestBackCatalogueJobValue =
    Result<Option<DistributionJobPayload>, SharedBatchError>;

pub(crate) type LatestBackCatalogueJobLoader =
    Loader<Uuid, LatestBackCatalogueJobValue, LatestBackCatalogueJobBatcher>;

/// Composite batch function for
/// `PublisherServiceConfigurationSummary.latestBackCatalogueJob`
/// (specification section 17.4.2).
///
/// The key is exactly `publisher_id` — the key the report already holds at
/// resolver entry — and the value is the whole field: the latest
/// `PUBLISHER_BACK_CATALOGUE` job *together with* its targets and attempts, or
/// `None`. The field takes no result-changing argument, so no second key
/// dimension exists.
///
/// This is not two loaders merged to make a count fit. It is `ADR-0007` section
/// 4.4's "one loader represents one reviewed logical field/query family" applied
/// to the family this field actually is: the targets and attempts of the latest
/// job are sub-structure of the one returned value, reachable from nowhere else
/// in the report. Loading them here is what makes the whole field **one
/// loader-first cohort**, which is the only shape with a stated bound. The
/// rejected alternative — a latest-job loader feeding two loaders keyed by
/// `distribution_job_id` — produces a dependent-arrival cohort whose only
/// provable bound is `ceil(N / max_batch_size) <= dispatches <= N`.
///
/// The load shape deliberately does not depend on the query's own selection:
/// targets and attempts are materialized whenever the field is selected at all,
/// even for `latestBackCatalogueJob { status }`. Deciding otherwise would need
/// Juniper look-ahead at the resolver, which is the retired `ADR-0006`
/// mechanism.
pub(crate) struct LatestBackCatalogueJobBatcher {
    pool: Arc<PgPool>,
    #[cfg(all(test, feature = "backend"))]
    stats: Option<Arc<fixture::BatchStats>>,
}

impl BatchFn<Uuid, LatestBackCatalogueJobValue> for LatestBackCatalogueJobBatcher {
    /// Load one dispatch chunk inside **one** `spawn_blocking` boundary on
    /// **one** pooled connection acquired and dropped inside that closure, so no
    /// connection is ever held across an `.await` (`ADR-0007` section 4.7).
    ///
    /// Three set-based statements for a chunk whose L1 returns at least one job,
    /// one for a chunk whose L1 returns none. There is no per-publisher,
    /// per-job, per-target or per-attempt statement, no fallback query and no
    /// retry.
    ///
    /// The result is **total** over the requested keys: every key is seeded with
    /// a successful `None` before jobs are placed, so a publisher with no job
    /// returns the absent value rather than a missing map entry. A failure in
    /// any of L1, L2 or L3 replaces every key's value with the shared error, so
    /// the chunk fails closed for all of them, with no partially populated job
    /// and no successful empty substitution.
    async fn load(&mut self, keys: &[Uuid]) -> HashMap<Uuid, LatestBackCatalogueJobValue> {
        let pool = Arc::clone(&self.pool);
        let owned_keys = keys.to_vec();
        let result = tokio::task::spawn_blocking(move || {
            let mut connection = pool.get().map_err(ThothError::from)?;
            latest_back_catalogue_job_payloads(&mut connection, &owned_keys)
        })
        .await;

        // Recorded after the load, so the chunk's size and whether its L1
        // returned a job — which is what decides whether L2 and L3 ran — are
        // one atomic observation.
        #[cfg(all(test, feature = "backend"))]
        if let Some(stats) = &self.stats {
            stats.record_outcome(
                keys,
                match &result {
                    Ok(Ok(payloads)) => Some(!payloads.is_empty()),
                    _ => None,
                },
            );
        }

        let mut output: HashMap<Uuid, LatestBackCatalogueJobValue> =
            keys.iter().map(|key| (*key, Ok(None))).collect();
        match result {
            Ok(Ok(payloads)) => {
                for payload in payloads {
                    if let Some(slot) = output.get_mut(&payload.job.publisher_id) {
                        *slot = Ok(Some(payload));
                    }
                }
            }
            Ok(Err(error)) => fail_batch(&mut output, keys, error),
            Err(join_error) => fail_batch(
                &mut output,
                keys,
                ThothError::InternalError(join_error.to_string()),
            ),
        }
        output
    }
}

/// The loaded value of one `DistributionJob.targets` key.
pub(crate) type DistributionJobTargetValue = Result<Vec<DistributionJobTarget>, SharedBatchError>;

pub(crate) type DistributionJobTargetLoader =
    Loader<Uuid, DistributionJobTargetValue, DistributionJobTargetBatcher>;

/// Batch function for `DistributionJob.targets`, keyed by `distribution_job_id`.
pub(crate) struct DistributionJobTargetBatcher {
    pool: Arc<PgPool>,
    #[cfg(all(test, feature = "backend"))]
    stats: Option<Arc<fixture::BatchStats>>,
}

impl BatchFn<Uuid, DistributionJobTargetValue> for DistributionJobTargetBatcher {
    async fn load(&mut self, keys: &[Uuid]) -> HashMap<Uuid, DistributionJobTargetValue> {
        #[cfg(all(test, feature = "backend"))]
        if let Some(stats) = &self.stats {
            stats.record(keys);
        }
        let pool = Arc::clone(&self.pool);
        let owned_keys = keys.to_vec();
        let result = tokio::task::spawn_blocking(move || {
            let mut connection = pool.get().map_err(ThothError::from)?;
            targets_for_jobs(&mut connection, &owned_keys)
        })
        .await;

        let mut output: HashMap<Uuid, DistributionJobTargetValue> =
            keys.iter().map(|key| (*key, Ok(Vec::new()))).collect();
        match result {
            Ok(Ok(rows)) => {
                // Rows arrive ordered by `(distribution_job_id, platform)`, so
                // pushing in order preserves canonical per-job platform order.
                for target in rows {
                    if let Some(Ok(targets)) = output.get_mut(&target.distribution_job_id) {
                        targets.push(target);
                    }
                }
            }
            Ok(Err(error)) => fail_batch(&mut output, keys, error),
            Err(join_error) => fail_batch(
                &mut output,
                keys,
                ThothError::InternalError(join_error.to_string()),
            ),
        }
        output
    }
}

/// The loaded value of one `DistributionJob.attempts` key.
pub(crate) type DistributionJobAttemptValue = Result<Vec<DistributionJobAttempt>, SharedBatchError>;

pub(crate) type DistributionJobAttemptLoader =
    Loader<Uuid, DistributionJobAttemptValue, DistributionJobAttemptBatcher>;

/// Batch function for `DistributionJob.attempts`, keyed by
/// `distribution_job_id`.
///
/// This is a **separate** loader from the target loader, with a different value
/// and a different statement. The two are deliberately not merged to lower a
/// statement count: that would trade a correct, independently batched,
/// independently fail-closed pair for a contrived join.
pub(crate) struct DistributionJobAttemptBatcher {
    pool: Arc<PgPool>,
    #[cfg(all(test, feature = "backend"))]
    stats: Option<Arc<fixture::BatchStats>>,
}

impl BatchFn<Uuid, DistributionJobAttemptValue> for DistributionJobAttemptBatcher {
    async fn load(&mut self, keys: &[Uuid]) -> HashMap<Uuid, DistributionJobAttemptValue> {
        #[cfg(all(test, feature = "backend"))]
        if let Some(stats) = &self.stats {
            stats.record(keys);
        }
        let pool = Arc::clone(&self.pool);
        let owned_keys = keys.to_vec();
        let result = tokio::task::spawn_blocking(move || {
            let mut connection = pool.get().map_err(ThothError::from)?;
            attempts_for_jobs(&mut connection, &owned_keys)
        })
        .await;

        let mut output: HashMap<Uuid, DistributionJobAttemptValue> =
            keys.iter().map(|key| (*key, Ok(Vec::new()))).collect();
        match result {
            Ok(Ok(rows)) => {
                // Rows arrive most recent first within each parent.
                for attempt in rows {
                    if let Some(Ok(attempts)) = output.get_mut(&attempt.distribution_job_id) {
                        attempts.push(attempt);
                    }
                }
            }
            Ok(Err(error)) => fail_batch(&mut output, keys, error),
            Err(join_error) => fail_batch(
                &mut output,
                keys,
                ThothError::InternalError(join_error.to_string()),
            ),
        }
        output
    }
}

/// Replace every key's value with one shared error.
///
/// A batch-wide backend failure fails closed for **every** requested key. It
/// never becomes successful empty data, and no per-key fallback query runs
/// afterwards to recover individual results (`ADR-0007` invariant 9).
fn fail_batch<V, E>(
    output: &mut HashMap<Uuid, Result<V, SharedBatchError>>,
    keys: &[Uuid],
    error: E,
) where
    E: Into<ThothError>,
{
    let error = SharedBatchError::from_thoth(error.into(), JOB_ERROR_CONVENTION);
    for key in keys {
        output.insert(*key, Err(error.clone()));
    }
}

/// The `BE-04` job fields join the same field family as
/// `enabledDistributionPlatforms`, whose merged convention is
/// `ThothResult -> FieldResult` through `.map_err(Into::into)`. The loaders must
/// therefore produce the same message-only shape as a direct read would and
/// invent no `extensions.type`.
pub(crate) const JOB_ERROR_CONVENTION: FieldErrorConvention = FieldErrorConvention::Conventional;

/// Project one loaded value onto its field's `FieldResult`.
///
/// `try_load` is the only approved database-loader API: a defective batch that
/// omits a requested key fails closed here instead of panicking inside
/// `Loader::load`.
pub(crate) fn unpack_loaded<T>(
    outcome: Result<Result<T, SharedBatchError>, std::io::Error>,
) -> Result<T, FieldError> {
    match outcome {
        Ok(Ok(value)) => Ok(value),
        Ok(Err(error)) => Err(error.to_field_error()),
        Err(missing) => Err(FieldError::new(
            format!("loader returned no entry: {missing}"),
            graphql_value!(None),
        )),
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
