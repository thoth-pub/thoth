use serde::Deserialize;
use serde::Serialize;

use crate::api::models::series::Series;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Issue {
    pub work_id: String,
    pub series_id: String,
    pub issue_ordinal: i32,
    pub series: Series,
}

impl Default for Issue {
    fn default() -> Issue {
        Issue {
            work_id: "".to_string(),
            series_id: "".to_string(),
            issue_ordinal: 1,
            series: Default::default(),
        }
    }
}
