mod arguments;
mod commands;

lazy_static::lazy_static! {
    static ref THOTH: clap::Command = clap::Command::new(env!("CARGO_PKG_NAME"))
        .version(clap::crate_version!())
        .author(clap::crate_authors!("\n"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(commands::MIGRATE.clone())
        .subcommand(commands::start::COMMAND.clone())
        .subcommand(commands::INIT.clone())
        .subcommand(commands::account::COMMAND.clone())
        .subcommand(commands::cache::COMMAND.clone());
}

fn main() -> thoth::errors::ThothResult<()> {
    // load environment variables from `.env`
    dotenv::dotenv().ok();

    match THOTH.clone().get_matches().subcommand() {
        Some(("start", start_arguments)) => match start_arguments.subcommand() {
            Some(("graphql-api", arguments)) => commands::start::graphql_api(arguments),
            Some(("export-api", arguments)) => commands::start::export_api(arguments),
            _ => unreachable!(),
        },
        Some(("migrate", arguments)) => commands::migrate(arguments),
        Some(("init", arguments)) => {
            commands::run_migrations(arguments)?;
            commands::start::graphql_api(arguments)
        }
        Some(("account", arguments)) => match arguments.subcommand() {
            Some(("register", _)) => commands::account::register(arguments),
            Some(("publishers", _)) => commands::account::publishers(arguments),
            Some(("password", _)) => commands::account::password(arguments),
            _ => unreachable!(),
        },
        Some(("cache", arguments)) => match arguments.subcommand() {
            Some(("delete", _)) => commands::cache::delete(arguments),
            _ => unreachable!(),
        },
        _ => unreachable!(),
    }
}

#[test]
fn test_cli() {
    THOTH.clone().debug_assert();
}
