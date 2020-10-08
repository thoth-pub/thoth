use serde::Deserialize;
use serde::Serialize;

use crate::api::models::work::Work;

pub const WORKS_QUERY: &str = "
    query PublicationsQuery($limit: Int, $offset: Int, $filter: String) {
        works(limit: $limit, offset: $offset, filter: $filter) {
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
                workId
                contributorId
                contributionType
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
        workCount(filter: $filter)
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
#[serde(rename_all = "camelCase")]
pub struct WorksResponseData {
    pub works: Vec<Work>,
    pub work_count: i32,
}

impl Default for WorksResponseData {
    fn default() -> WorksResponseData {
        WorksResponseData { works: vec![], work_count: 0 }
    }
}
