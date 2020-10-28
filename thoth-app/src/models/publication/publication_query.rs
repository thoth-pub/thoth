use serde::Deserialize;
use serde::Serialize;

use super::Publication;

pub const PUBLICATION_QUERY: &str = "
    query PublicationQuery($publicationId: Uuid!) {
        publication(publicationId: $publicationId) {
            publicationId
            publicationType
            workId
            isbn
            publicationUrl
        }
    }
";

graphql_query_builder! {
    PublicationRequest,
    PublicationRequestBody,
    Variables,
    PUBLICATION_QUERY,
    PublicationResponseBody,
    PublicationResponseData,
    FetchPublication,
    FetchActionPublication
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub publication_id: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct PublicationResponseData {
    pub publication: Option<Publication>,
}
