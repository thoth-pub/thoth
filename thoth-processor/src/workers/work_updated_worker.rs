use super::fire_webhook_worker::{FireWebhookWorker, FireWebhookWorkerArgs};
use crate::{common::settings::Settings, requests::graphql::query_webhooks};
use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};
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
        tracing::info!("Event: {:?}", args.event);
        let work_id = args.event.work_id;
        let webhooks = query_webhooks(
            format!(
                "{}/graphql",
                Settings::from_json(&self.ctx.config.settings.as_ref().unwrap())?.thoth_graphql_api
            ),
            args.event,
        )
        .await?;
        tracing::info!("Webhooks: {:?}", webhooks);

        for webhook in webhooks {
            let _ = FireWebhookWorker::perform_later(
                &self.ctx,
                FireWebhookWorkerArgs { work_id, webhook },
            )
            .await;
        }

        Ok(())
    }
}
