use crate::arguments;
use clap::{ArgMatches, Command};
use lazy_static::lazy_static;
use thoth::{
    api::{
        async_worker::{launch, WorkerArgs},
        graphql::MutationGuardMode,
        model::{
            distribution_job::DistributionJobCreation, metric_dashboard::MetricMonthServing,
            metric_widget::MetricWidgetSwitch,
        },
    },
    api_server,
    errors::{ThothError, ThothResult},
    export_server,
};

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
                .arg(arguments::key())
                .arg(arguments::zitadel_url())
                .arg(arguments::mutation_guard_mode())
                .arg(arguments::distribution_job_creation())
                .arg(arguments::metric_widget())
                .arg(arguments::metric_month_serving())
                .arg(arguments::ingest_pool_size())
                .arg(arguments::aws_access_key_id())
                .arg(arguments::aws_secret_access_key())
                .arg(arguments::aws_region()),
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
        )
        .subcommand(worker_command());
}

/// `thoth start worker` (THOTH-ASYNC-01-WORKER-01, section 4). The database
/// argument is the shared one with its value hidden from `--help`; the
/// worker takes no AWS or key argument (its AWS authority is the ECS task
/// role).
pub(crate) fn worker_command() -> Command {
    Command::new("worker")
        .about("Start the trusted in-process async job worker (idle while no kind is listed)")
        .arg(arguments::database().hide_env_values(true))
        .arg(arguments::worker_name())
        .arg(arguments::worker_kinds())
        .arg(arguments::worker_capabilities())
        .arg(arguments::worker_slots())
        .arg(arguments::worker_poll_seconds())
        .arg(arguments::worker_shutdown_grace_seconds())
        .arg(arguments::worker_liveness_file())
}

/// Resolve the effective mutation-guard mode from parsed arguments.
///
/// This is the single accessor every command path uses, so a command that
/// dispatches here must register [`arguments::mutation_guard_mode`]. Both
/// `start graphql-api` and `init` do.
///
/// Defaults to `OFF`; `clap`'s `value_parser` has already restricted the string
/// to the three accepted modes, so an invalid value fails during parsing rather
/// than reaching this function.
pub(crate) fn mutation_guard_mode(arguments: &ArgMatches) -> ThothResult<MutationGuardMode> {
    arguments
        .get_one::<String>("mutation-guard-mode")
        .map(String::as_str)
        .unwrap_or("OFF")
        .parse::<MutationGuardMode>()
        .map_err(ThothError::InternalError)
}

/// Resolve the effective automatic distribution-job creation setting from parsed
/// arguments.
///
/// This is the single accessor every command path uses, so a command that
/// dispatches here must register [`arguments::distribution_job_creation`]. Both
/// `start graphql-api` and `init` do.
///
/// Defaults to `OFF`; `clap`'s `value_parser` has already restricted the string
/// to the two accepted values, so an invalid value fails during parsing rather
/// than reaching this function.
pub(crate) fn distribution_job_creation(
    arguments: &ArgMatches,
) -> ThothResult<DistributionJobCreation> {
    arguments
        .get_one::<String>("distribution-job-creation")
        .map(String::as_str)
        .unwrap_or("OFF")
        .parse::<DistributionJobCreation>()
        .map_err(ThothError::InternalError)
}

/// Resolve the effective public widget read switch from parsed arguments.
///
/// This is the single accessor every command path uses, so a command that
/// dispatches here must register [`arguments::metric_widget`]. Both
/// `start graphql-api` and `init` do.
///
/// Defaults to `OFF`; `clap`'s `value_parser` has already restricted the string
/// to the two accepted values, so an invalid value fails during parsing rather
/// than reaching this function.
pub(crate) fn metric_widget(arguments: &ArgMatches) -> ThothResult<MetricWidgetSwitch> {
    arguments
        .get_one::<String>("metric-widget")
        .map(String::as_str)
        .unwrap_or("OFF")
        .parse::<MetricWidgetSwitch>()
        .map_err(ThothError::InternalError)
}

/// Resolve the effective monthly serving switch from parsed arguments
/// (`MET-WP4-MONTH-01`).
///
/// This is the single accessor every command path uses, so a command that
/// dispatches here must register [`arguments::metric_month_serving`]. Both
/// `start graphql-api` and `init` do.
///
/// Defaults to `OFF`; `clap`'s `value_parser` has already restricted the string
/// to the two accepted values, so an invalid value fails during parsing rather
/// than reaching this function.
pub(crate) fn metric_month_serving(arguments: &ArgMatches) -> ThothResult<MetricMonthServing> {
    arguments
        .get_one::<String>("metric-month-serving")
        .map(String::as_str)
        .unwrap_or("OFF")
        .parse::<MetricMonthServing>()
        .map_err(ThothError::InternalError)
}

pub fn graphql_api(arguments: &ArgMatches) -> ThothResult<()> {
    let database_url = arguments.get_one::<String>("db").unwrap().to_owned();
    let host = arguments.get_one::<String>("host").unwrap().to_owned();
    let port = arguments.get_one::<String>("port").unwrap().to_owned();
    let threads = *arguments.get_one::<usize>("threads").unwrap();
    let keep_alive = *arguments.get_one::<u64>("keep-alive").unwrap();
    let url = arguments.get_one::<String>("gql-url").unwrap().to_owned();
    let private_key = arguments.get_one::<String>("key").unwrap().to_owned();
    let zitadel_url = arguments
        .get_one::<String>("zitadel-url")
        .unwrap()
        .to_owned();
    let mutation_guard_mode = mutation_guard_mode(arguments)?;
    let distribution_job_creation = distribution_job_creation(arguments)?;
    let metric_widget = metric_widget(arguments)?;
    let metric_month_serving = metric_month_serving(arguments)?;
    let ingest_pool_size = *arguments.get_one::<u32>("ingest-pool-size").unwrap();

    api_server(
        database_url,
        host,
        port,
        threads,
        keep_alive,
        url,
        private_key,
        zitadel_url,
        mutation_guard_mode,
        distribution_job_creation,
        metric_widget,
        metric_month_serving,
        ingest_pool_size,
        arguments
            .get_one::<String>("aws-access-key-id")
            .unwrap()
            .to_owned(),
        arguments
            .get_one::<String>("aws-secret-access-key")
            .unwrap()
            .to_owned(),
        arguments
            .get_one::<String>("aws-region")
            .unwrap()
            .to_owned(),
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

/// Run the worker until it exits, answering its exit code. The binary's
/// entry point is the only caller of `std::process::exit` (section 6.8 step
/// 7). The first `SIGTERM` or `SIGINT` requests shutdown and a second one
/// ends the grace phase; the signal listener runs on its own small runtime
/// thread, so the worker's own threads stay plain threads.
pub fn worker(arguments: &ArgMatches) -> i32 {
    let text = |name: &str| {
        arguments
            .get_one::<String>(name)
            .cloned()
            .unwrap_or_default()
    };
    let args = WorkerArgs {
        database_url: arguments.get_one::<String>("db").cloned(),
        worker_name: text("worker-name"),
        kinds: text("kinds"),
        capabilities: text("capabilities"),
        slots: text("slots"),
        poll_seconds: text("poll-seconds"),
        shutdown_grace_seconds: text("shutdown-grace-seconds"),
        liveness_file: text("liveness-file"),
    };
    let process = match launch(args) {
        Ok(process) => process,
        Err(code) => return code,
    };
    let handle = process.handle();
    let spawned = std::thread::Builder::new()
        .name("async-worker-signals".to_string())
        .spawn(move || {
            let runtime = match tokio::runtime::Builder::new_current_thread()
                .enable_io()
                .build()
            {
                Ok(runtime) => runtime,
                Err(_) => {
                    handle.signal_setup_failed();
                    return;
                }
            };
            runtime.block_on(async move {
                use tokio::signal::unix::{signal, SignalKind};
                let (mut terminate, mut interrupt) = match (
                    signal(SignalKind::terminate()),
                    signal(SignalKind::interrupt()),
                ) {
                    (Ok(terminate), Ok(interrupt)) => (terminate, interrupt),
                    _ => {
                        handle.signal_setup_failed();
                        return;
                    }
                };
                loop {
                    tokio::select! {
                        _ = terminate.recv() => handle.request_shutdown(),
                        _ = interrupt.recv() => handle.request_shutdown(),
                    }
                }
            });
        });
    if spawned.is_err() {
        process.handle().signal_setup_failed();
    }
    process.run()
}
