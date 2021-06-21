use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
#[cfg(feature = "backend")]
use uuid::Uuid;

use crate::errors::{ThothError, ThothResult};

pub const DOI_DOMAIN: &str = "https://doi.org/";
pub const ORCID_DOMAIN: &str = "https://orcid.org/";

#[cfg_attr(
    feature = "backend",
    derive(DieselNewType, juniper::GraphQLScalarValue),
    graphql(
        description = r#"Digital Object Identifier. Expressed as `^https:\/\/doi\.org\/10\.\d{4,9}\/[-._\;\(\)\/:a-zA-Z0-9]+$`"#
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Doi(String);

#[cfg_attr(
    feature = "backend",
    derive(DieselNewType, juniper::GraphQLScalarValue),
    graphql(
        description = r#"ORCID (Open Researcher and Contributor ID) identifier. Expressed as `^https:\/\/orcid\.org\/0000-000(1-[5-9]|2-[0-9]|3-[0-4])\d{3}-\d{3}[\dX]$`"#
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Orcid(String);

#[cfg_attr(
    feature = "backend",
    derive(DieselNewType, juniper::GraphQLScalarValue),
    graphql(description = "RFC 3339 combined date and time in UTC time zone")
)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Timestamp(DateTime<Utc>);

impl Default for Doi {
    fn default() -> Doi {
        Doi(Default::default())
    }
}

impl Default for Orcid {
    fn default() -> Orcid {
        Orcid(Default::default())
    }
}

impl Default for Timestamp {
    fn default() -> Timestamp {
        Timestamp(TimeZone::timestamp(&Utc, 0, 0))
    }
}

impl fmt::Display for Doi {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", &self.0.replace(DOI_DOMAIN, ""))
    }
}

impl fmt::Display for Orcid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", &self.0.replace(ORCID_DOMAIN, ""))
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
            r#"^(?i:(?:https?://)?(?:www\.)?(?:dx\.)?doi\.org/)?(10\.\d{4,9}/[-._;()/:a-zA-Z0-9]+$)"#).unwrap();
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
            // Matches strings of format "[[http[s]://][www.]orcid.org/]0000-000X-XXXX-XXXX"
            // and captures the 16-digit identifier segment
            // Corresponds to database constraints although regex syntax differs slightly
            r#"^(?i:(?:https?://)?(?:www\.)?orcid\.org/)?(0000-000(?:1-[5-9]|2-[0-9]|3-[0-4])\d{3}-\d{3}[\dX]$)"#).unwrap();
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

impl Doi {
    pub fn to_lowercase_string(&self) -> String {
        self.0.to_lowercase()
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
        filter_param_1: Option<Self::FilterParameter1>,
        filter_param_2: Option<Self::FilterParameter2>,
    ) -> ThothResult<Vec<Self>>;

    /// Query the database to obtain the total number of entities satisfying the search criteria
    fn count(
        db: &crate::db::PgPool,
        filter: Option<String>,
        publishers: Vec<Uuid>,
        filter_param_1: Option<Self::FilterParameter1>,
        filter_param_2: Option<Self::FilterParameter2>,
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

    fn insert(&self, connection: &diesel::PgConnection) -> ThothResult<Self::MainEntity>;
}

/// Declares function implementations for the `Crud` trait, reducing the boilerplate needed to define
/// the CRUD functionality for each entity.
///
/// Example usage
/// -------------
///
/// ```ignore
/// use crate::imprint::model::{NewImprint, PatchImprint, Imprint, NewImprintHistory};
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
        fn from_id(db: &crate::db::PgPool, entity_id: &Uuid) -> ThothResult<Self> {
            use diesel::{QueryDsl, RunQueryDsl};

            let connection = db.get().unwrap();
            match $entity_dsl.find(entity_id).get_result::<Self>(&connection) {
                Ok(t) => Ok(t),
                Err(e) => Err(ThothError::from(e)),
            }
        }

        fn create(db: &crate::db::PgPool, data: &Self::NewEntity) -> ThothResult<Self> {
            use diesel::RunQueryDsl;

            let connection = db.get().unwrap();
            match diesel::insert_into($table_dsl)
                .values(data)
                .get_result::<Self>(&connection)
            {
                Ok(t) => Ok(t),
                Err(e) => Err(ThothError::from(e)),
            }
        }

        /// Makes a database transaction that first updates the entity and then creates a new
        /// history entity record.
        fn update(
            &self,
            db: &crate::db::PgPool,
            data: &Self::PatchEntity,
            account_id: &Uuid,
        ) -> ThothResult<Self> {
            use diesel::{Connection, QueryDsl, RunQueryDsl};

            let connection = db.get().unwrap();
            connection.transaction(|| {
                match diesel::update($entity_dsl.find(&self.pk()))
                    .set(data)
                    .get_result(&connection)
                {
                    Ok(c) => match self.new_history_entry(&account_id).insert(&connection) {
                        Ok(_) => Ok(c),
                        Err(e) => Err(e),
                    },
                    Err(e) => Err(ThothError::from(e)),
                }
            })
        }

        fn delete(self, db: &crate::db::PgPool) -> ThothResult<Self> {
            use diesel::{QueryDsl, RunQueryDsl};

            let connection = db.get().unwrap();
            match diesel::delete($entity_dsl.find(&self.pk())).execute(&connection) {
                Ok(_) => Ok(self),
                Err(e) => Err(ThothError::from(e)),
            }
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
/// use crate::imprint::model::{ImprintHistory, NewImprintHistory};
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
        fn insert(&self, connection: &diesel::PgConnection) -> ThothResult<Self::MainEntity> {
            use diesel::RunQueryDsl;

            match diesel::insert_into($table_dsl)
                .values(self)
                .get_result(connection)
            {
                Ok(t) => Ok(t),
                Err(e) => Err(ThothError::from(e)),
            }
        }
    };
}

#[test]
fn test_doi_default() {
    let doi: Doi = Default::default();
    assert_eq!(doi, Doi("".to_string()));
}

#[test]
fn test_orcid_default() {
    let orcid: Orcid = Default::default();
    assert_eq!(orcid, Orcid("".to_string()));
}

#[test]
fn test_timestamp_default() {
    let stamp: Timestamp = Default::default();
    assert_eq!(stamp, Timestamp(TimeZone::timestamp(&Utc, 0, 0)));
}

#[test]
fn test_doi_display() {
    let doi = Doi("https://doi.org/10.12345/Test-Suffix.01".to_string());
    assert_eq!(format!("{}", doi), "10.12345/Test-Suffix.01");
}

#[test]
fn test_orcid_display() {
    let orcid = Orcid("https://orcid.org/0000-0002-1234-5678".to_string());
    assert_eq!(format!("{}", orcid), "0000-0002-1234-5678");
}

#[test]
fn test_timestamp_display() {
    let stamp: Timestamp = Default::default();
    assert_eq!(format!("{}", stamp), "1970-01-01 00:00:00");
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
}
