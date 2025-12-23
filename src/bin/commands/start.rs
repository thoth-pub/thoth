use crate::arguments;
use clap::{ArgMatches, Command};
use lazy_static::lazy_static;
use thoth::{api_server, errors::ThothResult, export_server};

lazy_static! {
    pub(crate) static ref COMMAND: Command = Command::new("start")
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
            Command::new("export-api")
                .about("Start the thoth metadata export API")
                .arg(arguments::redis())
                .arg(arguments::host("EXPORT_API_HOST"))
                .arg(arguments::port("8181", "EXPORT_API_PORT"))
                .arg(arguments::threads("EXPORT_API_THREADS"))
                .arg(arguments::keep_alive("EXPORT_API_KEEP_ALIVE"))
                .arg(arguments::export_url())
                .arg(arguments::gql_endpoint()),
        );
}

pub fn graphql_api(arguments: &ArgMatches) -> ThothResult<()> {
    let database_url = arguments.get_one::<String>("db").unwrap().to_owned();
    let host = arguments.get_one::<String>("host").unwrap().to_owned();
    let port = arguments.get_one::<String>("port").unwrap().to_owned();
    let threads = *arguments.get_one::<usize>("threads").unwrap();
    let keep_alive = *arguments.get_one::<u64>("keep-alive").unwrap();
    let url = arguments.get_one::<String>("gql-url").unwrap().to_owned();
    let domain = arguments.get_one::<String>("domain").unwrap().to_owned();
    let secret_str = arguments.get_one::<String>("key").unwrap().to_owned();
    let session_duration = *arguments.get_one::<i64>("duration").unwrap();
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
pub fn export_api(arguments: &ArgMatches) -> ThothResult<()> {
    let redis_url = arguments.get_one::<String>("redis").unwrap().to_owned();
    let host = arguments.get_one::<String>("host").unwrap().to_owned();
    let port = arguments.get_one::<String>("port").unwrap().to_owned();
    let threads = *arguments.get_one::<usize>("threads").unwrap();
    let keep_alive = *arguments.get_one::<u64>("keep-alive").unwrap();
    let url = arguments
        .get_one::<String>("export-url")
        .unwrap()
        .to_owned();
    let gql_endpoint = arguments
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
