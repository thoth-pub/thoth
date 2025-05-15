use serde::{Deserialize, Serialize};
use loco_rs::prelude::*;
use thoth_api::event::model::Event;

pub struct TestWorker {
    pub ctx: AppContext,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct TestWorkerArgs {
    pub event: Event,
}

#[async_trait]
impl BackgroundWorker<TestWorkerArgs> for TestWorker {
    fn build(ctx: &AppContext) -> Self {
        Self { ctx: ctx.clone() }
    }

    async fn perform(&self, args: TestWorkerArgs) -> Result<()> {
        tracing::info!("TestWorker start");
        tracing::info!("Event: {:?}", args.event);
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
        tracing::info!("TestWorker end");

        Ok(())
    }
}
