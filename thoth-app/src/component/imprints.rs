use crate::models::imprint::imprints_query::FetchActionImprints;
use crate::models::imprint::imprints_query::FetchImprints;
use crate::models::imprint::imprints_query::ImprintsRequest;
use crate::models::imprint::imprints_query::ImprintsRequestBody;
use crate::models::imprint::imprints_query::Variables;
use thoth_api::model::imprint::ImprintField;
use thoth_api::model::imprint::ImprintOrderBy;
use thoth_api::model::imprint::ImprintWithPublisher;

pagination_component! {
    ImprintsComponent,
    ImprintWithPublisher,
    imprints,
    imprint_count,
    ImprintsRequest,
    FetchActionImprints,
    FetchImprints,
    ImprintsRequestBody,
    Variables,
    SEARCH_IMPRINTS,
    PAGINATION_COUNT_IMPRINTS,
    vec![
        ImprintField::ImprintId.to_string(),
        ImprintField::ImprintName.to_string(),
        "Publisher".to_string(),
        ImprintField::ImprintUrl.to_string(),
        ImprintField::UpdatedAt.to_string(),
    ],
    ImprintOrderBy,
    ImprintField,
}
