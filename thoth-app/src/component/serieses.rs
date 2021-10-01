use crate::models::series::serieses_query::FetchActionSerieses;
use crate::models::series::serieses_query::FetchSerieses;
use crate::models::series::serieses_query::SeriesesRequest;
use crate::models::series::serieses_query::SeriesesRequestBody;
use crate::models::series::serieses_query::Variables;
use thoth_api::model::series::SeriesField;
use thoth_api::model::series::SeriesOrderBy;
use thoth_api::model::series::SeriesWithImprint;

pagination_component! {
    SeriesesComponent,
    SeriesWithImprint,
    serieses,
    series_count,
    SeriesesRequest,
    FetchActionSerieses,
    FetchSerieses,
    SeriesesRequestBody,
    Variables,
    SEARCH_SERIESES,
    PAGINATION_COUNT_SERIESES,
    vec![
        SeriesField::SeriesId.to_string(),
        SeriesField::SeriesName.to_string(),
        SeriesField::SeriesType.to_string(),
        SeriesField::IssnPrint.to_string(),
        SeriesField::IssnDigital.to_string(),
        SeriesField::UpdatedAt.to_string(),
    ],
    SeriesOrderBy,
    SeriesField,
}
