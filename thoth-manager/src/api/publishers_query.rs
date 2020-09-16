use serde::Deserialize;
use serde::Serialize;

use crate::api::models::Publisher;

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

query_builder!{
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

impl Default for PublishersResponseBody {
    fn default() -> PublishersResponseBody {
        PublishersResponseBody {
            data: PublishersResponseData { publishers: vec![] }
        }
    }
}
