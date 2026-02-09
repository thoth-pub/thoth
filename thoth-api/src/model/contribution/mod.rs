use serde::{Deserialize, Serialize};
use strum::Display;
use strum::EnumString;
use uuid::Uuid;

use crate::model::Timestamp;
#[cfg(feature = "backend")]
use crate::schema::contribution;
#[cfg(feature = "backend")]
use crate::schema::contribution_history;

#[cfg_attr(
    feature = "backend",
    derive(DbEnum, juniper::GraphQLEnum),
    graphql(description = "Role describing the type of contribution to the work"),
    ExistingTypePath = "crate::schema::sql_types::ContributionType"
)]
#[derive(
    Debug, Clone, Default, Copy, PartialEq, Eq, Deserialize, Serialize, EnumString, Display,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "title_case")]
pub enum ContributionType {
    #[cfg_attr(feature = "backend", graphql(description = "Author of the work"))]
    #[default]
    Author,
    #[cfg_attr(feature = "backend", graphql(description = "Editor of the work"))]
    Editor,
    #[cfg_attr(feature = "backend", graphql(description = "Translator of the work"))]
    Translator,
    #[cfg_attr(
        feature = "backend",
        graphql(
            description = "Photographer when named as the primary creator of, eg, a book of photographs"
        )
    )]
    Photographer,
    #[cfg_attr(
        feature = "backend",
        graphql(
            description = "Artist when named as the creator of artwork which illustrates a work"
        )
    )]
    Illustrator,
    #[cfg_attr(
        feature = "backend",
        db_rename = "music-editor",
        graphql(
            description = "Person responsible for editing any piece of music referenced in the work"
        )
    )]
    MusicEditor,
    #[cfg_attr(
        feature = "backend",
        db_rename = "foreword-by",
        graphql(description = "Author of foreword")
    )]
    ForewordBy,
    #[cfg_attr(
        feature = "backend",
        db_rename = "introduction-by",
        graphql(description = "Author of introduction")
    )]
    IntroductionBy,
    #[cfg_attr(
        feature = "backend",
        db_rename = "afterword-by",
        graphql(description = "Author of afterword")
    )]
    AfterwordBy,
    #[cfg_attr(
        feature = "backend",
        db_rename = "preface-by",
        graphql(description = "Author of preface")
    )]
    PrefaceBy,
    #[cfg_attr(
        feature = "backend",
        db_rename = "software-by",
        graphql(description = "Writer of computer programs ancillary to the work")
    )]
    SoftwareBy,
    #[cfg_attr(
        feature = "backend",
        db_rename = "research-by",
        graphql(
            description = "Person responsible for performing research on which the work is based"
        )
    )]
    ResearchBy,
    #[cfg_attr(
        feature = "backend",
        db_rename = "contributions-by",
        graphql(description = "Author of additional contributions to the work")
    )]
    ContributionsBy,
    #[cfg_attr(feature = "backend", graphql(description = "Compiler of index"))]
    Indexer,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLEnum),
    graphql(description = "Field to use when sorting contributions list")
)]
pub enum ContributionField {
    ContributionId,
    WorkId,
    ContributorId,
    ContributionType,
    MainContribution,
    Biography,
    CreatedAt,
    UpdatedAt,
    FirstName,
    LastName,
    FullName,
    ContributionOrdinal,
}

#[cfg_attr(feature = "backend", derive(Queryable))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Contribution {
    pub contribution_id: Uuid,
    pub work_id: Uuid,
    pub contributor_id: Uuid,
    pub contribution_type: ContributionType,
    pub main_contribution: bool,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub first_name: Option<String>,
    pub last_name: String,
    pub full_name: String,
    pub contribution_ordinal: i32,
}
#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, Insertable),
    graphql(description = "Set of values required to define a new individual involvement in the production of a work"),
    diesel(table_name = contribution)
)]
pub struct NewContribution {
    pub work_id: Uuid,
    pub contributor_id: Uuid,
    pub contribution_type: ContributionType,
    pub main_contribution: bool,
    pub first_name: Option<String>,
    pub last_name: String,
    pub full_name: String,
    pub contribution_ordinal: i32,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, AsChangeset),
    graphql(description = "Set of values required to update an individual involvement in the production of a work"),
    diesel(table_name = contribution, treat_none_as_null = true)
)]
pub struct PatchContribution {
    pub contribution_id: Uuid,
    pub work_id: Uuid,
    pub contributor_id: Uuid,
    pub contribution_type: ContributionType,
    pub main_contribution: bool,
    pub first_name: Option<String>,
    pub last_name: String,
    pub full_name: String,
    pub contribution_ordinal: i32,
}

#[cfg_attr(feature = "backend", derive(Queryable))]
pub struct ContributionHistory {
    pub contribution_history_id: Uuid,
    pub contribution_id: Uuid,
    pub user_id: String,
    pub data: serde_json::Value,
    pub timestamp: Timestamp,
}

#[cfg_attr(
    feature = "backend",
    derive(Insertable),
    diesel(table_name = contribution_history)
)]
pub struct NewContributionHistory {
    pub contribution_id: Uuid,
    pub user_id: String,
    pub data: serde_json::Value,
}

impl Default for Contribution {
    fn default() -> Contribution {
        Contribution {
            contribution_id: Default::default(),
            work_id: Default::default(),
            contributor_id: Default::default(),
            contribution_type: Default::default(),
            main_contribution: true,
            created_at: Default::default(),
            updated_at: Default::default(),
            first_name: Default::default(),
            last_name: Default::default(),
            full_name: Default::default(),
            contribution_ordinal: 1,
        }
    }
}

#[cfg(feature = "backend")]
pub mod crud;
#[cfg(feature = "backend")]
mod policy;
#[cfg(feature = "backend")]
pub(crate) use policy::ContributionPolicy;
#[cfg(test)]
mod tests;
