use serde::Deserialize;
use serde::Serialize;

use super::WorkStatusDefinition;

const WORK_STATUSES_QUERY: &str = "
    {
        work_statuses: __type(name: \"WorkStatus\") {
            enumValues {
                name
            }
        }
    }
";

graphql_query_builder! {
    WorkStatusesRequest,
    WorkStatusesRequestBody,
    Variables,
    WORK_STATUSES_QUERY,
    WorkStatusesResponseBody,
    WorkStatusesResponseData,
    FetchWorkStatuses,
    FetchActionWorkStatuses
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Variables {}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkStatusesResponseData {
    pub work_statuses: WorkStatusDefinition,
}
