use serde::Deserialize;
use serde::Serialize;

use super::Publisher;

const PUBLISHERS_QUERY: &str = "
    query PublishersQuery($limit: Int, $offset: Int, $filter: String) {
        publishers(limit: $limit, offset: $offset, filter: $filter) {
            publisherId
            publisherName
            publisherShortname
            publisherUrl
        }
        publisherCount(filter: $filter)
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
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PublishersResponseData {
    pub publishers: Vec<Publisher>,
    pub publisher_count: i32,
}
