use serde::Deserialize;
use serde::Serialize;
use thoth_api::publication::model::PublicationExtended as Publication;
use uuid::Uuid;

const DELETE_PUBLICATION_MUTATION: &str = "
    mutation DeletePublication(
        $publicationId: Uuid!
    ) {
        deletePublication(
            publicationId: $publicationId
        ){
            publicationId
            publicationType
            workId
            work {
                workId
                title
                imprint {
                    publisher {
                        publisherId
                        publisherName
                    }
                }
            }
        }
    }
";

graphql_query_builder! {
    DeletePublicationRequest,
    DeletePublicationRequestBody,
    Variables,
    DELETE_PUBLICATION_MUTATION,
    DeletePublicationResponseBody,
    DeletePublicationResponseData,
    PushDeletePublication,
    PushActionDeletePublication
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub publication_id: Uuid,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DeletePublicationResponseData {
    pub delete_publication: Option<Publication>,
}
