use crate::models::work::works_query::FetchActionWorks;
use crate::models::work::works_query::FetchWorks;
use crate::models::work::works_query::Variables;
use crate::models::work::works_query::WorksRequest;
use crate::models::work::works_query::WorksRequestBody;
use thoth_api::model::work::WorkField;
use thoth_api::model::work::WorkOrderBy;
use thoth_api::model::work::WorkWithRelations;

use super::ToElementValue;

pagination_component! {
    WorksComponent,
    WorkWithRelations,
    works,
    work_count,
    WorksRequest,
    FetchActionWorks,
    FetchWorks,
    WorksRequestBody,
    Variables,
    SEARCH_WORKS,
    PAGINATION_COUNT_WORKS,
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
