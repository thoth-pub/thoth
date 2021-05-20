use actix_web::{http::StatusCode, Error, HttpRequest, Responder};
use paperclip::actix::web::HttpResponse;
use paperclip::actix::OperationModifier;
use paperclip::util::{ready, Ready};
use paperclip::v2::models::{DefaultOperationRaw, Either, Response};
use paperclip::v2::schema::Apiv2Schema;
use std::fmt;

pub struct Xml<String>(pub String);

impl<String> fmt::Debug for Xml<String> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status: StatusCode = StatusCode::OK;
        let status_str = status.canonical_reason().unwrap_or(status.as_str());
        write!(f, "{} Xml: {:?}", status_str, self)
    }
}

impl<String> fmt::Display for Xml<String> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", &self)
    }
}

impl<String> Responder for Xml<String>
where
    actix_web::dev::Body: From<String>,
{
    type Error = Error;
    type Future = Ready<Result<HttpResponse, Error>>;

    fn respond_to(self, _: &HttpRequest) -> Self::Future {
        ready(Ok(HttpResponse::build(StatusCode::OK)
            .content_type("text/xml; charset=utf-8")
            .body(self.0)))
    }
}

impl<String> Apiv2Schema for Xml<String> {
    const DESCRIPTION: &'static str = "ONIX";
}

impl<String> OperationModifier for Xml<String> {
    fn update_response(op: &mut DefaultOperationRaw) {
        let status: StatusCode = StatusCode::OK;
        op.responses.insert(
            status.as_str().into(),
            Either::Right(Response {
                description: status.canonical_reason().map(ToString::to_string),
                schema: None,
                ..Default::default()
            }),
        );
    }
}
