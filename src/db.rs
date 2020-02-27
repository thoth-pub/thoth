use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool, PoolError};
use std::env;

pub type PgPool = Pool<ConnectionManager<PgConnection>>;

fn init_pool(database_url: &str) -> Result<PgPool, PoolError> {
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = Pool::builder()
        .build(manager)
        .expect("Failed to create database pool.");
    Ok(pool)
}

fn get_database_url() -> String {
    let database_url = if cfg!(test) {
        env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL must be set")
    } else {
        env::var("DATABASE_URL").expect("DATABASE_URL must be set")
    };
    database_url
}

pub fn establish_connection() -> PgPool {
    let database_url = get_database_url();
    init_pool(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}
