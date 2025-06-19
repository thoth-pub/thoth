use crate::requests::graphql::query_webhooks;
use crate::requests::target::fire_webhook;
use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};
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
        let response = query_webhooks(args.event).await?;
        tracing::info!("Response: {:?}", response);

        let target_rsp = fire_webhook().await?;
        tracing::info!("Target response: {:?}", target_rsp);

        tracing::info!("WorkPublishedWorker end");

        Ok(())
    }
}
