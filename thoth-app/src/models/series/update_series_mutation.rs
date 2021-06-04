use serde::Deserialize;
use serde::Serialize;
use thoth_api::series::model::SeriesType;
use thoth_api::series::model::SlimSeries;
use uuid::Uuid;

const UPDATE_SERIES_MUTATION: &str = "
    mutation UpdateSeries(
            $seriesId: Uuid!,
            $seriesType: SeriesType!,
            $seriesName: String!,
            $issnPrint: String!,
            $issnDigital: String!,
            $seriesUrl: String,
            $imprintId: Uuid!
    ) {
        updateSeries(data: {
            seriesId: $seriesId
            seriesType: $seriesType
            seriesName: $seriesName
            issnPrint: $issnPrint
            issnDigital: $issnDigital
            seriesUrl: $seriesUrl
            imprintId: $imprintId
        }){
            seriesId
            seriesName
        }
    }
";

graphql_query_builder! {
    UpdateSeriesRequest,
    UpdateSeriesRequestBody,
    Variables,
    UPDATE_SERIES_MUTATION,
    UpdateSeriesResponseBody,
    UpdateSeriesResponseData,
    PushUpdateSeries,
    PushActionUpdateSeries
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub series_id: Uuid,
    pub series_type: SeriesType,
    pub series_name: String,
    pub issn_print: String,
    pub issn_digital: String,
    pub series_url: Option<String>,
    pub imprint_id: Uuid,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSeriesResponseData {
    pub update_series: Option<SlimSeries>,
}
