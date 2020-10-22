use serde::Deserialize;
use serde::Serialize;

use super::Publisher;

const PUBLISHERS_QUERY: &str = "
    {
        publishers(limit: 9999) {
            publisherId
            publisherName
            publisherShortname
            publisherUrl
        }
    }
";

graphql_query_builder! {
    PublishersRequest,
    PublishersRequestBody,
    PUBLISHERS_QUERY,
    PublishersResponseBody,
    PublishersResponseData,
    FetchPublishers,
    FetchActionPublishers
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PublishersResponseData {
    pub publishers: Vec<Publisher>,
}

impl Default for PublishersResponseData {
    fn default() -> PublishersResponseData {
        PublishersResponseData { publishers: vec![] }
    }
}
