use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::Connection;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

use thoth_errors::ThothResult;

pub type PgPool = Pool<ConnectionManager<PgConnection>>;

/// Default size of the dedicated metric-ingestion pool (`THOTH_INGEST_POOL_SIZE`).
///
/// A batch call holds at most two connections (guard and coordinator), so four
/// admit two concurrent batch calls and leave room for the lifecycle's
/// short calls (claim, begin, checkpoint, complete).
pub const DEFAULT_INGEST_POOL_SIZE: u32 = 4;

/// The dedicated pool of the metric-ingestion lifecycle.
///
/// A distinct type, so that it can sit beside the ordinary [`PgPool`] in
/// actix `Data` without the two being confused. Read traffic never draws from
/// it, and ingestion never draws from the ordinary pool.
#[derive(Clone)]
pub struct IngestPool(pub std::sync::Arc<PgPool>);

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

pub fn init_pool(database_url: &str) -> PgPool {
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    Pool::builder()
        .build(manager)
        .expect("Failed to create database pool.")
}

/// Build the metric-ingestion pool with at most `size` connections.
pub fn init_ingest_pool(database_url: &str, size: u32) -> IngestPool {
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    IngestPool(std::sync::Arc::new(
        Pool::builder()
            .max_size(size.max(2))
            .build(manager)
            .expect("Failed to create ingestion database pool."),
    ))
}

pub fn run_migrations(database_url: &str) -> ThothResult<()> {
    let mut connection = PgConnection::establish(database_url)?;
    connection
        .run_pending_migrations(MIGRATIONS)
        .map(|_| ())
        .map_err(Into::into)
}

pub fn revert_migrations(database_url: &str) -> ThothResult<()> {
    let mut connection = PgConnection::establish(database_url)?;
    connection
        .revert_all_migrations(MIGRATIONS)
        .map(|_| ())
        .map_err(Into::into)
}
