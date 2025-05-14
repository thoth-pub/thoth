use async_trait::async_trait;
use loco_rs::prelude::*;
use thoth_api::{
    event::{model::Event, handler::QUEUE_KEY},
    redis::{blpop, init_pool},
};
use crate::workers::test_worker::{TestWorker, TestWorkerArgs};

pub struct TestInitializer;

#[async_trait]
impl Initializer for TestInitializer {
    fn name(&self) -> String {
        "test-initializer".to_string()
    }

    async fn before_run(&self, ctx: &AppContext) -> Result<()> {
        let redis = init_pool("redis://localhost:6379");
        let ctx = ctx.clone();

        tokio::spawn(async move {
            loop {
                if let Ok(payload) = blpop(&redis, QUEUE_KEY).await {
                    tracing::info!("Initializer received payload: {:?}", payload);
                    match serde_json::from_str::<Event>(&payload) {
                        Ok(event) => {
                            tracing::info!("Received event: {:?}", event);
                            let _ = TestWorker::perform_later(
                                &ctx,
                                TestWorkerArgs {},
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