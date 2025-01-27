use thoth_api::db::{init_pool as init_pg_pool, PgPool};

pub(crate) mod account;

fn get_pg_pool(arguments: &clap::ArgMatches) -> PgPool {
    let database_url = arguments.get_one::<String>("db").unwrap();
    init_pg_pool(database_url)
}
