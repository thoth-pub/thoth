use async_trait::async_trait;
use loco_rs::{
    app::{AppContext, Initializer},
    Result,
};

pub struct TestInitializer;

#[async_trait]
impl Initializer for TestInitializer {
    fn name(&self) -> String {
        "test-initializer".to_string()
    }

    async fn before_run(&self, _ctx: &AppContext) -> Result<()> {
        let redis: &deadpool_redis::Pool = &deadpool_redis::Config::from_url("redis://localhost:6379")
            .builder()
            .expect("Failed to create redis pool.")
            .build()
            .expect("Failed to build redis pool.");
        let mut conn = redis.get().await.expect("Failed to connect to redis pool.");

        tokio::spawn(async move {
            loop {
                if let Ok((_, payload)) = deadpool_redis::redis::AsyncCommands::blpop::<_,(String, String)>(&mut conn, "events:graphql", 0.0).await {
                    tracing::info!("Initializer received payload: {:?}", payload);
                }
            }
        });

        Ok(())
    }
}