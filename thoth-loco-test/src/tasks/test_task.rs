use loco_rs::prelude::*;
use crate::workers::test_worker::{TestWorker, TestWorkerArgs};

pub struct TestTask;

#[async_trait]
impl Task for TestTask {
    fn task(&self) -> TaskInfo {
        TaskInfo {
            name: "test_task".to_string(),
            detail: "This is a test task".to_string(),
        }
    }

    async fn run(&self, ctx: &AppContext, _vars: &task::Vars) -> Result<()> {
        TestWorker::perform_later(
            &ctx,
            TestWorkerArgs {
                // user_guid: "foo".to_string(),
            },
        )
        .await
    }
}
