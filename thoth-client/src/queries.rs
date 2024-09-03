use std::fmt;

use chrono::naive::NaiveDate;
use graphql_client::GraphQLQuery;
use thoth_api::model::contribution::ContributionType;
use thoth_api::model::language::LanguageRelation;
use thoth_api::model::publication::PublicationType;
use thoth_api::model::Doi;
use thoth_api::model::Isbn;
use thoth_api::model::Orcid;
use thoth_api::model::Ror;
use uuid::Uuid;

// Juniper v0.16 onwards converts Rust `NaiveDate` to GraphQL scalar `Date`,
// so we need to convert it back explicitly here (was previously automatic)
pub type Date = NaiveDate;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "assets/schema.graphql",
    query_path = "assets/queries.graphql",
    response_derives = "Debug,Clone,Deserialize,Serialize,PartialEq",
    variables_derives = "Debug,PartialEq"
)]
pub struct WorkQuery;

impl fmt::Display for work_query::LanguageCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl fmt::Display for work_query::CurrencyCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl fmt::Display for work_query::SubjectType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl fmt::Display for work_query::PublicationType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl fmt::Display for work_query::LocationPlatform {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "assets/schema.graphql",
    query_path = "assets/queries.graphql",
    response_derives = "Debug,Clone,Deserialize,Serialize,PartialEq",
    variables_derives = "Debug,PartialEq"
)]
pub struct WorksQuery;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "assets/schema.graphql",
    query_path = "assets/queries.graphql",
    response_derives = "Debug,Clone,Deserialize,Serialize,PartialEq",
    variables_derives = "Debug,PartialEq"
)]
pub struct WorkCountQuery;

// Needed to set work_query::Work as the canonical struct for the shared fragment in the two queries
// until https://github.com/graphql-rust/graphql-client/issues/312 gets fixed
impl From<works_query::Work> for work_query::Work {
    fn from(w: works_query::Work) -> Self {
        let se = serde_json::to_string(&w).unwrap();
        serde_json::from_str(&se).unwrap()
    }
}

// As above: enables shared processing of parent Works and child RelatedWorks in doideposit format
impl From<work_query::Work> for work_query::WorkRelationsRelatedWork {
    fn from(w: work_query::Work) -> Self {
        let se = serde_json::to_string(&w).unwrap();
        serde_json::from_str(&se).unwrap()
    }
}

// Allow conversion to the original ContributionType to benefit from trait implementations
impl From<work_query::ContributionType> for ContributionType {
    fn from(value: work_query::ContributionType) -> Self {
        match value {
            work_query::ContributionType::AUTHOR => ContributionType::Author,
            work_query::ContributionType::EDITOR => ContributionType::Editor,
            work_query::ContributionType::TRANSLATOR => ContributionType::Translator,
            work_query::ContributionType::PHOTOGRAPHER => ContributionType::Photographer,
            work_query::ContributionType::ILLUSTRATOR => ContributionType::Illustrator,
            work_query::ContributionType::MUSIC_EDITOR => ContributionType::MusicEditor,
            work_query::ContributionType::FOREWORD_BY => ContributionType::ForewordBy,
            work_query::ContributionType::INTRODUCTION_BY => ContributionType::IntroductionBy,
            work_query::ContributionType::AFTERWORD_BY => ContributionType::AfterwordBy,
            work_query::ContributionType::PREFACE_BY => ContributionType::PrefaceBy,
            work_query::ContributionType::SOFTWARE_BY => ContributionType::SoftwareBy,
            work_query::ContributionType::RESEARCH_BY => ContributionType::ResearchBy,
            work_query::ContributionType::CONTRIBUTIONS_BY => ContributionType::ContributionsBy,
            work_query::ContributionType::INDEXER => ContributionType::Indexer,
            _ => unreachable!(),
        }
    }
}

impl From<work_query::PublicationType> for PublicationType {
    fn from(value: crate::PublicationType) -> Self {
        match value {
            work_query::PublicationType::PAPERBACK => PublicationType::Paperback,
            work_query::PublicationType::HARDBACK => PublicationType::Hardback,
            work_query::PublicationType::PDF => PublicationType::Pdf,
            work_query::PublicationType::HTML => PublicationType::Html,
            work_query::PublicationType::XML => PublicationType::Xml,
            work_query::PublicationType::EPUB => PublicationType::Epub,
            work_query::PublicationType::MOBI => PublicationType::Mobi,
            work_query::PublicationType::AZW3 => PublicationType::Azw3,
            work_query::PublicationType::DOCX => PublicationType::Docx,
            work_query::PublicationType::FICTION_BOOK => PublicationType::FictionBook,
            work_query::PublicationType::MP3 => PublicationType::Mp3,
            work_query::PublicationType::WAV => PublicationType::Wav,
            work_query::PublicationType::Other(_) => unreachable!(),
        }
    }
}

impl From<work_query::LanguageRelation> for LanguageRelation {
    fn from(value: crate::LanguageRelation) -> Self {
        match value {
            work_query::LanguageRelation::ORIGINAL => LanguageRelation::Original,
            work_query::LanguageRelation::TRANSLATED_FROM => LanguageRelation::TranslatedFrom,
            work_query::LanguageRelation::TRANSLATED_INTO => LanguageRelation::TranslatedInto,
            work_query::LanguageRelation::Other(_) => unreachable!(),
        }
    }
}
