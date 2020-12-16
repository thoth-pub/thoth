use serde::Deserialize;
use serde::Serialize;

use super::Work;

pub const WORKS_QUERY: &str = "
    query WorksQuery($limit: Int, $offset: Int, $filter: String) {
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
                    lastName
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

graphql_query_builder! {
    WorksRequest,
    WorksRequestBody,
    Variables,
    WORKS_QUERY,
    WorksResponseBody,
    WorksResponseData,
    FetchWorks,
    FetchActionWorks
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub limit: Option<i32>,
    pub offset: Option<i32>,
    pub filter: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WorksResponseData {
    pub works: Vec<Work>,
    pub work_count: i32,
}
