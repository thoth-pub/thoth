use crate::models::work::works_query::FetchWorks;
use crate::models::work::works_query::WorksRequestBody;
use crate::models::work::works_query::Variables;
use crate::models::work::works_query::WorksRequest;
use crate::models::work::works_query::FetchActionWorks;
use crate::models::work::Work;

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
    NewWork,
    SEARCH_WORKS,
    PAGINATION_COUNT_WORKS,
    vec!["ID".to_string(), "Title".to_string(), "Type".to_string(), "Contributors".to_string(), "DOI".to_string(), "Publisher".to_string()]
}
