use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::Connection;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

use thoth_errors::{ThothError, ThothResult};

pub type PgPool = Pool<ConnectionManager<PgConnection>>;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

pub fn init_pool(database_url: &str) -> PgPool {
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    Pool::builder()
        .build(manager)
        .expect("Failed to create database pool.")
}

pub fn run_migrations(database_url: &str) -> ThothResult<()> {
    let mut connection = PgConnection::establish(database_url)?;
    connection
        .run_pending_migrations(MIGRATIONS)
        .map(|_| ())
        .map_err(|error| ThothError::InternalError(error.to_string()))
}

pub fn revert_migrations(database_url: &str) -> ThothResult<()> {
    let mut connection = PgConnection::establish(database_url)?;
    connection
        .revert_all_migrations(MIGRATIONS)
        .map(|_| ())
        .map_err(|error| ThothError::InternalError(error.to_string()))
}
