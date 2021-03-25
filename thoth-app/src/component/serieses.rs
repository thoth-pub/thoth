use crate::models::series::serieses_query::FetchActionSerieses;
use crate::models::series::serieses_query::FetchSerieses;
use crate::models::series::serieses_query::SeriesesRequest;
use crate::models::series::serieses_query::SeriesesRequestBody;
use crate::models::series::serieses_query::Variables;
use crate::models::series::Series;
use thoth_api::series::model::SeriesField;
use thoth_api::series::model::SeriesOrderBy;

pagination_component! {
    SeriesesComponent,
    Series,
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
        SeriesField::SeriesID.to_string(),
        SeriesField::SeriesName.to_string(),
        SeriesField::SeriesType.to_string(),
        SeriesField::ISSNPrint.to_string(),
        SeriesField::ISSNDigital.to_string(),
        SeriesField::UpdatedAt.to_string(),
    ],
    SeriesOrderBy,
    SeriesField,
}
