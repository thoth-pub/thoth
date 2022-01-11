use crate::models::chapter::chapters_query::ChaptersRequest;
use crate::models::chapter::chapters_query::ChaptersRequestBody;
use crate::models::chapter::chapters_query::FetchActionChapters;
use crate::models::chapter::chapters_query::FetchChapters;
use crate::models::chapter::chapters_query::Variables;
use thoth_api::model::work::WorkField;
use thoth_api::model::work::WorkOrderBy;
use thoth_api::model::work::WorkWithRelations;

pagination_component! {
    ChaptersComponent,
    WorkWithRelations,
    chapters,
    chapter_count,
    ChaptersRequest,
    FetchActionChapters,
    FetchChapters,
    ChaptersRequestBody,
    Variables,
    SEARCH_WORKS,
    PAGINATION_COUNT_CHAPTERS,
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
