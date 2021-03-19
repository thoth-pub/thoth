use serde::Deserialize;
use serde::Serialize;
use thoth_api::work::model::WorkOrderBy;

use super::Work;

pub const WORKS_QUERY: &str = "
    query WorksQuery($limit: Int, $offset: Int, $filter: String, $publishers: [Uuid!]) {
        works(limit: $limit, offset: $offset, filter: $filter, publishers: $publishers) {
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
                lastName
                fullName
                contributor {
                    contributorId
                    lastName
                    fullName
                    updatedAt
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
        workCount(filter: $filter, publishers: $publishers)
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
    pub order: Option<WorkOrderBy>,
    pub publishers: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WorksResponseData {
    pub works: Vec<Work>,
    pub work_count: i32,
}
