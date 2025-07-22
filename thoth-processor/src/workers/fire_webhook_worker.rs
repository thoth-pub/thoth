use crate::requests::graphql::Webhook;
use crate::requests::target::fire_webhook;
use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub struct FireWebhookWorker {
    pub ctx: AppContext,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct FireWebhookWorkerArgs {
    pub work_id: Uuid,
    pub webhook: Webhook,
}

#[async_trait]
impl BackgroundWorker<FireWebhookWorkerArgs> for FireWebhookWorker {
    fn build(ctx: &AppContext) -> Self {
        Self { ctx: ctx.clone() }
    }

    async fn perform(&self, args: FireWebhookWorkerArgs) -> Result<()> {
        tracing::info!("Webhook: {:?}", args.webhook);
        let target_rsp =
            fire_webhook(args.webhook.endpoint, args.webhook.token, args.work_id).await?;
        tracing::info!("Target response: {:?}", target_rsp);

        Ok(())
    }
}
