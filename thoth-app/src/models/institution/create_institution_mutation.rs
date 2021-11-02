use serde::Deserialize;
use serde::Serialize;
use thoth_api::model::institution::Institution;
use thoth_api::model::Doi;
use thoth_api::model::Ror;

const CREATE_INSTITUTION_MUTATION: &str = "
    mutation CreateInstitution(
        $institutionName: String!,
        $institutionDoi: Doi,
        $ror: Ror
    ) {
        createInstitution(data: {
            institutionName: $institutionName
            institutionDoi: $institutionDoi
            ror: $ror
        }){
            institutionId
            institutionName
            ror
            createdAt
            updatedAt
        }
    }
";

graphql_query_builder! {
    CreateInstitutionRequest,
    CreateInstitutionRequestBody,
    Variables,
    CREATE_INSTITUTION_MUTATION,
    CreateInstitutionResponseBody,
    CreateInstitutionResponseData,
    PushCreateInstitution,
    PushActionCreateInstitution
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub institution_name: String,
    pub institution_doi: Option<Doi>,
    pub ror: Option<Ror>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CreateInstitutionResponseData {
    pub create_institution: Option<Institution>,
}
