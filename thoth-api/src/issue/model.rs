use chrono::DateTime;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(feature = "backend")]
use crate::schema::issue;
#[cfg(feature = "backend")]
use crate::schema::issue_history;
use crate::series::model::SeriesExtended as Series;

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLEnum),
    graphql(description = "Field to use when sorting issues list")
)]
pub enum IssueField {
    SeriesId,
    WorkId,
    IssueOrdinal,
    CreatedAt,
    UpdatedAt,
}

#[cfg_attr(feature = "backend", derive(Queryable))]
#[derive(Serialize, Deserialize)]
pub struct Issue {
    pub series_id: Uuid,
    pub work_id: Uuid,
    pub issue_ordinal: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct IssueExtended {
    pub work_id: Uuid,
    pub series_id: Uuid,
    pub issue_ordinal: i32,
    pub series: Series,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, Insertable),
    table_name = "issue"
)]
pub struct NewIssue {
    pub series_id: Uuid,
    pub work_id: Uuid,
    pub issue_ordinal: i32,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, AsChangeset),
    changeset_options(treat_none_as_null = "true"),
    table_name = "issue"
)]
pub struct PatchIssue {
    pub series_id: Uuid,
    pub work_id: Uuid,
    pub issue_ordinal: i32,
}

#[cfg_attr(feature = "backend", derive(Queryable))]
pub struct IssueHistory {
    pub issue_history_id: Uuid,
    pub series_id: Uuid,
    pub work_id: Uuid,
    pub account_id: Uuid,
    pub data: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

#[cfg_attr(feature = "backend", derive(Insertable), table_name = "issue_history")]
pub struct NewIssueHistory {
    pub series_id: Uuid,
    pub work_id: Uuid,
    pub account_id: Uuid,
    pub data: serde_json::Value,
}

impl Default for IssueExtended {
    fn default() -> IssueExtended {
        IssueExtended {
            work_id: Default::default(),
            series_id: Default::default(),
            issue_ordinal: 1,
            series: Default::default(),
        }
    }
}
