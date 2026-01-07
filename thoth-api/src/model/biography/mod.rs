use crate::model::locale::LocaleCode;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::graphql::utils::Direction;

#[cfg(feature = "backend")]
use crate::schema::biography;
#[cfg(feature = "backend")]
use crate::schema::biography_history;

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLEnum),
    graphql(description = "Field to use when sorting biography list")
)]
pub enum BiographyField {
    BiographyId,
    ContributionId,
    Content,
    Canonical,
    LocaleCode,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject),
    graphql(description = "Field and order to use when sorting biography list")
)]
pub struct BiographyOrderBy {
    pub field: BiographyField,
    pub direction: Direction,
}

impl Default for BiographyOrderBy {
    fn default() -> Self {
        Self {
            field: BiographyField::Canonical,
            direction: Direction::Desc,
        }
    }
}

#[cfg_attr(feature = "backend", derive(Queryable))]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Biography {
    pub biography_id: Uuid,
    pub contribution_id: Uuid,
    pub content: String,
    pub canonical: bool,
    pub locale_code: LocaleCode,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, Insertable, Clone),
    graphql(description = "Set of values required to define a new work's biography"),
    diesel(table_name = biography)
)]
#[derive(Default)]
pub struct NewBiography {
    pub contribution_id: Uuid,
    pub content: String,
    pub canonical: bool,
    pub locale_code: LocaleCode,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, AsChangeset, Clone),
    graphql(description = "Set of values required to update an existing work's biography"),
    diesel(table_name = biography, treat_none_as_null = true)
)]
pub struct PatchBiography {
    pub biography_id: Uuid,
    pub contribution_id: Uuid,
    pub content: String,
    pub canonical: bool,
    pub locale_code: LocaleCode,
}

#[cfg_attr(
    feature = "backend",
    derive(Insertable),
    diesel(table_name = biography_history)
)]
pub struct NewBiographyHistory {
    pub biography_id: Uuid,
    pub user_id: String,
    pub data: serde_json::Value,
}

#[cfg_attr(feature = "backend", derive(Queryable))]
pub struct BiographyHistory {
    pub biography_history_id: Uuid,
    pub biography_id: Uuid,
    pub user_id: String,
    pub data: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[cfg(feature = "backend")]
pub mod crud;
