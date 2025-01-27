use clap::{crate_authors, crate_version, Arg, ArgAction, Command};
use dotenv::dotenv;
use std::env;
use thoth::errors::ThothResult;

mod arguments;
mod commands;

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
                .arg(arguments::database())
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
                        .arg(arguments::database())
                        .arg(arguments::host("GRAPHQL_API_HOST"))
                        .arg(arguments::port("8000", "GRAPHQL_API_PORT"))
                        .arg(arguments::threads("GRAPHQL_API_THREADS"))
                        .arg(arguments::keep_alive("GRAPHQL_API_KEEP_ALIVE"))
                        .arg(arguments::gql_url())
                        .arg(arguments::domain())
                        .arg(arguments::key())
                        .arg(arguments::session()),
                )
                .subcommand(
                    Command::new("app")
                        .about("Start the thoth client GUI")
                        .arg(arguments::host("APP_HOST"))
                        .arg(arguments::port("8080", "APP_PORT"))
                        .arg(arguments::threads("APP_THREADS"))
                        .arg(arguments::keep_alive("APP_KEEP_ALIVE")),
                )
                .subcommand(
                    Command::new("export-api")
                        .about("Start the thoth metadata export API")
                        .arg(arguments::redis())
                        .arg(arguments::host("EXPORT_API_HOST"))
                        .arg(arguments::port("8181", "EXPORT_API_PORT"))
                        .arg(arguments::threads("EXPORT_API_THREADS"))
                        .arg(arguments::keep_alive("EXPORT_API_KEEP_ALIVE"))
                        .arg(arguments::export_url())
                        .arg(arguments::gql_endpoint()),
                ),
        )
        .subcommand(
            Command::new("init")
                .about("Run the database migrations and start the thoth API server")
                .arg(arguments::database())
                .arg(arguments::host("GRAPHQL_API_HOST"))
                .arg(arguments::port("8000", "GRAPHQL_API_PORT"))
                .arg(arguments::threads("GRAPHQL_API_THREADS"))
                .arg(arguments::keep_alive("GRAPHQL_API_KEEP_ALIVE"))
                .arg(arguments::gql_url())
                .arg(arguments::domain())
                .arg(arguments::key())
                .arg(arguments::session()),
        )
        .subcommand(
            Command::new("account")
                .about("Manage user accounts")
                .arg(arguments::database())
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
                .arg(arguments::redis())
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
