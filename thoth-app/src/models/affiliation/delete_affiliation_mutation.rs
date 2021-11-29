use serde::Deserialize;
use serde::Serialize;
use thoth_api::model::affiliation::Affiliation;
use uuid::Uuid;

const DELETE_AFFILIATION_MUTATION: &str = "
    mutation DeleteAffiliation(
        $affiliationId: Uuid!,
    ) {
        deleteAffiliation(
            affiliationId: $affiliationId
        ){
            affiliationId
            contributionId
            institutionId
            affiliationOrdinal
            createdAt
            updatedAt
        }
    }
";

graphql_query_builder! {
    DeleteAffiliationRequest,
    DeleteAffiliationRequestBody,
    Variables,
    DELETE_AFFILIATION_MUTATION,
    DeleteAffiliationResponseBody,
    DeleteAffiliationResponseData,
    PushDeleteAffiliation,
    PushActionDeleteAffiliation
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub affiliation_id: Uuid,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DeleteAffiliationResponseData {
    pub delete_affiliation: Option<Affiliation>,
}
