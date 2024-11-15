use deadpool_redis::{redis::AsyncCommands, Config, Connection, Pool};
use dotenv::dotenv;
use std::env;
use thoth_errors::ThothResult;

pub type RedisPool = Pool;
type RedisConnection = Connection;

pub fn init_pool() -> RedisPool {
    dotenv().ok();
    let redis_url = env::var("REDIS_URL").expect("REDIS_URL must be set");
    Config::from_url(redis_url)
        .builder()
        .expect("Failed to create redis pool.")
        .build()
        .expect("Failed to build redis pool.")
}

async fn create_connection(pool: &RedisPool) -> ThothResult<RedisConnection> {
    pool.get().await.map_err(Into::into)
}

pub async fn set(pool: &RedisPool, key: &str, value: &str) -> ThothResult<()> {
    let mut con = create_connection(pool).await?;
    con.set(key, value).await.map_err(Into::into)
}

pub async fn get(pool: &RedisPool, key: &str) -> ThothResult<String> {
    let mut con = create_connection(pool).await?;
    con.get(key).await.map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_init_pool() {
        // Ensure that the pool initializes successfully
        let pool = init_pool();
        assert!(pool.get().await.is_ok());
    }

    #[tokio::test]
    async fn test_set_and_get() {
        let pool = init_pool();

        let test_key = "test_key";
        let test_value = "test_value";

        let set_result = set(&pool, test_key, test_value).await;
        assert!(set_result.is_ok());

        let get_result = get(&pool, test_key).await;
        assert!(get_result.is_ok());
        assert_eq!(get_result.unwrap(), test_value);
    }

    #[tokio::test]
    async fn test_get_nonexistent_key() {
        let pool = init_pool();

        let test_key = "nonexistent_key";
        let get_result = get(&pool, test_key).await;
        assert!(get_result.is_err());
    }
}
