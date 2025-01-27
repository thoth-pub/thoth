use clap::{value_parser, Arg, ArgAction};

pub fn database() -> Arg {
    Arg::new("db")
        .short('D')
        .long("database-url")
        .value_name("DATABASE_URL")
        .env("DATABASE_URL")
        .help("Full postgres database url, e.g. postgres://thoth:thoth@localhost/thoth")
        .num_args(1)
}

pub fn redis() -> Arg {
    Arg::new("redis")
        .short('R')
        .long("redis-url")
        .value_name("REDIS_URL")
        .env("REDIS_URL")
        .help("Full redis url, e.g. redis://localhost:6379")
        .num_args(1)
}

pub fn host(env_value: &'static str) -> Arg {
    Arg::new("host")
        .short('H')
        .long("host")
        .value_name("HOST")
        .env(env_value)
        .default_value("0.0.0.0")
        .help("host to bind")
        .num_args(1)
}

pub fn port(default_value: &'static str, env_value: &'static str) -> Arg {
    Arg::new("port")
        .short('p')
        .long("port")
        .value_name("PORT")
        .env(env_value)
        .default_value(default_value)
        .help("Port to bind")
        .num_args(1)
}

pub fn domain() -> Arg {
    Arg::new("domain")
        .short('d')
        .long("domain")
        .value_name("THOTH_DOMAIN")
        .env("THOTH_DOMAIN")
        .default_value("localhost")
        .help("Authentication cookie domain")
        .num_args(1)
}

pub fn key() -> Arg {
    Arg::new("key")
        .short('k')
        .long("secret-key")
        .value_name("SECRET")
        .env("SECRET_KEY")
        .help("Authentication cookie secret key")
        .num_args(1)
}

pub fn session() -> Arg {
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

pub fn gql_url() -> Arg {
    Arg::new("gql-url")
        .short('u')
        .long("gql-url")
        .value_name("THOTH_GRAPHQL_API")
        .env("THOTH_GRAPHQL_API")
        .default_value("http://localhost:8000")
        .help("Thoth GraphQL's, public facing, root URL.")
        .num_args(1)
}

pub fn gql_endpoint() -> Arg {
    Arg::new("gql-endpoint")
        .short('g')
        .long("gql-endpoint")
        .value_name("THOTH_GRAPHQL_ENDPOINT")
        .env("THOTH_GRAPHQL_ENDPOINT")
        .default_value("http://localhost:8000/graphql")
        .help("Thoth GraphQL's endpoint")
        .num_args(1)
}

pub fn export_url() -> Arg {
    Arg::new("export-url")
        .short('u')
        .long("export-url")
        .value_name("THOTH_EXPORT_API")
        .env("THOTH_EXPORT_API")
        .default_value("http://localhost:8181")
        .help("Thoth Export API's, public facing, root URL.")
        .num_args(1)
}

pub fn threads(env_value: &'static str) -> Arg {
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

pub fn keep_alive(env_value: &'static str) -> Arg {
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

pub fn revert() -> Arg {
    Arg::new("revert")
        .long("revert")
        .help("Revert all database migrations")
        .action(ArgAction::SetTrue)
}
