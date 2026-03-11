use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::graphql::types::inputs::Direction;
use crate::model::Doi;
use crate::model::Timestamp;
#[cfg(feature = "backend")]
use crate::schema::book_review;
#[cfg(feature = "backend")]
use crate::schema::book_review_history;

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLEnum),
    graphql(description = "Field to use when sorting book reviews list")
)]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BookReviewField {
    BookReviewId,
    WorkId,
    #[default]
    ReviewOrdinal,
    Title,
    AuthorName,
    JournalName,
    ReviewDate,
    CreatedAt,
    UpdatedAt,
}

#[cfg_attr(feature = "backend", derive(diesel::Queryable))]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BookReview {
    pub book_review_id: Uuid,
    pub work_id: Uuid,
    pub title: Option<String>,
    pub author_name: Option<String>,
    pub url: Option<String>,
    pub doi: Option<Doi>,
    pub review_date: Option<NaiveDate>,
    pub journal_name: Option<String>,
    pub journal_volume: Option<String>,
    pub journal_number: Option<String>,
    pub journal_issn: Option<String>,
    pub text: Option<String>,
    pub review_ordinal: i32,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, diesel::Insertable),
    graphql(description = "Set of values required to define a new book review linked to a work"),
    diesel(table_name = book_review)
)]
pub struct NewBookReview {
    pub work_id: Uuid,
    pub title: Option<String>,
    pub author_name: Option<String>,
    pub url: Option<String>,
    pub doi: Option<Doi>,
    pub review_date: Option<NaiveDate>,
    pub journal_name: Option<String>,
    pub journal_volume: Option<String>,
    pub journal_number: Option<String>,
    pub journal_issn: Option<String>,
    pub text: Option<String>,
    pub review_ordinal: i32,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, diesel::AsChangeset),
    graphql(description = "Set of values required to update an existing book review"),
    diesel(table_name = book_review, treat_none_as_null = true)
)]
pub struct PatchBookReview {
    pub book_review_id: Uuid,
    pub work_id: Uuid,
    pub title: Option<String>,
    pub author_name: Option<String>,
    pub url: Option<String>,
    pub doi: Option<Doi>,
    pub review_date: Option<NaiveDate>,
    pub journal_name: Option<String>,
    pub journal_volume: Option<String>,
    pub journal_number: Option<String>,
    pub journal_issn: Option<String>,
    pub text: Option<String>,
    pub review_ordinal: i32,
}

#[cfg_attr(feature = "backend", derive(diesel::Queryable))]
pub struct BookReviewHistory {
    pub book_review_history_id: Uuid,
    pub book_review_id: Uuid,
    pub user_id: String,
    pub data: serde_json::Value,
    pub timestamp: Timestamp,
}

#[cfg_attr(
    feature = "backend",
    derive(diesel::Insertable),
    diesel(table_name = book_review_history)
)]
pub struct NewBookReviewHistory {
    pub book_review_id: Uuid,
    pub user_id: String,
    pub data: serde_json::Value,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject),
    graphql(description = "Field and order to use when sorting book reviews list")
)]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct BookReviewOrderBy {
    pub field: BookReviewField,
    pub direction: Direction,
}

#[cfg(feature = "backend")]
pub mod crud;
#[cfg(feature = "backend")]
mod policy;
#[cfg(feature = "backend")]
pub(crate) use policy::BookReviewPolicy;
#[cfg(test)]
mod tests;
