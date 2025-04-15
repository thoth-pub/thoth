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
        Ok(())
    }
}
