use serde::Deserialize;
use serde::Serialize;
use thoth_api::model::work::WorkOrderBy;
use thoth_api::model::work::WorkWithRelations;

pub const BOOKS_QUERY: &str = "
    query BooksQuery($limit: Int, $offset: Int, $filter: String, $publishers: [Uuid!], $order: WorkOrderBy) {
        books(limit: $limit, offset: $offset, filter: $filter, publishers: $publishers, order: $order) {
            workId
            workType
            workStatus
            fullTitle
            title
            edition
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
        bookCount(filter: $filter, publishers: $publishers)
    }
";

graphql_query_builder! {
    BooksRequest,
    BooksRequestBody,
    Variables,
    BOOKS_QUERY,
    BooksResponseBody,
    BooksResponseData,
    FetchBooks,
    FetchActionBooks
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
pub struct BooksResponseData {
    pub books: Vec<WorkWithRelations>,
    pub book_count: i32,
}
