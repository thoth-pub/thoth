use crate::event::model::EventWrapper;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Job {
    pub queue: String,
    pub args: Vec<EventWrapper>,
    pub retry: bool,
    pub class: String,
    pub jid: String,
    pub created_at: f64,
    pub enqueued_at: f64,
    pub failed_at: f64,
    pub error_message: String,
    pub error_class: Option<String>,
    pub retry_count: i32,
    pub retried_at: f64,
}
