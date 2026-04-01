use crate::arguments;
use crate::commands::get_redis_pool;
use clap::{ArgMatches, Command};
use dialoguer::{console::Term, theme::ColorfulTheme, MultiSelect};
use lazy_static::lazy_static;
use thoth::{
    api::redis::{del, scan_match},
    errors::{ThothError, ThothResult},
    ALL_SPECIFICATIONS,
};

lazy_static! {
    pub(crate) static ref COMMAND: Command = Command::new("cache")
        .about("Manage cached records")
        .arg(arguments::redis())
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(Command::new("delete").about("Delete cached records"));
}

pub fn delete(arguments: &ArgMatches) -> ThothResult<()> {
    let pool = get_redis_pool(arguments);
    let chosen: Vec<usize> = MultiSelect::with_theme(&ColorfulTheme::default())
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
            let keys = scan_match(&pool, &format!("{specification}*")).await?;
            for key in keys {
                del(&pool, &key).await?;
            }
        }
        Ok::<(), ThothError>(())
    })
}
