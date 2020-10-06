use serde::Deserialize;
use serde::Serialize;

use crate::api::models::Publication;

const PUBLICATIONS_QUERY: &str = "
    {
        publications(limit: 9999) {
            publicationId
            publicationType
            workId
            isbn
            publicationUrl
            work {
                workId
                workType
                workStatus
                fullTitle
                doi
                title
                edition
                copyrightHolder
                imprint {
                    imprintId
                    imprintName
                    publisher {
                        publisherId
                        publisherName
                        publisherShortname
                        publisherUrl
                    }
                }
            }
        }
    }
";

query_builder! {
    PublicationsRequest,
    PublicationsRequestBody,
    PUBLICATIONS_QUERY,
    PublicationsResponseBody,
    PublicationsResponseData,
    FetchPublications,
    FetchActionPublications
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PublicationsResponseData {
    pub publications: Vec<Publication>,
}

impl Default for PublicationsResponseData {
    fn default() -> PublicationsResponseData {
        PublicationsResponseData { publications: vec![] }
    }
}
