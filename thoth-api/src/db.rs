use std::env;

use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use dotenv::dotenv;

use thoth_errors::{ThothError, ThothResult};

pub type PgPool = Pool<ConnectionManager<PgConnection>>;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

fn init_pool(database_url: &str) -> PgPool {
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    Pool::builder()
        .build(manager)
        .expect("Failed to create database pool.")
}

fn get_database_url() -> String {
    dotenv().ok();
    if cfg!(test) {
        env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL must be set")
    } else {
        env::var("DATABASE_URL").expect("DATABASE_URL must be set")
    }
}

pub fn establish_connection() -> PgPool {
    let database_url = get_database_url();
    init_pool(&database_url)
}

pub fn run_migrations() -> ThothResult<()> {
    let mut connection = establish_connection().get().unwrap();
    match connection.run_pending_migrations(MIGRATIONS) {
        Ok(_) => Ok(()),
        Err(_) => Err(ThothError::DatabaseError(
            "Could not run migrations".to_string(),
        )),
    }
}
