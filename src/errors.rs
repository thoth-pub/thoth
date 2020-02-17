use actix_http::ResponseBuilder;
use actix_web::{error, http::header, http::StatusCode, HttpResponse};
use failure::Fail;

#[derive(Fail, Debug)]
pub enum SubjectError {
    #[fail(display = "{} is not a valid {} code", _0, _1)]
    InvalidCode(String, String),
}

impl error::ResponseError for SubjectError {
    fn error_response(&self) -> HttpResponse {
        ResponseBuilder::new(self.status_code())
            .set_header(header::CONTENT_TYPE, "text/html; charset=utf-8")
            .body(self.to_string())
    }
    fn status_code(&self) -> StatusCode {
        match *self {
            SubjectError::InvalidCode { .. } => StatusCode::BAD_REQUEST,
        }
    }
}
