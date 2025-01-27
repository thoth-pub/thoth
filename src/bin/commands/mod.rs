use thoth::api::{
    db::{init_pool as init_pg_pool, PgPool},
    redis::{init_pool as init_redis_pool, RedisPool},
};

pub(super) mod account;
pub(super) mod cache;
pub(super) mod start;

fn get_pg_pool(arguments: &clap::ArgMatches) -> PgPool {
    let database_url = arguments.get_one::<String>("db").unwrap();
    init_pg_pool(database_url)
}

fn get_redis_pool(arguments: &clap::ArgMatches) -> RedisPool {
    let redis_url = arguments.get_one::<String>("redis").unwrap();
    init_redis_pool(redis_url)
}
