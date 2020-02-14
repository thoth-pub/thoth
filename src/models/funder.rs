use uuid::Uuid;
use crate::schema::funder;
use crate::schema::funding;

#[derive(Queryable)]
pub struct Funder {
    pub funder_id: Uuid,
    pub funder_name: String,
    pub funder_doi: Option<String>,
}

#[derive(juniper::GraphQLInputObject, Insertable)]
#[table_name = "funder"]
pub struct NewFunder {
    pub funder_name: String,
    pub funder_doi: Option<String>,
}

#[derive(Queryable)]
pub struct Funding {
    pub funding_id: Uuid,
    pub work_id: Uuid,
    pub funder_id: Uuid,
    pub program: Option<String>,
    pub project_name: Option<String>,
    pub project_shortname: Option<String>,
    pub grant_number: Option<String>,
    pub jurisdiction: Option<String>,
}

#[derive(juniper::GraphQLInputObject, Insertable)]
#[table_name = "funding"]
pub struct NewFunding {
    pub work_id: Uuid,
    pub funder_id: Uuid,
    pub program: Option<String>,
    pub project_name: Option<String>,
    pub project_shortname: Option<String>,
    pub grant_number: Option<String>,
    pub jurisdiction: Option<String>,
}
