use serde::{Deserialize, Serialize};
use std::fmt;
use strum::{Display, EnumString};
use uuid::Uuid;

use crate::graphql::types::inputs::Direction;
use crate::model::{CountryCode, Doi, Ror, Timestamp};
#[cfg(feature = "backend")]
use crate::schema::institution;
#[cfg(feature = "backend")]
use crate::schema::institution_history;

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLEnum),
    graphql(description = "Field to use when sorting institutions list")
)]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, EnumString, Display)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InstitutionField {
    #[strum(serialize = "ID")]
    InstitutionId,
    #[strum(serialize = "Institution")]
    #[default]
    InstitutionName,
    #[strum(serialize = "DOI")]
    InstitutionDoi,
    #[strum(serialize = "ROR ID")]
    Ror,
    #[strum(serialize = "Country")]
    CountryCode,
    CreatedAt,
    UpdatedAt,
}

#[cfg_attr(feature = "backend", derive(diesel::Queryable))]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Institution {
    pub institution_id: Uuid,
    pub institution_name: String,
    pub institution_doi: Option<Doi>,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub ror: Option<Ror>,
    pub country_code: Option<CountryCode>,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, diesel::Insertable),
    graphql(description = "Set of values required to define a new organisation with which contributors may be affiliated or by which works may be funded"),
    diesel(table_name = institution)
)]
pub struct NewInstitution {
    pub institution_name: String,
    pub institution_doi: Option<Doi>,
    pub ror: Option<Ror>,
    pub country_code: Option<CountryCode>,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, diesel::AsChangeset),
    graphql(description = "Set of values required to update an existing organisation with which contributors may be affiliated or by which works may be funded"),
    diesel(table_name = institution, treat_none_as_null = true)
)]
pub struct PatchInstitution {
    pub institution_id: Uuid,
    pub institution_name: String,
    pub institution_doi: Option<Doi>,
    pub ror: Option<Ror>,
    pub country_code: Option<CountryCode>,
}

#[cfg_attr(feature = "backend", derive(diesel::Queryable))]
pub struct InstitutionHistory {
    pub institution_history_id: Uuid,
    pub institution_id: Uuid,
    pub user_id: String,
    pub data: serde_json::Value,
    pub timestamp: Timestamp,
}

#[cfg_attr(
    feature = "backend",
    derive(diesel::Insertable),
    diesel(table_name = institution_history)
)]
pub struct NewInstitutionHistory {
    pub institution_id: Uuid,
    pub user_id: String,
    pub data: serde_json::Value,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject),
    graphql(description = "Field and order to use when sorting institutions list")
)]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct InstitutionOrderBy {
    pub field: InstitutionField,
    pub direction: Direction,
}

impl fmt::Display for Institution {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ror) = &self.ror {
            write!(f, "{} - {}", &self.institution_name, ror)
        } else if let Some(doi) = &self.institution_doi {
            write!(f, "{} - {}", &self.institution_name, doi)
        } else {
            write!(f, "{}", &self.institution_name)
        }
    }
}

#[cfg(feature = "backend")]
pub mod crud;
#[cfg(feature = "backend")]
mod policy;
#[cfg(feature = "backend")]
pub(crate) use policy::InstitutionPolicy;
#[cfg(test)]
mod tests;
