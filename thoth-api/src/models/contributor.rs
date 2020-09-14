use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(feature = "backend")]
use crate::schema::contribution;
#[cfg(feature = "backend")]
use crate::schema::contributor;

#[cfg_attr(feature = "backend", derive(DbEnum, juniper::GraphQLEnum))]
#[cfg_attr(feature = "backend", DieselType = "Contribution_type")]
#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub enum ContributionType {
    Author,
    Editor,
    Translator,
    Photographer,
    Ilustrator,
    #[cfg_attr(feature = "backend", db_rename = "music-editor")]
    MusicEditor,
    #[cfg_attr(feature = "backend", db_rename = "foreword-by")]
    ForewordBy,
    #[cfg_attr(feature = "backend", db_rename = "introduction-by")]
    IntroductionBy,
    #[cfg_attr(feature = "backend", db_rename = "afterword-by")]
    AfterwordBy,
    #[cfg_attr(feature = "backend", db_rename = "preface-by")]
    PrefaceBy,
}

#[cfg_attr(feature = "backend", derive(Queryable))]
pub struct Contributor {
    pub contributor_id: Uuid,
    pub first_name: Option<String>,
    pub last_name: String,
    pub full_name: String,
    pub orcid: Option<String>,
    pub website: Option<String>,
}

#[cfg_attr(feature = "backend", derive(juniper::GraphQLInputObject, Insertable))]
#[cfg_attr(feature = "backend", table_name = "contributor")]
pub struct NewContributor {
    pub first_name: Option<String>,
    pub last_name: String,
    pub full_name: String,
    pub orcid: Option<String>,
    pub website: Option<String>,
}

#[cfg_attr(feature = "backend", derive(Queryable))]
pub struct Contribution {
    pub work_id: Uuid,
    pub contributor_id: Uuid,
    pub contribution_type: ContributionType,
    pub main_contribution: bool,
    pub biography: Option<String>,
    pub institution: Option<String>,
}

#[cfg_attr(feature = "backend", derive(juniper::GraphQLInputObject, Insertable))]
#[cfg_attr(feature = "backend", table_name = "contribution")]
pub struct NewContribution {
    pub work_id: Uuid,
    pub contributor_id: Uuid,
    pub contribution_type: ContributionType,
    pub main_contribution: bool,
    pub biography: Option<String>,
    pub institution: Option<String>,
}
