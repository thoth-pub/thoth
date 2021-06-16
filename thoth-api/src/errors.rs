use failure::Fail;

/// A specialised result type for returning Thoth data
pub type ThothResult<T> = std::result::Result<T, ThothError>;

#[derive(Fail, Debug, PartialEq)]
/// Represents anything that can go wrong in Thoth
///
/// This type is not intended to be exhaustively matched, and new variants may
/// be added in the future without a major version bump.
pub enum ThothError {
    #[fail(display = "{} is not a valid {} code", _0, _1)]
    InvalidSubjectCode(String, String),
    #[fail(display = "Database error: {}", _0)]
    DatabaseError(String),
    #[fail(display = "Internal error: {}", _0)]
    InternalError(String),
    #[fail(display = "Invalid credentials.")]
    Unauthorised,
    #[fail(display = "Failed to validate token.")]
    InvalidToken,
    #[fail(display = "No record was found for the given ID.")]
    EntityNotFound,
    #[fail(display = "Issue's Work and Series cannot have different Imprints.")]
    IssueImprintsError,
    #[fail(display = "{} is not a valid metadata specification", _0)]
    InvalidMetadataSpecification(String),
    #[fail(display = "Invalid UUID supplied.")]
    InvalidUuid,
    #[fail(display = "CSV Error: {}", _0)]
    CsvError(String),
    #[fail(display = "Could not generate {}: {}", _0, _1)]
    IncompleteMetadataRecord(String, String),
    #[fail(
        display = "{} is not a validly formatted {} and will not be saved",
        _0, _1
    )]
    IdentifierParseError(String, String),
}

impl juniper::IntoFieldError for ThothError {
    fn into_field_error(self) -> juniper::FieldError {
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

#[cfg(feature = "backend")]
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
            _ => HttpResponse::InternalServerError().json(self.to_string()),
        }
    }
}

#[cfg(feature = "backend")]
impl From<diesel::result::Error> for ThothError {
    fn from(error: diesel::result::Error) -> ThothError {
        use diesel::result::Error;
        match error {
            Error::DatabaseError(_kind, info) => {
                let message = info.details().unwrap_or_else(|| info.message()).to_string();
                ThothError::DatabaseError(message)
            }
            Error::NotFound => ThothError::EntityNotFound,
            _ => ThothError::InternalError("".into()),
        }
    }
}

#[cfg(feature = "backend")]
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

impl From<xml::writer::Error> for ThothError {
    fn from(error: xml::writer::Error) -> ThothError {
        ThothError::InternalError(error.to_string())
    }
}

impl From<failure::Error> for ThothError {
    fn from(error: failure::Error) -> ThothError {
        if error.downcast_ref::<ThothError>().is_some() {
            return error.downcast::<ThothError>().unwrap();
        }
        ThothError::InternalError(error.to_string())
    }
}

impl From<uuid::parser::ParseError> for ThothError {
    fn from(_: uuid::parser::ParseError) -> ThothError {
        ThothError::InvalidUuid
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uuid_error() {
        assert_eq!(
            ThothError::from(uuid::Uuid::parse_str("not-a-uuid").unwrap_err()),
            ThothError::InvalidUuid
        );
    }
}
