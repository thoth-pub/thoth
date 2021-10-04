use serde::Deserialize;
use serde::Serialize;
use thoth_api::model::publisher::Publisher;
use uuid::Uuid;

pub const PUBLISHER_QUERY: &str = "
    query PublisherQuery($publisherId: Uuid!) {
        publisher(publisherId: $publisherId) {
            publisherId
            publisherName
            publisherShortname
            publisherUrl
            createdAt
            updatedAt
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
    pub publisher_id: Option<Uuid>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct PublisherResponseData {
    pub publisher: Option<Publisher>,
}
