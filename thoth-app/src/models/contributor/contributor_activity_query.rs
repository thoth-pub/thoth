use serde::Deserialize;
use serde::Serialize;
use thoth_api::contribution::model::SlimContribution;
use uuid::Uuid;

use crate::graphql_query_builder;

pub const CONTRIBUTOR_ACTIVITY_QUERY: &str = "
    query ContributorActivityQuery($contributorId: Uuid!) {
        contributor(contributorId: $contributorId) {
            contributions {
                work {
                    workId
                    title
                    imprint {
                        publisher {
                            publisherId
                            publisherName
                        }
                    }
                }
            }
        }
    }
";

graphql_query_builder! {
    ContributorActivityRequest,
    ContributorActivityRequestBody,
    Variables,
    CONTRIBUTOR_ACTIVITY_QUERY,
    ContributorActivityResponseBody,
    ContributorActivityResponseData,
    FetchContributorActivity,
    FetchActionContributorActivity
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub contributor_id: Option<Uuid>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ContributorActivityResponseData {
    pub contributor: Option<ContributorActivity>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ContributorActivity {
    pub contributions: Option<Vec<SlimContribution>>,
}
