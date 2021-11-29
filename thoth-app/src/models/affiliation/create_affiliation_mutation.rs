use serde::Deserialize;
use serde::Serialize;
use thoth_api::model::affiliation::AffiliationWithInstitution;
use uuid::Uuid;

const CREATE_AFFILIATION_MUTATION: &str = "
    mutation CreateAffiliation(
        $contributionId: Uuid!,
        $institutionId: Uuid!,
        $affiliationOrdinal: Int!
        $position: String,
    ) {
        createAffiliation(data: {
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
    CreateAffiliationRequest,
    CreateAffiliationRequestBody,
    Variables,
    CREATE_AFFILIATION_MUTATION,
    CreateAffiliationResponseBody,
    CreateAffiliationResponseData,
    PushCreateAffiliation,
    PushActionCreateAffiliation
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub contribution_id: Uuid,
    pub institution_id: Uuid,
    pub affiliation_ordinal: i32,
    pub position: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CreateAffiliationResponseData {
    pub create_affiliation: Option<AffiliationWithInstitution>,
}
