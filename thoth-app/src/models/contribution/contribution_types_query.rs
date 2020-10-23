use serde::Deserialize;
use serde::Serialize;

use super::ContributionTypeDefinition;

const CONTRIBUTION_TYPES_QUERY: &str = "
    {
        contribution_types: __type(name: \"ContributionType\") {
            enumValues {
                name
            }
        }
    }
";

graphql_query_builder! {
    ContributionTypesRequest,
    ContributionTypesRequestBody,
    Variables,
    CONTRIBUTION_TYPES_QUERY,
    ContributionTypesResponseBody,
    ContributionTypesResponseData,
    FetchContributionTypes,
    FetchActionContributionTypes
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Variables {}

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
