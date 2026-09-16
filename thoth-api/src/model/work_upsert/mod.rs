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
mod tests;
