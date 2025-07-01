use crate::workers::{
    work_created_worker::{WorkCreatedWorker, WorkCreatedWorkerArgs},
    work_published_worker::{WorkPublishedWorker, WorkPublishedWorkerArgs},
    work_updated_worker::{WorkUpdatedWorker, WorkUpdatedWorkerArgs},
};
use async_trait::async_trait;
use loco_rs::{config::QueueConfig, prelude::*};
use thoth_api::{
    event::{
        handler::QUEUE_KEY,
        model::{Event, EventType},
    },
    redis::{blpop, init_pool},
};

pub struct HandleEvents;

#[async_trait]
impl Initializer for HandleEvents {
    fn name(&self) -> String {
        "handle-events".to_string()
    }

    async fn before_run(&self, ctx: &AppContext) -> Result<()> {
        let ctx = ctx.clone();
        let redis_url = match ctx.config.queue.as_ref().unwrap() {
            QueueConfig::Redis(queue) => &queue.uri,
            _ => unreachable!(),
        };
        let redis = init_pool(redis_url);

        tokio::spawn(async move {
            loop {
                if let Ok(payload) = blpop(&redis, QUEUE_KEY).await {
                    tracing::info!("Initializer received payload: {:?}", payload);
                    match serde_json::from_str::<Event>(&payload) {
                        Ok(event) => {
                            tracing::info!("Received event: {:?}", event);
                            let _ = match event.event_type {
                                EventType::WorkCreated => {
                                    WorkCreatedWorker::perform_later(
                                        &ctx,
                                        WorkCreatedWorkerArgs { event },
                                    )
                                    .await
                                }
                                EventType::WorkUpdated => {
                                    WorkUpdatedWorker::perform_later(
                                        &ctx,
                                        WorkUpdatedWorkerArgs { event },
                                    )
                                    .await
                                }
                                EventType::WorkPublished => {
                                    WorkPublishedWorker::perform_later(
                                        &ctx,
                                        WorkPublishedWorkerArgs { event },
                                    )
                                    .await
                                }
                            };
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
