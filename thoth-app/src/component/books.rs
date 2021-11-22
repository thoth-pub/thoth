use crate::models::book::books_query::BooksRequest;
use crate::models::book::books_query::BooksRequestBody;
use crate::models::book::books_query::FetchActionBooks;
use crate::models::book::books_query::FetchBooks;
use crate::models::book::books_query::Variables;
use thoth_api::model::work::WorkField;
use thoth_api::model::work::WorkOrderBy;
use thoth_api::model::work::WorkWithRelations;

pagination_component! {
    BooksComponent,
    WorkWithRelations,
    books,
    book_count,
    BooksRequest,
    FetchActionBooks,
    FetchBooks,
    BooksRequestBody,
    Variables,
    SEARCH_WORKS,
    PAGINATION_COUNT_BOOKS,
    vec![
        WorkField::WorkId.to_string(),
        WorkField::FullTitle.to_string(),
        WorkField::WorkType.to_string(),
        "Contributors".to_string(),
        WorkField::Doi.to_string(),
        "Publisher".to_string(),
        WorkField::UpdatedAt.to_string(),
    ],
    WorkOrderBy,
    WorkField,
}
