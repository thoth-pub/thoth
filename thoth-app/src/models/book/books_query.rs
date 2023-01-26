use serde::Deserialize;
use serde::Serialize;
use thoth_api::model::work::WorkWithRelations;

pub use crate::models::work::works_query::Variables;
use crate::models::work::works_query::WORKS_QUERY_BODY;

pub const BOOKS_QUERY_HEADER: &str = "
    query BooksQuery($limit: Int, $offset: Int, $filter: String, $publishers: [Uuid!], $order: WorkOrderBy) {
        books(limit: $limit, offset: $offset, filter: $filter, publishers: $publishers, order: $order) {";

pub const BOOKS_QUERY_FOOTER: &str = "
        bookCount(filter: $filter, publishers: $publishers)
    }
";

graphql_query_builder! {
    BooksRequest,
    BooksRequestBody,
    Variables,
    format!("{BOOKS_QUERY_HEADER}{WORKS_QUERY_BODY}{BOOKS_QUERY_FOOTER}"),
    BooksResponseBody,
    BooksResponseData,
    FetchBooks,
    FetchActionBooks
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BooksResponseData {
    pub books: Vec<WorkWithRelations>,
    pub book_count: i32,
}
