use super::model::Job;
use crate::redis::{zrange, RedisPool};
use thoth_errors::ThothResult;

pub const RETRY_QUEUE_KEY: &str = "retry";

pub async fn get_jobs(redis: &RedisPool) -> ThothResult<Vec<Job>> {
    let jobs_vec_string = zrange(redis, RETRY_QUEUE_KEY, 0, -1).await?;
    let jobs_vec_json_result = jobs_vec_string
        .into_iter()
        .map(|j| serde_json::from_str(&j).map_err(|e| e.into()))
        .collect::<ThothResult<Vec<Job>>>();
    jobs_vec_json_result
}
