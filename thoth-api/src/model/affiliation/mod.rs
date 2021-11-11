use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::graphql::utils::Direction;
use crate::model::contribution::ContributionWithWork;
use crate::model::institution::Institution;
use crate::model::Timestamp;
#[cfg(feature = "backend")]
use crate::schema::affiliation;
#[cfg(feature = "backend")]
use crate::schema::affiliation_history;

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLEnum),
    graphql(description = "Field to use when sorting affiliations list")
)]
pub enum AffiliationField {
    AffiliationId,
    ContributionId,
    InstitutionId,
    AffiliationOrdinal,
    Position,
    CreatedAt,
    UpdatedAt,
}

#[cfg_attr(feature = "backend", derive(Queryable))]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Affiliation {
    pub affiliation_id: Uuid,
    pub contribution_id: Uuid,
    pub institution_id: Uuid,
    pub affiliation_ordinal: i32,
    pub position: Option<String>,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AffiliationWithInstitution {
    pub affiliation_id: Uuid,
    pub contribution_id: Uuid,
    pub institution_id: Uuid,
    pub affiliation_ordinal: i32,
    pub position: Option<String>,
    pub institution: Institution,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AffiliationWithContribution {
    pub contribution: ContributionWithWork,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, Insertable),
    table_name = "affiliation"
)]
pub struct NewAffiliation {
    pub contribution_id: Uuid,
    pub institution_id: Uuid,
    pub affiliation_ordinal: i32,
    pub position: Option<String>,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, AsChangeset),
    changeset_options(treat_none_as_null = "true"),
    table_name = "affiliation"
)]
pub struct PatchAffiliation {
    pub affiliation_id: Uuid,
    pub contribution_id: Uuid,
    pub institution_id: Uuid,
    pub affiliation_ordinal: i32,
    pub position: Option<String>,
}

#[cfg_attr(feature = "backend", derive(Queryable))]
pub struct AffiliationHistory {
    pub affiliation_history_id: Uuid,
    pub affiliation_id: Uuid,
    pub account_id: Uuid,
    pub data: serde_json::Value,
    pub timestamp: Timestamp,
}

#[cfg_attr(
    feature = "backend",
    derive(Insertable),
    table_name = "affiliation_history"
)]
pub struct NewAffiliationHistory {
    pub affiliation_id: Uuid,
    pub account_id: Uuid,
    pub data: serde_json::Value,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject),
    graphql(description = "Field and order to use when sorting affiliations list")
)]
pub struct AffiliationOrderBy {
    pub field: AffiliationField,
    pub direction: Direction,
}

impl Default for AffiliationWithInstitution {
    fn default() -> AffiliationWithInstitution {
        AffiliationWithInstitution {
            affiliation_id: Default::default(),
            institution_id: Default::default(),
            contribution_id: Default::default(),
            affiliation_ordinal: 1,
            position: Default::default(),
            institution: Default::default(),
        }
    }
}

#[cfg(feature = "backend")]
pub mod crud;
