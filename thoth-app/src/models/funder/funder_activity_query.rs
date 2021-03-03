use serde::Deserialize;
use serde::Serialize;

use crate::graphql_query_builder;

pub const FUNDER_ACTIVITY_QUERY: &str = "
    query FunderActivityQuery($funderId: Uuid!) {
        funder(funderId: $funderId) {
            fundings {
                work {
                    workId
                    title
                    imprint {
                        publisher {
                            publisherName
                        }
                    }
                }
            }
        }
    }
";

graphql_query_builder! {
    FunderActivityRequest,
    FunderActivityRequestBody,
    Variables,
    FUNDER_ACTIVITY_QUERY,
    FunderActivityResponseBody,
    FunderActivityResponseData,
    FetchFunderActivity,
    FetchActionFunderActivity
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub funder_id: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct FunderActivityResponseData {
    pub funder: Option<FunderActivity>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FunderActivity {
    pub fundings: Option<Vec<SlimFunding>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SlimFunding {
    pub work: SlimWork,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SlimWork {
    pub work_id: String,
    pub title: String,
    pub imprint: SlimImprint,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SlimImprint {
    pub publisher: SlimPublisher,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SlimPublisher {
    pub publisher_name: String,
}
