use serde::Deserialize;
use serde::Serialize;
use thoth_api::model::institution::Institution;
use uuid::Uuid;

const DELETE_INSTITUTION_MUTATION: &str = "
    mutation DeleteInstitution(
        $institutionId: Uuid!
    ) {
        deleteInstitution(
            institutionId: $institutionId
        ){
            institutionId
            institutionName
            createdAt
            updatedAt
        }
    }
";

graphql_query_builder! {
    DeleteInstitutionRequest,
    DeleteInstitutionRequestBody,
    Variables,
    DELETE_INSTITUTION_MUTATION,
    DeleteInstitutionResponseBody,
    DeleteInstitutionResponseData,
    PushDeleteInstitution,
    PushActionDeleteInstitution
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub institution_id: Uuid,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DeleteInstitutionResponseData {
    pub delete_institution: Option<Institution>,
}
