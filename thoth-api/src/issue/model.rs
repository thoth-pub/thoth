use chrono::naive::NaiveDateTime;
use uuid::Uuid;

#[cfg(feature = "backend")]
use crate::schema::issue;

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLEnum),
    graphql(description = "Field to use when sorting issues list")
)]
pub enum IssueField {
    SeriesID,
    WorkID,
    IssueOrdinal,
    CreatedAt,
    UpdatedAt,
}

#[cfg_attr(feature = "backend", derive(Queryable))]
pub struct Issue {
    pub series_id: Uuid,
    pub work_id: Uuid,
    pub issue_ordinal: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, Insertable),
    table_name = "issue"
)]
pub struct NewIssue {
    pub series_id: Uuid,
    pub work_id: Uuid,
    pub issue_ordinal: i32,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, AsChangeset),
    changeset_options(treat_none_as_null = "true"),
    table_name = "issue"
)]
pub struct PatchIssue {
    pub series_id: Uuid,
    pub work_id: Uuid,
    pub issue_ordinal: i32,
}
