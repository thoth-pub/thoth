use serde::Deserialize;
use serde::Serialize;
use thoth_api::model::series::Series;
use uuid::Uuid;

const DELETE_SERIES_MUTATION: &str = "
    mutation DeleteSeries(
        $seriesId: Uuid!
    ) {
        deleteSeries(
            seriesId: $seriesId
        ){
            seriesId
            seriesType
            seriesName
            issnPrint
            issnDigital
            imprintId
            createdAt
            updatedAt
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

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub series_id: Uuid,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DeleteSeriesResponseData {
    pub delete_series: Option<Series>,
}
