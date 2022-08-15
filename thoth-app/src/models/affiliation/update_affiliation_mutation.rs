use serde::Deserialize;
use serde::Serialize;
use thoth_api::model::affiliation::AffiliationWithInstitution;
use uuid::Uuid;

const UPDATE_AFFILIATION_MUTATION: &str = "
    mutation UpdateAffiliation(
        $affiliationId: Uuid!,
        $contributionId: Uuid!,
        $institutionId: Uuid!,
        $affiliationOrdinal: Int!
        $position: String,
    ) {
        updateAffiliation(data: {
            affiliationId: $affiliationId
            contributionId: $contributionId
            institutionId: $institutionId
            affiliationOrdinal: $affiliationOrdinal
            position: $position
        }){
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
";

graphql_query_builder! {
    UpdateAffiliationRequest,
    UpdateAffiliationRequestBody,
    Variables,
    UPDATE_AFFILIATION_MUTATION,
    UpdateAffiliationResponseBody,
    UpdateAffiliationResponseData,
    PushUpdateAffiliation,
    PushActionUpdateAffiliation
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub affiliation_id: Uuid,
    pub contribution_id: Uuid,
    pub institution_id: Uuid,
    pub affiliation_ordinal: i32,
    pub position: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAffiliationResponseData {
    pub update_affiliation: Option<AffiliationWithInstitution>,
}
