use deadpool_redis::{redis::AsyncCommands, Config, Connection, Pool};
use dotenv::dotenv;
use std::env;
use thoth_errors::{ThothError, ThothResult};

pub type RedisPool = Pool;
type RedisConnection = Connection;

pub fn init_pool() -> RedisPool {
    Config::from_url(get_redis_url())
        .builder()
        .expect("Failed to create redis pool.")
        .build()
        .expect("Failed to build redis pool.")
}
fn get_redis_url() -> String {
    dotenv().ok();
    env::var("REDIS_URL").expect("REDIS_URL must be set")
}

async fn create_connection(pool: &RedisPool) -> ThothResult<RedisConnection> {
    pool.get().await.map_err(ThothError::from)
}

pub async fn set(pool: &RedisPool, key: &str, value: &str) -> ThothResult<()> {
    let mut con = create_connection(pool).await?;
    con.set(key, value).await.map_err(ThothError::from)
}

pub async fn get(pool: &RedisPool, key: &str) -> ThothResult<String> {
    let mut con = create_connection(pool).await?;
    con.get(key).await.map_err(ThothError::from)
}
