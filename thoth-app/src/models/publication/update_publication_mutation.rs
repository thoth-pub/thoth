use serde::Deserialize;
use serde::Serialize;
use thoth_api::publication::model::PublicationType;

use super::Publication;

const UPDATE_PUBLICATION_MUTATION: &str = "
    mutation UpdatePublication(
        $publicationId: Uuid!,
        $publicationType: PublicationType!,
        $work_id: Uuid!,
        $isbn: String,
        $publicationUrl: String,
    ) {
        updatePublication(data: {
            publicationId: $publicationId
            publicationType: $publicationType
            workId: $workId
            isbn: $isbn
            publicationUrl: $publicationUrl
        }){
            publicationId
            publicationType
            workId
        }
    }
";

graphql_query_builder! {
    UpdatePublicationRequest,
    UpdatePublicationRequestBody,
    Variables,
    UPDATE_PUBLICATION_MUTATION,
    UpdatePublicationResponseBody,
    UpdatePublicationResponseData,
    PushUpdatePublication,
    PushActionUpdatePublication
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub publication_id: String,
    pub publication_type: PublicationType,
    pub work_id: String,
    pub isbn: Option<String>,
    pub publication_url: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePublicationResponseData {
    pub update_publication: Option<Publication>,
}
