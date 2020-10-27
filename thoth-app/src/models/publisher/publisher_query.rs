use serde::Deserialize;
use serde::Serialize;

use super::Publisher;

pub const PUBLISHER_QUERY: &str = "
    query PublisherQuery($publisherId: Uuid!) {
        publisher(publisherId: $publisherId) {
            publisherId
            publisherName
        }
    }
";

graphql_query_builder! {
    PublisherRequest,
    PublisherRequestBody,
    Variables,
    PUBLISHER_QUERY,
    PublisherResponseBody,
    PublisherResponseData,
    FetchPublisher,
    FetchActionPublisher
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub publisher_id: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct PublisherResponseData {
    pub publisher: Option<Publisher>,
}
