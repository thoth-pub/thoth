use loco_rs::{bgworker::BackgroundWorker, testing::prelude::*};
use serial_test::serial;
use thoth_processor::{
    app::App,
    requests::graphql::Webhook,
    workers::fire_webhook_worker::{FireWebhookWorker, FireWebhookWorkerArgs},
};

#[tokio::test]
#[serial]
async fn test_run_work_created_worker_worker() {
    let boot = boot_test::<App>().await.unwrap();

    // Execute the worker ensuring that it operates in 'ForegroundBlocking' mode, which prevents the addition of your worker to the background
    assert!(FireWebhookWorker::perform_later(
        &boot.app_context,
        FireWebhookWorkerArgs {
            webhook: Webhook {
                endpoint: Default::default(),
                token: Default::default(),
                is_published: Default::default(),
                event_type: Default::default(),
            }
        }
    )
    .await
    .is_ok());
    // Include additional assert validations after the execution of the worker
}
