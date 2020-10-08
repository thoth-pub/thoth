use serde::Deserialize;
use serde::Serialize;

use thoth_api::models::series::SeriesType;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Series {
    pub series_id: String,
    pub series_type: SeriesType,
    pub series_name: String,
    pub issn_print: String,
    pub issn_digital: String,
    pub series_url: Option<String>,
}

impl Default for Series {
    fn default() -> Series {
        Series {
            series_id: "".to_string(),
            series_type: SeriesType::BookSeries,
            series_name: "".to_string(),
            issn_print: "".to_string(),
            issn_digital: "".to_string(),
            series_url: None,
        }
    }
}
