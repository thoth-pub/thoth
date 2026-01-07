use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::model::Timestamp;
#[cfg(feature = "backend")]
use crate::schema::funding;
#[cfg(feature = "backend")]
use crate::schema::funding_history;

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLEnum),
    graphql(description = "Field to use when sorting fundings list")
)]
pub enum FundingField {
    InstitutionId,
    WorkId,
    FundingId,
    Program,
    ProjectName,
    ProjectShortname,
    GrantNumber,
    Jurisdiction,
    CreatedAt,
    UpdatedAt,
}

#[cfg_attr(feature = "backend", derive(Queryable))]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Funding {
    pub funding_id: Uuid,
    pub work_id: Uuid,
    pub institution_id: Uuid,
    pub program: Option<String>,
    pub project_name: Option<String>,
    pub project_shortname: Option<String>,
    pub grant_number: Option<String>,
    pub jurisdiction: Option<String>,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, Insertable),
    graphql(description = "Set of values required to define a new grant awarded for the publication of a work by an institution"),
    diesel(table_name = funding)
)]
pub struct NewFunding {
    pub work_id: Uuid,
    pub institution_id: Uuid,
    pub program: Option<String>,
    pub project_name: Option<String>,
    pub project_shortname: Option<String>,
    pub grant_number: Option<String>,
    pub jurisdiction: Option<String>,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, AsChangeset),
    graphql(description = "Set of values required to update an existing grant awarded for the publication of a work by an institution"),
    diesel(table_name = funding, treat_none_as_null = true)
)]
pub struct PatchFunding {
    pub funding_id: Uuid,
    pub work_id: Uuid,
    pub institution_id: Uuid,
    pub program: Option<String>,
    pub project_name: Option<String>,
    pub project_shortname: Option<String>,
    pub grant_number: Option<String>,
    pub jurisdiction: Option<String>,
}

#[cfg_attr(feature = "backend", derive(Queryable))]
pub struct FundingHistory {
    pub funding_history_id: Uuid,
    pub funding_id: Uuid,
    pub user_id: String,
    pub data: serde_json::Value,
    pub timestamp: Timestamp,
}

#[cfg_attr(
    feature = "backend",
    derive(Insertable),
    diesel(table_name = funding_history)
)]
pub struct NewFundingHistory {
    pub funding_id: Uuid,
    pub user_id: String,
    pub data: serde_json::Value,
}

#[cfg(feature = "backend")]
pub mod crud;
