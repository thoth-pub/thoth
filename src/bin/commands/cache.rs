use crate::commands::get_redis_pool;
use clap::ArgMatches;
use dialoguer::{console::Term, MultiSelect};
use thoth::{
    api::redis::{del, scan_match},
    errors::{ThothError, ThothResult},
    ALL_SPECIFICATIONS,
};

pub fn delete(arguments: &ArgMatches) -> ThothResult<()> {
    let pool = get_redis_pool(arguments);
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
