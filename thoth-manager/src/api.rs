use serde::Deserialize;
use serde::Serialize;
use serde::de;
use serde::de::Deserializer;
use yewtil::fetch::Fetch;
use yewtil::fetch::FetchAction;
use yewtil::fetch::FetchRequest;
use yewtil::fetch::Json;
use yewtil::fetch::MethodBody;

use crate::string::GRAPHQL_ENDPOINT;

pub type FetchWork = Fetch<WorkRequest, WorkResponseBody>;
pub type FetchActionWork = FetchAction<WorkResponseBody>;
pub type FetchWorks = Fetch<WorksRequest, WorksResponseBody>;
pub type FetchActionWorks = FetchAction<WorksResponseBody>;
pub type FetchPublishers = Fetch<PublishersRequest, PublishersResponseBody>;
pub type FetchActionPublishers = FetchAction<PublishersResponseBody>;

#[derive(Default, Debug, Clone)]
pub struct WorkRequest {
    pub body: WorkRequestBody,
}

#[derive(Default, Debug, Clone)]
pub struct WorksRequest {
    body: WorksRequestBody,
}

#[derive(Default, Debug, Clone)]
pub struct PublishersRequest {
    body: PublishersRequestBody,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkResponseBody {
    pub data: WorkResponseData,
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
pub struct WorkResponseData {
    pub work: Work,
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
pub struct WorkRequestBody {
    pub query: String,
    pub variables: String,
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
    pub full_title: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub doi: String,
    pub cover_url: String,
    pub license: License,
    pub place: String,
    pub publication_date: Option<String>,
    pub contributions: Option<Vec<Contribution>>,
    pub imprint: Imprint,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub enum License {
    By,
    BySa,
    ByNd,
    ByNc,
    ByNcSa,
    ByNcNd,
    Zero,
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

impl<'de> Deserialize<'de> for License {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let l = String::deserialize(deserializer)?.to_lowercase();
        let license = match l.as_str() {
            "http://creativecommons.org/licenses/by/1.0/"
                | "http://creativecommons.org/licenses/by/2.0/"
                | "http://creativecommons.org/licenses/by/2.5/"
                | "http://creativecommons.org/licenses/by/3.0/"
                | "http://creativecommons.org/licenses/by/4.0/" => License::By,
            "http://creativecommons.org/licenses/by-sa/1.0/"
                  | "http://creativecommons.org/licenses/by-sa/2.0/"
                  | "http://creativecommons.org/licenses/by-sa/2.5/"
                  | "http://creativecommons.org/licenses/by-sa/3.0/"
                  | "http://creativecommons.org/licenses/by-sa/4.0/" => License::BySa,
            "http://creativecommons.org/licenses/by-nd/1.0/"
                  | "http://creativecommons.org/licenses/by-nd/2.0/"
                  | "http://creativecommons.org/licenses/by-nd/2.5/"
                  | "http://creativecommons.org/licenses/by-nd/3.0/"
                  | "http://creativecommons.org/licenses/by-nd/4.0/" => License::ByNd,
            "http://creativecommons.org/licenses/by-nc/1.0/"
                  | "http://creativecommons.org/licenses/by-nc/2.0/"
                  | "http://creativecommons.org/licenses/by-nc/2.5/"
                  | "http://creativecommons.org/licenses/by-nc/3.0/"
                  | "http://creativecommons.org/licenses/by-nc/4.0/" => License::ByNc,
            "http://creativecommons.org/licenses/by-nc-sa/1.0/"
                  | "http://creativecommons.org/licenses/by-nc-sa/2.0/"
                  | "http://creativecommons.org/licenses/by-nc-sa/2.5/"
                  | "http://creativecommons.org/licenses/by-nc-sa/3.0/"
                  | "http://creativecommons.org/licenses/by-nc-sa/4.0/" => License::ByNcSa,
            "http://creativecommons.org/licenses/by-nc-nd/1.0/"
                  | "http://creativecommons.org/licenses/by-nc-nd/2.0/"
                  | "http://creativecommons.org/licenses/by-nc-nd/2.5/"
                  | "http://creativecommons.org/licenses/by-nc-nd/3.0/"
                  | "http://creativecommons.org/licenses/by-nc-nd/4.0/" => License::ByNcNd,
            "https://creativecommons.org/publicdomain/zero/1.0/" => License::Zero,
            other => { return Err(de::Error::custom(format!("Invalid license '{}'", other))); },
        };
        Ok(license)
    }
}

impl FetchRequest for WorkRequest {
    type RequestBody = WorkRequestBody;
    type ResponseBody = WorkResponseBody;
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

impl Default for WorkResponseBody {
    fn default() -> WorkResponseBody {
        WorkResponseBody { data: Default::default() }
    }
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

impl Default for WorkResponseData {
    fn default() -> WorkResponseData {
        WorkResponseData {
            work: Work {
                work_id: "".to_string(),
                full_title: "".to_string(),
                title: "".to_string(),
                subtitle: None,
                doi: "".to_string(),
                cover_url: "".to_string(),
                license: License::By,
                place: "".to_string(),
                publication_date: None,
                contributions: None,
                imprint: Imprint {
                    publisher: Publisher {
                        publisher_id: "".to_string(),
                        publisher_name: "".to_string(),
                        publisher_shortname: None,
                        publisher_url: None,
                    }
                },
            },
        }
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

impl Default for WorkRequestBody {
    fn default() -> WorkRequestBody {
        WorkRequestBody { query: "".to_string(), variables: "null".to_string() }
    }
}

impl Default for WorksRequestBody {
    fn default() -> WorksRequestBody {
        WorksRequestBody {
            query: "
                {
                works(limit: 9999) {
                    workId
                    fullTitle
                    title
                    doi
                    coverUrl
                    license
                    publicationDate
                    place
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
