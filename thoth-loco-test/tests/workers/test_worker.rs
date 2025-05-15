use loco_rs::{bgworker::BackgroundWorker, testing::prelude::*};
use thoth_api::event::model::{Event, EventType};
use thoth_loco_test::{
    app::App,
    workers::test_worker::{TestWorker, TestWorkerArgs},
};
use serial_test::serial;

#[tokio::test]
#[serial]
async fn test_run_test_worker_worker() {
    let boot = boot_test::<App>().await.unwrap();

    // Execute the worker ensuring that it operates in 'ForegroundBlocking' mode, which prevents the addition of your worker to the background
    assert!(
        TestWorker::perform_later(&boot.app_context, TestWorkerArgs {
            event: Event {
                event_type: EventType::WorkUpdated,
                work_id: Default::default(),
                is_published: Default::default(),
                event_timestamp: Default::default(),
                thoth_version: Default::default(),
            }
        })
            .await
            .is_ok()
    );
    // Include additional assert validations after the execution of the worker
}
