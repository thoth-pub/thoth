use serde::Deserialize;
use serde::Serialize;
use thoth_api::model::biography::Biography;
use uuid::Uuid;

const DELETE_BIOGRAPHY_MUTATION: &str = "
    mutation DeleteBiography(
        $biographyId: Uuid!
    ) {
        deleteBiography(
            biographyId: $biographyId
        ){
            biographyId
            contributionId
            workId
            content
            canonical
            localeCode
        }
    }
";

graphql_query_builder! {
    DeleteBiographyRequest,
    DeleteBiographyRequestBody,
    Variables,
    DELETE_BIOGRAPHY_MUTATION,
    DeleteBiographyResponseBody,
    DeleteBiographyResponseData,
    PushDeleteBiography,
    PushActionDeleteBiography
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub biography_id: Uuid,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DeleteBiographyResponseData {
    pub delete_biography: Option<Biography>,
}
