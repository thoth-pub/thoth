use serde::Deserialize;
use serde::Serialize;
use thoth_api::publication::model::PublicationType;
use uuid::Uuid;

use super::Publication;

const CREATE_PUBLICATION_MUTATION: &str = "
    mutation CreatePublication(
        $publicationType: PublicationType!,
        $workId: Uuid!,
        $isbn: String,
        $publicationUrl: String,
    ) {
        createPublication(data: {
            publicationType: $publicationType
            workId: $workId
            isbn: $isbn
            publicationUrl: $publicationUrl
        }){
            publicationId
            publicationType
            isbn
            publicationUrl
            workId
            work {
                imprint {
                    publisher {
                        publisherId
                    }
                }
            }
        }
    }
";

graphql_query_builder! {
    CreatePublicationRequest,
    CreatePublicationRequestBody,
    Variables,
    CREATE_PUBLICATION_MUTATION,
    CreatePublicationResponseBody,
    CreatePublicationResponseData,
    PushCreatePublication,
    PushActionCreatePublication
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub publication_type: PublicationType,
    pub work_id: Uuid,
    pub isbn: Option<String>,
    pub publication_url: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CreatePublicationResponseData {
    pub create_publication: Option<Publication>,
}
