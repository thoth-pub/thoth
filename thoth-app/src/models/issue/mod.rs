use serde::Deserialize;
use serde::Serialize;

use super::series::Series;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Issue {
    pub issue_id: String,
    pub work_id: String,
    pub series_id: String,
    pub issue_ordinal: i32,
    pub series: Series,
}

impl Default for Issue {
    fn default() -> Issue {
        Issue {
            issue_id: "".to_string(),
            work_id: "".to_string(),
            series_id: "".to_string(),
            issue_ordinal: 1,
            series: Default::default(),
        }
    }
}

pub mod create_issue_mutation;
pub mod delete_issue_mutation;
