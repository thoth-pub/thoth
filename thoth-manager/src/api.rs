use serde::Deserialize;
use serde::Serialize;
use yewtil::fetch::Fetch;
use yewtil::fetch::FetchAction;
use yewtil::fetch::FetchRequest;
use yewtil::fetch::Json;
use yewtil::fetch::MethodBody;

pub type FetchPublishers = Fetch<Request, ResponseBody>;
pub type FetchActionPublishers = FetchAction<ResponseBody>;

#[derive(Default, Debug, Clone)]
pub struct Request {
    body: RequestBody,
}

impl FetchRequest for Request {
    type RequestBody = RequestBody;
    type ResponseBody = ResponseBody;
    type Format = Json;

    fn url(&self) -> String {
        "http://localhost:8000/graphql".to_string()
    }

    fn method(&self) -> MethodBody<Self::RequestBody> {
        MethodBody::Post(&self.body)
    }

    fn headers(&self) -> Vec<(String, String)> {
        vec![("Content-Type".to_string(), "application/json".to_string())]
    }

    fn use_cors(&self) -> bool {
        true
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResponseBody {
    pub data: ResponseData,
}

impl Default for ResponseBody {
    fn default() -> ResponseBody {
        ResponseBody {
            data: ResponseData { publishers: vec![] },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResponseData {
    pub publishers: Vec<Publisher>,
}

impl Default for ResponseData {
    fn default() -> ResponseData {
        ResponseData {
            publishers: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RequestBody {
    pub query: String,
    pub variables: String,
}

impl Default for RequestBody {
    fn default() -> RequestBody {
        RequestBody {
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

