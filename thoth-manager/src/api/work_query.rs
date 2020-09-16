use serde::Deserialize;
use serde::Serialize;

use crate::api::models::Work;

// $query here is filled in when instantiated. TODO make use of variables and predefine the query
query_builder!{
    WorkRequest,
    WorkRequestBody,
    "",
    WorkResponseBody,
    WorkResponseData,
    FetchWork,
    FetchActionWork
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkResponseData {
    pub work: Option<Work>,
}

impl Default for WorkResponseData {
    fn default() -> WorkResponseData {
        WorkResponseData {
            work: None
        }
    }
}
