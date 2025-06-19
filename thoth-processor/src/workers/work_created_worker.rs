use super::fire_webhook_worker::{FireWebhookWorker, FireWebhookWorkerArgs};
use crate::requests::graphql::query_webhooks;
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
        let webhooks = query_webhooks(args.event).await?;
        tracing::info!("Webhooks: {:?}", webhooks);

        for webhook in webhooks {
            let _ = FireWebhookWorker::perform_later(&self.ctx, FireWebhookWorkerArgs { webhook })
                .await;
        }

        tracing::info!("WorkCreatedWorker end");

        Ok(())
    }
}
