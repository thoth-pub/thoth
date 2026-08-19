use crate::arguments;
use clap::Command;
use lazy_static::lazy_static;
use thoth::{
    api::{
        db::{revert_migrations as revert_db_migrations, run_migrations as run_db_migrations},
        redis::{init_pool as init_redis_pool, RedisPool},
    },
    errors::ThothResult,
};

pub(super) mod cache;
pub(super) mod publisher_services;
pub(super) mod start;
pub(super) mod zitadel;

lazy_static! {
    pub(super) static ref INIT: Command = Command::new("init")
        .about("Run the database migrations and start the thoth API server")
        .arg(arguments::database())
        .arg(arguments::host("GRAPHQL_API_HOST"))
        .arg(arguments::port("8000", "GRAPHQL_API_PORT"))
        .arg(arguments::threads("GRAPHQL_API_THREADS"))
        .arg(arguments::keep_alive("GRAPHQL_API_KEEP_ALIVE"))
        .arg(arguments::gql_url())
        .arg(arguments::key())
        .arg(arguments::zitadel_url())
        // `init` dispatches into the same handler as `start graphql-api`
        // (`src/bin/thoth.rs`), which reads `mutation-guard-mode`. Registering
        // the argument here is what makes that read find it: without it, a
        // release build silently resolved the default `OFF` and a debug build
        // panicked on the unknown argument (`THOTH-GQL-OPS-02`).
        .arg(arguments::mutation_guard_mode())
        // Same reason as `mutation-guard-mode` above, for the same dispatch:
        // `init` reaches the `graphql_api` handler, which reads
        // `distribution-job-creation`. Registering it on only one production
        // command path is the exact defect `THOTH-GQL-OPS-02` had to fix.
        .arg(arguments::distribution_job_creation())
        .arg(arguments::aws_access_key_id())
        .arg(arguments::aws_secret_access_key())
        .arg(arguments::aws_region());
}

lazy_static! {
    pub(super) static ref MIGRATE: Command = Command::new("migrate")
        .about("Run the database migrations")
        .arg(arguments::database())
        .arg(arguments::revert());
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

/// The `init` sequence: run the database migrations **first**, and start the
/// API only if they succeeded.
///
/// Generic over the two steps purely so that the ordering guarantee and the
/// abort-on-migration-failure guarantee can be asserted without a database or a
/// bound socket. The semantics are exactly those of the previous inline
/// `run_migrations(arguments)?; start::graphql_api(arguments)`.
pub(super) fn run_init<M, A>(run_migrations: M, start_api: A) -> ThothResult<()>
where
    M: FnOnce() -> ThothResult<()>,
    A: FnOnce() -> ThothResult<()>,
{
    run_migrations()?;
    start_api()
}

fn revert_migrations(arguments: &clap::ArgMatches) -> ThothResult<()> {
    let database_url = arguments.get_one::<String>("db").unwrap();
    revert_db_migrations(database_url)
}
