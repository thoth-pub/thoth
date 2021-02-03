use serde::Deserialize;
use serde::Serialize;

use crate::graphql_query_builder;

pub const CONTRIBUTOR_LINKS_QUERY: &str = "
    query ContributorLinksQuery($contributorId: Uuid!) {
        contributor(contributorId: $contributorId) {
            contributions {
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
    ContributorLinksRequest,
    ContributorLinksRequestBody,
    Variables,
    CONTRIBUTOR_LINKS_QUERY,
    ContributorLinksResponseBody,
    ContributorLinksResponseData,
    FetchContributorLinks,
    FetchActionContributorLinks
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub contributor_id: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ContributorLinksResponseData {
    pub contributor: Option<ContributorLink>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ContributorLink {
    pub contributions: Option<Vec<SlimContribution>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SlimContribution {
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