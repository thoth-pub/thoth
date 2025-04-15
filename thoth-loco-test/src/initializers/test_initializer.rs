use async_trait::async_trait;
use loco_rs::{
    app::{AppContext, Initializer},
    task::Task,
    Result,
};
use tracing::error;

use crate::tasks::test_task::TestTask;

pub struct TestInitializer;

#[async_trait]
impl Initializer for TestInitializer {
    fn name(&self) -> String {
        "test-initializer".to_string()
    }

    async fn before_run(&self, ctx: &AppContext) -> Result<()> {
        let ctx = ctx.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;

                if let Err(e) = (TestTask)
                    .run(&ctx, &loco_rs::task::Vars::default())
                    .await
                {
                    error!("TestTask error: {:?}", e);
                }
            }
        });

        Ok(())
    }
}