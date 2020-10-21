use std::env;
use std::io;

use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool, PoolError};
use diesel_migrations::embed_migrations;
use dotenv::dotenv;

use crate::errors;

pub type PgPool = Pool<ConnectionManager<PgConnection>>;

fn init_pool(database_url: &str) -> Result<PgPool, PoolError> {
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = Pool::builder()
        .build(manager)
        .expect("Failed to create database pool.");
    Ok(pool)
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
    init_pool(&database_url).unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

pub fn run_migrations() -> errors::Result<()> {
    embed_migrations!("migrations");
    let connection = establish_connection().get().unwrap();
    match embedded_migrations::run_with_output(&connection, &mut io::stdout()) {
        Ok(_) => Ok(()),
        Err(_) => {
            Err(errors::ThothError::DatabaseError("Could not run migrations".to_string()).into())
        }
    }
}
