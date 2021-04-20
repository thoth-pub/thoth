use serde::Deserialize;
use serde::Serialize;
use thoth_api::publisher::model::PublisherOrderBy;

use super::Publisher;

const PUBLISHERS_QUERY: &str = "
    query PublishersQuery($limit: Int, $offset: Int, $filter: String, $publishers: [Uuid!], $order: PublisherOrderBy) {
        publishers(limit: $limit, offset: $offset, filter: $filter, publishers: $publishers, order: $order) {
            publisherId
            publisherName
            publisherShortname
            publisherUrl
            createdAt
            updatedAt
        }
        publisherCount(filter: $filter, publishers: $publishers)
    }
";

graphql_query_builder! {
    PublishersRequest,
    PublishersRequestBody,
    Variables,
    PUBLISHERS_QUERY,
    PublishersResponseBody,
    PublishersResponseData,
    FetchPublishers,
    FetchActionPublishers
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub limit: Option<i32>,
    pub offset: Option<i32>,
    pub filter: Option<String>,
    pub order: Option<PublisherOrderBy>,
    pub publishers: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PublishersResponseData {
    pub publishers: Vec<Publisher>,
    pub publisher_count: i32,
}
