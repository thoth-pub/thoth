use serde::Deserialize;
use serde::Serialize;

use crate::api::models::Work;

const WORKS_QUERY: &str = "
    {
        works(limit: 9999) {
            workId
            workType
            workStatus
            fullTitle
            title
            edition
            copyrightHolder
            doi
            coverUrl
            license
            place
            publicationDate
            contributions {
                mainContribution
                contributor {
                    contributorId
                    fullName
                }
            }
            imprint {
                imprintId
                imprintName
                publisher {
                    publisherId
                    publisherName
                    publisherShortname
                    publisherUrl
                }
            }
        }
    }
";

query_builder! {
    WorksRequest,
    WorksRequestBody,
    WORKS_QUERY,
    WorksResponseBody,
    WorksResponseData,
    FetchWorks,
    FetchActionWorks
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorksResponseData {
    pub works: Vec<Work>,
}

impl Default for WorksResponseData {
    fn default() -> WorksResponseData {
        WorksResponseData { works: vec![] }
    }
}
