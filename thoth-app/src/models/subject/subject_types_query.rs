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

query_builder! {
    SubjectTypesRequest,
    SubjectTypesRequestBody,
    SUBJECT_TYPES_QUERY,
    SubjectTypesResponseBody,
    SubjectTypesResponseData,
    FetchSubjectTypes,
    FetchActionSubjectTypes
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubjectTypesResponseData {
    pub subject_types: SubjectTypeDefinition,
}

impl Default for SubjectTypesResponseData {
    fn default() -> SubjectTypesResponseData {
        SubjectTypesResponseData {
            subject_types: SubjectTypeDefinition {
                enum_values: vec![],
            },
        }
    }
}
