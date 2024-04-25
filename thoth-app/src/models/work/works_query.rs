use serde::Deserialize;
use serde::Serialize;
use thoth_api::model::work::WorkOrderBy;
use thoth_api::model::work::WorkWithRelations;

pub const WORKS_QUERY_HEADER: &str = "
    query WorksQuery($limit: Int, $offset: Int, $filter: String, $publishers: [Uuid!], $order: WorkOrderBy) {
        works(limit: $limit, offset: $offset, filter: $filter, publishers: $publishers, order: $order) {";

// The same object and attributes are retrieved across Works, Books and Chapters queries.
// Pull this section out so it can be reused and consistently updated.
pub const WORKS_QUERY_BODY: &str = "
            workId
            workType
            workStatus
            fullTitle
            title
            landingPage
            doi
            coverUrl
            license
            place
            publicationDate
            withdrawnDate
            updatedAt
            contributions {
                contributionId
                workId
                contributorId
                contributionType
                mainContribution
                createdAt
                updatedAt
                lastName
                fullName
                contributionOrdinal
            }
            imprint {
                imprintId
                imprintName
                updatedAt
                publisher {
                    publisherId
                    publisherName
                    publisherShortname
                    publisherUrl
                    createdAt
                    updatedAt
                }
            }
        }";

pub const WORKS_QUERY_FOOTER: &str = "
        workCount(filter: $filter, publishers: $publishers)
    }
";

graphql_query_builder! {
    WorksRequest,
    WorksRequestBody,
    Variables,
    format!("{WORKS_QUERY_HEADER}{WORKS_QUERY_BODY}{WORKS_QUERY_FOOTER}"),
    WorksResponseBody,
    WorksResponseData,
    FetchWorks,
    FetchActionWorks
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
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
    pub works: Vec<WorkWithRelations>,
    pub work_count: i32,
}
