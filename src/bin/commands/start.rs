use clap::ArgMatches;
use thoth::{
    api_server, app_server,
    errors::ThothResult,
    export_server,
};

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

pub fn app(arguments: &ArgMatches) -> ThothResult<()> {
    let host = arguments.get_one::<String>("host").unwrap().to_owned();
    let port = arguments.get_one::<String>("port").unwrap().to_owned();
    let threads = *arguments.get_one::<usize>("threads").unwrap();
    let keep_alive = *arguments.get_one::<u64>("keep-alive").unwrap();
    app_server(host, port, threads, keep_alive).map_err(|e| e.into())
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
