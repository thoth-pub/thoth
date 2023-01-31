use serde::Deserialize;
use serde::Serialize;
use thoth_api::model::work::Work;

pub use crate::models::work::works_query::Variables;
use crate::models::work::works_query::{WORKS_QUERY_FOOTER, WORKS_QUERY_HEADER};

pub const SLIM_WORKS_QUERY_BODY: &str = "
            workId
            workType
            workStatus
            fullTitle
            title
            imprintId
            doi
            copyrightHolder
            createdAt
            updatedAt
        }";

graphql_query_builder! {
    SlimWorksRequest,
    SlimWorksRequestBody,
    Variables,
    format!("{WORKS_QUERY_HEADER}{SLIM_WORKS_QUERY_BODY}{WORKS_QUERY_FOOTER}"),
    SlimWorksResponseBody,
    SlimWorksResponseData,
    FetchSlimWorks,
    FetchActionSlimWorks
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SlimWorksResponseData {
    pub works: Vec<Work>,
    pub work_count: i32,
}
