#[cfg(feature = "backend")]
use actix_web::{error::ResponseError, HttpResponse};
#[cfg(feature = "backend")]
use diesel::result::Error as DBError;
use failure::Fail;

pub type Result<T> = std::result::Result<T, failure::Error>;

#[derive(Fail, Debug)]
pub enum ThothError {
    #[fail(display = "{} is not a valid {} code", _0, _1)]
    InvalidSubjectCode(String, String),
    #[fail(display = "{} is not a valid Language Code", _0)]
    InvalidLanguageCode(String),
    #[fail(display = "{} is not a valid Work Type", _0)]
    InvalidWorkType(String),
    #[fail(display = "{} is not a valid Work Status", _0)]
    InvalidWorkStatus(String),
    #[fail(display = "{} is not a valid Contribution Type", _0)]
    InvalidContributionType(String),
    #[fail(display = "{} is not a valid Publication Type", _0)]
    InvalidPublicationType(String),
    #[fail(display = "{} is not a valid Series Type", _0)]
    InvalidSeriesType(String),
    #[fail(display = "{} is not a valid Subject Type", _0)]
    InvalidSubjectType(String),
    #[fail(display = "{} is not a valid Language Relation", _0)]
    InvalidLanguageRelation(String),
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
