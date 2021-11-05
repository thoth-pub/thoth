use serde::Deserialize;
use serde::Serialize;
use thoth_api::model::institution::CountryCode;
use thoth_api::model::institution::Institution;
use thoth_api::model::Doi;
use thoth_api::model::Ror;
use uuid::Uuid;

const UPDATE_INSTITUTION_MUTATION: &str = "
    mutation UpdateInstitution(
        $institutionId: Uuid!,
        $institutionName: String!,
        $institutionDoi: Doi,
        $ror: Ror,
        $countryCode: CountryCode
    ) {
        updateInstitution(data: {
            institutionId: $institutionId
            institutionName: $institutionName
            institutionDoi: $institutionDoi
            ror: $ror
            countryCode: $countryCode
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
    pub ror: Option<Ror>,
    pub country_code: Option<CountryCode>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInstitutionResponseData {
    pub update_institution: Option<Institution>,
}
