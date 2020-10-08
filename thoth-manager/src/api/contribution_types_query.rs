use serde::Deserialize;
use serde::Serialize;

use crate::api::models::contribution::ContributionTypeDefinition;

const CONTRIBUTION_TYPES_QUERY: &str = "
    {
        contribution_types: __type(name: \"ContributionType\") {
            enumValues {
                name
            }
        }
    }
";

query_builder! {
    ContributionTypesRequest,
    ContributionTypesRequestBody,
    CONTRIBUTION_TYPES_QUERY,
    ContributionTypesResponseBody,
    ContributionTypesResponseData,
    FetchContributionTypes,
    FetchActionContributionTypes
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContributionTypesResponseData {
    pub contribution_types: ContributionTypeDefinition,
}

impl Default for ContributionTypesResponseData {
    fn default() -> ContributionTypesResponseData {
        ContributionTypesResponseData {
            contribution_types: ContributionTypeDefinition {
                enum_values: vec![],
            },
        }
    }
}
