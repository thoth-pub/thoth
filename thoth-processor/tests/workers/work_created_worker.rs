use loco_rs::{bgworker::BackgroundWorker, testing::prelude::*};
use thoth_api::event::model::{Event, EventType};
use thoth_processor::{
    app::App,
    workers::work_created_worker::{WorkCreatedWorker, WorkCreatedWorkerArgs},
};
use serial_test::serial;

#[tokio::test]
#[serial]
async fn test_run_work_created_worker_worker() {
    let boot = boot_test::<App>().await.unwrap();

    // Execute the worker ensuring that it operates in 'ForegroundBlocking' mode, which prevents the addition of your worker to the background
    assert!(
        WorkCreatedWorker::perform_later(&boot.app_context, WorkCreatedWorkerArgs {
            event: Event {
                event_type: EventType::WorkCreated,
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
