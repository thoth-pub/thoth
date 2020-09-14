use uuid::Uuid;

#[cfg(feature = "backend")]
use crate::schema::funder;
#[cfg(feature = "backend")]
use crate::schema::funding;

#[cfg_attr(feature = "backend", derive(Queryable))]
pub struct Funder {
    pub funder_id: Uuid,
    pub funder_name: String,
    pub funder_doi: Option<String>,
}

#[cfg_attr(feature = "backend", derive(juniper::GraphQLInputObject, Insertable))]
#[cfg_attr(feature = "backend", table_name = "funder")]
pub struct NewFunder {
    pub funder_name: String,
    pub funder_doi: Option<String>,
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
}

#[cfg_attr(feature = "backend", derive(juniper::GraphQLInputObject, Insertable))]
#[cfg_attr(feature = "backend", table_name = "funding")]
pub struct NewFunding {
    pub work_id: Uuid,
    pub funder_id: Uuid,
    pub program: Option<String>,
    pub project_name: Option<String>,
    pub project_shortname: Option<String>,
    pub grant_number: Option<String>,
    pub jurisdiction: Option<String>,
}
