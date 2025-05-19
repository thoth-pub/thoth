use async_trait::async_trait;
use loco_rs::prelude::*;
use thoth_api::{
    event::{model::Event, handler::QUEUE_KEY},
    redis::{blpop, init_pool},
};
use crate::workers::work_updated_worker::{WorkUpdatedWorker, WorkUpdatedWorkerArgs};

pub struct HandleEvents;

#[async_trait]
impl Initializer for HandleEvents {
    fn name(&self) -> String {
        "handle-events".to_string()
    }

    async fn before_run(&self, ctx: &AppContext) -> Result<()> {
        //TODO remove hardcoding
        let redis = init_pool("redis://localhost:6379");
        let ctx = ctx.clone();

        tokio::spawn(async move {
            loop {
                if let Ok(payload) = blpop(&redis, QUEUE_KEY).await {
                    tracing::info!("Initializer received payload: {:?}", payload);
                    match serde_json::from_str::<Event>(&payload) {
                        Ok(event) => {
                            tracing::info!("Received event: {:?}", event);
                            //TODO match on the type of event & pass to different workers
                            let _ = WorkUpdatedWorker::perform_later(
                                &ctx,
                                WorkUpdatedWorkerArgs {
                                    event: event,
                                },
                            )
                            .await;
                        }
                        Err(e) => {
                            tracing::error!("Invalid event payload: {}", e);
                        }
                    }
                }
            }
        });

        Ok(())
    }
}