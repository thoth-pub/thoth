//! API-owned work-level incremental distribution (`BE-06`, #848): the generic
//! `WORK_UPSERT` substrate over the released `BE-04` durable jobs.
//!
//! The specification is `docs/publisher-services/specifications/BE-06-R52B.md`
//! as amended by #848 Amendments 1-3. Crossref is the only implemented
//! work-level execution profile; its permit, timestamp and version-floor model
//! lives in [`crate::model::crossref_write_permit`].
//!
//! Everything here is Publisher-Services-specific. There is no generic
//! cross-programme queue, scheduler or job framework (frozen rule 1, R52B §5).

pub mod policy;
pub mod registry;

#[cfg(feature = "backend")]
pub mod crud;

#[cfg(feature = "backend")]
use diesel::result::Error as DatabaseError;
#[cfg(feature = "backend")]
use diesel::{Connection, PgConnection, QueryResult};
#[cfg(feature = "backend")]
use thoth_errors::{ThothError, ThothResult};

#[cfg(feature = "backend")]
use crate::db::PgPool;

#[cfg(feature = "backend")]
use crate::model::publisher_distribution_platform::DistributionPlatform;

/// One profile's work-level control row (R52B sections 18.4 and 21.1).
///
/// `updated_at` is deliberately not carried: the transitions write only their
/// own flag and the row carries no binding (Amendment 3 section 4.5).
#[cfg_attr(feature = "backend", derive(diesel::Queryable))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkUpsertControl {
    pub execution_profile: crate::model::publisher_distribution_platform::DistributionPlatform,
    pub capture_enabled: bool,
    pub execution_enabled: bool,
}

/// How one seed unit ended (Amendment 3 section 9.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeedUnitOutcome {
    /// A `0` or absent generation row was raised to `1`.
    Seeded,
    /// The generation row was already positive; nothing was written.
    Observed,
    /// The Work no longer belongs to the publisher; nothing was written.
    BindingMovedRetryLater,
    /// The Work no longer exists; nothing was written.
    NoWork,
}

/// The result of one `seedCrossrefWorkUpsert` call (Amendment 3 section 9.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SeedCrossrefWorkUpsertResult {
    pub examined: i32,
    pub seeded: i32,
    pub observed: i32,
    pub binding_moved_retry_later: i32,
    pub no_work: i32,
    pub remaining_uncovered: i32,
}

/// One admission row (R52B section 18.5; Amendment 3 section 9.3).
///
/// `activation_id` is internal: it is never part of any public type
/// (Amendment 3 section 6).
#[cfg_attr(
    feature = "backend",
    derive(diesel::Queryable, diesel::QueryableByName)
)]
#[cfg_attr(feature = "backend", diesel(table_name = crate::schema::work_upsert_admission))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkUpsertAdmission {
    pub execution_profile: crate::model::publisher_distribution_platform::DistributionPlatform,
    pub publisher_id: uuid::Uuid,
    pub activation_id: uuid::Uuid,
    pub evidence_reference: String,
    pub actor: String,
    pub admitted_at: crate::model::Timestamp,
}

/// How one materialization unit ended (R52B section 9.3; Amendment 3 section
/// 4.5). Every end is an outcome, never an error.
#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLEnum),
    graphql(description = "How one work-level materialization unit ended")
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkUpsertMaterializationOutcome {
    #[cfg_attr(
        feature = "backend",
        graphql(description = "A job was created through the single creation helper")
    )]
    Created,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "A current PENDING job already exists; nothing was written")
    )]
    PendingCurrent,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "A RUNNING job exists and is never rewritten by materialization")
    )]
    RunningInFlight,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "The source generation is already resolved")
    )]
    Resolved,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "The Work is not eligible now; its residue stays durable")
    )]
    Ineligible,
    #[cfg_attr(
        feature = "backend",
        graphql(
            description = "The Work's current binding is not admitted; its residue stays durable"
        )
    )]
    ResidueNotAdmitted,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Capture is not enabled for the profile")
    )]
    CaptureDisabled,
    #[cfg_attr(
        feature = "backend",
        graphql(
            description = "The Work's binding moved while the unit ran; a later drain revisits it"
        )
    )]
    BindingMovedRetryLater,
    #[cfg_attr(feature = "backend", graphql(description = "The Work does not exist"))]
    NoWork,
}

/// The result of one materialization unit (Amendment 3 section 4.5).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkUpsertMaterialization {
    pub outcome: WorkUpsertMaterializationOutcome,
    /// Whether step 7 retired a stale `PENDING` job.
    pub rebound: bool,
    /// The created job, or the actionable job step 7 found.
    pub job: Option<crate::model::distribution_job::DistributionJob>,
}

/// The result of one `materializeWorkUpsertJobs` call (Amendment 3 section 9.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MaterializeWorkUpsertJobsResult {
    pub examined: i32,
    pub created: i32,
    pub rebound: i32,
    pub skipped_resolved: i32,
    pub skipped_ineligible: i32,
    pub skipped_not_admitted: i32,
    pub remaining_candidates: i32,
}

/// The advisory-lock namespace of the execution gates `Q` (R52B section 21.1),
/// distinct from the DOI keys' `1948572001`.
#[cfg(feature = "backend")]
const EXECUTION_GATE_NAMESPACE: i32 = 1_948_572_002;

/// Take the profile's execution gate `Q` for the rest of the transaction: in
/// SHARE mode for a consumer, EXCLUSIVE for `setWorkUpsertExecution` only.
#[cfg(feature = "backend")]
pub(crate) fn take_execution_gate(
    connection: &mut PgConnection,
    profile: DistributionPlatform,
    exclusive: bool,
) -> QueryResult<()> {
    use diesel::sql_types::{Integer, Text};
    use diesel::RunQueryDsl;

    let function = if exclusive {
        "pg_advisory_xact_lock"
    } else {
        "pg_advisory_xact_lock_shared"
    };
    diesel::sql_query(format!(
        "SELECT {function}($1, hashtext('be06:work_upsert:execution:' || $2))::text"
    ))
    .bind::<Integer, _>(EXECUTION_GATE_NAMESPACE)
    .bind::<Text, _>(profile.to_string())
    .execute(connection)
    .map(|_| ())
}

/// Take the execution gates `Q` of several profiles in SHARE mode, deduplicated
/// and ascending by key as signed `integer` (R52B section 10.1 rule 2).
#[cfg(feature = "backend")]
pub(crate) fn take_execution_gates(
    connection: &mut PgConnection,
    profiles: &[&'static registry::WorkLevelExecutionProfile],
) -> QueryResult<()> {
    use diesel::sql_types::{Array, Integer, Text};
    use diesel::RunQueryDsl;

    #[derive(diesel::QueryableByName)]
    struct Key {
        #[diesel(sql_type = Integer)]
        key: i32,
    }
    let labels: Vec<String> = profiles
        .iter()
        .map(|profile| profile.key.to_string())
        .collect();
    let mut keys: Vec<i32> = diesel::sql_query(
        "SELECT DISTINCT hashtext('be06:work_upsert:execution:' || p) AS key FROM unnest($1::text[]) AS p",
    )
    .bind::<Array<Text>, _>(labels)
    .load::<Key>(connection)?
    .into_iter()
    .map(|row| row.key)
    .collect();
    keys.sort_unstable();
    keys.dedup();
    for key in keys {
        diesel::sql_query("SELECT pg_advisory_xact_lock_shared($1, $2)::text")
            .bind::<Integer, _>(EXECUTION_GATE_NAMESPACE)
            .bind::<Integer, _>(key)
            .execute(connection)?;
    }
    Ok(())
}

/// The error a BE-06 transaction closure returns (Amendment 3 section 10.3).
///
/// `?` on a Diesel result inside the closure produces `Database`, which the
/// boundary converts through the scoped conversion; `?` on a `ThothResult`
/// produces `Thoth`, which passes through unchanged.
#[cfg(feature = "backend")]
#[derive(Debug)]
pub(crate) enum WorkUpsertTxError {
    Database(DatabaseError),
    Thoth(ThothError),
}

#[cfg(feature = "backend")]
impl From<DatabaseError> for WorkUpsertTxError {
    fn from(error: DatabaseError) -> Self {
        WorkUpsertTxError::Database(error)
    }
}

#[cfg(feature = "backend")]
impl From<ThothError> for WorkUpsertTxError {
    fn from(error: ThothError) -> Self {
        WorkUpsertTxError::Thoth(error)
    }
}

/// The one transaction boundary of every new BE-06 entry point (Amendment 3
/// section 10.3, EB1).
///
/// A pool acquisition failure, and every database error the closure, `BEGIN`,
/// `COMMIT` or rollback raises, is converted by the scoped conversion; nothing
/// about the pool, the driver or PostgreSQL's text reaches the caller, and an
/// unmapped failure is recorded server-side. The isolation level is
/// PostgreSQL's default `READ COMMITTED`, which this never changes.
#[cfg(feature = "backend")]
pub(crate) fn work_upsert_transaction<T, F>(db: &PgPool, f: F) -> ThothResult<T>
where
    F: FnOnce(&mut PgConnection) -> Result<T, WorkUpsertTxError>,
{
    let mut connection = match db.get() {
        Ok(connection) => connection,
        Err(pool_error) => {
            log::error!("BE-06 work-level database connection unavailable: {pool_error}");
            return Err(ThothError::WorkUpsertDatabaseFailure);
        }
    };
    let connection: &mut PgConnection = &mut connection;
    connection.transaction(f).map_err(|error| match error {
        WorkUpsertTxError::Database(error) => work_upsert_database_error(error),
        WorkUpsertTxError::Thoth(error) => error,
    })
}

/// Apply the scoped conversion to a BE-06 database error, recording an
/// unmapped failure server-side.
#[cfg(feature = "backend")]
fn work_upsert_database_error(error: DatabaseError) -> ThothError {
    let detail = error.to_string();
    let converted = ThothError::from_work_upsert_database_error(error);
    if converted == ThothError::WorkUpsertDatabaseFailure {
        log::error!("BE-06 work-level database operation failed: {detail}");
    }
    converted
}

/// The scoped conversion for a BE-06-added statement inside a released shared
/// operation (Amendment 3 section 10.3, EB2): `.work_upsert()?`, never a bare
/// `?`.
#[cfg(feature = "backend")]
pub(crate) trait WorkUpsertQueryResultExt<T> {
    fn work_upsert(self) -> ThothResult<T>;
}

#[cfg(feature = "backend")]
impl<T> WorkUpsertQueryResultExt<T> for QueryResult<T> {
    fn work_upsert(self) -> ThothResult<T> {
        self.map_err(work_upsert_database_error)
    }
}

#[cfg(all(test, feature = "backend"))]
pub(crate) mod tests;
