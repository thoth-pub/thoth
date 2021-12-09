use serde::Deserialize;
use serde::Serialize;
use thoth_api::model::work::WorkOrderBy;
use thoth_api::model::work::WorkWithRelations;

pub const CHAPTERS_QUERY: &str = "
    query ChaptersQuery($limit: Int, $offset: Int, $filter: String, $publishers: [Uuid!], $order: WorkOrderBy) {
        chapters(limit: $limit, offset: $offset, filter: $filter, publishers: $publishers, order: $order) {
            workId
            workType
            workStatus
            fullTitle
            title
            copyrightHolder
            landingPage
            doi
            coverUrl
            license
            place
            publicationDate
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
        }
        chapterCount(filter: $filter, publishers: $publishers)
    }
";

graphql_query_builder! {
    ChaptersRequest,
    ChaptersRequestBody,
    Variables,
    CHAPTERS_QUERY,
    ChaptersResponseBody,
    ChaptersResponseData,
    FetchChapters,
    FetchActionChapters
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
pub struct ChaptersResponseData {
    pub chapters: Vec<WorkWithRelations>,
    pub chapter_count: i32,
}
