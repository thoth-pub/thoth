use serde::Deserialize;
use serde::Serialize;
use thoth_api::model::work::WorkWithRelations;

pub use crate::models::work::works_query::Variables;
use crate::models::work::works_query::WORKS_QUERY_BODY;

pub const CHAPTERS_QUERY_HEADER: &str = "
    query ChaptersQuery($limit: Int, $offset: Int, $filter: String, $publishers: [Uuid!], $order: WorkOrderBy) {
        chapters(limit: $limit, offset: $offset, filter: $filter, publishers: $publishers, order: $order) {";

pub const CHAPTERS_QUERY_FOOTER: &str = "
        chapterCount(filter: $filter, publishers: $publishers)
    }
";

graphql_query_builder! {
    ChaptersRequest,
    ChaptersRequestBody,
    Variables,
    format!("{}{}{}", CHAPTERS_QUERY_HEADER, WORKS_QUERY_BODY, CHAPTERS_QUERY_FOOTER),
    ChaptersResponseBody,
    ChaptersResponseData,
    FetchChapters,
    FetchActionChapters
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ChaptersResponseData {
    pub chapters: Vec<WorkWithRelations>,
    pub chapter_count: i32,
}
