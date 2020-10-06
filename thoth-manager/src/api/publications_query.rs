use serde::Deserialize;
use serde::Serialize;

use crate::api::models::Publication;

pub const PUBLICATIONS_QUERY: &str = "
    query PublicationsQuery($filter: String) {
        publications(limit: 9999, filter: $filter) {
            publicationId
            publicationType
            workId
            isbn
            publicationUrl
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
