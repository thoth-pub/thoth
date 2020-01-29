#[derive(Debug, PartialEq, DbEnum)]
#[derive(juniper::GraphQLEnum)]
#[DieselType = "Contribution_type"]
pub enum ContributionType {
    Author,
    Editor,
    Translator,
    Photographer,
    Ilustrator,
    ForewordBy,
    IntroductionBy,
    AfterwordBy,
    PrefaceBy,
}

#[derive(Debug, PartialEq, DbEnum)]
#[derive(juniper::GraphQLEnum)]
#[DieselType = "Work_type"]
pub enum WorkType {
    BookChapter,
    Book,
}

#[derive(Debug, PartialEq, DbEnum)]
#[derive(juniper::GraphQLEnum)]
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
