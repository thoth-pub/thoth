use crate::models::series::serieses_query::FetchActionSerieses;
use crate::models::series::serieses_query::FetchSerieses;
use crate::models::series::serieses_query::SeriesesRequest;
use crate::models::series::serieses_query::SeriesesRequestBody;
use crate::models::series::serieses_query::Variables;
use crate::models::series::Series;

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
    vec!["ID".to_string(), "Series".to_string(), "SeriesType".to_string(), "ISSNPrint".to_string(), "ISSNDigital".to_string()]
}
