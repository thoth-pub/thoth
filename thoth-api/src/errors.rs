#[cfg(feature = "backend")]
use actix_web::{error::ResponseError, HttpResponse};
#[cfg(feature = "backend")]
use diesel::result::Error as DBError;
use failure::Fail;

/// A specialised result type for any returning Thoth data
pub type ThothResult<T> = std::result::Result<T, ThothError>;

#[derive(Fail, Debug)]
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
    #[fail(display = "No cookie found.")]
    CookieError(),
    #[fail(display = "No record was found for the given ID.")]
    EntityNotFound,
    #[fail(display = "Issue's Work and Series cannot have different Imprints.")]
    IssueImprintsError,
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
impl ResponseError for ThothError {
    fn error_response(&self) -> HttpResponse {
        match self {
            ThothError::Unauthorised => HttpResponse::Unauthorized().json("Unauthorized"),
            ThothError::DatabaseError { .. } => {
                HttpResponse::InternalServerError().json("DB error")
            }
            _ => HttpResponse::InternalServerError().json("Internal error"),
        }
    }
}

#[cfg(feature = "backend")]
impl From<DBError> for ThothError {
    fn from(error: DBError) -> ThothError {
        match error {
            DBError::DatabaseError(_kind, info) => {
                let message = info.details().unwrap_or_else(|| info.message()).to_string();
                ThothError::DatabaseError(message)
            }
            DBError::NotFound => ThothError::EntityNotFound,
            _ => ThothError::InternalError("".into()),
        }
    }
}

impl From<std::io::Error> for ThothError {
    fn from(error: std::io::Error) -> ThothError {
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
