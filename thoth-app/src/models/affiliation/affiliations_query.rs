use serde::Deserialize;
use serde::Serialize;
use thoth_api::model::contribution::ContributionWithAffiliations;
use uuid::Uuid;

pub const AFFILIATIONS_QUERY: &str = "
    query AffiliationsQuery($contributionId: Uuid!) {
        contribution(contributionId: $contributionId) {
            affiliations {
                affiliationId
                contributionId
                institutionId
                affiliationOrdinal
                position
                institution {
                    institutionId
                    institutionName
                    createdAt
                    updatedAt
                }
            }
        }
    }
";

graphql_query_builder! {
    AffiliationsRequest,
    AffiliationsRequestBody,
    Variables,
    AFFILIATIONS_QUERY,
    AffiliationsResponseBody,
    AffiliationsResponseData,
    FetchAffiliations,
    FetchActionAffiliations
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub contribution_id: Uuid,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AffiliationsResponseData {
    pub contribution: Option<ContributionWithAffiliations>,
}
