use core::convert::From;
use serde::Deserialize;
use std::fmt;
use thiserror::Error;

/// A specialised result type for returning Thoth data
pub type ThothResult<T> = std::result::Result<T, ThothError>;

#[derive(Error, Debug, PartialEq, Eq)]
/// Represents anything that can go wrong in Thoth
///
/// This type is not intended to be exhaustively matched, and new variants may
/// be added in the future without a major version bump.
pub enum ThothError {
    #[error("{0} is not a valid {1} code")]
    InvalidSubjectCode(String, String),
    #[error("Database error: {0}")]
    DatabaseError(String),
    #[error("Internal error: {0}")]
    InternalError(String),
    #[error("Invalid credentials.")]
    Unauthorised,
    #[error("Failed to validate token.")]
    InvalidToken,
    #[error("No record was found for the given ID.")]
    EntityNotFound,
    #[error("Issue's Work and Series cannot have different Imprints.")]
    IssueImprintsError,
    #[error("{0} is not a valid metadata specification")]
    InvalidMetadataSpecification(String),
    #[error("Invalid UUID supplied.")]
    InvalidUuid,
    #[error("CSV Error: {0}")]
    CsvError(String),
    #[error("Could not generate {0}: {1}")]
    IncompleteMetadataRecord(String, String),
    #[error("{0} is not a validly formatted ORCID and will not be saved")]
    OrcidParseError(String),
    #[error("{0} is not a validly formatted DOI and will not be saved")]
    DoiParseError(String),
    #[error("{0} is not a validly formatted ISBN and will not be saved")]
    IsbnParseError(String),
    #[error("{0} is not a validly formatted ROR ID and will not be saved")]
    RorParseError(String),
    #[error("Cannot parse ORCID: no value provided")]
    OrcidEmptyError,
    #[error("Cannot parse DOI: no value provided")]
    DoiEmptyError,
    #[error("Cannot parse ISBN: no value provided")]
    IsbnEmptyError,
    #[error("Cannot parse ROR ID: no value provided")]
    RorEmptyError,
    #[error("Works of type Book Chapter cannot have ISBNs in their Publications.")]
    ChapterIsbnError,
    #[error(
        "Works of type Book Chapter cannot have Width, Height, Depth or Weight in their Publications."
    )]
    ChapterDimensionError,
    #[error("Each Publication must have exactly one canonical Location.")]
    CanonicalLocationError,
    #[error(
        "Canonical Locations for digital Publications must have both a Landing Page and a Full Text URL."
    )]
    LocationUrlError,
    #[error("When specifying Weight, both values (g and oz) must be supplied.")]
    WeightEmptyError,
    #[error("When specifying Width, both values (mm and in) must be supplied.")]
    WidthEmptyError,
    #[error("When specifying Height, both values (mm and in) must be supplied.")]
    HeightEmptyError,
    #[error("When specifying Depth, both values (mm and in) must be supplied.")]
    DepthEmptyError,
    #[error(
        "Width/Height/Depth/Weight are only applicable to physical (Paperback/Hardback) Publications."
    )]
    DimensionDigitalError,
    #[error(
        "Price values must be greater than zero. To indicate an unpriced Publication, omit all Prices."
    )]
    PriceZeroError,
    #[error("{0}")]
    RequestError(String),
    #[error("{0}")]
    GraphqlError(String),
}

#[cfg(not(target_arch = "wasm32"))]
impl juniper::IntoFieldError for ThothError {
    fn into_field_error(self) -> juniper::FieldError {
        use juniper::graphql_value;
        match self {
            ThothError::InvalidSubjectCode { .. } => juniper::FieldError::new(
                self.to_string(),
                graphql_value!({
                    "type": "INVALID_SUBJECT_CODE"
                }),
            ),
            ThothError::Unauthorised => juniper::FieldError::new(
                "Unauthorized",
                graphql_value!({
                    "type": "NO_ACCESS"
                }),
            ),
            _ => juniper::FieldError::new(
                self.to_string(),
                graphql_value!({
                    "type": "INTERNAL_ERROR"
                }),
            ),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl actix_web::error::ResponseError for ThothError {
    fn error_response(&self) -> actix_web::HttpResponse {
        use actix_web::HttpResponse;
        match self {
            ThothError::Unauthorised | ThothError::InvalidToken => {
                HttpResponse::Unauthorized().json(self.to_string())
            }
            ThothError::EntityNotFound => HttpResponse::NotFound().json(self.to_string()),
            ThothError::InvalidMetadataSpecification(_) | ThothError::InvalidUuid => {
                HttpResponse::BadRequest().json(self.to_string())
            }
            ThothError::DatabaseError { .. } => {
                HttpResponse::InternalServerError().json("DB error")
            }
            ThothError::IncompleteMetadataRecord(_, _) => {
                HttpResponse::NotFound().json(self.to_string())
            }
            _ => HttpResponse::InternalServerError().json(self.to_string()),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
/// An array of tuples containing a database constraint name and a corresponding error to output
/// when the constraint is violated.
///
/// To obtain a list of unique and check constraints:
/// ```sql
/// SELECT conname
/// FROM pg_catalog.pg_constraint con
/// INNER JOIN pg_catalog.pg_class rel ON rel.oid = con.conrelid
/// INNER JOIN pg_catalog.pg_namespace nsp ON nsp.oid = connamespace
/// WHERE nsp.nspname = 'public'
/// AND contype in ('u', 'c');
/// ```
const DATABASE_CONSTRAINT_ERRORS: [(&str, &str); 3] = [
    (
        "contribution_contribution_ordinal_work_id_uniq",
        "A contribution with this ordinal number already exists.",
    ),
    (
        "work_relation_ordinal_type_uniq",
        "A relation with this ordinal number already exists.",
    ),
    (
        "affiliation_uniq_ord_in_contribution_idx",
        "An affiliation with this ordinal number already exists.",
    ),
];

#[cfg(not(target_arch = "wasm32"))]
impl From<diesel::result::Error> for ThothError {
    fn from(error: diesel::result::Error) -> ThothError {
        use diesel::result::Error;
        match error {
            Error::DatabaseError(_kind, info) => {
                let mut message = info.message();
                for (constranint, error) in DATABASE_CONSTRAINT_ERRORS {
                    if message.contains(constranint) {
                        message = error;
                        break;
                    }
                }
                ThothError::DatabaseError(message.to_string())
            }
            Error::NotFound => ThothError::EntityNotFound,
            _ => ThothError::InternalError("".into()),
        }
    }
}

#[derive(Debug, Deserialize)]
struct GraphqlError {
    message: String,
}

#[derive(Debug, Deserialize)]
struct GrqphqlErrorMessage {
    errors: Vec<GraphqlError>,
}

impl fmt::Display for GraphqlError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl fmt::Display for GrqphqlErrorMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for error in &self.errors {
            write!(f, "{}", error)?;
        }
        Ok(())
    }
}

impl From<yewtil::fetch::FetchError> for ThothError {
    fn from(error: yewtil::fetch::FetchError) -> Self {
        use serde_json::error::Result;
        use yewtil::fetch::FetchError;
        match error {
            FetchError::DeserializeError { error: _, content } => {
                let message: Result<GrqphqlErrorMessage> = serde_json::from_str(&content);
                match message {
                    Ok(m) => ThothError::GraphqlError(m.to_string()),
                    Err(_) => ThothError::RequestError(content),
                }
            }
            FetchError::CouldNotCreateFetchFuture => {
                ThothError::RequestError("Could not connect to the API.".to_string())
            }
            _ => ThothError::RequestError(error.to_string()),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<csv::Error> for ThothError {
    fn from(e: csv::Error) -> Self {
        ThothError::CsvError(e.to_string())
    }
}

impl From<std::io::Error> for ThothError {
    fn from(error: std::io::Error) -> ThothError {
        ThothError::InternalError(error.to_string())
    }
}

impl From<&std::io::Error> for ThothError {
    fn from(error: &std::io::Error) -> ThothError {
        ThothError::InternalError(error.to_string())
    }
}

impl From<reqwest::Error> for ThothError {
    fn from(error: reqwest::Error) -> ThothError {
        ThothError::InternalError(error.to_string())
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<xml::writer::Error> for ThothError {
    fn from(error: xml::writer::Error) -> ThothError {
        ThothError::InternalError(error.to_string())
    }
}

impl From<uuid_07::parser::ParseError> for ThothError {
    fn from(_: uuid_07::parser::ParseError) -> ThothError {
        ThothError::InvalidUuid
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<uuid_08::Error> for ThothError {
    fn from(_: uuid_08::Error) -> ThothError {
        ThothError::InvalidUuid
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csv_error() {
        // We are just testing that _a_ `csv::error` is converted to `ThothError::CsvError`.
        // The test instantiation is copied from the library: https://github.com/BurntSushi/rust-csv/blob/40ea4c49d7467d2b607a6396424f8e0e101adae1/src/writer.rs#L1268
        let mut wtr = csv::WriterBuilder::new().from_writer(vec![]);
        wtr.write_record(&csv::ByteRecord::from(vec!["a", "b", "c"]))
            .unwrap();
        let err = wtr
            .write_record(&csv::ByteRecord::from(vec!["a"]))
            .unwrap_err();
        assert!(matches!(ThothError::from(err), ThothError::CsvError { .. }));
    }

    #[test]
    fn test_uuid_error() {
        assert_eq!(
            ThothError::from(uuid_07::Uuid::parse_str("not-a-uuid").unwrap_err()),
            ThothError::InvalidUuid
        );
        assert_eq!(
            ThothError::from(uuid_08::Uuid::parse_str("not-a-uuid").unwrap_err()),
            ThothError::InvalidUuid
        );
    }

    #[test]
    fn test_fetch_error() {
        use yewtil::fetch::FetchError;
        let error = "{\"data\":null,\"errors\":[{\"message\":\"A relation with this ordinal already exists.\",\"locations\":[{\"line\":8,\"column\":9}],\"path\":[\"createWorkRelation\"]}]}";
        let fetch_error = FetchError::DeserializeError {
            error: "".to_string(),
            content: error.to_string(),
        };
        assert_eq!(
            ThothError::from(fetch_error),
            ThothError::GraphqlError("A relation with this ordinal already exists.".to_string())
        )
    }
}
