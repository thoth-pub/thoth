use chrono::{DateTime, TimeZone, Utc};
use isbn2::Isbn13;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use strum::Display;
use strum::EnumString;
use thoth_errors::{ThothError, ThothResult};
#[cfg(feature = "backend")]
use uuid::Uuid;

pub const DOI_DOMAIN: &str = "https://doi.org/";
pub const ORCID_DOMAIN: &str = "https://orcid.org/";
pub const ROR_DOMAIN: &str = "https://ror.org/";

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLEnum),
    graphql(description = "Unit of measurement for physical Work dimensions (mm, cm or in)")
)]
#[derive(
    Debug, Copy, Clone, Default, Serialize, Deserialize, PartialEq, Eq, EnumString, Display,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "lowercase")]
pub enum LengthUnit {
    #[cfg_attr(feature = "backend", graphql(description = "Millimetres"))]
    #[default]
    Mm,
    #[cfg_attr(feature = "backend", graphql(description = "Centimetres"))]
    Cm,
    #[cfg_attr(feature = "backend", graphql(description = "Inches"))]
    In,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLEnum),
    graphql(description = "Unit of measurement for physical Work weight (grams or ounces)")
)]
#[derive(
    Debug, Copy, Clone, Default, Serialize, Deserialize, PartialEq, Eq, EnumString, Display,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "lowercase")]
pub enum WeightUnit {
    #[cfg_attr(feature = "backend", graphql(description = "Grams"))]
    #[default]
    G,
    #[cfg_attr(feature = "backend", graphql(description = "Ounces"))]
    Oz,
}

#[cfg_attr(
    feature = "backend",
    derive(DieselNewType, juniper::GraphQLScalar),
    graphql(
        transparent,
        description = r#"Digital Object Identifier. Expressed as `^https:\/\/doi\.org\/10\.\d{4,9}\/[-._;()\/:a-zA-Z0-9<>+\[\]]+$`"#
    )
)]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Doi(String);

#[cfg_attr(
    feature = "backend",
    derive(DieselNewType, juniper::GraphQLScalar),
    graphql(
        transparent,
        description = "13-digit International Standard Book Number, with its parts separated by hyphens"
    )
)]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Isbn(String);

#[cfg_attr(
    feature = "backend",
    derive(DieselNewType, juniper::GraphQLScalar),
    graphql(
        transparent,
        description = r#"ORCID (Open Researcher and Contributor ID) identifier. Expressed as `^https:\/\/orcid\.org\/\d{4}-\d{4}-\d{4}-\d{3}[\dX]$`"#
    )
)]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Orcid(String);

#[cfg_attr(
    feature = "backend",
    derive(DieselNewType, juniper::GraphQLScalar),
    graphql(
        transparent,
        description = r#"ROR (Research Organization Registry) identifier. Expressed as `^https:\/\/ror\.org\/0[a-hjkmnp-z0-9]{6}\d{2}$`"#
    )
)]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Ror(String);

#[cfg_attr(
    feature = "backend",
    derive(DieselNewType, juniper::GraphQLScalar),
    graphql(
        transparent,
        description = "RFC 3339 combined date and time in UTC time zone (e.g. \"1999-12-31T23:59:00Z\")"
    )
)]
#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Ord, PartialOrd)]
pub struct Timestamp(DateTime<Utc>);

impl Timestamp {
    pub fn to_rfc3339(&self) -> String {
        self.0.to_rfc3339()
    }

    pub fn parse_from_rfc3339(input: &str) -> ThothResult<Self> {
        let timestamp = DateTime::parse_from_rfc3339(input)?.with_timezone(&Utc);
        Ok(Timestamp(timestamp))
    }
}

impl Default for Timestamp {
    fn default() -> Timestamp {
        Timestamp(TimeZone::timestamp_opt(&Utc, 0, 0).unwrap())
    }
}

impl fmt::Display for Doi {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", &self.0.replace(DOI_DOMAIN, ""))
    }
}

impl fmt::Display for Isbn {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", &self.0)
    }
}

impl fmt::Display for Orcid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", &self.0.replace(ORCID_DOMAIN, ""))
    }
}

impl fmt::Display for Ror {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", &self.0.replace(ROR_DOMAIN, ""))
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> fmt::Result {
        write!(f, "{}", &self.0.format("%F %T"))
    }
}

impl FromStr for Doi {
    type Err = ThothError;

    fn from_str(input: &str) -> ThothResult<Doi> {
        use lazy_static::lazy_static;
        use regex::Regex;
        lazy_static! {
            static ref RE: Regex = Regex::new(
            // ^    = beginning of string
            // (?:) = non-capturing group
            // i    = case-insensitive flag
            // $    = end of string
            // Matches strings of format "[[http[s]://][www.][dx.]doi.org/]10.XXX/XXX"
            // and captures the identifier segment starting with the "10." directory indicator
            // Corresponds to database constraints although regex syntax differs slightly
            // (e.g. `;()/` do not need to be escaped here)
            r"^(?i:(?:https?://)?(?:www\.)?(?:dx\.)?doi\.org/)?(10\.\d{4,9}/[-._;()\/:a-zA-Z0-9<>+\[\]]+$)").unwrap();
        }
        if input.is_empty() {
            Err(ThothError::DoiEmptyError)
        } else if let Some(matches) = RE.captures(input) {
            // The 0th capture always corresponds to the entire match
            if let Some(identifier) = matches.get(1) {
                let standardised = format!("{}{}", DOI_DOMAIN, identifier.as_str());
                let doi: Doi = Doi(standardised);
                Ok(doi)
            } else {
                Err(ThothError::DoiParseError(input.to_string()))
            }
        } else {
            Err(ThothError::DoiParseError(input.to_string()))
        }
    }
}

impl FromStr for Isbn {
    type Err = ThothError;

    fn from_str(input: &str) -> ThothResult<Isbn> {
        if input.is_empty() {
            Err(ThothError::IsbnEmptyError)
        } else {
            match input.parse::<Isbn13>() {
                Ok(parsed) => match parsed.hyphenate() {
                    Ok(hyphenated) => Ok(Isbn(hyphenated.to_string())),
                    Err(_) => Err(ThothError::IsbnParseError(input.to_string())),
                },
                Err(_) => Err(ThothError::IsbnParseError(input.to_string())),
            }
        }
    }
}

impl FromStr for Orcid {
    type Err = ThothError;

    fn from_str(input: &str) -> ThothResult<Orcid> {
        use lazy_static::lazy_static;
        use regex::Regex;
        lazy_static! {
            static ref RE: Regex = Regex::new(
            // ^    = beginning of string
            // (?:) = non-capturing group
            // i    = case-insensitive flag
            // $    = end of string
            // Matches strings of format "[[http[s]://][www.]orcid.org/]XXXX-XXXX-XXXX-XXXX"
            // and captures the 16-digit identifier segment
            // Corresponds to database constraints although regex syntax differs slightly
            r"^(?i:(?:https?://)?(?:www\.)?orcid\.org/)?(\d{4}-\d{4}-\d{4}-\d{3}[\dX]$)").unwrap();
        }
        if input.is_empty() {
            Err(ThothError::OrcidEmptyError)
        } else if let Some(matches) = RE.captures(input) {
            // The 0th capture always corresponds to the entire match
            if let Some(identifier) = matches.get(1) {
                let standardised = format!("{}{}", ORCID_DOMAIN, identifier.as_str());
                let orcid: Orcid = Orcid(standardised);
                Ok(orcid)
            } else {
                Err(ThothError::OrcidParseError(input.to_string()))
            }
        } else {
            Err(ThothError::OrcidParseError(input.to_string()))
        }
    }
}

impl FromStr for Ror {
    type Err = ThothError;

    fn from_str(input: &str) -> ThothResult<Ror> {
        use lazy_static::lazy_static;
        use regex::Regex;
        lazy_static! {
            static ref RE: Regex = Regex::new(
            // ^    = beginning of string
            // (?:) = non-capturing group
            // i    = case-insensitive flag
            // $    = end of string
            // Matches strings of format "[[[http[s]://]|[https://www.]]ror.org/]0XXXXXXNN"
            // and captures the 7-character/2-digit-checksum identifier segment
            // Corresponds to database constraints although regex syntax differs slightly
            r"^(?i:(?:https?://|https://www\.)?ror\.org/)?(0[a-hjkmnp-z0-9]{6}\d{2}$)").unwrap();
        }
        if input.is_empty() {
            Err(ThothError::RorEmptyError)
        } else if let Some(matches) = RE.captures(input) {
            // The 0th capture always corresponds to the entire match
            if let Some(identifier) = matches.get(1) {
                let standardised = format!("{}{}", ROR_DOMAIN, identifier.as_str());
                let ror: Ror = Ror(standardised);
                Ok(ror)
            } else {
                Err(ThothError::RorParseError(input.to_string()))
            }
        } else {
            Err(ThothError::RorParseError(input.to_string()))
        }
    }
}

impl Doi {
    pub fn to_lowercase_string(&self) -> String {
        self.0.to_lowercase()
    }
}

impl Isbn {
    pub fn to_hyphenless_string(&self) -> String {
        self.0.replace('-', "")
    }
}

#[cfg(feature = "backend")]
#[allow(clippy::too_many_arguments)]
/// Common functionality to perform basic CRUD actions on Thoth entities
pub trait Crud
where
    Self: Sized,
{
    /// The structure used to create a new entity, e.g. `NewImprint`
    type NewEntity;
    /// The structure used to modify an existing entity, e.g. `PatchImprint`
    type PatchEntity;
    /// The structure used to sort database results, e.g. `ImprintOrderBy`
    type OrderByEntity;
    /// A generic structure to constrain search results, e.g. `WorkType`
    type FilterParameter1;
    /// A second such structure, e.g. `WorkStatus`
    type FilterParameter2;
    /// A third such structure, e.g. `TimeExpression`
    type FilterParameter3;

    /// Specify the entity's primary key
    fn pk(&self) -> Uuid;

    /// Query the database to obtain a list of entities based on some criteria.
    ///
    /// `parent_id` is used, when nesting, to constrain results by a particular foreign key.
    ///
    /// `filter_param`s are used for filtering by a structure specific parameter,
    /// e.g. `WorkType` for `Work`
    fn all(
        db: &crate::db::PgPool,
        limit: i32,
        offset: i32,
        filter: Option<String>,
        order: Self::OrderByEntity,
        publishers: Vec<Uuid>,
        parent_id_1: Option<Uuid>,
        parent_id_2: Option<Uuid>,
        filter_param_1: Vec<Self::FilterParameter1>,
        filter_param_2: Vec<Self::FilterParameter2>,
        filter_param_3: Option<Self::FilterParameter3>,
    ) -> ThothResult<Vec<Self>>;

    /// Query the database to obtain the total number of entities satisfying the search criteria
    fn count(
        db: &crate::db::PgPool,
        filter: Option<String>,
        publishers: Vec<Uuid>,
        filter_param_1: Vec<Self::FilterParameter1>,
        filter_param_2: Vec<Self::FilterParameter2>,
        filter_param_3: Option<Self::FilterParameter3>,
    ) -> ThothResult<i32>;

    /// Query the database to obtain an instance of the entity given its ID
    fn from_id(db: &crate::db::PgPool, entity_id: &Uuid) -> ThothResult<Self>;

    /// Insert a new record in the database and obtain the resulting instance
    fn create(db: &crate::db::PgPool, data: &Self::NewEntity) -> ThothResult<Self>;

    /// Modify the record in the database and obtain the resulting instance
    fn update(
        &self,
        db: &crate::db::PgPool,
        data: &Self::PatchEntity,
        account_id: &Uuid,
    ) -> ThothResult<Self>;

    /// Delete the record from the database and obtain the deleted instance
    fn delete(self, db: &crate::db::PgPool) -> ThothResult<Self>;

    /// Retrieve the ID of the publisher linked to this entity (if applicable)
    fn publisher_id(&self, db: &crate::db::PgPool) -> ThothResult<Uuid>;
}

#[cfg(feature = "backend")]
/// Common functionality to store history
pub trait HistoryEntry
where
    Self: Sized,
{
    /// The structure used to create a new history entity, e.g. `NewImprintHistory` for `Imprint`
    type NewHistoryEntity;

    fn new_history_entry(&self, account_id: &Uuid) -> Self::NewHistoryEntity;
}

#[cfg(feature = "backend")]
pub trait DbInsert
where
    Self: Sized,
{
    /// The structure that is returned by the insert statement
    type MainEntity;

    fn insert(&self, connection: &mut diesel::PgConnection) -> ThothResult<Self::MainEntity>;
}

/// Declares function implementations for the `Crud` trait, reducing the boilerplate needed to define
/// the CRUD functionality for each entity.
///
/// Example usage
/// -------------
///
/// ```ignore
/// use crate::model::imprint::{NewImprint, PatchImprint, Imprint, NewImprintHistory};
/// use crate::crud_methods;
/// use crate::model::Crud;
///
/// impl Crud for Imprint {
///     type NewEntity = NewImprint;
///     type PatchEntity = PatchImprint;
///
///     fn pk(&self) -> Uuid {
///         self.imprint_id
///     }
///
///     crud_methods!(imprint::table, imprint::dsl::imprint, NewImprintHistory);
/// }
/// ```
///
///
#[cfg(feature = "backend")]
#[macro_export]
macro_rules! crud_methods {
    ($table_dsl:expr, $entity_dsl:expr) => {
        fn from_id(db: &$crate::db::PgPool, entity_id: &Uuid) -> ThothResult<Self> {
            use diesel::{QueryDsl, RunQueryDsl};

            let mut connection = db.get()?;
            $entity_dsl
                .find(entity_id)
                .get_result::<Self>(&mut connection)
                .map_err(Into::into)
        }

        fn create(db: &$crate::db::PgPool, data: &Self::NewEntity) -> ThothResult<Self> {
            let mut connection = db.get()?;
            diesel::insert_into($table_dsl)
                .values(data)
                .get_result::<Self>(&mut connection)
                .map_err(Into::into)
        }

        /// Makes a database transaction that first updates the entity and then creates a new
        /// history entity record.
        fn update(
            &self,
            db: &$crate::db::PgPool,
            data: &Self::PatchEntity,
            account_id: &Uuid,
        ) -> ThothResult<Self> {
            use diesel::{Connection, QueryDsl, RunQueryDsl};

            let mut connection = db.get()?;
            connection.transaction(|connection| {
                diesel::update($entity_dsl.find(&self.pk()))
                    .set(data)
                    .get_result(connection)
                    .map_err(Into::into)
                    .and_then(|c| {
                        self.new_history_entry(&account_id)
                            .insert(connection)
                            .map(|_| c)
                    })
                    .map_err(Into::into)
            })
        }

        fn delete(self, db: &$crate::db::PgPool) -> ThothResult<Self> {
            use diesel::{QueryDsl, RunQueryDsl};

            let mut connection = db.get()?;
            diesel::delete($entity_dsl.find(&self.pk()))
                .execute(&mut connection)
                .map(|_| self)
                .map_err(Into::into)
        }
    };
}

/// Declares an insert function implementation for any insertable. Useful together with the
/// `DbInsert` trait.
///
/// Example usage
/// -------------
///
/// ```ignore
/// use crate::model::imprint::{ImprintHistory, NewImprintHistory};
/// use crate::db_insert;
/// use crate::model::DbInsert;
///
/// impl DbInsert for NewImprintHistory {
///    type MainEntity = ImprintHistory;
///
///    db_insert!(imprint_history::table);
///}
/// ```
///
///
#[cfg(feature = "backend")]
#[macro_export]
macro_rules! db_insert {
    ($table_dsl:expr) => {
        fn insert(&self, connection: &mut diesel::PgConnection) -> ThothResult<Self::MainEntity> {
            use diesel::RunQueryDsl;

            diesel::insert_into($table_dsl)
                .values(self)
                .get_result(connection)
                .map_err(Into::into)
        }
    };
}

pub trait Convert {
    fn convert_length_from_to(&self, current_units: &LengthUnit, new_units: &LengthUnit) -> f64;
    fn convert_weight_from_to(&self, current_units: &WeightUnit, new_units: &WeightUnit) -> f64;
}

impl Convert for f64 {
    fn convert_length_from_to(&self, current_units: &LengthUnit, new_units: &LengthUnit) -> f64 {
        match (current_units, new_units) {
            // If current units and new units are the same, no conversion is needed
            (LengthUnit::Mm, LengthUnit::Mm)
            | (LengthUnit::Cm, LengthUnit::Cm)
            | (LengthUnit::In, LengthUnit::In) => *self,
            // Return cm values rounded to max 1 decimal place (1 cm = 10 mm)
            (LengthUnit::Mm, LengthUnit::Cm) => self.round() / 10.0,
            // Return mm values rounded to nearest mm (1 cm = 10 mm)
            (LengthUnit::Cm, LengthUnit::Mm) => (self * 10.0).round(),
            // Return inch values rounded to 2 decimal places (1 inch = 25.4 mm)
            (LengthUnit::Mm, LengthUnit::In) => {
                let unrounded_inches = self / 25.4;
                // To round to a non-integer scale, multiply by the appropriate factor,
                // round to the nearest integer, then divide again by the same factor
                (unrounded_inches * 100.0).round() / 100.0
            }
            // Return mm values rounded to nearest mm (1 inch = 25.4 mm)
            (LengthUnit::In, LengthUnit::Mm) => (self * 25.4).round(),
            // We don't currently support conversion between cm and in as it is not required
            _ => unimplemented!(),
        }
    }

    fn convert_weight_from_to(&self, current_units: &WeightUnit, new_units: &WeightUnit) -> f64 {
        match (current_units, new_units) {
            // If current units and new units are the same, no conversion is needed
            (WeightUnit::G, WeightUnit::G) | (WeightUnit::Oz, WeightUnit::Oz) => *self,
            // Return ounce values rounded to 4 decimal places (1 ounce = 28.349523125 grams)
            (WeightUnit::G, WeightUnit::Oz) => {
                let unrounded_ounces = self / 28.349523125;
                // To round to a non-integer scale, multiply by the appropriate factor,
                // round to the nearest integer, then divide again by the same factor
                (unrounded_ounces * 10000.0).round() / 10000.0
            }
            // Return gram values rounded to nearest gram (1 ounce = 28.349523125 grams)
            (WeightUnit::Oz, WeightUnit::G) => (self * 28.349523125).round(),
        }
    }
}

/// Assign the leading domain of an identifier
pub trait UrlIdentifier {
    fn domain(&self) -> &'static str;
}

/// Output an identifier with its leading domain
pub trait IdentifierWithDomain
where
    Self: UrlIdentifier + fmt::Display,
{
    fn with_domain(&self) -> String {
        format!("{}{}", self.domain(), self)
    }
}

impl UrlIdentifier for Doi {
    fn domain(&self) -> &'static str {
        DOI_DOMAIN
    }
}

impl UrlIdentifier for Orcid {
    fn domain(&self) -> &'static str {
        ORCID_DOMAIN
    }
}

impl UrlIdentifier for Ror {
    fn domain(&self) -> &'static str {
        ROR_DOMAIN
    }
}

impl IdentifierWithDomain for Doi {}
impl IdentifierWithDomain for Orcid {}
impl IdentifierWithDomain for Ror {}

/// Enum to represent the markup format
#[cfg_attr(
    feature = "backend",
    derive(DbEnum, juniper::GraphQLEnum),
    graphql(description = "Allowed markup formats for multilingual paradigm"),
    ExistingTypePath = "crate::schema::sql_types::MarkupFormat"
)]
#[derive(
    Debug, Copy, Clone, Default, PartialEq, Eq, Deserialize, Serialize, EnumString, Display,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "UPPERCASE")]
pub enum MarkupFormat {
    #[cfg_attr(feature = "backend", graphql(description = "HTML format"))]
    Html,
    #[cfg_attr(feature = "backend", graphql(description = "Markdown format"))]
    Markdown,
    #[cfg_attr(feature = "backend", graphql(description = "Plain text format"))]
    PlainText,
    #[cfg_attr(feature = "backend", graphql(description = "JATS XML format"))]
    #[default]
    JatsXml,
}

// impl MarkupFormat {
// pub fn parse(input: &str) -> ThothResult<Self> {
//     // Extract format from input extension
//     let format = input
//         .split('.')
//         .last()
//         .ok_or_else(|| ThothError::UnsuportedFileFormatError)?;

//     match format.to_lowercase().as_str() {
//         "html" | "htm" => Ok(MarkupFormat::Html),
//         "md" | "markdown" => Ok(MarkupFormat::Markdown),
//         "txt" | "text" => Ok(MarkupFormat::PlainText),
//         "xml" => Ok(MarkupFormat::JatsXml),
//         _ => Err(ThothError::UnsuportedFileFormatError),
//     }
// }
// }

/// Enum to represent abstract types
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum AbstractType {
    Short,
    Long,
}

/// Validate content format based on markup format
pub fn validate_format(content: &str, format: &MarkupFormat) -> ThothResult<()> {
    match format {
        MarkupFormat::Html => {
            // Basic HTML validation - check for opening and closing tags
            if !content.contains('<') || !content.contains('>') {
                return Err(ThothError::UnsuportedFileFormatError);
            }
            // Check for basic HTML structure
            if !content.contains("</") {
                return Err(ThothError::UnsuportedFileFormatError);
            }
        }
        MarkupFormat::Markdown => {
            // Basic Markdown validation - check for markdown syntax
            if !content.contains('#') && !content.contains('*') && !content.contains('`') {
                // At least one markdown element should be present
                return Err(ThothError::UnsuportedFileFormatError);
            }
        }
        MarkupFormat::JatsXml => {
            // Basic JATS XML validation
            if !content.contains("<?xml") && !content.contains("<article") {
                return Err(ThothError::UnsuportedFileFormatError);
            }
            if !content.contains("</") {
                return Err(ThothError::UnsuportedFileFormatError);
            }
        }
        MarkupFormat::PlainText => {
            // Plain text validation - should not contain markup
            if content.contains('<') && content.contains('>') {
                return Err(ThothError::UnsuportedFileFormatError);
            }
        }
    }
    Ok(())
}

/// Extract title and subtitle from content based on format
pub fn extract_title(content: &str, format: &MarkupFormat) -> ThothResult<(String, String)> {
    // Validate format first
    validate_format(content, format)?;

    match format {
        MarkupFormat::Html => {
            let title_regex =
                Regex::new(r"<h1>(.*?)</h1>").map_err(|_| ThothError::UnsuportedFileFormatError)?;
            let subtitle_regex =
                Regex::new(r"<h2>(.*?)</h2>").map_err(|_| ThothError::UnsuportedFileFormatError)?;

            let title = title_regex
                .captures(content)
                .and_then(|caps| caps.get(1))
                .map(|m| m.as_str().trim().to_string())
                .unwrap_or_default();

            let subtitle = subtitle_regex
                .captures(content)
                .and_then(|caps| caps.get(1))
                .map(|m| m.as_str().trim().to_string())
                .unwrap_or_default();

            Ok((title, subtitle))
        }
        MarkupFormat::Markdown => {
            let title_regex =
                Regex::new(r"^#\s+(.*?)$").map_err(|_| ThothError::UnsuportedFileFormatError)?;
            let subtitle_regex =
                Regex::new(r"^##\s+(.*?)$").map_err(|_| ThothError::UnsuportedFileFormatError)?;

            let title = content
                .lines()
                .find_map(|line| {
                    title_regex
                        .captures(line)
                        .and_then(|caps| caps.get(1))
                        .map(|m| m.as_str().trim().to_string())
                })
                .unwrap_or_default();

            let subtitle = content
                .lines()
                .find_map(|line| {
                    subtitle_regex
                        .captures(line)
                        .and_then(|caps| caps.get(1))
                        .map(|m| m.as_str().trim().to_string())
                })
                .unwrap_or_default();

            Ok((title, subtitle))
        }
        MarkupFormat::JatsXml => {
            let title_regex = Regex::new(r"<title>(.*?)</title>")
                .map_err(|_| ThothError::UnsuportedFileFormatError)?;
            let subtitle_regex = Regex::new(r"<subtitle>(.*?)</subtitle>")
                .map_err(|_| ThothError::UnsuportedFileFormatError)?;

            let title = title_regex
                .captures(content)
                .and_then(|caps| caps.get(1))
                .map(|m| m.as_str().trim().to_string())
                .unwrap_or_default();

            let subtitle = subtitle_regex
                .captures(content)
                .and_then(|caps| caps.get(1))
                .map(|m| m.as_str().trim().to_string())
                .unwrap_or_default();

            Ok((title, subtitle))
        }
        MarkupFormat::PlainText => {
            let mut lines = content.lines();
            let first_line = lines
                .next()
                .ok_or(ThothError::UnsuportedFileFormatError)?
                .trim();
            let second_line = lines.next().unwrap_or("").trim();

            // Check if first line is all uppercase (potential title)
            let is_title = first_line.chars().all(|c| !c.is_lowercase());

            // Check if second line is title case (potential subtitle)
            let is_subtitle = second_line.chars().next().is_some_and(|c| c.is_uppercase())
                && second_line.chars().any(|c| c.is_lowercase());

            let title = if is_title && !first_line.is_empty() {
                first_line.to_string()
            } else {
                return Err(ThothError::UnsuportedFileFormatError);
            };

            let subtitle = if is_subtitle && !second_line.is_empty() {
                second_line.to_string()
            } else if !second_line.is_empty() {
                // If second line exists but doesn't follow title case pattern, return error
                return Err(ThothError::UnsuportedFileFormatError);
            } else {
                String::new()
            };

            Ok((title, subtitle))
        }
    }
}

/// Convert content to JATS XML format with specified tag
pub fn convert_to_jats(content: String, tag_name: String) -> ThothResult<String> {
    if content.is_empty() {
        return Ok(String::new());
    }

    // Escape XML special characters
    let escaped_content = content
        .replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace("\"", "&quot;")
        .replace("'", "&apos;");

    Ok(format!("<{}>{}</{}>", tag_name, escaped_content, tag_name))
}

/// Convert from JATS XML to specified format
pub fn convert_from_jats(jats_xml: &str, format: MarkupFormat) -> ThothResult<String> {
    validate_format(jats_xml, &MarkupFormat::JatsXml)?;

    let content_regex =
        Regex::new(r"<([^>]+)>(.*?)</\1>").map_err(|_| ThothError::UnsuportedFileFormatError)?;

    let mut elements = Vec::new();
    for caps in content_regex.captures_iter(jats_xml) {
        let tag = caps
            .get(1)
            .map(|m| m.as_str().to_string())
            .unwrap_or_default();
        let content = caps
            .get(2)
            .map(|m| m.as_str().trim().to_string())
            .unwrap_or_default();
        elements.push((tag, content));
    }

    match format {
        MarkupFormat::Html => {
            let mut html = String::new();
            for (tag, content) in elements {
                html.push_str(&format!("<{}>{}</{}>\n", tag, content, tag));
            }
            Ok(html)
        }
        MarkupFormat::Markdown => {
            let mut markdown = String::new();
            for (tag, content) in elements {
                match tag.as_str() {
                    "title" => markdown.push_str(&format!("# {}\n", content)),
                    "subtitle" => markdown.push_str(&format!("## {}\n", content)),
                    _ => markdown.push_str(&format!("{}\n", content)),
                }
            }
            Ok(markdown)
        }
        MarkupFormat::PlainText => {
            let mut text = String::new();
            for (tag, content) in elements {
                match tag.as_str() {
                    "title" => text.push_str(&format!("{}\n", content.to_uppercase())),
                    "subtitle" => {
                        let title_case = content
                            .split_whitespace()
                            .map(|word| {
                                let mut chars = word.chars();
                                match chars.next() {
                                    None => String::new(),
                                    Some(first) => first
                                        .to_uppercase()
                                        .chain(chars.flat_map(|c| c.to_lowercase()))
                                        .collect(),
                                }
                            })
                            .collect::<Vec<String>>()
                            .join(" ");
                        text.push_str(&format!("{}\n", title_case));
                    }
                    _ => text.push_str(&format!("{}\n", content)),
                }
            }
            Ok(text)
        }
        MarkupFormat::JatsXml => Ok(jats_xml.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_doi_default() {
        let doi: Doi = Default::default();
        assert_eq!(doi, Doi("".to_string()));
    }

    #[test]
    fn test_isbn_default() {
        let isbn: Isbn = Default::default();
        assert_eq!(isbn, Isbn("".to_string()));
    }

    #[test]
    fn test_orcid_default() {
        let orcid: Orcid = Default::default();
        assert_eq!(orcid, Orcid("".to_string()));
    }

    #[test]
    fn test_ror_default() {
        let ror: Ror = Default::default();
        assert_eq!(ror, Ror("".to_string()));
    }

    #[test]
    fn test_timestamp_default() {
        let stamp: Timestamp = Default::default();
        assert_eq!(
            stamp,
            Timestamp(TimeZone::timestamp_opt(&Utc, 0, 0).unwrap())
        );
    }

    #[test]
    fn test_doi_display() {
        let doi = Doi("https://doi.org/10.12345/Test-Suffix.01".to_string());
        assert_eq!(format!("{doi}"), "10.12345/Test-Suffix.01");
    }

    #[test]
    fn test_isbn_display() {
        let isbn = Isbn("978-3-16-148410-0".to_string());
        assert_eq!(format!("{isbn}"), "978-3-16-148410-0");
    }

    #[test]
    fn test_orcid_display() {
        let orcid = Orcid("https://orcid.org/0000-0002-1234-5678".to_string());
        assert_eq!(format!("{orcid}"), "0000-0002-1234-5678");
    }

    #[test]
    fn test_ror_display() {
        let ror = Ror("https://ror.org/0abcdef12".to_string());
        assert_eq!(format!("{ror}"), "0abcdef12");
    }

    #[test]
    fn test_timestamp_display() {
        let stamp: Timestamp = Default::default();
        assert_eq!(format!("{stamp}"), "1970-01-01 00:00:00");
    }

    #[test]
    fn test_doi_fromstr() {
        let standardised = Doi("https://doi.org/10.12345/Test-Suffix.01".to_string());
        assert_eq!(
            Doi::from_str("https://doi.org/10.12345/Test-Suffix.01").unwrap(),
            standardised
        );
        assert_eq!(
            Doi::from_str("http://doi.org/10.12345/Test-Suffix.01").unwrap(),
            standardised
        );
        assert_eq!(
            Doi::from_str("doi.org/10.12345/Test-Suffix.01").unwrap(),
            standardised
        );
        assert_eq!(
            Doi::from_str("10.12345/Test-Suffix.01").unwrap(),
            standardised
        );
        assert_eq!(
            Doi::from_str("HTTPS://DOI.ORG/10.12345/Test-Suffix.01").unwrap(),
            standardised
        );
        assert_eq!(
            Doi::from_str("Https://DOI.org/10.12345/Test-Suffix.01").unwrap(),
            standardised
        );
        assert_eq!(
            Doi::from_str("https://www.doi.org/10.12345/Test-Suffix.01").unwrap(),
            standardised
        );
        assert_eq!(
            Doi::from_str("http://www.doi.org/10.12345/Test-Suffix.01").unwrap(),
            standardised
        );
        assert_eq!(
            Doi::from_str("www.doi.org/10.12345/Test-Suffix.01").unwrap(),
            standardised
        );
        assert_eq!(
            Doi::from_str("https://dx.doi.org/10.12345/Test-Suffix.01").unwrap(),
            standardised
        );
        assert_eq!(
            Doi::from_str("http://dx.doi.org/10.12345/Test-Suffix.01").unwrap(),
            standardised
        );
        assert_eq!(
            Doi::from_str("dx.doi.org/10.12345/Test-Suffix.01").unwrap(),
            standardised
        );
        assert_eq!(
            Doi::from_str("https://www.dx.doi.org/10.12345/Test-Suffix.01").unwrap(),
            standardised
        );
        assert_eq!(
            Doi::from_str("http://www.dx.doi.org/10.12345/Test-Suffix.01").unwrap(),
            standardised
        );
        assert_eq!(
            Doi::from_str("www.dx.doi.org/10.12345/Test-Suffix.01").unwrap(),
            standardised
        );
        assert!(Doi::from_str("htts://doi.org/10.12345/Test-Suffix.01").is_err());
        assert!(Doi::from_str("https://10.12345/Test-Suffix.01").is_err());
        assert!(Doi::from_str("https://test.org/10.12345/Test-Suffix.01").is_err());
        assert!(Doi::from_str("http://test.org/10.12345/Test-Suffix.01").is_err());
        assert!(Doi::from_str("test.org/10.12345/Test-Suffix.01").is_err());
        assert!(Doi::from_str("//doi.org/10.12345/Test-Suffix.01").is_err());
        assert!(Doi::from_str("https://doi-org/10.12345/Test-Suffix.01").is_err());
        assert!(Doi::from_str("10.https://doi.org/12345/Test-Suffix.01").is_err());
        assert!(
            Doi::from_str("http://dx.doi.org/10.2990/1471-5457(2005)24[2:tmpwac]2.0.co;2").is_ok()
        );
        assert!(Doi::from_str(
            "https://doi.org/10.1002/(SICI)1098-2736(199908)36:6<637::AID-TEA4>3.0.CO;2-9"
        )
        .is_ok());
        assert!(Doi::from_str(
            "https://doi.org/10.1002/(sici)1096-8644(1996)23+<91::aid-ajpa4>3.0.co;2-c"
        )
        .is_ok());
    }

    #[test]
    fn test_isbn_fromstr() {
        // Note the `isbn2` crate contains tests of valid/invalid ISBN values -
        // this focuses on testing that a valid ISBN in any format is standardised
        let standardised = Isbn("978-3-16-148410-0".to_string());
        assert_eq!(Isbn::from_str("978-3-16-148410-0").unwrap(), standardised);
        assert_eq!(Isbn::from_str("9783161484100").unwrap(), standardised);
        assert_eq!(Isbn::from_str("978 3 16 148410 0").unwrap(), standardised);
        assert_eq!(Isbn::from_str("978 3 16-148410-0").unwrap(), standardised);
        assert_eq!(Isbn::from_str("9-7-831614-8-4-100").unwrap(), standardised);
        assert_eq!(
            Isbn::from_str("   97831    614 84  100    ").unwrap(),
            standardised
        );
        assert_eq!(
            Isbn::from_str("---97--831614----8-4100--").unwrap(),
            standardised
        );
        assert!(Isbn::from_str("978-3-16-148410-1").is_err());
        assert!(Isbn::from_str("1234567890123").is_err());
        assert!(Isbn::from_str("0-684-84328-5").is_err());
        assert!(Isbn::from_str("abcdef").is_err());
    }

    #[test]
    fn test_orcid_fromstr() {
        let standardised = Orcid("https://orcid.org/0000-0002-1234-5678".to_string());
        assert_eq!(
            Orcid::from_str("https://orcid.org/0000-0002-1234-5678").unwrap(),
            standardised
        );
        assert_eq!(
            Orcid::from_str("http://orcid.org/0000-0002-1234-5678").unwrap(),
            standardised
        );
        assert_eq!(
            Orcid::from_str("orcid.org/0000-0002-1234-5678").unwrap(),
            standardised
        );
        assert_eq!(
            Orcid::from_str("0000-0002-1234-5678").unwrap(),
            standardised
        );
        assert_eq!(
            Orcid::from_str("HTTPS://ORCID.ORG/0000-0002-1234-5678").unwrap(),
            standardised
        );
        assert_eq!(
            Orcid::from_str("Https://ORCiD.org/0000-0002-1234-5678").unwrap(),
            standardised
        );
        assert_eq!(
            Orcid::from_str("https://www.orcid.org/0000-0002-1234-5678").unwrap(),
            standardised
        );
        assert_eq!(
            Orcid::from_str("http://www.orcid.org/0000-0002-1234-5678").unwrap(),
            standardised
        );
        assert_eq!(
            Orcid::from_str("www.orcid.org/0000-0002-1234-5678").unwrap(),
            standardised
        );
        assert!(Orcid::from_str("htts://orcid.org/0000-0002-1234-5678").is_err());
        assert!(Orcid::from_str("https://0000-0002-1234-5678").is_err());
        assert!(Orcid::from_str("https://test.org/0000-0002-1234-5678").is_err());
        assert!(Orcid::from_str("http://test.org/0000-0002-1234-5678").is_err());
        assert!(Orcid::from_str("test.org/0000-0002-1234-5678").is_err());
        assert!(Orcid::from_str("//orcid.org/0000-0002-1234-5678").is_err());
        assert!(Orcid::from_str("https://orcid-org/0000-0002-1234-5678").is_err());
        assert!(Orcid::from_str("0000-0002-1234-5678https://orcid.org/").is_err());
        assert!(Orcid::from_str("0009-0002-1234-567X").is_ok());
    }

    #[test]
    fn test_ror_fromstr() {
        let standardised = Ror("https://ror.org/0abcdef12".to_string());
        assert_eq!(
            Ror::from_str("https://ror.org/0abcdef12").unwrap(),
            standardised
        );
        assert_eq!(
            Ror::from_str("http://ror.org/0abcdef12").unwrap(),
            standardised
        );
        assert_eq!(Ror::from_str("ror.org/0abcdef12").unwrap(), standardised);
        assert_eq!(Ror::from_str("0abcdef12").unwrap(), standardised);
        assert_eq!(
            Ror::from_str("HTTPS://ROR.ORG/0abcdef12").unwrap(),
            standardised
        );
        assert_eq!(
            Ror::from_str("Https://Ror.org/0abcdef12").unwrap(),
            standardised
        );
        assert_eq!(
            Ror::from_str("https://www.ror.org/0abcdef12").unwrap(),
            standardised
        );
        // Testing shows that while leading http://ror and https://www.ror
        // resolve successfully, leading www.ror and http://www.ror do not.
        assert!(Ror::from_str("http://www.ror.org/0abcdef12").is_err());
        assert!(Ror::from_str("www.ror.org/0abcdef12").is_err());
        assert!(Ror::from_str("htts://ror.org/0abcdef12").is_err());
        assert!(Ror::from_str("https://0abcdef12").is_err());
        assert!(Ror::from_str("https://test.org/0abcdef12").is_err());
        assert!(Ror::from_str("http://test.org/0abcdef12").is_err());
        assert!(Ror::from_str("test.org/0abcdef12").is_err());
        assert!(Ror::from_str("//ror.org/0abcdef12").is_err());
        assert!(Ror::from_str("https://ror-org/0abcdef12").is_err());
        assert!(Ror::from_str("0abcdef12https://ror.org/").is_err());
    }

    #[test]
    fn test_isbn_to_hyphenless_string() {
        let hyphenless_isbn = Isbn("978-3-16-148410-0".to_string()).to_hyphenless_string();
        assert_eq!(hyphenless_isbn, "9783161484100");
    }

    #[test]
    // Float equality comparison is fine here because the floats
    // have already been rounded by the functions under test
    #[allow(clippy::float_cmp)]
    fn test_convert_length_from_to() {
        use LengthUnit::*;
        assert_eq!(123.456.convert_length_from_to(&Mm, &Cm), 12.3);
        assert_eq!(123.456.convert_length_from_to(&Mm, &In), 4.86);
        assert_eq!(123.456.convert_length_from_to(&Cm, &Mm), 1235.0);
        assert_eq!(123.456.convert_length_from_to(&In, &Mm), 3136.0);
        // Test some standard print sizes
        assert_eq!(4.25.convert_length_from_to(&In, &Mm), 108.0);
        assert_eq!(108.0.convert_length_from_to(&Mm, &In), 4.25);
        assert_eq!(6.0.convert_length_from_to(&In, &Mm), 152.0);
        assert_eq!(152.0.convert_length_from_to(&Mm, &In), 5.98);
        assert_eq!(8.5.convert_length_from_to(&In, &Mm), 216.0);
        assert_eq!(216.0.convert_length_from_to(&Mm, &In), 8.5);
        // Test that converting and then converting back again
        // returns a value within a reasonable margin of error
        assert_eq!(
            5.06.convert_length_from_to(&In, &Mm)
                .convert_length_from_to(&Mm, &In),
            5.08
        );
        assert_eq!(
            6.5.convert_length_from_to(&In, &Mm)
                .convert_length_from_to(&Mm, &In),
            6.5
        );
        assert_eq!(
            7.44.convert_length_from_to(&In, &Mm)
                .convert_length_from_to(&Mm, &In),
            7.44
        );
        assert_eq!(
            8.27.convert_length_from_to(&In, &Mm)
                .convert_length_from_to(&Mm, &In),
            8.27
        );
        assert_eq!(
            9.0.convert_length_from_to(&In, &Mm)
                .convert_length_from_to(&Mm, &In),
            9.02
        );
        assert_eq!(
            10.88
                .convert_length_from_to(&In, &Mm)
                .convert_length_from_to(&Mm, &In),
            10.87
        );
        assert_eq!(
            102.0
                .convert_length_from_to(&Mm, &In)
                .convert_length_from_to(&In, &Mm),
            102.0
        );
        assert_eq!(
            120.0
                .convert_length_from_to(&Mm, &In)
                .convert_length_from_to(&In, &Mm),
            120.0
        );
        assert_eq!(
            168.0
                .convert_length_from_to(&Mm, &In)
                .convert_length_from_to(&In, &Mm),
            168.0
        );
        assert_eq!(
            190.0
                .convert_length_from_to(&Mm, &In)
                .convert_length_from_to(&In, &Mm),
            190.0
        );
    }

    #[test]
    // Float equality comparison is fine here because the floats
    // have already been rounded by the functions under test
    #[allow(clippy::float_cmp)]
    fn test_convert_weight_from_to() {
        use WeightUnit::*;
        assert_eq!(123.456.convert_weight_from_to(&G, &Oz), 4.3548);
        assert_eq!(123.456.convert_weight_from_to(&Oz, &G), 3500.0);
        assert_eq!(4.25.convert_weight_from_to(&Oz, &G), 120.0);
        assert_eq!(108.0.convert_weight_from_to(&G, &Oz), 3.8096);
        assert_eq!(6.0.convert_weight_from_to(&Oz, &G), 170.0);
        assert_eq!(152.0.convert_weight_from_to(&G, &Oz), 5.3616);
        assert_eq!(8.5.convert_weight_from_to(&Oz, &G), 241.0);
        assert_eq!(216.0.convert_weight_from_to(&G, &Oz), 7.6192);
        // Test that converting and then converting back again
        // returns a value within a reasonable margin of error
        assert_eq!(
            5.0.convert_weight_from_to(&Oz, &G)
                .convert_weight_from_to(&G, &Oz),
            5.0089
        );
        assert_eq!(
            5.125
                .convert_weight_from_to(&Oz, &G)
                .convert_weight_from_to(&G, &Oz),
            5.1147
        );
        assert_eq!(
            6.5.convert_weight_from_to(&Oz, &G)
                .convert_weight_from_to(&G, &Oz),
            6.4904
        );
        assert_eq!(
            7.25.convert_weight_from_to(&Oz, &G)
                .convert_weight_from_to(&G, &Oz),
            7.2664
        );
        assert_eq!(
            7.44.convert_weight_from_to(&Oz, &G)
                .convert_weight_from_to(&G, &Oz),
            7.4428
        );
        assert_eq!(
            8.0625
                .convert_weight_from_to(&Oz, &G)
                .convert_weight_from_to(&G, &Oz),
            8.0777
        );
        assert_eq!(
            9.0.convert_weight_from_to(&Oz, &G)
                .convert_weight_from_to(&G, &Oz),
            8.9949
        );
        assert_eq!(
            10.75
                .convert_weight_from_to(&Oz, &G)
                .convert_weight_from_to(&G, &Oz),
            10.7586
        );
        assert_eq!(
            10.88
                .convert_weight_from_to(&Oz, &G)
                .convert_weight_from_to(&G, &Oz),
            10.8644
        );
        assert_eq!(
            102.0
                .convert_weight_from_to(&G, &Oz)
                .convert_weight_from_to(&Oz, &G),
            102.0
        );
        assert_eq!(
            120.0
                .convert_weight_from_to(&G, &Oz)
                .convert_weight_from_to(&Oz, &G),
            120.0
        );
        assert_eq!(
            168.0
                .convert_weight_from_to(&G, &Oz)
                .convert_weight_from_to(&Oz, &G),
            168.0
        );
        assert_eq!(
            190.0
                .convert_weight_from_to(&G, &Oz)
                .convert_weight_from_to(&Oz, &G),
            190.0
        );
    }

    #[test]
    fn test_doi_with_domain() {
        let doi = "https://doi.org/10.12345/Test-Suffix.01";
        assert_eq!(format!("{}", Doi(doi.to_string()).with_domain()), doi);
    }

    #[test]
    fn test_orcid_with_domain() {
        let orcid = "https://orcid.org/0000-0002-1234-5678";
        assert_eq!(format!("{}", Orcid(orcid.to_string()).with_domain()), orcid);
    }

    #[test]
    fn test_ror_with_domain() {
        let ror = "https://ror.org/0abcdef12";
        assert_eq!(format!("{}", Ror(ror.to_string()).with_domain()), ror);
    }

    #[test]
    fn test_timestamp_parse_from_rfc3339_valid() {
        let input = "1999-12-31T23:59:00Z";
        let timestamp = Timestamp::parse_from_rfc3339(input);
        assert!(timestamp.is_ok());

        let expected = Timestamp(Utc.with_ymd_and_hms(1999, 12, 31, 23, 59, 0).unwrap());
        assert_eq!(timestamp.unwrap(), expected);
    }

    #[test]
    fn test_timestamp_parse_from_rfc3339_invalid_format() {
        let input = "1999-12-31 23:59:00"; // Missing 'T' and 'Z'
        let timestamp = Timestamp::parse_from_rfc3339(input);
        assert!(timestamp.is_err());
    }

    #[test]
    fn test_timestamp_parse_from_rfc3339_invalid_date() {
        let input = "1999-02-30T23:59:00Z"; // Invalid date
        let timestamp = Timestamp::parse_from_rfc3339(input);
        assert!(timestamp.is_err());
    }

    #[test]
    fn test_timestamp_to_rfc3339() {
        let timestamp = Timestamp(Utc.with_ymd_and_hms(1999, 12, 31, 23, 59, 0).unwrap());
        assert_eq!(timestamp.to_rfc3339(), "1999-12-31T23:59:00+00:00");
    }

    #[test]
    fn test_timestamp_round_trip_rfc3339_conversion() {
        let original_string = "2023-11-13T12:34:56Z";
        let timestamp = Timestamp::parse_from_rfc3339(original_string).unwrap();
        let converted_string = timestamp.to_rfc3339();

        let round_trip_timestamp = Timestamp::parse_from_rfc3339(&converted_string).unwrap();
        assert_eq!(timestamp, round_trip_timestamp);
    }

    #[test]
    fn test_extract_title_html() {
        let html = "<h1>Main Title</h1>\n<h2>Subtitle Here</h2>";
        let (title, subtitle) = extract_title(html, &MarkupFormat::Html).unwrap();
        assert_eq!(title, "Main Title");
        assert_eq!(subtitle, "Subtitle Here");
    }

    #[test]
    fn test_extract_title_markdown() {
        let md = "# Main Title\n## Subtitle Here";
        let (title, subtitle) = extract_title(md, &MarkupFormat::Markdown).unwrap();
        assert_eq!(title, "Main Title");
        assert_eq!(subtitle, "Subtitle Here");
    }

    #[test]
    fn test_extract_title_jatsxml() {
        let xml = "<title>Main Title</title>\n<subtitle>Subtitle Here</subtitle>";
        let (title, subtitle) = extract_title(xml, &MarkupFormat::JatsXml).unwrap();
        assert_eq!(title, "Main Title");
        assert_eq!(subtitle, "Subtitle Here");
    }

    #[test]
    fn test_extract_title_plaintext() {
        let txt = "MAIN TITLE\nSubtitle Here";
        let (title, subtitle) = extract_title(txt, &MarkupFormat::PlainText).unwrap();
        assert_eq!(title, "MAIN TITLE");
        assert_eq!(subtitle, "Subtitle Here");
    }

    #[test]
    fn test_convert_to_jats() {
        let title = "Sample Title".to_string();
        let subtitle = "Sample Subtitle".to_string();

        let title_jats = convert_to_jats(title.clone(), "title".to_string()).unwrap();
        let subtitle_jats = convert_to_jats(subtitle.clone(), "subtitle".to_string()).unwrap();

        assert_eq!(title_jats, "<title>Sample Title</title>");
        assert_eq!(subtitle_jats, "<subtitle>Sample Subtitle</subtitle>");
    }

    #[test]
    fn test_convert_from_jats_html() {
        let jats = r#"<title>Main Title</title>\n<subtitle>Subtitle Here</subtitle>"#;
        let html = convert_from_jats(jats, MarkupFormat::Html).unwrap();
        assert!(html.contains("<h1>Main Title</h1>"));
        assert!(html.contains("<h2>Subtitle Here</h2>"));
    }

    #[test]
    fn test_convert_from_jats_markdown() {
        let jats = r#"<title>Main Title</title>\n<subtitle>Subtitle Here</subtitle>"#;
        let md = convert_from_jats(jats, MarkupFormat::Markdown).unwrap();
        assert!(md.contains("# Main Title"));
        assert!(md.contains("## Subtitle Here"));
    }

    #[test]
    fn test_convert_from_jats_plaintext() {
        let jats = r#"<title>Main Title</title>\n<subtitle>Subtitle Here</subtitle>"#;
        let txt = convert_from_jats(jats, MarkupFormat::PlainText).unwrap();
        assert!(txt.contains("MAIN TITLE"));
        assert!(txt.contains("Subtitle Here"));
    }

    #[test]
    fn test_convert_from_jats_jatsxml() {
        let jats = r#"<title>Main Title</title>\n<subtitle>Subtitle Here</subtitle>"#;
        let xml = convert_from_jats(jats, MarkupFormat::JatsXml).unwrap();
        assert_eq!(xml, jats);
    }

    #[test]
    fn test_validate_format_html_valid() {
        let html = "<h1>Title</h1><h2>Subtitle</h2>";
        assert!(validate_format(html, &MarkupFormat::Html).is_ok());
    }

    #[test]
    fn test_validate_format_html_invalid() {
        let invalid_html = "Just plain text without tags";
        assert!(validate_format(invalid_html, &MarkupFormat::Html).is_err());
    }

    #[test]
    fn test_validate_format_markdown_valid() {
        let md = "# Title\n## Subtitle\n*Some content*";
        assert!(validate_format(md, &MarkupFormat::Markdown).is_ok());
    }

    #[test]
    fn test_validate_format_markdown_invalid() {
        let invalid_md = "Just plain text without markdown";
        assert!(validate_format(invalid_md, &MarkupFormat::Markdown).is_err());
    }

    #[test]
    fn test_validate_format_jatsxml_valid() {
        let xml = "<?xml version=\"1.0\"?><article><title>Title</title></article>";
        assert!(validate_format(xml, &MarkupFormat::JatsXml).is_ok());
    }

    #[test]
    fn test_validate_format_jatsxml_invalid() {
        let invalid_xml = "Just plain text without XML";
        assert!(validate_format(invalid_xml, &MarkupFormat::JatsXml).is_err());
    }

    #[test]
    fn test_validate_format_plaintext_valid() {
        let text = "Plain text content without markup";
        assert!(validate_format(text, &MarkupFormat::PlainText).is_ok());
    }

    #[test]
    fn test_validate_format_plaintext_invalid() {
        let invalid_text = "Text with <html> tags";
        assert!(validate_format(invalid_text, &MarkupFormat::PlainText).is_err());
    }

    #[test]
    fn test_extract_title_with_validation() {
        // Test that extract_title now validates format
        let invalid_html = "Just plain text";
        assert!(extract_title(invalid_html, &MarkupFormat::Html).is_err());

        let valid_html = "<h1>Title</h1><h2>Subtitle</h2>";
        let (title, subtitle) = extract_title(valid_html, &MarkupFormat::Html).unwrap();
        assert_eq!(title, "Title");
        assert_eq!(subtitle, "Subtitle");
    }
}

pub mod affiliation;
pub mod contribution;
pub mod contributor;
pub mod funding;
pub mod imprint;
pub mod institution;
pub mod issue;
pub mod language;
pub mod location;
pub mod price;
pub mod publication;
pub mod publisher;
pub mod reference;
pub mod series;
pub mod subject;
pub mod title;
pub mod work;
pub mod work_relation;

pub mod locale;

// Explicitly list the items to avoid ambiguous glob re-exports
pub use affiliation::{Affiliation, NewAffiliation};
pub use contribution::{Contribution, NewContribution};
pub use contributor::{Contributor, NewContributor};
pub use funding::{Funding, NewFunding};
pub use imprint::{Imprint, NewImprint};
pub use institution::{Institution, NewInstitution};
pub use issue::{Issue, NewIssue};
pub use language::{Language, NewLanguage};
pub use locale::LocaleCode;
pub use location::{Location, NewLocation};
pub use price::{NewPrice, Price};
pub use publication::{NewPublication, Publication};
pub use publisher::{NewPublisher, Publisher};
pub use reference::{NewReference, Reference};
pub use series::{NewSeries, Series};
pub use subject::{NewSubject, Subject};
pub use title::{NewTitle, Title, TitleOrderBy};
pub use work::{NewWork, Work};
pub use work_relation::{NewWorkRelation, WorkRelation};
