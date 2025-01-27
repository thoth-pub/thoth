use clap::{crate_authors, crate_version, value_parser, Arg, ArgAction, Command};
use dotenv::dotenv;
use std::env;
use thoth::errors::ThothResult;

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
        Some(("start", start_arguments)) => match start_arguments.subcommand() {
            Some(("graphql-api", arguments)) => commands::start::graphql_api(arguments),
            Some(("app", arguments)) => commands::start::app(arguments),
            Some(("export-api", arguments)) => commands::start::export_api(arguments),
            _ => unreachable!(),
        },
        Some(("migrate", aguments)) => commands::migrate(aguments),
        Some(("init", arguments)) => {
            commands::run_migrations(arguments)?;
            commands::start::graphql_api(arguments)
        }
        Some(("account", aguments)) => match aguments.subcommand() {
            Some(("register", _)) => commands::account::register(aguments),
            Some(("publishers", _)) => commands::account::publishers(aguments),
            Some(("password", _)) => commands::account::password(aguments),
            _ => unreachable!(),
        },
        Some(("cache", aguments)) => match aguments.subcommand() {
            Some(("delete", _)) => commands::cache::delete(aguments),
            _ => unreachable!(),
        },
        _ => unreachable!(),
    }
}

#[test]
fn test_cli() {
    thoth_commands().debug_assert();
}
