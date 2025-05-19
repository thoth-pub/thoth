use serde::{Deserialize, Serialize};
use loco_rs::prelude::*;
use thoth_api::event::model::Event;

pub struct WorkUpdatedWorker {
    pub ctx: AppContext,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct WorkUpdatedWorkerArgs {
    pub event: Event,
}

#[async_trait]
impl BackgroundWorker<WorkUpdatedWorkerArgs> for WorkUpdatedWorker {
    fn build(ctx: &AppContext) -> Self {
        Self { ctx: ctx.clone() }
    }

    async fn perform(&self, args: WorkUpdatedWorkerArgs) -> Result<()> {
        tracing::info!("WorkUpdatedWorker start");
        tracing::info!("Event: {:?}", args.event);
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
        tracing::info!("WorkUpdatedWorker end");

        Ok(())
    }
}
