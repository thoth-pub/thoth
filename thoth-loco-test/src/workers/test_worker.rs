use serde::{Deserialize, Serialize};
use loco_rs::prelude::*;

pub struct TestWorker {
    pub ctx: AppContext,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct TestWorkerArgs {
}

#[async_trait]
impl BackgroundWorker<TestWorkerArgs> for TestWorker {
    fn build(ctx: &AppContext) -> Self {
        Self { ctx: ctx.clone() }
    }

    async fn perform(&self, _args: TestWorkerArgs) -> Result<()> {
        println!("=================TestWorker=======================");
        // TODO: Some actual work goes here...
        let redis: &deadpool_redis::Pool = &deadpool_redis::Config::from_url("redis://localhost:6379")
            .builder()
            .expect("Failed to create redis pool.")
            .build()
            .expect("Failed to build redis pool.");
        let mut conn = redis.get().await.expect("Failed to connect to redis pool.");
        loop {
            if let Ok((_, payload)) = deadpool_redis::redis::AsyncCommands::blpop::<_,(String, String)>(&mut conn, "events:graphql", 0.0).await {
                tracing::info!("Received payload: {:?}", payload);
            }
        }
    }
}
