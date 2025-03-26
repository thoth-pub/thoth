#[cfg(not(target_arch = "wasm32"))]
mod database_errors;

use core::convert::From;
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, fmt};
use thiserror::Error;

/// A specialised result type for returning Thoth data
pub type ThothResult<T> = Result<T, ThothError>;

#[derive(Error, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// Represents anything that can go wrong in Thoth
///
/// This type is not intended to be exhaustively matched, and new variants may
/// be added in the future without a major version bump.
pub enum ThothError {
    #[error("{input:?} is not a valid {subject_type:?} code")]
    InvalidSubjectCode { input: String, subject_type: String },
    #[error("Database error: {0}")]
    DatabaseError(String),
    #[error("Redis error: {0}")]
    RedisError(String),
    #[error("{0}")]
    DatabaseConstraintError(Cow<'static, str>),
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
    #[error("Invalid timestamp supplied.")]
    InvalidTimestamp,
    #[error("CSV Error: {0}")]
    CsvError(String),
    #[error("MARC Error: {0}")]
    MarcError(String),
    #[error("Could not generate {0}: {1}")]
    IncompleteMetadataRecord(String, String),
    #[error("The metadata record has not yet been generated.")]
    MetadataRecordNotGenerated,
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
    #[error("Publication Date is required for Active, Withdrawn, and Superseded Works.")]
    PublicationDateError,
    #[error("{0}")]
    RequestError(String),
    #[error("{0}")]
    GraphqlError(String),
    #[error("Withdrawn Date must be later than Publication Date.")]
    WithdrawnDateBeforePublicationDateError,
    #[error("Withdrawn Date can only be added to a Superseded or Withdrawn Work.")]
    WithdrawnDateError,
    #[error("A Superseded or Withdrawn Work must have a Withdrawn Date.")]
    NoWithdrawnDateError,
    #[error("Only superusers can create, edit, or delete Locations where the Location Platform is Thoth.")]
    ThothLocationError,
    #[error("Only superusers can update the canonical location when Thoth Location Platform is already set as canonical.")]
    ThothUpdateCanonicalError,
    #[error("Once a Work has been published, it cannot return to an unpublished Work Status.")]
    ThothSetWorkStatusError,
}

impl ThothError {
    /// Serialise to JSON
    pub fn to_json(&self) -> ThothResult<String> {
        serde_json::to_string(&self).map_err(Into::into)
    }

    /// Deserialise from JSON
    pub fn from_json(s: &str) -> ThothResult<ThothError> {
        serde_json::from_str(s).map_err(Into::into)
    }
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
            ThothError::RedisError { .. } => {
                HttpResponse::InternalServerError().json("Redis error")
            }
            ThothError::IncompleteMetadataRecord(_, _) => {
                HttpResponse::NotFound().json(self.to_string())
            }
            _ => HttpResponse::InternalServerError().json(self.to_string()),
        }
    }
}

#[derive(Debug, Deserialize)]
struct GraphqlError {
    message: String,
}

#[derive(Debug, Deserialize)]
struct GraqphqlErrorMessage {
    errors: Vec<GraphqlError>,
}

impl fmt::Display for GraphqlError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl fmt::Display for GraqphqlErrorMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for error in &self.errors {
            write!(f, "{error}")?;
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
                let message: Result<GraqphqlErrorMessage> = serde_json::from_str(&content);
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
impl From<reqwest_middleware::Error> for ThothError {
    fn from(error: reqwest_middleware::Error) -> ThothError {
        ThothError::InternalError(error.to_string())
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<xml::writer::Error> for ThothError {
    fn from(error: xml::writer::Error) -> ThothError {
        ThothError::InternalError(error.to_string())
    }
}

impl From<uuid::Error> for ThothError {
    fn from(_: uuid::Error) -> ThothError {
        ThothError::InvalidUuid
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<marc::Error> for ThothError {
    fn from(e: marc::Error) -> Self {
        ThothError::MarcError(e.to_string())
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<dialoguer::Error> for ThothError {
    fn from(e: dialoguer::Error) -> Self {
        ThothError::InternalError(e.to_string())
    }
}

impl From<chrono::ParseError> for ThothError {
    fn from(_: chrono::ParseError) -> Self {
        ThothError::InvalidTimestamp
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<deadpool_redis::redis::RedisError> for ThothError {
    fn from(e: deadpool_redis::redis::RedisError) -> Self {
        ThothError::RedisError(e.to_string())
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<deadpool_redis::PoolError> for ThothError {
    fn from(e: deadpool_redis::PoolError) -> Self {
        ThothError::InternalError(e.to_string())
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<Box<dyn std::error::Error + Send + Sync>> for ThothError {
    fn from(e: Box<dyn std::error::Error + Send + Sync>) -> Self {
        ThothError::InternalError(e.to_string())
    }
}

impl From<serde_json::Error> for ThothError {
    fn from(e: serde_json::Error) -> Self {
        ThothError::InternalError(e.to_string())
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
            ThothError::from(uuid::Uuid::parse_str("not-a-uuid").unwrap_err()),
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

    #[test]
    fn test_round_trip_serialisation() {
        let original_error = ThothError::InvalidSubjectCode {
            input: "002".to_string(),
            subject_type: "BIC".to_string(),
        };
        let json = original_error.to_json().unwrap();
        let deserialised_error = ThothError::from_json(&json).unwrap();
        assert_eq!(original_error, deserialised_error);
    }

    #[test]
    fn test_to_json_valid_error() {
        let error = ThothError::InvalidSubjectCode {
            input: "001".to_string(),
            subject_type: "BIC".to_string(),
        };
        let json = error.to_json().unwrap();

        assert!(json.contains("\"InvalidSubjectCode\""));
        assert!(json.contains("\"001\""));
        assert!(json.contains("\"BIC\""));
    }

    #[test]
    fn test_invalid_json_deserialisation() {
        let invalid_json = r#"{"UnknownError":"Unexpected field"}"#;
        let error = ThothError::from_json(invalid_json);
        assert!(error.is_err());
    }
}
