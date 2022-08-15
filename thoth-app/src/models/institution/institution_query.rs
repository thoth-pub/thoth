use serde::Deserialize;
use serde::Serialize;
use thoth_api::model::institution::Institution;
use uuid::Uuid;

pub const INSTITUTION_QUERY: &str = "
    query InstitutionQuery($institutionId: Uuid!) {
        institution(institutionId: $institutionId) {
            institutionId
            institutionName
            institutionDoi
            ror
            countryCode
            createdAt
            updatedAt
        }
    }
";

graphql_query_builder! {
    InstitutionRequest,
    InstitutionRequestBody,
    Variables,
    INSTITUTION_QUERY,
    InstitutionResponseBody,
    InstitutionResponseData,
    FetchInstitution,
    FetchActionInstitution
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub institution_id: Option<Uuid>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct InstitutionResponseData {
    pub institution: Option<Institution>,
}
