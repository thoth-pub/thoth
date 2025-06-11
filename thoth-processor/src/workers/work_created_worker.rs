use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};
use thoth_api::event::model::Event;

pub struct WorkCreatedWorker {
    pub ctx: AppContext,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct WorkCreatedWorkerArgs {
    pub event: Event,
}

#[async_trait]
impl BackgroundWorker<WorkCreatedWorkerArgs> for WorkCreatedWorker {
    fn build(ctx: &AppContext) -> Self {
        Self { ctx: ctx.clone() }
    }

    async fn perform(&self, args: WorkCreatedWorkerArgs) -> Result<()> {
        tracing::info!("WorkCreatedWorker start");
        tracing::info!("Event: {:?}", args.event);
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
        tracing::info!("WorkCreatedWorker end");

        Ok(())
    }
}
