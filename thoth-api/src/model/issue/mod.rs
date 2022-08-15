use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::model::series::SeriesWithImprint;
use crate::model::Timestamp;
#[cfg(feature = "backend")]
use crate::schema::issue;
#[cfg(feature = "backend")]
use crate::schema::issue_history;

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLEnum),
    graphql(description = "Field to use when sorting issues list")
)]
pub enum IssueField {
    IssueId,
    SeriesId,
    WorkId,
    IssueOrdinal,
    CreatedAt,
    UpdatedAt,
}

#[cfg_attr(feature = "backend", derive(Queryable))]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Issue {
    pub issue_id: Uuid,
    pub series_id: Uuid,
    pub work_id: Uuid,
    pub issue_ordinal: i32,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct IssueWithSeries {
    pub issue_id: Uuid,
    pub work_id: Uuid,
    pub series_id: Uuid,
    pub issue_ordinal: i32,
    pub series: SeriesWithImprint,
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
    pub issue_id: Uuid,
    pub series_id: Uuid,
    pub work_id: Uuid,
    pub issue_ordinal: i32,
}

#[cfg_attr(feature = "backend", derive(Queryable))]
pub struct IssueHistory {
    pub issue_history_id: Uuid,
    pub issue_id: Uuid,
    pub account_id: Uuid,
    pub data: serde_json::Value,
    pub timestamp: Timestamp,
}

#[cfg_attr(feature = "backend", derive(Insertable), table_name = "issue_history")]
pub struct NewIssueHistory {
    pub issue_id: Uuid,
    pub account_id: Uuid,
    pub data: serde_json::Value,
}

impl Default for IssueWithSeries {
    fn default() -> IssueWithSeries {
        IssueWithSeries {
            issue_id: Default::default(),
            work_id: Default::default(),
            series_id: Default::default(),
            issue_ordinal: 1,
            series: Default::default(),
        }
    }
}

#[cfg(feature = "backend")]
pub mod crud;
