use thoth::api::{
    db::{
        init_pool as init_pg_pool, revert_migrations as revert_db_migrations,
        run_migrations as run_db_migrations, PgPool,
    },
    redis::{init_pool as init_redis_pool, RedisPool},
};
use thoth_errors::ThothResult;

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

pub(super) fn migrate(arguments: &clap::ArgMatches) -> ThothResult<()> {
    match arguments.get_flag("revert") {
        true => revert_migrations(arguments),
        false => run_migrations(arguments),
    }
}

pub(super) fn run_migrations(arguments: &clap::ArgMatches) -> ThothResult<()> {
    let database_url = arguments.get_one::<String>("db").unwrap();
    run_db_migrations(database_url)
}

fn revert_migrations(arguments: &clap::ArgMatches) -> ThothResult<()> {
    let database_url = arguments.get_one::<String>("db").unwrap();
    revert_db_migrations(database_url)
}
