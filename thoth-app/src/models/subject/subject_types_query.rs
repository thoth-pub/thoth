use serde::Deserialize;
use serde::Serialize;

use super::SubjectTypeDefinition;

const SUBJECT_TYPES_QUERY: &str = "
    {
        subject_types: __type(name: \"SubjectType\") {
            enumValues {
                name
            }
        }
    }
";

graphql_query_builder! {
    SubjectTypesRequest,
    SubjectTypesRequestBody,
    Variables,
    SUBJECT_TYPES_QUERY,
    SubjectTypesResponseBody,
    SubjectTypesResponseData,
    FetchSubjectTypes,
    FetchActionSubjectTypes
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Variables {}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct SubjectTypesResponseData {
    pub subject_types: SubjectTypeDefinition,
}
