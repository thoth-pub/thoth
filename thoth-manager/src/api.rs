use serde::Deserialize;
use serde::Serialize;
use yewtil::fetch::Fetch;
use yewtil::fetch::FetchAction;
use yewtil::fetch::FetchRequest;
use yewtil::fetch::Json;
use yewtil::fetch::MethodBody;

use crate::string::GRAPHQL_ENDPOINT;

pub type FetchPublishers = Fetch<PublishersRequest, PublishersResponseBody>;
pub type FetchActionPublishers = FetchAction<PublishersResponseBody>;

#[derive(Default, Debug, Clone)]
pub struct PublishersRequest {
    body: PublishersRequestBody,
}

impl FetchRequest for PublishersRequest {
    type RequestBody = PublishersRequestBody;
    type ResponseBody = PublishersResponseBody;
    type Format = Json;

    fn url(&self) -> String { GRAPHQL_ENDPOINT.to_string() }

    fn method(&self) -> MethodBody<Self::RequestBody> {
        MethodBody::Post(&self.body)
    }

    fn headers(&self) -> Vec<(String, String)> {
        vec![("Content-Type".to_string(), "application/json".to_string())]
    }

    fn use_cors(&self) -> bool { true }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PublishersResponseBody {
    pub data: PublishersResponseData,
}

impl Default for PublishersResponseBody {
    fn default() -> PublishersResponseBody {
        PublishersResponseBody {
            data: Default::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PublishersResponseData {
    pub publishers: Vec<Publisher>,
}

impl Default for PublishersResponseData {
    fn default() -> PublishersResponseData {
        PublishersResponseData {
            publishers: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PublishersRequestBody {
    pub query: String,
    pub variables: String,
}

impl Default for PublishersRequestBody {
    fn default() -> PublishersRequestBody {
        PublishersRequestBody {
            query: "
                {
                    publishers(limit: 9999) {
                        publisherId
                        publisherName
                        publisherShortname
                        publisherUrl
                    }
                }
            ".to_string(),
            variables: "null".to_string()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Publisher {
    pub publisher_id: String,
    pub publisher_name: String,
    pub publisher_shortname: Option<String>,
    pub publisher_url: Option<String>,
}

