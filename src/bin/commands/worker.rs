use crate::arguments;
use clap::{ArgMatches, Command};
use lazy_static::lazy_static;
use thoth::api::async_worker::health_command;

lazy_static! {
    pub(crate) static ref COMMAND: Command = Command::new("worker")
        .about("Inspect the async job worker")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(health_subcommand());
}

/// `thoth worker health`: exit 0 when the liveness file exists, parses and
/// is at most `--max-age-seconds` old, otherwise 1. It opens no database
/// connection and no socket.
pub(crate) fn health_subcommand() -> Command {
    Command::new("health")
        .about("Exit 0 if the worker's liveness file is fresh, otherwise 1")
        .arg(arguments::worker_liveness_file())
        .arg(arguments::worker_health_max_age_seconds())
}

pub fn health(arguments: &ArgMatches) -> i32 {
    health_command(
        arguments
            .get_one::<String>("liveness-file")
            .map(String::as_str),
        arguments
            .get_one::<String>("max-age-seconds")
            .map(String::as_str),
    )
}
