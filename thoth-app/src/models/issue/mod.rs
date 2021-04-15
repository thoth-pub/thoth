use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

use super::series::Series;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Issue {
    pub work_id: Uuid,
    pub series_id: Uuid,
    pub issue_ordinal: i32,
    pub series: Series,
}

impl Default for Issue {
    fn default() -> Issue {
        Issue {
            work_id: Default::default(),
            series_id: Default::default(),
            issue_ordinal: 1,
            series: Default::default(),
        }
    }
}

pub mod create_issue_mutation;
pub mod delete_issue_mutation;
