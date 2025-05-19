use serde::{Deserialize, Serialize};
use loco_rs::prelude::*;
use thoth_api::event::model::Event;

pub struct WorkPublishedWorker {
    pub ctx: AppContext,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct WorkPublishedWorkerArgs {
    pub event: Event,
}

#[async_trait]
impl BackgroundWorker<WorkPublishedWorkerArgs> for WorkPublishedWorker {
    fn build(ctx: &AppContext) -> Self {
        Self { ctx: ctx.clone() }
    }

    async fn perform(&self, args: WorkPublishedWorkerArgs) -> Result<()> {
        tracing::info!("WorkPublishedWorker start");
        tracing::info!("Event: {:?}", args.event);
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
        tracing::info!("WorkPublishedWorker end");

        Ok(())
    }
}
