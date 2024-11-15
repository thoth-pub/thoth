use std::env;

use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::Connection;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use dotenv::dotenv;

use thoth_errors::{ThothError, ThothResult};

pub type PgPool = Pool<ConnectionManager<PgConnection>>;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

pub fn init_pool() -> PgPool {
    let manager = ConnectionManager::<PgConnection>::new(get_database_url());
    Pool::builder()
        .build(manager)
        .expect("Failed to create database pool.")
}

fn get_database_url() -> String {
    dotenv().ok();
    env::var("DATABASE_URL").expect("DATABASE_URL must be set")
}

pub fn run_migrations() -> ThothResult<()> {
    let mut connection = PgConnection::establish(&get_database_url())?;
    connection
        .run_pending_migrations(MIGRATIONS)
        .map(|_| ())
        .map_err(|error| ThothError::InternalError(error.to_string()))
}

pub fn revert_migrations() -> ThothResult<()> {
    let mut connection = PgConnection::establish(&get_database_url())?;
    connection
        .revert_all_migrations(MIGRATIONS)
        .map(|_| ())
        .map_err(|error| ThothError::InternalError(error.to_string()))
}
