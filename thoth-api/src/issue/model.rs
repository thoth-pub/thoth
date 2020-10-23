use uuid::Uuid;

#[cfg(feature = "backend")]
use crate::schema::issue;

#[cfg_attr(feature = "backend", derive(Queryable))]
pub struct Issue {
    pub series_id: Uuid,
    pub work_id: Uuid,
    pub issue_ordinal: i32,
}

#[cfg_attr(feature = "backend", derive(juniper::GraphQLInputObject, Insertable))]
#[cfg_attr(feature = "backend", table_name = "issue")]
pub struct NewIssue {
    pub series_id: Uuid,
    pub work_id: Uuid,
    pub issue_ordinal: i32,
}
