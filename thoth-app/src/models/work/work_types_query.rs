use serde::Deserialize;
use serde::Serialize;

use super::WorkTypeDefinition;

const WORK_TYPES_QUERY: &str = "
    {
        work_types: __type(name: \"WorkType\") {
            enumValues {
                name
            }
        }
    }
";

graphql_query_builder! {
    WorkTypesRequest,
    WorkTypesRequestBody,
    Variables,
    WORK_TYPES_QUERY,
    WorkTypesResponseBody,
    WorkTypesResponseData,
    FetchWorkTypes,
    FetchActionWorkTypes
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Variables {}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct WorkTypesResponseData {
    pub work_types: WorkTypeDefinition,
}
