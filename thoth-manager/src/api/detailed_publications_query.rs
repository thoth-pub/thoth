use serde::Deserialize;
use serde::Serialize;
// We are using DetailedPublication instead of Publication so we can get more info
use crate::api::models::DetailedPublication;

pub const DETAILED_PUBLICATIONS_QUERY: &str = "
    query PublicationsQuery($filter: String) {
        publications(limit: 9999, filter: $filter) {
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
    DetailedPublicationsRequest,
    DetailedPublicationsRequestBody,
    DETAILED_PUBLICATIONS_QUERY,
    DetailedPublicationsResponseBody,
    DetailedPublicationsResponseData,
    FetchDetailedPublications,
    FetchActionDetailedPublications
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DetailedPublicationsResponseData {
    pub publications: Vec<DetailedPublication>,
}

impl Default for DetailedPublicationsResponseData {
    fn default() -> DetailedPublicationsResponseData {
        DetailedPublicationsResponseData { publications: vec![] }
    }
}
