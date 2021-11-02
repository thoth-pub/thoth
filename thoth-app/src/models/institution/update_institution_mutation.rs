use serde::Deserialize;
use serde::Serialize;
use thoth_api::model::institution::Institution;
use thoth_api::model::Doi;
use uuid::Uuid;

const UPDATE_INSTITUTION_MUTATION: &str = "
    mutation UpdateInstitution(
        $institutionId: Uuid!,
        $institutionName: String!,
        $institutionDoi: Doi
    ) {
        updateInstitution(data: {
            institutionId: $institutionId
            institutionName: $institutionName
            institutionDoi: $institutionDoi
        }){
            institutionId
            institutionName
            createdAt
            updatedAt
        }
    }
";

graphql_query_builder! {
    UpdateInstitutionRequest,
    UpdateInstitutionRequestBody,
    Variables,
    UPDATE_INSTITUTION_MUTATION,
    UpdateInstitutionResponseBody,
    UpdateInstitutionResponseData,
    PushUpdateInstitution,
    PushActionUpdateInstitution
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub institution_id: Uuid,
    pub institution_name: String,
    pub institution_doi: Option<Doi>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInstitutionResponseData {
    pub update_institution: Option<Institution>,
}
