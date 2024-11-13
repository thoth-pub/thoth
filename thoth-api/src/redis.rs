use dotenv::dotenv;
pub use redis::{AsyncCommands, Client as RedisClient};
use std::env;

fn get_redis_url() -> String {
    dotenv().ok();
    env::var("REDIS_URL").expect("REDIS_URL must be set")
}

pub fn redis_client() -> RedisClient {
    let redis_url = get_redis_url();
    RedisClient::open(redis_url).expect("Failed to open redis client")
}
