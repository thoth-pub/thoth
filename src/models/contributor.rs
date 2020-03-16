use crate::schema::contribution;
use crate::schema::contributor;
use uuid::Uuid;

#[derive(Debug, PartialEq, DbEnum, juniper::GraphQLEnum)]
#[DieselType = "Contribution_type"]
pub enum ContributionType {
    Author,
    Editor,
    Translator,
    Photographer,
    Ilustrator,
    #[db_rename = "music-editor"]
    MusicEditor,
    #[db_rename = "foreword-by"]
    ForewordBy,
    #[db_rename = "introduction-by"]
    IntroductionBy,
    #[db_rename = "afterword-by"]
    AfterwordBy,
    #[db_rename = "preface-by"]
    PrefaceBy,
}

#[derive(Queryable)]
pub struct Contributor {
    pub contributor_id: Uuid,
    pub first_name: Option<String>,
    pub last_name: String,
    pub full_name: String,
    pub orcid: Option<String>,
    pub website: Option<String>,
}

#[derive(juniper::GraphQLInputObject, Insertable)]
#[table_name = "contributor"]
pub struct NewContributor {
    pub first_name: Option<String>,
    pub last_name: String,
    pub full_name: String,
    pub orcid: Option<String>,
    pub website: Option<String>,
}

#[derive(Queryable)]
pub struct Contribution {
    pub work_id: Uuid,
    pub contributor_id: Uuid,
    pub contribution_type: ContributionType,
    pub main_contribution: bool,
    pub biography: Option<String>,
    pub institution: Option<String>,
}

#[derive(juniper::GraphQLInputObject, Insertable)]
#[table_name = "contribution"]
pub struct NewContribution {
    pub work_id: Uuid,
    pub contributor_id: Uuid,
    pub contribution_type: ContributionType,
    pub main_contribution: bool,
    pub biography: Option<String>,
    pub institution: Option<String>,
}
