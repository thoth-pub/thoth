use chrono::naive::NaiveDateTime;
use uuid::Uuid;

#[cfg(feature = "backend")]
use crate::schema::funding;

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLEnum),
    graphql(description = "Field to use when sorting fundings list")
)]
pub enum FundingField {
    FundingID,
    WorkID,
    FunderID,
    Program,
    ProjectName,
    ProjectShortname,
    GrantNumber,
    Jurisdiction,
    CreatedAt,
    UpdatedAt,
}

#[cfg_attr(feature = "backend", derive(Queryable))]
pub struct Funding {
    pub funding_id: Uuid,
    pub work_id: Uuid,
    pub funder_id: Uuid,
    pub program: Option<String>,
    pub project_name: Option<String>,
    pub project_shortname: Option<String>,
    pub grant_number: Option<String>,
    pub jurisdiction: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, Insertable),
    table_name = "funding"
)]
pub struct NewFunding {
    pub work_id: Uuid,
    pub funder_id: Uuid,
    pub program: Option<String>,
    pub project_name: Option<String>,
    pub project_shortname: Option<String>,
    pub grant_number: Option<String>,
    pub jurisdiction: Option<String>,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, AsChangeset),
    changeset_options(treat_none_as_null = "true"),
    table_name = "funding"
)]
pub struct PatchFunding {
    pub funding_id: Uuid,
    pub work_id: Uuid,
    pub funder_id: Uuid,
    pub program: Option<String>,
    pub project_name: Option<String>,
    pub project_shortname: Option<String>,
    pub grant_number: Option<String>,
    pub jurisdiction: Option<String>,
}
