use uuid::Uuid;

#[derive(Queryable)]
pub struct Keyword {
    pub keyword_id: Uuid,
    pub work_id: Uuid,
    pub keyword_term: String,
    pub keyword_ordinal: i32,
}
