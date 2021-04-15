use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

use super::Publication;

pub const PUBLICATION_QUERY: &str = "
    query PublicationQuery($publicationId: Uuid!) {
        publication(publicationId: $publicationId) {
            publicationId
            publicationType
            workId
            isbn
            publicationUrl
            prices {
                priceId
                publicationId
                currencyCode
                unitPrice
            }
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
    pub publication_id: Option<Uuid>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct PublicationResponseData {
    pub publication: Option<Publication>,
}
