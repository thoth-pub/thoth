use serde::Deserialize;
use serde::Serialize;
use thoth_api::funding::model::FundingWithWork;
use uuid::Uuid;

use crate::graphql_query_builder;

pub const FUNDER_ACTIVITY_QUERY: &str = "
    query FunderActivityQuery($funderId: Uuid!) {
        funder(funderId: $funderId) {
            fundings {
                work {
                    workId
                    workType
                    workStatus
                    fullTitle
                    title
                    edition
                    copyrightHolder
                    updatedAt
                    imprint {
                        imprintId
                        imprintName
                        updatedAt
                        publisher {
                            publisherId
                            publisherName
                            createdAt
                            updatedAt
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
    pub funder_id: Option<Uuid>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct FunderActivityResponseData {
    pub funder: Option<FunderActivity>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FunderActivity {
    pub fundings: Option<Vec<FundingWithWork>>,
}
