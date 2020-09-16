use serde::Deserialize;
use serde::Serialize;
use yewtil::fetch::Fetch;
use yewtil::fetch::FetchAction;
use yewtil::fetch::FetchRequest;
use yewtil::fetch::Json;
use yewtil::fetch::MethodBody;

use crate::string::GRAPHQL_ENDPOINT;

pub type FetchWorks = Fetch<WorksRequest, WorksResponseBody>;
pub type FetchActionWorks = FetchAction<WorksResponseBody>;
pub type FetchPublishers = Fetch<PublishersRequest, PublishersResponseBody>;
pub type FetchActionPublishers = FetchAction<PublishersResponseBody>;

#[derive(Default, Debug, Clone)]
pub struct WorksRequest {
    body: WorksRequestBody,
}

#[derive(Default, Debug, Clone)]
pub struct PublishersRequest {
    body: PublishersRequestBody,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorksResponseBody {
    pub data: WorksResponseData,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PublishersResponseBody {
    pub data: PublishersResponseData,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorksResponseData {
    pub works: Vec<Work>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PublishersResponseData {
    pub publishers: Vec<Publisher>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorksRequestBody {
    pub query: String,
    pub variables: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PublishersRequestBody {
    pub query: String,
    pub variables: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Work {
    pub work_id: String,
    pub title: String,
    pub doi: String,
    pub contributions: Option<Vec<Contribution>>,
    pub imprint: Imprint,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Imprint {
    pub publisher: Publisher,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Publisher {
    pub publisher_id: String,
    pub publisher_name: String,
    pub publisher_shortname: Option<String>,
    pub publisher_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Contribution {
    pub main_contribution: bool,
    pub contributor: Contributor,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Contributor {
    pub full_name: String,
}

impl FetchRequest for WorksRequest {
    type RequestBody = WorksRequestBody;
    type ResponseBody = WorksResponseBody;
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

impl Default for WorksResponseBody {
    fn default() -> WorksResponseBody {
        WorksResponseBody { data: Default::default() }
    }
}

impl Default for PublishersResponseBody {
    fn default() -> PublishersResponseBody {
        PublishersResponseBody { data: Default::default() }
    }
}

impl Default for WorksResponseData {
    fn default() -> WorksResponseData {
        WorksResponseData { works: vec![] }
    }
}

impl Default for PublishersResponseData {
    fn default() -> PublishersResponseData {
        PublishersResponseData { publishers: vec![] }
    }
}

impl Default for WorksRequestBody {
    fn default() -> WorksRequestBody {
        WorksRequestBody {
            query: "
                {
                works(limit: 9999) {
                    title
                    workId
                    doi
                    contributions {
                        mainContribution
                        contributor {
                            fullName
                        }
                    }
                    imprint {
                        publisher {
                            publisherId
                            publisherName
                            publisherShortname
                            publisherUrl
                        }
                    }
                }
            }
            ".to_string(),
            variables: "null".to_string()
        }
    }
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
