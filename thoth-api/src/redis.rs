use deadpool_redis::{redis::AsyncCommands, Config, Connection, Pool};
use futures::StreamExt;
use thoth_errors::ThothResult;

pub type RedisPool = Pool;
type RedisConnection = Connection;

pub fn init_pool(redis_url: &str) -> RedisPool {
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

pub async fn del(pool: &RedisPool, key: &str) -> ThothResult<String> {
    let mut con = create_connection(pool).await?;
    con.del(key).await.map_err(Into::into)
}

pub async fn scan_match(pool: &RedisPool, pattern: &str) -> ThothResult<Vec<String>> {
    let mut con = create_connection(pool).await?;
    let keys: Vec<String> = con.scan_match(pattern).await?.collect().await;
    Ok(keys)
}

pub async fn rpush(pool: &RedisPool, key: &str, value: &str) -> ThothResult<String> {
    let mut con = create_connection(pool).await?;
    con.rpush(key, value).await.map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;
    use dotenv::dotenv;
    use std::env;

    async fn get_pool() -> RedisPool {
        dotenv().ok();
        let redis_url = env::var("TEST_REDIS_URL").expect("TEST_REDIS_URL must be set");
        init_pool(&redis_url)
    }

    #[tokio::test]
    async fn test_init_pool() {
        // Ensure that the pool initializes successfully
        let pool = get_pool().await;
        assert!(pool.get().await.is_ok());
    }

    #[tokio::test]
    async fn test_set_and_get() {
        let pool = get_pool().await;

        let test_key = "test_key";
        let test_value = "test_value";

        let set_result = set(&pool, test_key, test_value).await;
        assert!(set_result.is_ok());

        let get_result = get(&pool, test_key).await;
        assert!(get_result.is_ok());
        assert_eq!(get_result.unwrap(), test_value);
    }

    #[tokio::test]
    async fn test_rpush() {
        let pool = get_pool().await;

        let test_key = "test_queue";
        let test_value_1 = "test_value_1";
        let test_value_2 = "test_value_2";

        let rpush_result_1 = rpush(&pool, test_key, test_value_1).await;
        assert!(rpush_result_1.is_ok());

        let rpush_result_2 = rpush(&pool, test_key, test_value_2).await;
        assert!(rpush_result_2.is_ok());

    }

    #[tokio::test]
    async fn test_get_nonexistent_key() {
        let pool = get_pool().await;

        let test_key = "nonexistent_key";
        let get_result = get(&pool, test_key).await;
        assert!(get_result.is_err());
    }

    #[tokio::test]
    async fn test_del() {
        let pool = get_pool().await;
        let test_key = "test_key_to_delete";
        let test_value = "test_value";
        set(&pool, test_key, test_value).await.unwrap();

        let del_result = del(&pool, test_key).await;
        assert!(del_result.is_ok());
        let get_result = get(&pool, test_key).await;
        assert!(get_result.is_err());
    }

    #[tokio::test]
    async fn test_scan_match() {
        let pool = get_pool().await;
        set(&pool, "onix_3.0::key1", "value1").await.unwrap();
        set(&pool, "onix_3.0::key2", "value2").await.unwrap();
        set(&pool, "onix_3.0::key3", "value3").await.unwrap();

        let keys = scan_match(&pool, "onix_3.0::*").await.unwrap();
        assert_eq!(keys.len(), 3);
        assert!(keys.contains(&"onix_3.0::key1".to_string()));
        assert!(keys.contains(&"onix_3.0::key2".to_string()));
        assert!(keys.contains(&"onix_3.0::key3".to_string()));
    }
}
