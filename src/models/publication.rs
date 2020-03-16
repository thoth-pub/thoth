use crate::schema::publication;
use uuid::Uuid;

#[derive(Debug, PartialEq, DbEnum, juniper::GraphQLEnum)]
#[DieselType = "Publication_type"]
pub enum PublicationType {
    #[db_rename = "Paperback"]
    Paperback,
    #[db_rename = "Hardback"]
    Hardback,
    #[db_rename = "PDF"]
    PDF,
    #[db_rename = "HTML"]
    HTML,
    #[db_rename = "XML"]
    XML,
    #[db_rename = "Epub"]
    Epub,
    #[db_rename = "Mobi"]
    Mobi,
}

#[derive(Queryable)]
pub struct Publication {
    pub publication_id: Uuid,
    pub publication_type: PublicationType,
    pub work_id: Uuid,
    pub isbn: Option<String>,
    pub publication_url: Option<String>,
}

#[derive(juniper::GraphQLInputObject, Insertable)]
#[table_name = "publication"]
pub struct NewPublication {
    pub publication_type: PublicationType,
    pub work_id: Uuid,
    pub isbn: Option<String>,
    pub publication_url: Option<String>,
}
