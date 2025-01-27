use clap::{crate_authors, crate_version, value_parser, Arg, ArgAction, Command};
use dialoguer::{console::Term, MultiSelect};
use dotenv::dotenv;
use std::env;
use thoth::{
    api::{
        db::{revert_migrations, run_migrations},
        redis::{del, init_pool as init_redis_pool, scan_match},
    },
    api_server, app_server,
    errors::{ThothError, ThothResult},
    export_server, ALL_SPECIFICATIONS,
};

mod commands;

fn database_argument() -> Arg {
    Arg::new("db")
        .short('D')
        .long("database-url")
        .value_name("DATABASE_URL")
        .env("DATABASE_URL")
        .help("Full postgres database url, e.g. postgres://thoth:thoth@localhost/thoth")
        .num_args(1)
}

fn redis_argument() -> Arg {
    Arg::new("redis")
        .short('R')
        .long("redis-url")
        .value_name("REDIS_URL")
        .env("REDIS_URL")
        .help("Full redis url, e.g. redis://localhost:6379")
        .num_args(1)
}

fn host_argument(env_value: &'static str) -> Arg {
    Arg::new("host")
        .short('H')
        .long("host")
        .value_name("HOST")
        .env(env_value)
        .default_value("0.0.0.0")
        .help("host to bind")
        .num_args(1)
}

fn port_argument(default_value: &'static str, env_value: &'static str) -> Arg {
    Arg::new("port")
        .short('p')
        .long("port")
        .value_name("PORT")
        .env(env_value)
        .default_value(default_value)
        .help("Port to bind")
        .num_args(1)
}

fn domain_argument() -> Arg {
    Arg::new("domain")
        .short('d')
        .long("domain")
        .value_name("THOTH_DOMAIN")
        .env("THOTH_DOMAIN")
        .default_value("localhost")
        .help("Authentication cookie domain")
        .num_args(1)
}

fn key_argument() -> Arg {
    Arg::new("key")
        .short('k')
        .long("secret-key")
        .value_name("SECRET")
        .env("SECRET_KEY")
        .help("Authentication cookie secret key")
        .num_args(1)
}

fn session_argument() -> Arg {
    Arg::new("duration")
        .short('s')
        .long("session-length")
        .value_name("DURATION")
        .env("SESSION_DURATION_SECONDS")
        .default_value("3600")
        .help("Authentication cookie session duration (seconds)")
        .num_args(1)
        .value_parser(value_parser!(i64))
}

fn gql_url_argument() -> Arg {
    Arg::new("gql-url")
        .short('u')
        .long("gql-url")
        .value_name("THOTH_GRAPHQL_API")
        .env("THOTH_GRAPHQL_API")
        .default_value("http://localhost:8000")
        .help("Thoth GraphQL's, public facing, root URL.")
        .num_args(1)
}

fn gql_endpoint_argument() -> Arg {
    Arg::new("gql-endpoint")
        .short('g')
        .long("gql-endpoint")
        .value_name("THOTH_GRAPHQL_ENDPOINT")
        .env("THOTH_GRAPHQL_ENDPOINT")
        .default_value("http://localhost:8000/graphql")
        .help("Thoth GraphQL's endpoint")
        .num_args(1)
}

fn export_url_argument() -> Arg {
    Arg::new("export-url")
        .short('u')
        .long("export-url")
        .value_name("THOTH_EXPORT_API")
        .env("THOTH_EXPORT_API")
        .default_value("http://localhost:8181")
        .help("Thoth Export API's, public facing, root URL.")
        .num_args(1)
}

fn threads_argument(env_value: &'static str) -> Arg {
    Arg::new("threads")
        .short('t')
        .long("threads")
        .value_name("THREADS")
        .env(env_value)
        .default_value("5")
        .help("Number of HTTP workers to start")
        .num_args(1)
        .value_parser(value_parser!(usize))
}

fn keep_alive_argument(env_value: &'static str) -> Arg {
    Arg::new("keep-alive")
        .short('K')
        .long("keep-alive")
        .value_name("THREADS")
        .env(env_value)
        .default_value("5")
        .help("Number of seconds to wait for subsequent requests")
        .num_args(1)
        .value_parser(value_parser!(u64))
}

fn thoth_commands() -> Command {
    Command::new(env!("CARGO_PKG_NAME"))
        .version(crate_version!())
        .author(crate_authors!("\n"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("migrate")
                .about("Run the database migrations")
                .arg(database_argument())
                .arg(
                    Arg::new("revert")
                        .long("revert")
                        .help("Revert all database migrations")
                        .action(ArgAction::SetTrue),
                ),
        )
        .subcommand(
            Command::new("start")
                .about("Start an instance of Thoth API or GUI")
                .subcommand_required(true)
                .arg_required_else_help(true)
                .subcommand(
                    Command::new("graphql-api")
                        .about("Start the thoth GraphQL API server")
                        .arg(database_argument())
                        .arg(host_argument("GRAPHQL_API_HOST"))
                        .arg(port_argument("8000", "GRAPHQL_API_PORT"))
                        .arg(threads_argument("GRAPHQL_API_THREADS"))
                        .arg(keep_alive_argument("GRAPHQL_API_KEEP_ALIVE"))
                        .arg(gql_url_argument())
                        .arg(domain_argument())
                        .arg(key_argument())
                        .arg(session_argument()),
                )
                .subcommand(
                    Command::new("app")
                        .about("Start the thoth client GUI")
                        .arg(host_argument("APP_HOST"))
                        .arg(port_argument("8080", "APP_PORT"))
                        .arg(threads_argument("APP_THREADS"))
                        .arg(keep_alive_argument("APP_KEEP_ALIVE")),
                )
                .subcommand(
                    Command::new("export-api")
                        .about("Start the thoth metadata export API")
                        .arg(redis_argument())
                        .arg(host_argument("EXPORT_API_HOST"))
                        .arg(port_argument("8181", "EXPORT_API_PORT"))
                        .arg(threads_argument("EXPORT_API_THREADS"))
                        .arg(keep_alive_argument("EXPORT_API_KEEP_ALIVE"))
                        .arg(export_url_argument())
                        .arg(gql_endpoint_argument()),
                ),
        )
        .subcommand(
            Command::new("init")
                .about("Run the database migrations and start the thoth API server")
                .arg(database_argument())
                .arg(host_argument("GRAPHQL_API_HOST"))
                .arg(port_argument("8000", "GRAPHQL_API_PORT"))
                .arg(threads_argument("GRAPHQL_API_THREADS"))
                .arg(keep_alive_argument("GRAPHQL_API_KEEP_ALIVE"))
                .arg(gql_url_argument())
                .arg(domain_argument())
                .arg(key_argument())
                .arg(session_argument()),
        )
        .subcommand(
            Command::new("account")
                .about("Manage user accounts")
                .arg(database_argument())
                .subcommand_required(true)
                .arg_required_else_help(true)
                .subcommand(Command::new("register").about("Create a new user account"))
                .subcommand(
                    Command::new("publishers")
                        .about("Select which publisher(s) this account can manage"),
                )
                .subcommand(Command::new("password").about("Reset a password")),
        )
        .subcommand(
            Command::new("cache")
                .about("Manage cached records")
                .arg(redis_argument())
                .subcommand_required(true)
                .arg_required_else_help(true)
                .subcommand(Command::new("delete").about("Delete cached records")),
        )
}

fn main() -> ThothResult<()> {
    // load environment variables from `.env`
    dotenv().ok();

    match thoth_commands().get_matches().subcommand() {
        Some(("start", start_matches)) => match start_matches.subcommand() {
            Some(("graphql-api", api_matches)) => {
                let database_url = api_matches.get_one::<String>("db").unwrap().to_owned();
                let host = api_matches.get_one::<String>("host").unwrap().to_owned();
                let port = api_matches.get_one::<String>("port").unwrap().to_owned();
                let threads = *api_matches.get_one::<usize>("threads").unwrap();
                let keep_alive = *api_matches.get_one::<u64>("keep-alive").unwrap();
                let url = api_matches.get_one::<String>("gql-url").unwrap().to_owned();
                let domain = api_matches.get_one::<String>("domain").unwrap().to_owned();
                let secret_str = api_matches.get_one::<String>("key").unwrap().to_owned();
                let session_duration = *api_matches.get_one::<i64>("duration").unwrap();
                api_server(
                    database_url,
                    host,
                    port,
                    threads,
                    keep_alive,
                    url,
                    domain,
                    secret_str,
                    session_duration,
                )
                .map_err(|e| e.into())
            }
            Some(("app", client_matches)) => {
                let host = client_matches.get_one::<String>("host").unwrap().to_owned();
                let port = client_matches.get_one::<String>("port").unwrap().to_owned();
                let threads = *client_matches.get_one::<usize>("threads").unwrap();
                let keep_alive = *client_matches.get_one::<u64>("keep-alive").unwrap();
                app_server(host, port, threads, keep_alive).map_err(|e| e.into())
            }
            Some(("export-api", client_matches)) => {
                let redis_url = client_matches
                    .get_one::<String>("redis")
                    .unwrap()
                    .to_owned();
                let host = client_matches.get_one::<String>("host").unwrap().to_owned();
                let port = client_matches.get_one::<String>("port").unwrap().to_owned();
                let threads = *client_matches.get_one::<usize>("threads").unwrap();
                let keep_alive = *client_matches.get_one::<u64>("keep-alive").unwrap();
                let url = client_matches
                    .get_one::<String>("export-url")
                    .unwrap()
                    .to_owned();
                let gql_endpoint = client_matches
                    .get_one::<String>("gql-endpoint")
                    .unwrap()
                    .to_owned();
                export_server(
                    redis_url,
                    host,
                    port,
                    threads,
                    keep_alive,
                    url,
                    gql_endpoint,
                )
                .map_err(|e| e.into())
            }
            _ => unreachable!(),
        },
        Some(("migrate", migrate_matches)) => {
            let database_url = migrate_matches.get_one::<String>("db").unwrap();
            match migrate_matches.get_flag("revert") {
                true => revert_migrations(database_url),
                false => run_migrations(database_url),
            }
        }
        Some(("init", init_matches)) => {
            let database_url = init_matches.get_one::<String>("db").unwrap().to_owned();
            let host = init_matches.get_one::<String>("host").unwrap().to_owned();
            let port = init_matches.get_one::<String>("port").unwrap().to_owned();
            let threads = *init_matches.get_one::<usize>("threads").unwrap();
            let keep_alive = *init_matches.get_one::<u64>("keep-alive").unwrap();
            let url = init_matches
                .get_one::<String>("gql-url")
                .unwrap()
                .to_owned();
            let domain = init_matches.get_one::<String>("domain").unwrap().to_owned();
            let secret_str = init_matches.get_one::<String>("key").unwrap().to_owned();
            let session_duration = *init_matches.get_one::<i64>("duration").unwrap();
            run_migrations(&database_url)?;
            api_server(
                database_url,
                host,
                port,
                threads,
                keep_alive,
                url,
                domain,
                secret_str,
                session_duration,
            )
            .map_err(|e| e.into())
        }
        Some(("account", account_matches)) => match account_matches.subcommand() {
            Some(("register", _)) => commands::account::register(account_matches),
            Some(("publishers", _)) => commands::account::publishers(account_matches),
            Some(("password", _)) => commands::account::password(account_matches),
            _ => unreachable!(),
        },
        Some(("cache", cache_matches)) => match cache_matches.subcommand() {
            Some(("delete", _)) => {
                let redis_url = cache_matches.get_one::<String>("redis").unwrap();
                let pool = init_redis_pool(redis_url);
                let chosen: Vec<usize> = MultiSelect::new()
                    .items(&ALL_SPECIFICATIONS)
                    .with_prompt("Select cached specifications to delete")
                    .interact_on(&Term::stdout())?;
                // run a separate tokio runtime to avoid interfering with actix's threads
                let runtime = tokio::runtime::Builder::new_multi_thread()
                    .worker_threads(1)
                    .enable_all()
                    .build()?;
                runtime.block_on(async {
                    for index in chosen {
                        let specification = ALL_SPECIFICATIONS.get(index).unwrap();
                        let keys = scan_match(&pool, &format!("{}*", specification)).await?;
                        for key in keys {
                            del(&pool, &key).await?;
                        }
                    }
                    Ok::<(), ThothError>(())
                })
            }
            _ => unreachable!(),
        },
        _ => unreachable!(),
    }
}

#[test]
fn test_cli() {
    thoth_commands().debug_assert();
}
