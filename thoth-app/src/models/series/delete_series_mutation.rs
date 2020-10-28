use serde::Deserialize;
use serde::Serialize;
use thoth_api::series::model::SeriesType;

const DELETE_SERIES_MUTATION: &str = "
    mutation DeleteSeries(
            $seriesId: Uuid!
    ) {
        deleteSeries(
            seriesId: $seriesId
        ){
            seriesId
            seriesName
        }
    }
";

graphql_query_builder! {
    DeleteSeriesRequest,
    DeleteSeriesRequestBody,
    Variables,
    DELETE_SERIES_MUTATION,
    DeleteSeriesResponseBody,
    DeleteSeriesResponseData,
    PushDeleteSeries,
    PushActionDeleteSeries
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub series_id: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SlimSeries {
    pub series_id: String,
    pub series_name: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DeleteSeriesResponseData {
    pub delete_series: Option<SlimSeries>,
}
