use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::graphql::utils::Direction;
use crate::model::{Doi, Isbn, Timestamp};
#[cfg(feature = "backend")]
use crate::schema::reference;
#[cfg(feature = "backend")]
use crate::schema::reference_history;

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLEnum),
    graphql(description = "Field to use when sorting references list")
)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReferenceField {
    ReferenceId,
    WorkId,
    ReferenceOrdinal,
    Doi,
    UnstructuredCitation,
    Issn,
    Isbn,
    JournalTitle,
    ArticleTitle,
    SeriesTitle,
    VolumeTitle,
    Edition,
    Author,
    Volume,
    Issue,
    FirstPage,
    ComponentNumber,
    StandardDesignator,
    StandardsBodyName,
    StandardsBodyAcronym,
    Url,
    PublicationDate,
    RetrievalDate,
    CreatedAt,
    UpdatedAt,
}

#[cfg_attr(feature = "backend", derive(Queryable))]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Reference {
    pub reference_id: Uuid,
    pub work_id: Uuid,
    pub reference_ordinal: i32,
    pub doi: Option<Doi>,
    pub unstructured_citation: Option<String>,
    pub issn: Option<String>,
    pub isbn: Option<Isbn>,
    pub journal_title: Option<String>,
    pub article_title: Option<String>,
    pub series_title: Option<String>,
    pub volume_title: Option<String>,
    pub edition: Option<i32>,
    pub author: Option<String>,
    pub volume: Option<String>,
    pub issue: Option<String>,
    pub first_page: Option<String>,
    pub component_number: Option<String>,
    pub standard_designator: Option<String>,
    pub standards_body_name: Option<String>,
    pub standards_body_acronym: Option<String>,
    pub url: Option<String>,
    pub publication_date: Option<NaiveDate>,
    pub retrieval_date: Option<NaiveDate>,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, Insertable),
    diesel(table_name = reference)
)]
pub struct NewReference {
    pub work_id: Uuid,
    pub reference_ordinal: i32,
    pub doi: Option<Doi>,
    pub unstructured_citation: Option<String>,
    pub issn: Option<String>,
    pub isbn: Option<Isbn>,
    pub journal_title: Option<String>,
    pub article_title: Option<String>,
    pub series_title: Option<String>,
    pub volume_title: Option<String>,
    pub edition: Option<i32>,
    pub author: Option<String>,
    pub volume: Option<String>,
    pub issue: Option<String>,
    pub first_page: Option<String>,
    pub component_number: Option<String>,
    pub standard_designator: Option<String>,
    pub standards_body_name: Option<String>,
    pub standards_body_acronym: Option<String>,
    pub url: Option<String>,
    pub publication_date: Option<NaiveDate>,
    pub retrieval_date: Option<NaiveDate>,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, AsChangeset),
    diesel(table_name = reference, treat_none_as_null = true)
)]
pub struct PatchReference {
    pub reference_id: Uuid,
    pub work_id: Uuid,
    pub reference_ordinal: i32,
    pub doi: Option<Doi>,
    pub unstructured_citation: Option<String>,
    pub issn: Option<String>,
    pub isbn: Option<Isbn>,
    pub journal_title: Option<String>,
    pub article_title: Option<String>,
    pub series_title: Option<String>,
    pub volume_title: Option<String>,
    pub edition: Option<i32>,
    pub author: Option<String>,
    pub volume: Option<String>,
    pub issue: Option<String>,
    pub first_page: Option<String>,
    pub component_number: Option<String>,
    pub standard_designator: Option<String>,
    pub standards_body_name: Option<String>,
    pub standards_body_acronym: Option<String>,
    pub url: Option<String>,
    pub publication_date: Option<NaiveDate>,
    pub retrieval_date: Option<NaiveDate>,
}

#[cfg_attr(feature = "backend", derive(Queryable))]
pub struct ReferenceHistory {
    pub reference_history_id: Uuid,
    pub reference_id: Uuid,
    pub account_id: Uuid,
    pub data: serde_json::Value,
    pub timestamp: Timestamp,
}

#[cfg_attr(
    feature = "backend",
    derive(Insertable),
    diesel(table_name = reference_history)
)]
pub struct NewReferenceHistory {
    pub reference_id: Uuid,
    pub account_id: Uuid,
    pub data: serde_json::Value,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject),
    graphql(description = "Field and order to use when sorting references list")
)]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReferenceOrderBy {
    pub field: ReferenceField,
    pub direction: Direction,
}

impl Default for ReferenceField {
    fn default() -> Self {
        ReferenceField::ReferenceOrdinal
    }
}

#[test]
fn test_referencefield_default() {
    let reffield: ReferenceField = Default::default();
    assert_eq!(reffield, ReferenceField::ReferenceOrdinal);
}

#[cfg(feature = "backend")]
pub mod crud;
