use crate::models::work::works_query::FetchActionWorks;
use crate::models::work::works_query::FetchWorks;
use crate::models::work::works_query::Variables;
use crate::models::work::works_query::WorksRequest;
use crate::models::work::works_query::WorksRequestBody;
use crate::models::work::Work;
use thoth_api::work::model::WorkField;
use thoth_api::work::model::WorkOrderBy;

pagination_component! {
    WorksComponent,
    Work,
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
        WorkField::WorkID.to_string(),
        WorkField::FullTitle.to_string(),
        WorkField::WorkType.to_string(),
        "Contributors".to_string(),
        WorkField::DOI.to_string(),
        "Publisher".to_string(),
        WorkField::UpdatedAt.to_string(),
    ],
    WorkOrderBy,
    WorkField,
}
