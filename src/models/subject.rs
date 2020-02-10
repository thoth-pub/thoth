use uuid::Uuid;

#[derive(Debug, PartialEq, DbEnum)]
#[derive(juniper::GraphQLEnum)]
#[DieselType = "Subject_type"]
pub enum SubjectType {
    Bic,
    Bisac,
    Thema,
    Custom,
    Keyword,
}

#[derive(Queryable)]
pub struct Subject {
    pub subject_id: Uuid,
    pub work_id: Uuid,
    pub subject_type: SubjectType,
    pub subject_code: String,
    pub subject_ordinal: i32,
}
